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
use polars::prelude::{DataFrame, IpcStreamReader, ParquetReader, SerReader, SortMultipleOptions};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

pub mod model;
pub mod theatre_tables;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub const DATA_REVISION: &str = "473ff2c0e640a7e457f500593f6ea1f06602ab35";
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
/// Reporting scope for actual observations, not a population or pooled-case
/// estimator. Replications may support a separately declared future estimand.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationUncertainty {
    pub schema: String,
    pub method: String,
    pub reason: String,
    pub observation_unit: String,
    pub experiment_unit: String,
    pub metrics: Vec<String>,
}
impl ObservationUncertainty {
    fn observations_only() -> Self {
        Self {
            schema: "careops.birthing.observation-uncertainty.v1".into(),
            method: "not_estimated".into(),
            reason: "case_observations_no_declared_sampling_estimand".into(),
            observation_unit: "synthetic_case".into(),
            experiment_unit: "independent_replication".into(),
            metrics: vec![
                "delay_seconds".into(),
                "origin_occupied_seconds".into(),
                "target_breach".into(),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplicationManifest {
    pub schema_version: u32,
    pub uncertainty: ObservationUncertainty,
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
                time_scale: "second".into(),
                priority: 0,
                sequence: (offset + i + 1) as u64,
                event_kind: row.kind.clone(),
                status: EventStatus::Dispatched,
                payload_ref: Some(serde_json::to_string(row)?),
            })
        })
        .collect()
}

fn capture_metadata(
    input: &BatchInput,
    run: &str,
    index: usize,
    seed: u64,
    first_sequence: usize,
    rows: usize,
) -> Result<BTreeMap<String, String>> {
    Ok([
        ("schema_version", SCHEMA_VERSION_V2.to_string()),
        ("run_id", run.to_string()),
        ("effective_input_sha256", input.hash()?),
        ("model_contract_sha256", MODEL_CONTRACT_SHA256.to_string()),
        ("runtime_revision", RUNTIME_REVISION.to_string()),
        ("data_revision", DATA_REVISION.to_string()),
        ("time_unit", "second".to_string()),
        ("replication_index", index.to_string()),
        ("seed", seed.to_string()),
        ("first_sequence", first_sequence.to_string()),
        ("rows", rows.to_string()),
    ]
    .into_iter()
    .map(|(k, v)| (format!("careops.{k}"), v))
    .collect())
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
            let metadata =
                capture_metadata(input, &run, index, seed, part * PART_ROWS + 1, chunk.len())?;
            output.write_batch_with_metadata(&capture, &metadata)?;
            let mut frame = io::records_to_dataframe(&capture)?;
            let mut bytes = Vec::new();
            io::write_ipc_stream_with_metadata(&mut frame, &mut bytes, &metadata)?;
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
            schema_version: 2,
            uncertainty: ObservationUncertainty::observations_only(),
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
// All package entry points share this barrier before any Polars operation.
// This coordinates the package's own callers; applications embedding other
// Polars users must still initialize the shared runtime before concurrent use.
fn initialize_native_data() -> Result<()> {
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
    Ok(())
}

fn canonical_event_frame(frame: DataFrame) -> Result<DataFrame> {
    initialize_native_data()?;
    let expected = io::records_to_dataframe(&[])?;
    if frame.schema() != expected.schema() || frame.height() > MAX_ROWS {
        return Err("unsupported native event schema or row bound".into());
    }
    for column in frame.columns() {
        if !matches!(column.name().as_str(), "entity_id_index" | "entity_id_gen")
            && column.null_count() != 0
        {
            return Err("null required B7 event field".into());
        }
    }
    Ok(frame.sort(["sequence"], SortMultipleOptions::default())?)
}

pub fn validate_replication(
    root: &Path,
    directory: &str,
    input: &BatchInput,
    index: usize,
    seed: u64,
) -> Result<ReplicationManifest> {
    replication_events(root, directory, input, index, seed).map(|(manifest, _)| manifest)
}

/// Read original native capture with exact manifest/model/input binding. Returned
/// buffers are owned by the offline table; the simulation is never invoked.
pub fn replication_events(
    root: &Path,
    directory: &str,
    input: &BatchInput,
    index: usize,
    seed: u64,
) -> Result<(ReplicationManifest, DataFrame)> {
    let capture = checked_replication_capture(root, directory, input, index, seed)?;
    Ok((capture.manifest, capture.events))
}

// Internal shared validation result: canonical Row decoding is retained once
// for domain identity/digest control. Offline views reuse these already checked
// original sequence/Row pairs instead of reparsing payload JSON a second time.
pub(crate) struct CheckedCapture {
    pub(crate) manifest: ReplicationManifest,
    pub(crate) events: DataFrame,
    pub(crate) run_id: String,
    pub(crate) rows: Vec<(u64, Row)>,
}
pub(crate) fn checked_replication_capture(
    root: &Path,
    directory: &str,
    input: &BatchInput,
    index: usize,
    seed: u64,
) -> Result<CheckedCapture> {
    initialize_native_data()?;
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
    if manifest.schema_version != 2
        || manifest.uncertainty != ObservationUncertainty::observations_only()
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
    let mut all = io::records_to_dataframe(&[])?;
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
        let expected = capture_metadata(
            input,
            &parquet.run_id,
            index,
            seed,
            all.height() + 1,
            receipt.rows,
        )?;
        let mut ipc_reader = IpcStreamReader::new(std::io::Cursor::new(bytes));
        let metadata = ipc_reader
            .custom_metadata()?
            .ok_or("missing IPC capture metadata")?;
        let actual: BTreeMap<String, String> = metadata
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        if actual != expected {
            return Err("IPC capture metadata mismatch".into());
        }
        let frame = canonical_event_frame(ipc_reader.finish()?)?;
        let mut parquet_reader = ParquetReader::new(std::io::Cursor::new(read_bounded(
            &dir.join(&parquet.parts[part].file_name),
            8 * 1024 * 1024,
        )?));
        let metadata = parquet_reader.get_metadata()?;
        let mut actual = BTreeMap::new();
        for entry in metadata.key_value_metadata.iter().flatten() {
            if entry.key == "ARROW:schema" {
                continue;
            }
            if actual
                .insert(
                    entry.key.clone(),
                    entry
                        .value
                        .clone()
                        .ok_or("missing Parquet metadata value")?,
                )
                .is_some()
            {
                return Err("duplicate Parquet metadata key".into());
            }
        }
        if actual != expected {
            return Err("Parquet capture metadata mismatch".into());
        }
        let other = canonical_event_frame(parquet_reader.finish()?)?;
        if !frame.equals_missing(&other)
            || frame.height() != receipt.rows
            || other.height() != parquet.parts[part].row_count
        {
            return Err("IPC/Parquet capture mismatch".into());
        }
        let sequences = frame.column("sequence")?.u64()?;
        for offset in 0..frame.height() {
            if sequences.get(offset) != Some((all.height() + offset + 1) as u64) {
                return Err("native part sequence metadata mismatch".into());
            }
        }
        if all
            .height()
            .checked_add(frame.height())
            .is_none_or(|n| n > MAX_ROWS)
        {
            return Err("capture row bound exceeded".into());
        }
        all.vstack_mut(&frame)?;
    }
    let all = canonical_event_frame(all)?;
    // Domain identity and retained canonical digest are control validation, not
    // tabular transformations. Actual sort/format equality stay in Polars.
    let records = io::dataframe_to_records(&all)?;
    let mut rows = Vec::with_capacity(records.len());
    for (i, record) in records.iter().enumerate() {
        let payload = record
            .payload_ref
            .as_deref()
            .ok_or("missing domain capture")?;
        if payload.len() > 8 * 1024 * 1024 {
            return Err("captured domain payload exceeds bound".into());
        }
        let row: Row = serde_json::from_str(payload)?;
        if record.sequence != (i + 1) as u64
            || record.schema_version != SCHEMA_VERSION_V2
            || record.time_scale != "second"
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
    let rows = records
        .into_iter()
        .map(|record| record.sequence)
        .zip(rows)
        .collect();
    Ok(CheckedCapture {
        manifest,
        events: all,
        run_id: parquet.run_id,
        rows,
    })
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
    initialize_native_data()?;
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

#[cfg(test)]
mod native_identity_vectors {
    use super::*;
    #[test]
    fn generic_entity_zero_stays_distinct_from_absence_and_domain_person_sentinel() {
        initialize_native_data().unwrap();
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../careops-ed/tests/fixtures/r121_analytical.json"
        ))
        .unwrap();
        let records = fixture["birthing"]["nullable_entities"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, entity)| EventLogRecord {
                schema_version: SCHEMA_VERSION_V2,
                run_id: "r".into(),
                event_id: EventId {
                    index: i as u64 + 1,
                    generation: 0,
                },
                entity_id: if entity.is_null() {
                    None
                } else {
                    Some(EntityId {
                        index: entity["index"].as_u64().unwrap(),
                        generation: u32::try_from(entity["generation"].as_u64().unwrap()).unwrap(),
                    })
                },
                time_ticks: 0,
                time_scale: "second".into(),
                priority: 0,
                sequence: i as u64 + 1,
                event_kind: "control".into(),
                status: EventStatus::Dispatched,
                payload_ref: Some("{}".into()),
            })
            .collect::<Vec<_>>();
        let frame =
            canonical_event_frame(io::records_to_dataframe(&records).unwrap().reverse()).unwrap();
        assert_eq!(
            frame
                .column("entity_id_index")
                .unwrap()
                .u64()
                .unwrap()
                .get(0),
            None
        );
        assert_eq!(
            frame
                .column("entity_id_index")
                .unwrap()
                .u64()
                .unwrap()
                .get(1),
            Some(0)
        );
        assert_eq!(
            frame.column("entity_id_gen").unwrap().u32().unwrap().get(1),
            Some(0)
        );
        // B7's domain PersonId format is nonzero; its explicit zero sentinel is
        // not a generic ECS EntityId(0,0) and must never become a mother/infant.
        let domain = Row {
            tick: 0,
            person: 0,
            episode: 0,
            kind: "control".into(),
            data: serde_json::Value::Null,
        };
        assert!(super::records("r", &[domain], 0).unwrap()[0]
            .entity_id
            .is_none());
    }
}
