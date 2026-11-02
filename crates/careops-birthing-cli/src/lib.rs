//! Native birthing batch consumer; runtime and data components are separately pinned.

use careops_core::experiment::{
    self, ExperimentError, ExperimentErrorKind, GenericBatchCheckpoint, IndexedReplication,
};
use kairo_data_types::{EntityId, EventId};
use kairo_ecs_arrow::{
    polars_interop as io,
    run_output::{self, RunOutput, RunOutputConfig, ThreadBudget},
};
use kairo_ecs_arrow::{EventLogRecord, EventStatus, SCHEMA_VERSION_V2};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

pub mod model;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub const DATA_REVISION: &str = "4134a3733ba7e833a8c520621ec2af5a4bd4b31a";
pub const RUNTIME_REVISION: &str = "07410266bf54b6f73c59db2f8cbfbc5697c53734";
pub const MODEL_CONTRACT_SHA256: &str = env!("CAREOPS_MODEL_CONTRACT_SHA256");
const MAX_ROWS: usize = 4096;
const PART_ROWS: usize = 64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub schema_version: u32,
    pub revision: String,
    pub provenance: String,
    pub time_unit: String,
    pub days: u32,
    pub infants_per_birth: u16,
    pub progression_rate: f64,
    pub service_min_seconds: u64,
    pub service_jitter_seconds: u64,
    pub travel_min_seconds: u64,
    pub travel_jitter_seconds: u64,
    pub theatre_open_delay_seconds: u64,
    pub destination_open_delay_seconds: u64,
    pub preparation_seconds: u64,
    pub incision_to_birth_seconds: u64,
    pub completion_after_birth_seconds: u64,
    pub cleaning_seconds: u64,
    pub target_birth_seconds: u64,
    pub acknowledgement_delay_seconds: u64,
}
impl Scenario {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || self.revision.trim().is_empty()
            || !self.provenance.starts_with("synthetic:")
            || self.time_unit != "second"
            || !(1..=7).contains(&self.days)
            || !(1..=3).contains(&self.infants_per_birth)
            || !self.progression_rate.is_finite()
            || self.progression_rate <= 0.0
            || self.service_min_seconds == 0
            || self.travel_min_seconds == 0
            || self.completion_after_birth_seconds < 3
            // This finite coordinator seals theatre entry before birth and
            // completes LDR cleaning before downstream arrival. Other timing
            // topologies need their own global-tick packet, not silent clamping.
            || (self.preparation_seconds == 0 && self.incision_to_birth_seconds == 0)
            || self.cleaning_seconds > self.travel_min_seconds
        {
            return Err("invalid bounded synthetic scenario declaration".into());
        }
        let times = [
            self.service_min_seconds,
            self.service_jitter_seconds,
            self.travel_min_seconds,
            self.travel_jitter_seconds,
            self.theatre_open_delay_seconds,
            self.destination_open_delay_seconds,
            self.preparation_seconds,
            self.incision_to_birth_seconds,
            self.completion_after_birth_seconds,
            self.cleaning_seconds,
            self.target_birth_seconds,
            self.acknowledgement_delay_seconds,
        ];
        if times.iter().any(|v| *v > 30)
            || times.iter().sum::<u64>() > 150
            || self.destination_open_delay_seconds
                > self.theatre_open_delay_seconds
                    + self.preparation_seconds
                    + self.incision_to_birth_seconds
                    + 2
        {
            return Err("scenario exceeds the reviewed finite case/time envelope".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub tick: u64,
    pub person: u64,
    pub episode: u64,
    pub kind: String,
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchInput {
    pub schema_version: u32,
    pub batch_id: String,
    pub seed: u64,
    pub replications: usize,
    pub scenario: Scenario,
}
/// The existing runner flattens metadata into its checkpoint envelope. Namespace
/// our versioned input so schema_version/batch_id cannot collide with that envelope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchMetadata {
    pub birthing_input: BatchInput,
    pub model_contract_sha256: String,
}
impl BatchInput {
    pub fn validate(&self) -> Result<()> {
        self.scenario.validate()?;
        if self.schema_version != 1
            || !(1..=32).contains(&self.replications)
            || self.batch_id.is_empty()
            || self.batch_id.len() > 64
            || !self
                .batch_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err("invalid batch identity/version/replication bound".into());
        }
        Ok(())
    }
    pub fn hash(&self) -> Result<String> {
        Ok(hash(self.effective_json()?.as_bytes()))
    }
    pub fn effective_json(&self) -> Result<String> {
        #[derive(Serialize)]
        struct Effective<'a> {
            input: &'a BatchInput,
            model_contract_sha256: &'static str,
        }
        Ok(serde_json::to_string(&Effective {
            input: self,
            model_contract_sha256: MODEL_CONTRACT_SHA256,
        })?)
    }
}
pub fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err("unsupported file or byte bound exceeded".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("file grew beyond byte bound".into());
    }
    Ok(bytes)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileReceipt {
    pub file: String,
    pub sha256: String,
    pub rows: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Replication {
    pub index: usize,
    pub seed: u64,
    pub directory: String,
    pub rows: usize,
    pub canonical_sha256: String,
    pub model_contract_sha256: String,
}
impl IndexedReplication for Replication {
    fn replication_index(&self) -> usize {
        self.index
    }
    fn seed(&self) -> u64 {
        self.seed
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplicationManifest {
    pub schema_version: u32,
    pub input: BatchInput,
    pub effective_input_sha256: String,
    /// Exact normalized UTF-8 bytes hashed by the writer. Readers parse these
    /// and compare effective values without re-spelling floating-point numbers.
    pub effective_input_json: String,
    pub runtime_revision: String,
    pub data_revision: String,
    pub tick_unit: String,
    pub replication: Replication,
    pub ipc: Vec<FileReceipt>,
    pub completed: bool,
}

fn records(run: &str, rows: &[Row], offset: usize) -> Result<Vec<EventLogRecord>> {
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            Ok(EventLogRecord {
                schema_version: SCHEMA_VERSION_V2,
                run_id: run.into(),
                event_id: EventId {
                    index: (offset + i + 1) as u64,
                    generation: 0,
                },
                entity_id: (row.person != 0).then_some(EntityId {
                    index: row.person,
                    generation: 0,
                }),
                time_ticks: u128::from(row.tick),
                time_scale: "ticks".into(),
                priority: 0,
                sequence: (offset + i + 1) as u64,
                event_kind: row.kind.clone(),
                status: EventStatus::Dispatched,
                payload_ref: Some(serde_json::to_string(row)?),
            })
        })
        .collect()
}

/// Publish original replication capture in finite parts. No export-time rerun.
/// A domain manifest completes only after both actual formats and the upstream
/// Parquet publication have completed. Partial attempts remain for diagnosis.
fn publish(
    root: &Path,
    input: &BatchInput,
    index: usize,
    seed: u64,
    fail: bool,
) -> Result<Replication> {
    for attempt in 1..=32 {
        let run = format!("{}-attempt{attempt}", input.batch_id);
        let directory = format!("{run}-rep{index}");
        let dir = root.join(&directory);
        if dir.exists() {
            if dir.join("birthing_manifest.json").exists() {
                let manifest = validate_replication(root, &directory, input, index, seed)?;
                return Ok(manifest.replication);
            }
            // Interrupted attempts are never overwritten or treated as complete.
            continue;
        }
        let mut output = RunOutput::create(
            RunOutputConfig {
                run_id: run.clone(),
                replication_id: index as u64,
                seed,
                time_scale: "second".into(),
                effective_input_hash: input.hash()?,
                raw_input_hash: None,
                max_rows_per_part: NonZeroUsize::new(PART_ROWS).unwrap(),
            },
            root,
        )?;
        let rows = model::simulate(&input.scenario, seed, index as u64 + 1, 0)?;
        if rows.is_empty() || rows.len() > MAX_ROWS {
            return Err("capture row bound violated".into());
        }
        let mut ipc = Vec::new();
        for (part, chunk) in rows.chunks(PART_ROWS).enumerate() {
            let capture = records(&run, chunk, part * PART_ROWS)?;
            output.write_batch(&capture)?;
            let mut frame = io::records_to_dataframe(&capture)?;
            let mut bytes = Vec::new();
            io::write_ipc_stream(&mut frame, &mut bytes)?;
            let file = format!("ipc-{part:05}.arrow");
            let mut sink = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(dir.join(&file))?;
            use std::io::Write;
            sink.write_all(&bytes)?;
            sink.sync_all()?;
            ipc.push(FileReceipt {
                file,
                sha256: hash(&bytes),
                rows: chunk.len(),
            });
            if fail {
                return Err("injected read-only synthetic publication interruption".into());
            }
        }
        output.complete()?;
        let replication = Replication {
            index,
            seed,
            directory: directory.clone(),
            rows: rows.len(),
            canonical_sha256: hash(&serde_json::to_vec(&rows)?),
            model_contract_sha256: MODEL_CONTRACT_SHA256.into(),
        };
        let manifest = ReplicationManifest {
            schema_version: 1,
            input: input.clone(),
            effective_input_sha256: input.hash()?,
            effective_input_json: input.effective_json()?,
            runtime_revision: RUNTIME_REVISION.into(),
            data_revision: DATA_REVISION.into(),
            tick_unit: "second".into(),
            replication: replication.clone(),
            ipc,
            completed: true,
        };
        let path = dir.join("birthing_manifest.json");
        experiment::atomic_write_file(&path, &serde_json::to_vec_pretty(&manifest)?)?;
        return Ok(replication);
    }
    Err("bounded publication attempts exhausted".into())
}

fn safe_component(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        && value != "."
        && value != ".."
}
pub fn validate_replication(
    root: &Path,
    directory: &str,
    input: &BatchInput,
    index: usize,
    seed: u64,
) -> Result<ReplicationManifest> {
    if !safe_component(directory)
        || !directory.starts_with(&format!("{}-attempt", input.batch_id))
        || !directory.ends_with(&format!("-rep{index}"))
    {
        return Err("unbound replication directory".into());
    }
    let dir = root.join(directory);
    if fs::symlink_metadata(&dir)?.file_type().is_symlink() {
        return Err("symlink replication directory".into());
    }
    let manifest: ReplicationManifest =
        serde_json::from_slice(&read_bounded(&dir.join("birthing_manifest.json"), 262144)?)?;
    if manifest.schema_version != 1
        || !manifest.completed
        || manifest.input != *input
        || manifest.effective_input_sha256 != input.hash()?
        || manifest.effective_input_json != input.effective_json()?
        || manifest.replication.model_contract_sha256 != MODEL_CONTRACT_SHA256
        || hash(manifest.effective_input_json.as_bytes()) != manifest.effective_input_sha256
        || manifest.runtime_revision != RUNTIME_REVISION
        || manifest.data_revision != DATA_REVISION
        || manifest.tick_unit != "second"
        || manifest.replication.index != index
        || manifest.replication.seed != seed
        || manifest.replication.directory != directory
        || manifest.replication.rows == 0
        || manifest.replication.rows > MAX_ROWS
        || manifest.ipc.is_empty()
        || manifest.ipc.len() > MAX_ROWS / PART_ROWS
    {
        return Err("unbound or incomplete replication manifest".into());
    }
    let declared: run_output::RunManifest =
        serde_json::from_slice(&read_bounded(&dir.join("run_manifest.json"), 262144)?)?;
    if declared.parts.len() != manifest.ipc.len() {
        return Err("unbounded or unmatched Parquet parts".into());
    }
    for (i, part) in declared.parts.iter().enumerate() {
        if part.file_name != format!("part-{:05}.parquet", i + 1)
            || part.row_count == 0
            || part.row_count > PART_ROWS
        {
            return Err("invalid bounded Parquet declaration".into());
        }
        read_bounded(&dir.join(&part.file_name), 8 * 1024 * 1024)?;
    }
    let parquet = run_output::validate_run_output(&dir)?;
    if parquet.run_id != directory.strip_suffix(&format!("-rep{index}")).unwrap()
        || parquet.replication_id != index as u64
        || parquet.seed != seed
        || parquet.effective_input_hash != input.hash()?
        || parquet.time_scale != "second"
        || parquet.parts.len() != manifest.ipc.len()
        || manifest.ipc.is_empty()
    {
        return Err("upstream/domain manifest identity mismatch".into());
    }
    let mut all = Vec::new();
    for (part, receipt) in manifest.ipc.iter().enumerate() {
        if receipt.file != format!("ipc-{part:05}.arrow")
            || receipt.rows == 0
            || receipt.rows > PART_ROWS
        {
            return Err("invalid IPC part declaration".into());
        }
        let bytes = read_bounded(&dir.join(&receipt.file), 8 * 1024 * 1024)?;
        if hash(&bytes) != receipt.sha256 {
            return Err("IPC content hash mismatch".into());
        }
        let frame = io::read_ipc_stream(std::io::Cursor::new(bytes))?;
        let mut ipc_rows = io::dataframe_to_records(&frame)?;
        let other = io::read_parquet(std::io::Cursor::new(read_bounded(
            &dir.join(&parquet.parts[part].file_name),
            8 * 1024 * 1024,
        )?))?;
        let mut parquet_rows = io::dataframe_to_records(&other)?;
        ipc_rows.sort_by_key(|r| r.sequence);
        parquet_rows.sort_by_key(|r| r.sequence);
        if ipc_rows != parquet_rows || ipc_rows.len() != receipt.rows {
            return Err("IPC/Parquet capture mismatch".into());
        }
        all.extend(ipc_rows);
    }
    all.sort_by_key(|r| r.sequence);
    let mut rows = Vec::new();
    for (i, record) in all.iter().enumerate() {
        let row: Row = serde_json::from_str(
            record
                .payload_ref
                .as_deref()
                .ok_or("missing domain capture")?,
        )?;
        if record.sequence != (i + 1) as u64
            || record.schema_version != SCHEMA_VERSION_V2
            || record.time_scale != "ticks"
            || record.status != EventStatus::Dispatched
            || record.priority != 0
            || record.event_id.index != record.sequence
            || record.event_id.generation != 0
            || record.run_id != parquet.run_id
            || record.time_ticks != u128::from(row.tick)
            || record.event_kind != row.kind
            || record.entity_id
                != (row.person != 0).then_some(EntityId {
                    index: row.person,
                    generation: 0,
                })
        {
            return Err("invalid canonical capture identity".into());
        }
        rows.push(row);
    }
    if rows.len() != manifest.replication.rows
        || hash(&serde_json::to_vec(&rows)?) != manifest.replication.canonical_sha256
    {
        return Err("canonical capture digest mismatch".into());
    }
    Ok(manifest)
}

fn execute_replication(
    root: &Path,
    input: &BatchInput,
    index: usize,
    seed: u64,
    fail: bool,
) -> std::result::Result<Replication, ExperimentError> {
    publish(root, input, index, seed, fail)
        .map_err(|e| ExperimentError::new(ExperimentErrorKind::ExecutionFailed, e.to_string()))
}
pub fn run_batch(
    root: &Path,
    input: &BatchInput,
    workers: usize,
    resume: bool,
    fail_replication: Option<usize>,
) -> Result<Vec<Replication>> {
    input.validate()?;
    if !(1..=4).contains(&workers) {
        return Err("workers must be1–4".into());
    }
    // Configure once, before native pool initialization; replication workers
    // remain independent of this one-thread native data pool.
    static NATIVE_INIT: std::sync::OnceLock<std::result::Result<(), String>> =
        std::sync::OnceLock::new();
    NATIVE_INIT
        .get_or_init(|| {
            let budget = ThreadBudget {
                worker_threads: NonZeroUsize::new(1).unwrap(),
            };
            match run_output::native_thread_budget().map_err(|e| e.to_string())? {
                Some(existing) if existing == budget => Ok(()),
                Some(_) => Err("conflicting native thread budget".into()),
                None => {
                    run_output::configure_native_thread_budget(budget).map_err(|e| e.to_string())
                }
            }
        })
        .clone()?;
    if !root.is_dir() || fs::symlink_metadata(root)?.file_type().is_symlink() {
        return Err("existing private output root required".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(root)?.permissions().mode() & 0o077 != 0 {
            return Err("output root must be private to its owner".into());
        }
    }
    let batch_root = root.join(&input.batch_id);
    if resume && fs::symlink_metadata(&batch_root)?.file_type().is_symlink() {
        return Err("symlink batch directory".into());
    }
    let runner = |index, seed, metadata: &BatchMetadata| {
        execute_replication(
            &batch_root,
            &metadata.birthing_input,
            index,
            seed,
            fail_replication == Some(index),
        )
    };
    let results = if resume {
        let checkpoint_path = batch_root.join("checkpoint.json");
        let checkpoint: GenericBatchCheckpoint<Replication, BatchMetadata> =
            serde_json::from_slice(&read_bounded(&checkpoint_path, 2 * 1024 * 1024)?)?;
        if checkpoint.metadata.birthing_input != *input
            || checkpoint.metadata.model_contract_sha256 != MODEL_CONTRACT_SHA256
            || checkpoint.batch_id != input.batch_id
            || checkpoint.base_seed != input.seed
            || checkpoint.total_replications != input.replications
        {
            return Err("checkpoint does not bind requested effective input".into());
        }
        for receipt in &checkpoint.completed_replications {
            let verified = validate_replication(
                &batch_root,
                &receipt.directory,
                input,
                receipt.index,
                receipt.seed,
            )?;
            if verified.replication != *receipt {
                return Err("checkpoint result does not bind actual capture".into());
            }
        }
        experiment::resume_generic_batch::<Replication, BatchMetadata, _>(
            &checkpoint_path,
            workers,
            &batch_root,
            None,
            None,
            runner,
        )?
        .1
    } else {
        fs::create_dir(&batch_root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&batch_root, fs::Permissions::from_mode(0o700))?;
        }
        experiment::run_generic_batch(
            &input.batch_id,
            input.seed,
            input.replications,
            workers,
            Some(1),
            BatchMetadata {
                birthing_input: input.clone(),
                model_contract_sha256: MODEL_CONTRACT_SHA256.into(),
            },
            Some(&batch_root),
            None,
            None,
            runner,
        )?
    };
    for r in &results {
        validate_replication(&batch_root, &r.directory, input, r.index, r.seed)?;
    }
    let completed = batch_root.join("batch_manifest.json");
    if completed.exists() {
        let existing: Vec<Replication> =
            serde_json::from_slice(&read_bounded(&completed, 262144)?)?;
        if existing != results {
            return Err("existing batch completion differs from verified results".into());
        }
    } else {
        experiment::atomic_write_file(&completed, &serde_json::to_vec_pretty(&results)?)?;
    }
    Ok(results)
}

pub fn batch_root(root: &Path, input: &BatchInput) -> PathBuf {
    root.join(&input.batch_id)
}
