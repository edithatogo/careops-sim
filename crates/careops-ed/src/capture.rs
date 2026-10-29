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
use std::io::{self, Read, Write};
use std::path::Path;

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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CapturedReplication {
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
    pub codec_id: String,
    pub effective_input_sha256: String,
    pub config: BatchConfig,
    pub limits: CaptureLimits,
}
pub type CaptureCheckpoint =
    experiment::GenericBatchCheckpoint<CapturedReplication, CaptureMetadata>;
const SCHEMA: &str = "careops.ed.capture-boundary.v1";

fn part_name(index: usize, sequence: usize) -> String {
    format!("rep-{index:08}-seq-{sequence:016}.part")
}
fn metadata(
    scenario: &ScenarioConfig,
    config: &BatchConfig,
    limits: CaptureLimits,
    store: &dyn PartStore,
) -> Result<CaptureMetadata> {
    limits.validate()?;
    if config.replications == 0 {
        return Err(invalid("replications must be positive"));
    }
    Ok(CaptureMetadata {
        capture_schema: SCHEMA.into(),
        codec_id: store.codec_id().into(),
        effective_input_sha256: compute_sha256_hex(
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

fn validate_parts(
    results: &[CapturedReplication],
    meta: &CaptureMetadata,
    scenario: &ScenarioConfig,
    store: &dyn PartStore,
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
        if seeds.get(index) != Some(&result.summary.seed) || !replications.insert(index) {
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
                return Err(invalid("duplicate, unsafe or discontinuous capture part"));
            }
            let bytes = store.read_part(&part.name, limits.max_bytes)?;
            if bytes.len() != part.bytes || compute_sha256_hex(&bytes) != part.sha256 {
                return Err(invalid("capture part hash or length mismatch"));
            }
            let batch = store.decode_part(&bytes)?;
            if batch.schema != SCHEMA
                || batch.run_id != meta.config.batch_id
                || batch.input_sha256 != meta.effective_input_sha256
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
    let actual = store.part_names()?;
    if actual.len() != names.len() || actual.into_iter().collect::<BTreeSet<_>>() != names {
        return Err(invalid(
            "unreferenced capture parts require explicit recovery; no implicit replay",
        ));
    }
    Ok(())
}

fn read_checkpoint(path: &Path) -> Result<CaptureCheckpoint> {
    let mut bytes = vec![];
    std::fs::File::open(path)
        .map_err(output)?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(output)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(invalid("capture checkpoint too large"));
    }
    serde_json::from_slice(&bytes).map_err(invalid_json)
}
fn check_binding(checkpoint: &CaptureCheckpoint, expected: &CaptureMetadata) -> Result<()> {
    let mut config = checkpoint.metadata.config.clone();
    config.num_workers = expected.config.num_workers;
    if checkpoint.metadata.capture_schema != SCHEMA
        || checkpoint.metadata.codec_id != expected.codec_id
        || checkpoint.metadata.limits != expected.limits
        || checkpoint.metadata.effective_input_sha256 != expected.effective_input_sha256
        || config != expected.config
        || checkpoint.batch_id != expected.config.batch_id
        || checkpoint.base_seed != expected.config.base_seed
        || checkpoint.total_replications != expected.config.replications
    {
        return Err(invalid("capture checkpoint input/config binding mismatch"));
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
    let meta = metadata(scenario, config, limits, store)?;
    let checkpoint_path = checkpoint_dir.join("checkpoint.json");
    if checkpoint_path.exists() || store.is_complete()? || !store.part_names()?.is_empty() {
        return Err(invalid(
            "capture run already exists; use completed-replication recovery",
        ));
    }
    let results = experiment::run_generic_batch(
        &config.batch_id,
        config.base_seed,
        config.replications,
        config.num_workers,
        Some(1),
        meta.clone(),
        Some(checkpoint_dir),
        None,
        None,
        |index, seed, m| execute(scenario, index, seed, m, store),
    )?;
    // The inherited generic runner treats checkpoint writes as best effort. A
    // captured run must not publish completion unless its actual checkpoint agrees.
    let checkpoint = read_checkpoint(&checkpoint_path)?;
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
    let meta = metadata(scenario, config, limits, store)?;
    if store.is_complete()? {
        return Err(invalid("completed capture cannot be overwritten"));
    }
    let path = checkpoint_dir.join("checkpoint.json");
    let checkpoint = read_checkpoint(&path)?;
    check_binding(&checkpoint, &meta)?;
    validate_parts(&checkpoint.completed_replications, &meta, scenario, store)?;
    // Exclusive checkpoint/store ownership is required throughout this operation.
    let (_, results) = experiment::resume_generic_batch::<CapturedReplication, CaptureMetadata, _>(
        &path,
        config.num_workers,
        checkpoint_dir,
        None,
        None,
        |index, seed, m| execute(scenario, index, seed, m, store),
    )?;
    validate_parts(&results, &meta, scenario, store)?;
    // Persist the complete capture checkpoint before publishing completion.
    let completed = CaptureCheckpoint {
        completed_replications: results.clone(),
        pending_indices: vec![],
        status: "complete".into(),
        ..checkpoint
    };
    experiment::atomic_write_file(
        &path,
        &serde_json::to_vec(&completed).map_err(invalid_json)?,
    )?;
    store.complete(&results)?;
    Ok(results)
}
