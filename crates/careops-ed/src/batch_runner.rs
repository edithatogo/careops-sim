//! Independent CPU worker pool, checkpointing, progress tracking, and atomic batch execution for E3.2.
//!
//! Provides:
//! - Multi-worker independent CPU simulation execution using `std::thread::scope`.
//! - Strict deterministic equivalence: 1, 2, or N workers produce identical results for matching seeds.
//! - Atomic checkpointing and safe resumption without duplicate work.
//! - Progress tracking with metric aggregates and standard errors.
//! - Fail-safe incomplete run protection: partial runs cannot be mistaken for complete experiments.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::disposition::api_runner::{
    CancellationToken, ExitCodeDef, ValidationError,
};
use crate::{run_scenario, ScenarioConfig};

pub use careops_core::experiment::allocate_replication_seeds;
pub use careops_core::experiment::current_epoch_ms;

/// Individual replication execution result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplicationResult {
    pub replication_index: usize,
    pub seed: u64,
    pub arrivals: usize,
    pub started: usize,
    pub completed: usize,
    pub unfinished: usize,
    pub mean_wait_ticks: f64,
    pub horizon_ticks: u64,
}

impl careops_core::experiment::IndexedReplication for ReplicationResult {
    fn replication_index(&self) -> usize {
        self.replication_index
    }
    fn seed(&self) -> u64 {
        self.seed
    }
    fn primary_metric(&self) -> f64 {
        self.mean_wait_ticks
    }
}

/// Metadata attached to ED simulation batches.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EdBatchMetadata {
    pub scenario_id: String,
    pub horizon_override: Option<u64>,
}

/// Progress snapshot of a running batch experiment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchProgress {
    pub batch_id: String,
    pub total_replications: usize,
    pub completed_replications: usize,
    pub percent_complete: f64,
    pub running_mean_wait: f64,
}

/// Configuration specifying a multi-replication batch run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchConfig {
    pub schema_version: u32,
    pub batch_id: String,
    pub scenario_id: String,
    pub base_seed: u64,
    pub replications: usize,
    pub num_workers: usize,
    pub horizon_override: Option<u64>,
    pub checkpoint_interval: Option<usize>,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            batch_id: "default-batch".to_string(),
            scenario_id: "nominal-ed-mvp".to_string(),
            base_seed: 42,
            replications: 4,
            num_workers: 2,
            horizon_override: None,
            checkpoint_interval: Some(2),
        }
    }
}

/// Checkpoint state persisted during a running batch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchCheckpoint {
    pub schema_version: u32,
    pub batch_id: String,
    pub scenario_id: String,
    pub base_seed: u64,
    pub total_replications: usize,
    pub horizon_override: Option<u64>,
    pub completed_replications: Vec<ReplicationResult>,
    pub pending_indices: Vec<usize>,
    pub status: String,
    pub checkpoint_epoch_ms: u64,
}

/// Marker persisted when a batch is interrupted, aborted, or incomplete.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialBatchMarker {
    pub status: String,
    pub batch_id: String,
    pub completed_replications: usize,
    pub total_replications: usize,
    pub error: String,
}

/// Final completed batch experiment document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchRunDocument {
    pub schema_version: u32,
    pub batch_id: String,
    pub scenario_id: String,
    pub status: String,
    pub num_workers: usize,
    pub total_replications: usize,
    pub replications: Vec<ReplicationResult>,
    pub aggregate_mean_wait: f64,
    pub aggregate_std_error: f64,
    pub mean_completion_rate: f64,
    pub completed_at_epoch_ms: u64,
}

/// Atomically writes content to path via temporary staging file.
pub fn atomic_write_file(path: &Path, content: &[u8]) -> Result<(), ValidationError> {
    careops_core::experiment::atomic_write_file(path, content).map_err(|e| {
        ValidationError::new(
            ExitCodeDef::OutputWriteFailed,
            e.message,
        )
    })
}

/// Executes a single replication of a scenario given its index and seed.
pub fn execute_single_replication(
    scenario: &ScenarioConfig,
    replication_index: usize,
    seed: u64,
    horizon_override: Option<u64>,
) -> Result<ReplicationResult, ValidationError> {
    let mut eff_scen = scenario.clone();
    eff_scen.seed = seed;
    if let Some(h) = horizon_override {
        eff_scen.horizon_ticks = h;
    }

    let summary = run_scenario(&eff_scen)
        .map_err(|e| ValidationError::new(ExitCodeDef::SimulationFailed, e.to_string()))?;

    let waits: Vec<u64> = summary
        .patients
        .iter()
        .filter_map(|p| p.wait_ticks)
        .collect();
    let mean_wait = if !waits.is_empty() {
        waits.iter().sum::<u64>() as f64 / waits.len() as f64
    } else {
        0.0
    };

    Ok(ReplicationResult {
        replication_index,
        seed,
        arrivals: summary.arrivals,
        started: summary.started,
        completed: summary.completed,
        unfinished: summary.unfinished,
        mean_wait_ticks: mean_wait,
        horizon_ticks: summary.horizon_ticks,
    })
}

/// Computes aggregate statistics (mean wait, standard error, completion rate) for replication results.
pub fn compute_batch_aggregates(results: &[ReplicationResult]) -> (f64, f64, f64) {
    let total_reps = results.len();
    if total_reps == 0 {
        return (0.0, 0.0, 0.0);
    }
    let mean_wait = results.iter().map(|r| r.mean_wait_ticks).sum::<f64>() / total_reps as f64;
    let std_error = if total_reps > 1 {
        let variance = results
            .iter()
            .map(|r| (r.mean_wait_ticks - mean_wait).powi(2))
            .sum::<f64>()
            / (total_reps - 1) as f64;
        (variance / total_reps as f64).sqrt()
    } else {
        0.0
    };

    let total_arrivals: usize = results.iter().map(|r| r.arrivals).sum();
    let total_completed: usize = results.iter().map(|r| r.completed).sum();
    let completion_rate = if total_arrivals > 0 {
        total_completed as f64 / total_arrivals as f64
    } else {
        0.0
    };

    (mean_wait, std_error, completion_rate)
}

/// Executes a batch of replications across independent CPU worker threads.
pub fn run_batch_with_workers(
    scenario: &ScenarioConfig,
    config: &BatchConfig,
    output_dir: Option<&Path>,
    cancel_token: Option<&CancellationToken>,
    progress_callback: Option<&(dyn Fn(BatchProgress) + Send + Sync)>,
) -> Result<BatchRunDocument, ValidationError> {
    if config.replications == 0 {
        return Err(ValidationError::new(
            ExitCodeDef::MalformedConfig,
            "replications count must be greater than zero",
        ));
    }

    let progress_adapter = progress_callback.map(|cb| {
        move |p: careops_core::experiment::BatchProgress| {
            cb(BatchProgress {
                batch_id: p.batch_id,
                total_replications: p.total_replications,
                completed_replications: p.completed_replications,
                percent_complete: p.percent_complete,
                running_mean_wait: p.running_metric,
            });
        }
    });

    let metadata = EdBatchMetadata {
        scenario_id: config.scenario_id.clone(),
        horizon_override: config.horizon_override,
    };

    let runner_fn = |idx: usize, seed: u64, meta: &EdBatchMetadata| -> Result<ReplicationResult, careops_core::experiment::ExperimentError> {
        execute_single_replication(scenario, idx, seed, meta.horizon_override)
            .map_err(|e| careops_core::experiment::ExperimentError::new(
                match e.code {
                    ExitCodeDef::Interrupted => careops_core::experiment::ExperimentErrorKind::Interrupted,
                    ExitCodeDef::OutputWriteFailed => careops_core::experiment::ExperimentErrorKind::OutputWriteFailed,
                    ExitCodeDef::MalformedConfig => careops_core::experiment::ExperimentErrorKind::MalformedConfig,
                    _ => careops_core::experiment::ExperimentErrorKind::ExecutionFailed,
                },
                e.message,
            ))
    };

    let cb_ref = progress_adapter.as_ref().map(|f| f as &(dyn Fn(careops_core::experiment::BatchProgress) + Send + Sync));

    let results = careops_core::experiment::run_generic_batch(
        &config.batch_id,
        config.base_seed,
        config.replications,
        config.num_workers,
        config.checkpoint_interval,
        metadata,
        output_dir,
        cancel_token,
        cb_ref,
        runner_fn,
    ).map_err(|e| {
        let code = match e.kind {
            careops_core::experiment::ExperimentErrorKind::Interrupted => ExitCodeDef::Interrupted,
            careops_core::experiment::ExperimentErrorKind::MalformedConfig => ExitCodeDef::MalformedConfig,
            careops_core::experiment::ExperimentErrorKind::FileNotFound => ExitCodeDef::FileNotFound,
            careops_core::experiment::ExperimentErrorKind::OutputWriteFailed => ExitCodeDef::OutputWriteFailed,
            _ => ExitCodeDef::SimulationFailed,
        };
        ValidationError::new(code, e.message)
    })?;

    // Compute aggregate metrics
    let (mean_wait, std_error, completion_rate) = compute_batch_aggregates(&results);

    let document = BatchRunDocument {
        schema_version: 1,
        batch_id: config.batch_id.clone(),
        scenario_id: config.scenario_id.clone(),
        status: "complete".to_string(),
        num_workers: config.num_workers.max(1),
        total_replications: results.len(),
        replications: results,
        aggregate_mean_wait: mean_wait,
        aggregate_std_error: std_error,
        mean_completion_rate: completion_rate,
        completed_at_epoch_ms: current_epoch_ms(),
    };

    // Atomically persist final completed document if output_dir provided
    if let Some(dir) = output_dir {
        let final_path = dir.join("batch_run.json");
        let bytes = serde_json::to_vec_pretty(&document)
            .map_err(|e| ValidationError::new(ExitCodeDef::MalformedConfig, e.to_string()))?;
        atomic_write_file(&final_path, &bytes)?;

        // Remove any partial marker on valid completion
        let partial_path = dir.join("batch_run.partial");
        if partial_path.exists() {
            let _ = fs::remove_file(partial_path);
        }
    }

    Ok(document)
}

/// Resumes a previously interrupted batch from its checkpoint file.
pub fn resume_batch_from_checkpoint(
    checkpoint_path: &Path,
    scenario: &ScenarioConfig,
    num_workers: usize,
    output_dir: &Path,
    cancel_token: Option<&CancellationToken>,
) -> Result<BatchRunDocument, ValidationError> {
    let runner_fn = |idx: usize, seed: u64, meta: &EdBatchMetadata| -> Result<ReplicationResult, careops_core::experiment::ExperimentError> {
        execute_single_replication(scenario, idx, seed, meta.horizon_override)
            .map_err(|e| careops_core::experiment::ExperimentError::new(
                match e.code {
                    ExitCodeDef::Interrupted => careops_core::experiment::ExperimentErrorKind::Interrupted,
                    ExitCodeDef::OutputWriteFailed => careops_core::experiment::ExperimentErrorKind::OutputWriteFailed,
                    ExitCodeDef::MalformedConfig => careops_core::experiment::ExperimentErrorKind::MalformedConfig,
                    _ => careops_core::experiment::ExperimentErrorKind::ExecutionFailed,
                },
                e.message,
            ))
    };

    let (checkpoint, all_results) = careops_core::experiment::resume_generic_batch::<ReplicationResult, EdBatchMetadata, _>(
        checkpoint_path,
        num_workers,
        output_dir,
        cancel_token,
        None,
        runner_fn,
    ).map_err(|e| {
        let code = match e.kind {
            careops_core::experiment::ExperimentErrorKind::Interrupted => ExitCodeDef::Interrupted,
            careops_core::experiment::ExperimentErrorKind::MalformedConfig => ExitCodeDef::MalformedConfig,
            careops_core::experiment::ExperimentErrorKind::FileNotFound => ExitCodeDef::FileNotFound,
            careops_core::experiment::ExperimentErrorKind::OutputWriteFailed => ExitCodeDef::OutputWriteFailed,
            _ => ExitCodeDef::SimulationFailed,
        };
        ValidationError::new(code, e.message)
    })?;

    let total_reps = all_results.len();
    let (mean_wait, std_error, completion_rate) = compute_batch_aggregates(&all_results);

    let doc = BatchRunDocument {
        schema_version: 1,
        batch_id: checkpoint.batch_id,
        scenario_id: checkpoint.metadata.scenario_id,
        status: "complete".to_string(),
        num_workers: num_workers.max(1),
        total_replications: total_reps,
        replications: all_results,
        aggregate_mean_wait: mean_wait,
        aggregate_std_error: std_error,
        mean_completion_rate: completion_rate,
        completed_at_epoch_ms: current_epoch_ms(),
    };

    let final_path = output_dir.join("batch_run.json");
    let bytes = serde_json::to_vec_pretty(&doc)
        .map_err(|e| ValidationError::new(ExitCodeDef::MalformedConfig, e.to_string()))?;
    atomic_write_file(&final_path, &bytes)?;

    let partial_path = output_dir.join("batch_run.partial");
    if partial_path.exists() {
        let _ = fs::remove_file(partial_path);
    }

    Ok(doc)
}

/// Safely loads a batch run document, rejecting any incomplete partial artifact.
pub fn load_completed_batch(path: &Path) -> Result<BatchRunDocument, ValidationError> {
    if !path.exists() {
        // Check if partial marker exists
        let partial_path = path.with_extension("partial");
        if partial_path.exists() {
            return Err(ValidationError::new(
                ExitCodeDef::MalformedConfig,
                "a partial run is clearly incomplete and cannot be mistaken for a valid finished experiment",
            ));
        }
        return Err(ValidationError::new(
            ExitCodeDef::FileNotFound,
            format!("batch artifact not found: '{}'", path.display()),
        ));
    }

    let bytes = fs::read(path)
        .map_err(|e| ValidationError::new(ExitCodeDef::FileNotFound, e.to_string()))?;

    // Check if file is a partial marker
    if let Ok(marker) = serde_json::from_slice::<PartialBatchMarker>(&bytes) {
        if marker.status == "incomplete" {
            return Err(ValidationError::new(
                ExitCodeDef::MalformedConfig,
                "a partial run is clearly incomplete and cannot be mistaken for a valid finished experiment",
            ));
        }
    }

    let doc: BatchRunDocument = serde_json::from_slice(&bytes).map_err(|e| {
        ValidationError::new(
            ExitCodeDef::MalformedConfig,
            format!("invalid batch artifact JSON: {e}"),
        )
    })?;

    if doc.status != "complete" {
        return Err(ValidationError::new(
            ExitCodeDef::MalformedConfig,
            "a partial run is clearly incomplete and cannot be mistaken for a valid finished experiment",
        ));
    }

    Ok(doc)
}
