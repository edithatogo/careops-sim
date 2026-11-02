//! Opt-in original-result capture boundaries for R11.1.
//!
//! No default codec/store is supplied. Rows are produced after the scheduler
//! finishes; the legacy model and RunSummary allocations remain. A synchronous
//! store acknowledgment bounds pending capture batches, not total simulation RSS.
//! Recovery is only for completed replications, never a simulation-state resume.
//! Per-worker pending rows and their patient IDs are capped separately from encoded
//! bytes. One lookahead row and bounded readback bytes/decoded rows may coexist.
//! Codec encode/decode scratch (especially decompression), the model, legacy
//! summary and accumulated checkpoint metadata require separate qualification.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

use careops_core::experiment::{self, ExperimentError, ExperimentErrorKind, IndexedReplication};
use serde::{Deserialize, Serialize};

use crate::disposition::api_runner::compute_sha256_hex;
use crate::disposition::batch_runner::{BatchConfig, ReplicationResult};
use crate::{PatientResult, ScenarioConfig};

pub type Result<T> = std::result::Result<T, ExperimentError>;

fn invalid(message: impl Into<String>) -> ExperimentError {
    ExperimentError::new(ExperimentErrorKind::MalformedConfig, message)
}
fn output(error: impl std::fmt::Display) -> ExperimentError {
    ExperimentError::new(ExperimentErrorKind::OutputWriteFailed, error.to_string())
}

/// Caps serialized part bytes DURING encoding, including framing/metadata.
/// A codec must use this boundary rather than allocate then inspect an unbounded Vec.
pub struct BoundedPartWriter<W> {
    inner: W,
    limit: usize,
    written: usize,
    exceeded: bool,
}
impl<W> BoundedPartWriter<W> {
    pub fn new(inner: W, limit: usize) -> Self {
        Self {
            inner,
            limit,
            written: 0,
            exceeded: false,
        }
    }
    pub fn written(&self) -> usize {
        self.written
    }
    pub fn into_inner(self) -> W {
        self.inner
    }
}
impl<W: Write> Write for BoundedPartWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.written) {
            self.exceeded = true;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "capture byte limit exceeded",
            ));
        }
        let count = self.inner.write(bytes)?;
        self.written += count;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CaptureLimits {
    pub max_rows: usize,
    pub max_bytes: usize,
    /// Independent logical row-memory bound; not an encoded-size estimate.
    pub max_patient_id_bytes: usize,
}
impl CaptureLimits {
    fn validate(self) -> Result<()> {
        if self.max_rows == 0 || self.max_bytes == 0 || self.max_patient_id_bytes == 0 {
            return Err(invalid("capture limits must be positive"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CaptureBatch {
    pub schema: String,
    pub run_id: String,
    pub input_sha256: String,
    pub model_contract_sha256: String,
    pub time_unit: String,
    pub replication_index: usize,
    pub seed: u64,
    pub first_sequence: usize,
    pub rows: Vec<PatientResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PartRef {
    pub name: String,
    pub first_sequence: usize,
    pub rows: usize,
    pub bytes: usize,
    pub sha256: String,
}

/// ED adapter boundary, not a new generic storage engine.
/// Implementations must be exclusive to this run, reject collisions/symlinks,
/// contain paths, bound reads/encoding, and return only after committing a part.
/// No hidden queue is permitted: write_part's return is the producer's ack.
/// Codec and physical publication/crash guarantees require separate qualification.
pub trait PartStore: Sync {
    /// Identifies codec/schema configuration for checkpoint binding.
    fn codec_id(&self) -> &str;
    fn begin_replication(&self, _index: usize) -> Result<()> {
        Ok(())
    }
    /// Pure deterministic streaming encoder, used for sizing and actual writing.
    /// Must not buffer an unbounded encoded part or hide output outside writer.
    fn encode_part(&self, batch: &CaptureBatch, writer: &mut dyn Write) -> Result<()>;
    fn write_part(&self, name: &str, batch: &CaptureBatch, max_bytes: usize) -> Result<()>;
    fn read_part(&self, name: &str, max_bytes: usize) -> Result<Vec<u8>>;
    /// Decoder must enforce its own allocation/decompression limits. Encoded
    /// byte limits alone do not bound decoder scratch memory.
    fn decode_part(&self, bytes: &[u8]) -> Result<CaptureBatch>;
    fn part_names(&self) -> Result<Vec<String>>;
    fn is_complete(&self) -> Result<bool>;
    fn complete(&self, replications: &[CapturedReplication]) -> Result<()>;
    /// Acknowledgment follows file sync, atomic replacement and directory sync.
    fn commit_checkpoint(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        durable_checkpoint(path, bytes)
    }
    /// Only explicit recovery may preserve/archive unacknowledged pending parts.
    fn prepare_recovery(&self, _pending: &[usize]) -> Result<()> {
        Ok(())
    }
    /// Stores must independently verify an already-published completion manifest.
    fn verify_complete(&self, _replications: &[CapturedReplication]) -> Result<()> {
        Err(invalid("completed store recovery verification unsupported"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CapturedReplication {
    pub model_contract_sha256: String,
    pub summary: ReplicationResult,
    pub parts: Vec<PartRef>,
}
impl IndexedReplication for CapturedReplication {
    fn replication_index(&self) -> usize {
        self.summary.replication_index
    }
    fn seed(&self) -> u64 {
        self.summary.seed
    }
    fn primary_metric(&self) -> f64 {
        self.summary.mean_wait_ticks
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CaptureMetadata {
    pub capture_schema: String,
    pub model_contract_sha256: String,
    pub codec_id: String,
    pub effective_input_sha256: String,
    pub parsed_input_sha256: String,
    pub config: BatchConfig,
    pub limits: CaptureLimits,
}
pub type CaptureCheckpoint =
    experiment::GenericBatchCheckpoint<CapturedReplication, CaptureMetadata>;
pub const SCHEMA: &str = "careops.ed.capture-boundary.v2";
pub const MODEL_CONTRACT_SHA256: &str = env!("CAREOPS_ED_MODEL_CONTRACT_SHA256");

pub(crate) fn part_name(index: usize, sequence: usize) -> String {
    format!("rep-{index:08}-seq-{sequence:016}.part")
}
pub(crate) fn validate_config(config: &BatchConfig) -> Result<()> {
    if config.schema_version != 1 {
        return Err(invalid("unsupported batch config schema version"));
    }
    Ok(())
}

pub(crate) fn metadata(
    scenario: &ScenarioConfig,
    config: &BatchConfig,
    limits: CaptureLimits,
    store: &dyn PartStore,
) -> Result<CaptureMetadata> {
    validate_config(config)?;
    limits.validate()?;
    if config.replications == 0 {
        return Err(invalid("replications must be positive"));
    }
    let mut effective = scenario.clone();
    effective.horizon_ticks = config.horizon_override.unwrap_or(scenario.horizon_ticks);
    effective.seed = config.base_seed;
    Ok(CaptureMetadata {
        capture_schema: SCHEMA.into(),
        model_contract_sha256: MODEL_CONTRACT_SHA256.into(),
        codec_id: store.codec_id().into(),
        effective_input_sha256: compute_sha256_hex(
            &serde_json::to_vec(&effective).map_err(invalid_json)?,
        ),
        parsed_input_sha256: compute_sha256_hex(
            &serde_json::to_vec(scenario).map_err(invalid_json)?,
        ),
        config: config.clone(),
        limits,
    })
}
fn invalid_json(error: serde_json::Error) -> ExperimentError {
    invalid(error.to_string())
}

fn fits_part(store: &dyn PartStore, batch: &CaptureBatch, limit: usize) -> Result<bool> {
    let mut counter = BoundedPartWriter::new(io::sink(), limit);
    let result = store.encode_part(batch, &mut counter);
    if counter.exceeded {
        return Ok(false);
    }
    result?;
    Ok(true)
}

struct Capture<'a> {
    store: &'a dyn PartStore,
    limits: CaptureLimits,
    batch: CaptureBatch,
    parts: Vec<PartRef>,
}
impl Capture<'_> {
    fn flush(&mut self) -> Result<()> {
        if self.batch.rows.is_empty() {
            return Ok(());
        }
        let name = part_name(self.batch.replication_index, self.batch.first_sequence);
        self.store
            .write_part(&name, &self.batch, self.limits.max_bytes)?;
        let bytes = self.store.read_part(&name, self.limits.max_bytes)?;
        if bytes.len() > self.limits.max_bytes || self.store.decode_part(&bytes)? != self.batch {
            return Err(invalid("committed capture differs from original rows"));
        }
        self.parts.push(PartRef {
            name,
            first_sequence: self.batch.first_sequence,
            rows: self.batch.rows.len(),
            bytes: bytes.len(),
            sha256: compute_sha256_hex(&bytes),
        });
        self.batch.first_sequence = self
            .batch
            .first_sequence
            .checked_add(self.batch.rows.len())
            .ok_or_else(|| invalid("capture sequence overflow"))?;
        self.batch.rows.clear();
        Ok(())
    }
    fn push(&mut self, row: &PatientResult) -> Result<()> {
        // Reject an oversized row before cloning it into the capture buffer.
        if row.patient_id.len() > self.limits.max_patient_id_bytes {
            return Err(invalid("capture patient ID memory limit exceeded"));
        }
        if self.batch.rows.len() == self.limits.max_rows {
            self.flush()?;
        }
        self.batch.rows.push(row.clone());
        if !fits_part(self.store, &self.batch, self.limits.max_bytes)? {
            let lookahead = self.batch.rows.pop().unwrap();
            self.flush()?;
            self.batch.rows.push(lookahead);
            if !fits_part(self.store, &self.batch, self.limits.max_bytes)? {
                return Err(invalid("single capture row exceeds encoded byte limit"));
            }
        }
        Ok(())
    }
}

fn execute(
    scenario: &ScenarioConfig,
    index: usize,
    seed: u64,
    meta: &CaptureMetadata,
    store: &dyn PartStore,
) -> Result<CapturedReplication> {
    store.begin_replication(index)?;
    let mut input = scenario.clone();
    input.seed = seed;
    if let Some(horizon) = meta.config.horizon_override {
        input.horizon_ticks = horizon;
    }
    let mut capture = Capture {
        store,
        limits: meta.limits,
        batch: CaptureBatch {
            schema: SCHEMA.into(),
            run_id: meta.config.batch_id.clone(),
            input_sha256: meta.effective_input_sha256.clone(),
            model_contract_sha256: meta.model_contract_sha256.clone(),
            time_unit: scenario.time_unit.clone(),
            replication_index: index,
            seed,
            first_sequence: 0,
            rows: vec![],
        },
        parts: vec![],
    };
    let mut capture_error = None;
    let summary = crate::run_scenario_with_capture(&input, &mut |row| {
        capture.push(row).map_err(|error| {
            let message = error.to_string();
            capture_error = Some(error);
            io::Error::other(message)
        })
    });
    if let Some(error) = capture_error {
        return Err(error);
    }
    let summary = summary
        .map_err(|e| ExperimentError::new(ExperimentErrorKind::ExecutionFailed, e.to_string()))?;
    capture.flush()?;
    let waits: Vec<u64> = summary
        .patients
        .iter()
        .filter_map(|p| p.wait_ticks)
        .collect();
    let wait_sum = waits.iter().try_fold(0u64, |total, wait| {
        total
            .checked_add(*wait)
            .ok_or_else(|| invalid("capture wait sum overflow"))
    })?;
    Ok(CapturedReplication {
        model_contract_sha256: meta.model_contract_sha256.clone(),
        summary: ReplicationResult {
            replication_index: index,
            seed,
            arrivals: summary.arrivals,
            started: summary.started,
            completed: summary.completed,
            unfinished: summary.unfinished,
            mean_wait_ticks: if waits.is_empty() {
                0.0
            } else {
                wait_sum as f64 / waits.len() as f64
            },
            horizon_ticks: summary.horizon_ticks,
        },
        parts: capture.parts,
    })
}

pub(crate) fn validate_parts(
    results: &[CapturedReplication],
    meta: &CaptureMetadata,
    scenario: &ScenarioConfig,
    store: &dyn PartStore,
) -> Result<()> {
    validate_parts_pending(results, meta, scenario, store, &BTreeSet::new())
}

fn validate_parts_pending(
    results: &[CapturedReplication],
    meta: &CaptureMetadata,
    scenario: &ScenarioConfig,
    store: &dyn PartStore,
    pending: &BTreeSet<usize>,
) -> Result<()> {
    let limits = meta.limits;
    let expected_patients: BTreeMap<_, _> = scenario
        .patients
        .iter()
        .map(|p| (p.patient_id.clone(), p))
        .collect();
    let expected_ids: BTreeSet<_> = expected_patients.keys().cloned().collect();
    let seeds =
        experiment::allocate_replication_seeds(meta.config.base_seed, meta.config.replications);
    let horizon = meta
        .config
        .horizon_override
        .unwrap_or(scenario.horizon_ticks);
    let mut replications = BTreeSet::new();
    let mut names = BTreeSet::new();
    for result in results {
        let index = result.summary.replication_index;
        if result.model_contract_sha256 != meta.model_contract_sha256
            || seeds.get(index) != Some(&result.summary.seed)
            || !replications.insert(index)
        {
            return Err(invalid("capture replication identity mismatch"));
        }
        let mut started = 0usize;
        let mut completed = 0usize;
        let mut wait_sum = 0u64;
        let mut next = 0usize;
        let mut patients = BTreeSet::new();
        for part in &result.parts {
            if part.name != part_name(result.summary.replication_index, next)
                || part.first_sequence != next
                || part.rows == 0
                || part.rows > limits.max_rows
                || part.bytes > limits.max_bytes
                || !names.insert(part.name.clone())
            {
                return Err(invalid("duplicate, invalid or discontinuous capture part"));
            }
            let bytes = store.read_part(&part.name, limits.max_bytes)?;
            if bytes.len() != part.bytes || compute_sha256_hex(&bytes) != part.sha256 {
                return Err(invalid("capture part hash or length mismatch"));
            }
            let batch = store.decode_part(&bytes)?;
            if batch.time_unit != scenario.time_unit
                || batch.schema != SCHEMA
                || batch.run_id != meta.config.batch_id
                || batch.input_sha256 != meta.effective_input_sha256
                || batch.model_contract_sha256 != meta.model_contract_sha256
                || batch.replication_index != result.summary.replication_index
                || batch.seed != result.summary.seed
                || batch.first_sequence != next
                || batch.rows.len() != part.rows
            {
                return Err(invalid("capture part identity or row count mismatch"));
            }
            for row in &batch.rows {
                let expected = expected_patients
                    .get(&row.patient_id)
                    .ok_or_else(|| invalid("unknown captured patient"))?;
                if !patients.insert(row.patient_id.clone())
                    || row.patient_id.len() > limits.max_patient_id_bytes
                    || row.arrival_tick != expected.arrival_tick
                    || row.work_ticks != expected.work_ticks
                    || row
                        .start_tick
                        .is_some_and(|start| start < row.arrival_tick || start > horizon)
                    || row.wait_ticks
                        != row
                            .start_tick
                            .and_then(|start| start.checked_sub(row.arrival_tick))
                    || row.completion_tick.is_some_and(|end| {
                        row.start_tick.is_none_or(|start| end < start) || end > horizon
                    })
                    || row.elapsed_ticks
                        != row
                            .completion_tick
                            .and_then(|end| end.checked_sub(row.arrival_tick))
                    || (row.status == crate::PatientStatus::Completed)
                        != row.completion_tick.is_some()
                {
                    return Err(invalid("capture patient identity or timing mismatch"));
                }
                if row.start_tick.is_some() {
                    started += 1;
                }
                if row.completion_tick.is_some() {
                    completed += 1;
                }
                wait_sum = wait_sum
                    .checked_add(row.wait_ticks.unwrap_or(0))
                    .ok_or_else(|| invalid("capture wait sum overflow"))?;
            }
            next = next
                .checked_add(part.rows)
                .ok_or_else(|| invalid("capture sequence overflow"))?;
        }
        let mean_wait = if started == 0 {
            0.0
        } else {
            wait_sum as f64 / started as f64
        };
        if patients != expected_ids
            || next != result.summary.arrivals
            || started != result.summary.started
            || completed != result.summary.completed
            || next - completed != result.summary.unfinished
            || result.summary.mean_wait_ticks != mean_wait
            || result.summary.horizon_ticks != horizon
        {
            return Err(invalid("capture coverage or summary mismatch"));
        }
    }
    let actual: BTreeSet<_> = store.part_names()?.into_iter().collect();
    if !names.is_subset(&actual)
        || actual
            .difference(&names)
            .any(|name| parse_part_name(name).is_none_or(|(index, _)| !pending.contains(&index)))
    {
        return Err(invalid(
            "unreferenced capture parts require explicit recovery; no implicit replay",
        ));
    }
    Ok(())
}

pub(crate) fn read_checkpoint(path: &Path) -> Result<CaptureCheckpoint> {
    let file_type = fs::symlink_metadata(path).map_err(output)?.file_type();
    if !file_type.is_file() || file_type.is_symlink() {
        return Err(invalid("checkpoint must be a regular file"));
    }
    let mut bytes = vec![];
    std::fs::File::open(path)
        .map_err(output)?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(output)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(invalid("capture checkpoint too large"));
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(invalid_json)?;
    let allowed = [
        "schema_version",
        "batch_id",
        "base_seed",
        "total_replications",
        "completed_replications",
        "pending_indices",
        "status",
        "checkpoint_epoch_ms",
        "capture_schema",
        "model_contract_sha256",
        "codec_id",
        "effective_input_sha256",
        "parsed_input_sha256",
        "config",
        "limits",
    ];
    if value
        .as_object()
        .is_none_or(|object| object.keys().any(|key| !allowed.contains(&key.as_str())))
    {
        return Err(invalid("unknown capture control fields"));
    }
    serde_json::from_value(value).map_err(invalid_json)
}
pub(crate) fn check_binding(
    checkpoint: &CaptureCheckpoint,
    expected: &CaptureMetadata,
) -> Result<()> {
    validate_config(&expected.config)?;
    validate_config(&checkpoint.metadata.config)?;
    let mut config = checkpoint.metadata.config.clone();
    config.num_workers = expected.config.num_workers;
    // Validate the represented control envelope before allocating a seed vector
    // from untrusted persisted counts. File-size bounds alone do not bound N.
    if expected.config.replications == 0
        || checkpoint
            .completed_replications
            .len()
            .checked_add(checkpoint.pending_indices.len())
            != Some(expected.config.replications)
        || expected.model_contract_sha256.len() != 64
        || !expected
            .model_contract_sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || checkpoint.schema_version != 2
        || checkpoint.status
            != if checkpoint.pending_indices.is_empty() {
                "complete"
            } else {
                "in_progress"
            }
        || checkpoint.metadata.model_contract_sha256 != expected.model_contract_sha256
        || checkpoint.metadata.parsed_input_sha256 != expected.parsed_input_sha256
        || checkpoint.metadata.capture_schema != SCHEMA
        || checkpoint.metadata.codec_id != expected.codec_id
        || checkpoint.metadata.limits != expected.limits
        || checkpoint.metadata.effective_input_sha256 != expected.effective_input_sha256
        || config != expected.config
        || checkpoint.batch_id != expected.config.batch_id
        || checkpoint.base_seed != expected.config.base_seed
        || checkpoint.total_replications != expected.config.replications
    {
        return Err(invalid(
            "capture checkpoint input/config/model binding mismatch",
        ));
    }
    let mut indices = BTreeSet::new();
    let seeds = experiment::allocate_replication_seeds(
        expected.config.base_seed,
        expected.config.replications,
    );
    for replication in &checkpoint.completed_replications {
        let index = replication.summary.replication_index;
        if seeds.get(index) != Some(&replication.summary.seed) || !indices.insert(index) {
            return Err(invalid("invalid completed replication envelope"));
        }
    }
    for &index in &checkpoint.pending_indices {
        if index >= expected.config.replications || !indices.insert(index) {
            return Err(invalid("invalid pending replication envelope"));
        }
    }
    if indices.len() != expected.config.replications {
        return Err(invalid("incomplete capture control coverage"));
    }
    Ok(())
}

/// Run original simulations with opt-in capture. Checkpoints contain only fully
/// committed replications. Partial/orphan replication parts require explicit recovery.
pub fn run_captured_batch(
    scenario: &ScenarioConfig,
    config: &BatchConfig,
    limits: CaptureLimits,
    checkpoint_dir: &Path,
    store: &dyn PartStore,
) -> Result<Vec<CapturedReplication>> {
    run_captured_batch_controlled(scenario, config, limits, checkpoint_dir, store, None, None)
}

#[allow(clippy::too_many_arguments)]
pub fn run_captured_batch_controlled(
    scenario: &ScenarioConfig,
    config: &BatchConfig,
    limits: CaptureLimits,
    checkpoint_dir: &Path,
    store: &dyn PartStore,
    cancel: Option<&experiment::CancellationToken>,
    progress: Option<&(dyn Fn(experiment::BatchProgress) + Send + Sync)>,
) -> Result<Vec<CapturedReplication>> {
    let meta = metadata(scenario, config, limits, store)?;
    let path = checkpoint_dir.join("checkpoint.json");
    if path.exists() || store.is_complete()? || !store.part_names()?.is_empty() {
        return Err(invalid(
            "capture run already exists; use completed-replication recovery",
        ));
    }
    fs::create_dir_all(checkpoint_dir).map_err(output)?;
    let initial = make_checkpoint(meta.clone(), vec![]);
    commit(store, &path, &initial)?; // Before the first worker or part exists.
    let journal = Mutex::new(initial);
    let results = experiment::run_generic_batch(
        &config.batch_id,
        config.base_seed,
        config.replications,
        config.num_workers,
        None,
        meta.clone(),
        None,
        cancel,
        progress,
        |index, seed, m| {
            let result = execute(scenario, index, seed, m, store)?;
            acknowledge(store, &path, &journal, result.clone())?;
            Ok(result)
        },
    )?;
    let checkpoint = read_checkpoint(&path)?;
    check_binding(&checkpoint, &meta)?;
    if checkpoint.completed_replications != results || !checkpoint.pending_indices.is_empty() {
        return Err(invalid("capture checkpoint did not commit all results"));
    }
    validate_parts(&results, &meta, scenario, store)?;
    store.complete(&results)?;
    Ok(results)
}

pub fn resume_captured_batch(
    scenario: &ScenarioConfig,
    config: &BatchConfig,
    limits: CaptureLimits,
    checkpoint_dir: &Path,
    store: &dyn PartStore,
) -> Result<Vec<CapturedReplication>> {
    resume_captured_batch_controlled(scenario, config, limits, checkpoint_dir, store, None)
}

pub fn resume_captured_batch_controlled(
    scenario: &ScenarioConfig,
    config: &BatchConfig,
    limits: CaptureLimits,
    checkpoint_dir: &Path,
    store: &dyn PartStore,
    cancel: Option<&experiment::CancellationToken>,
) -> Result<Vec<CapturedReplication>> {
    let meta = metadata(scenario, config, limits, store)?;
    let path = checkpoint_dir.join("checkpoint.json");
    let checkpoint = read_checkpoint(&path)?;
    check_binding(&checkpoint, &meta)?;
    let pending: BTreeSet<_> = checkpoint.pending_indices.iter().copied().collect();
    validate_parts_pending(
        &checkpoint.completed_replications,
        &meta,
        scenario,
        store,
        &pending,
    )?;
    if store.is_complete()? {
        if !pending.is_empty() {
            return Err(invalid("manifest precedes complete checkpoint"));
        }
        store.verify_complete(&checkpoint.completed_replications)?;
        return Ok(checkpoint.completed_replications);
    }
    // Validate every acknowledged part and control binding BEFORE preserving pending attempts.
    store.prepare_recovery(&checkpoint.pending_indices)?;
    validate_parts(&checkpoint.completed_replications, &meta, scenario, store)?;
    let cached: BTreeMap<_, _> = checkpoint
        .completed_replications
        .iter()
        .map(|r| (r.summary.replication_index, r.clone()))
        .collect();
    let journal = Mutex::new(make_checkpoint(
        meta.clone(),
        checkpoint.completed_replications,
    ));
    let results = experiment::run_generic_batch(
        &config.batch_id,
        config.base_seed,
        config.replications,
        config.num_workers,
        None,
        meta.clone(),
        None,
        cancel,
        None,
        |index, seed, m| {
            if let Some(result) = cached.get(&index) {
                return Ok(result.clone());
            }
            let result = execute(scenario, index, seed, m, store)?;
            acknowledge(store, &path, &journal, result.clone())?;
            Ok(result)
        },
    )?;
    let checkpoint = read_checkpoint(&path)?;
    check_binding(&checkpoint, &meta)?;
    if checkpoint.completed_replications != results || !checkpoint.pending_indices.is_empty() {
        return Err(invalid("recovery checkpoint did not commit every result"));
    }
    validate_parts(&results, &meta, scenario, store)?;
    store.complete(&results)?;
    Ok(results)
}

/// Executes one partition using the canonical index-derived seed. The caller owns
/// a disjoint part namespace and must finalize the complete set before export.
pub fn capture_replication(
    scenario: &ScenarioConfig,
    config: &BatchConfig,
    limits: CaptureLimits,
    store: &dyn PartStore,
    index: usize,
) -> Result<CapturedReplication> {
    let meta = metadata(scenario, config, limits, store)?;
    if index >= config.replications || store.is_complete()? {
        return Err(invalid("invalid partition index or completed store"));
    }
    let seeds = experiment::allocate_replication_seeds(config.base_seed, config.replications);
    execute(scenario, index, seeds[index], &meta, store)
}

/// Validates the entire genuine partition set before checkpoint/manifest commit.
pub fn finalize_captured_replications(
    scenario: &ScenarioConfig,
    config: &BatchConfig,
    limits: CaptureLimits,
    checkpoint_dir: &Path,
    store: &dyn PartStore,
    mut results: Vec<CapturedReplication>,
) -> Result<Vec<CapturedReplication>> {
    if store.is_complete()? || results.len() != config.replications {
        return Err(invalid("incomplete partition set or completed store"));
    }
    let meta = metadata(scenario, config, limits, store)?;
    results.sort_by_key(|r| r.summary.replication_index);
    validate_parts(&results, &meta, scenario, store)?;
    let checkpoint = CaptureCheckpoint {
        schema_version: 2,
        batch_id: config.batch_id.clone(),
        base_seed: config.base_seed,
        total_replications: config.replications,
        metadata: meta,
        completed_replications: results.clone(),
        pending_indices: vec![],
        status: "complete".into(),
        checkpoint_epoch_ms: experiment::current_epoch_ms(),
    };
    commit(store, &checkpoint_dir.join("checkpoint.json"), &checkpoint)?;
    store.complete(&results)?;
    Ok(results)
}

/// Canonical namespaces make pending-part recovery explicit and non-destructive.
pub(crate) fn parse_part_name(name: &str) -> Option<(usize, usize)> {
    let (index, sequence) = name
        .strip_prefix("rep-")?
        .strip_suffix(".part")?
        .split_once("-seq-")?;
    let pair = (index.parse().ok()?, sequence.parse().ok()?);
    (part_name(pair.0, pair.1) == name).then_some(pair)
}

fn make_checkpoint(
    meta: CaptureMetadata,
    mut results: Vec<CapturedReplication>,
) -> CaptureCheckpoint {
    results.sort_by_key(|r| r.summary.replication_index);
    let done: BTreeSet<_> = results
        .iter()
        .map(|r| r.summary.replication_index)
        .collect();
    let pending: Vec<_> = (0..meta.config.replications)
        .filter(|i| !done.contains(i))
        .collect();
    CaptureCheckpoint {
        schema_version: 2,
        batch_id: meta.config.batch_id.clone(),
        base_seed: meta.config.base_seed,
        total_replications: meta.config.replications,
        completed_replications: results,
        status: if pending.is_empty() {
            "complete"
        } else {
            "in_progress"
        }
        .into(),
        pending_indices: pending,
        checkpoint_epoch_ms: experiment::current_epoch_ms(),
        metadata: meta,
    }
}
fn commit(store: &dyn PartStore, path: &Path, checkpoint: &CaptureCheckpoint) -> Result<()> {
    let bytes = serde_json::to_vec(checkpoint).map_err(invalid_json)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(invalid("checkpoint exceeds read bound"));
    }
    store.commit_checkpoint(path, &bytes)
}
fn acknowledge(
    store: &dyn PartStore,
    path: &Path,
    journal: &Mutex<CaptureCheckpoint>,
    result: CapturedReplication,
) -> Result<()> {
    let mut guard = journal.lock().map_err(output)?;
    let mut results = guard.completed_replications.clone();
    if results
        .iter()
        .any(|r| r.summary.replication_index == result.summary.replication_index)
    {
        return Err(invalid("duplicate replication acknowledgment"));
    }
    results.push(result);
    let next = make_checkpoint(guard.metadata.clone(), results);
    commit(store, path, &next)?;
    *guard = next;
    Ok(())
}
/// Local-filesystem process-crash boundary; not a hardware power-loss guarantee.
pub fn durable_checkpoint(path: &Path, bytes: &[u8]) -> Result<()> {
    durable_checkpoint_with_sync(path, bytes, &File::sync_all)
}

/// Narrow storage-fault seam: production uses `File::sync_all` for both stages.
/// A caller-provided sync implementation does not qualify power-loss durability.
pub fn durable_checkpoint_with_sync(
    path: &Path,
    bytes: &[u8],
    sync: &dyn Fn(&File) -> io::Result<()>,
) -> Result<()> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = path
        .parent()
        .ok_or_else(|| invalid("checkpoint has no parent"))?;
    let parent_meta = fs::symlink_metadata(parent).map_err(output)?;
    if !parent_meta.is_dir() || parent_meta.file_type().is_symlink() {
        return Err(invalid("checkpoint parent must be a real directory"));
    }
    if let Ok(meta) = fs::symlink_metadata(path) {
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err(invalid("checkpoint must be a regular file"));
        }
    }
    let staging = parent.join(format!(
        ".checkpoint-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)
        .map_err(output)?;
    let result = (|| {
        file.write_all(bytes).map_err(output)?;
        file.flush().map_err(output)?;
        sync(&file).map_err(output)?;
        fs::rename(&staging, path).map_err(output)?;
        File::open(parent)
            .and_then(|directory| sync(&directory))
            .map_err(output)
    })();
    if staging.exists() {
        let _ = fs::remove_file(&staging);
    }
    result
}
