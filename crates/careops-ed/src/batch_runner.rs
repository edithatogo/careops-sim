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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::disposition::api_runner::{
    allocate_replication_seeds, CancellationToken, ExitCodeDef, ValidationError,
};
use crate::{run_scenario, ScenarioConfig};

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

fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Atomically writes content to path via temporary staging file.
pub fn atomic_write_file(path: &Path, content: &[u8]) -> Result<(), ValidationError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                ValidationError::new(
                    ExitCodeDef::OutputWriteFailed,
                    format!("failed to create directory '{}': {e}", parent.display()),
                )
            })?;
        }
    }

    let staging_path = path.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&staging_path, content).map_err(|e| {
        ValidationError::new(
            ExitCodeDef::OutputWriteFailed,
            format!(
                "failed to write staging file '{}': {e}",
                staging_path.display()
            ),
        )
    })?;

    fs::rename(&staging_path, path).map_err(|e| {
        let _ = fs::remove_file(&staging_path);
        ValidationError::new(
            ExitCodeDef::OutputWriteFailed,
            format!("failed to atomically commit file '{}': {e}", path.display()),
        )
    })?;

    Ok(())
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
    let num_workers = config.num_workers.max(1);
    let all_seeds = allocate_replication_seeds(config.base_seed, config.replications);

    // Prepare task queue: list of (index, seed), reversed so pop() yields index 0 first
    let mut tasks: Vec<(usize, u64)> = all_seeds.into_iter().enumerate().collect();
    tasks.reverse();
    let task_queue = Arc::new(Mutex::new(tasks));
    let completed_results: Arc<Mutex<Vec<ReplicationResult>>> = Arc::new(Mutex::new(Vec::new()));
    let cancel_flag = Arc::new(AtomicBool::new(false));

    // Check cancellation before launching
    if let Some(token) = cancel_token {
        if token.is_cancelled() {
            return Err(ValidationError::new(
                ExitCodeDef::Interrupted,
                "batch run cancelled before dispatch",
            ));
        }
    }

    let checkpoint_interval = config.checkpoint_interval.unwrap_or(usize::MAX);
    let checkpoint_counter = Arc::new(AtomicUsize::new(0));

    // Spawn independent CPU worker threads using standard library thread scope
    std::thread::scope(|scope| {
        let mut worker_handles = Vec::new();

        for _ in 0..num_workers {
            let q = Arc::clone(&task_queue);
            let results = Arc::clone(&completed_results);
            let cancel = Arc::clone(&cancel_flag);
            let chk_counter = Arc::clone(&checkpoint_counter);
            let scenario_clone = scenario.clone();
            let horizon_override = config.horizon_override;

            let handle = scope.spawn(move || -> Result<(), ValidationError> {
                loop {
                    if cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Some(token) = cancel_token {
                        if token.is_cancelled() {
                            cancel.store(true, Ordering::Relaxed);
                            break;
                        }
                    }

                    let next_task = {
                        let mut guard = q.lock().unwrap();
                        guard.pop()
                    };

                    let Some((idx, seed)) = next_task else {
                        break;
                    };

                    let rep_res =
                        execute_single_replication(&scenario_clone, idx, seed, horizon_override)?;

                    {
                        let mut guard = results.lock().unwrap();
                        guard.push(rep_res.clone());
                    }

                    // Progress reporting
                    if let Some(cb) = progress_callback {
                        let guard = results.lock().unwrap();
                        let count = guard.len();
                        let sum_wait: f64 = guard.iter().map(|r| r.mean_wait_ticks).sum();
                        let running_mean = if count > 0 {
                            sum_wait / count as f64
                        } else {
                            0.0
                        };
                        cb(BatchProgress {
                            batch_id: config.batch_id.clone(),
                            total_replications: config.replications,
                            completed_replications: count,
                            percent_complete: (count as f64 / config.replications as f64) * 100.0,
                            running_mean_wait: running_mean,
                        });
                    }

                    // Checkpointing
                    let completed_now = chk_counter.fetch_add(1, Ordering::SeqCst) + 1;
                    if completed_now.is_multiple_of(checkpoint_interval) {
                        if let Some(dir) = output_dir {
                            let guard = results.lock().unwrap();
                            let done_indices: std::collections::HashSet<usize> =
                                guard.iter().map(|r| r.replication_index).collect();
                            let pending: Vec<usize> = (0..config.replications)
                                .filter(|i| !done_indices.contains(i))
                                .collect();

                            let mut done_reps = guard.clone();
                            done_reps.sort_by_key(|r| r.replication_index);

                            let checkpoint = BatchCheckpoint {
                                schema_version: 1,
                                batch_id: config.batch_id.clone(),
                                scenario_id: config.scenario_id.clone(),
                                base_seed: config.base_seed,
                                total_replications: config.replications,
                                horizon_override: config.horizon_override,
                                completed_replications: done_reps,
                                pending_indices: pending,
                                status: "in_progress".to_string(),
                                checkpoint_epoch_ms: current_epoch_ms(),
                            };

                            let chk_path = dir.join("checkpoint.json");
                            if let Ok(bytes) = serde_json::to_vec_pretty(&checkpoint) {
                                let _ = atomic_write_file(&chk_path, &bytes);
                            }
                        }
                    }
                }
                Ok(())
            });

            worker_handles.push(handle);
        }

        // Wait for all workers to finish
        for h in worker_handles {
            h.join().unwrap()?;
        }
        Ok(())
    })?;

    // Check if run was cancelled or incomplete
    let mut results = Arc::try_unwrap(completed_results)
        .unwrap()
        .into_inner()
        .unwrap();

    // Sort by replication index for strict deterministic ordering
    results.sort_by_key(|r| r.replication_index);

    let is_cancelled = cancel_flag.load(Ordering::Relaxed)
        || cancel_token.map(|t| t.is_cancelled()).unwrap_or(false);

    if is_cancelled || results.len() < config.replications {
        if let Some(dir) = output_dir {
            let partial_marker = PartialBatchMarker {
                status: "incomplete".to_string(),
                batch_id: config.batch_id.clone(),
                completed_replications: results.len(),
                total_replications: config.replications,
                error: "batch execution was cancelled or interrupted".to_string(),
            };
            let partial_path = dir.join("batch_run.partial");
            if let Ok(bytes) = serde_json::to_vec_pretty(&partial_marker) {
                let _ = atomic_write_file(&partial_path, &bytes);
            }
        }
        return Err(ValidationError::new(
            ExitCodeDef::Interrupted,
            format!(
                "batch run incomplete: {}/{} replications completed",
                results.len(),
                config.replications
            ),
        ));
    }

    // Compute aggregate metrics
    let total_reps = results.len();
    let (mean_wait, std_error, completion_rate) = compute_batch_aggregates(&results);

    let document = BatchRunDocument {
        schema_version: 1,
        batch_id: config.batch_id.clone(),
        scenario_id: config.scenario_id.clone(),
        status: "complete".to_string(),
        num_workers,
        total_replications: total_reps,
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
    if !checkpoint_path.exists() {
        return Err(ValidationError::new(
            ExitCodeDef::FileNotFound,
            format!("checkpoint file not found: '{}'", checkpoint_path.display()),
        ));
    }

    let bytes = fs::read(checkpoint_path)
        .map_err(|e| ValidationError::new(ExitCodeDef::FileNotFound, e.to_string()))?;

    let checkpoint: BatchCheckpoint = serde_json::from_slice(&bytes).map_err(|e| {
        ValidationError::new(
            ExitCodeDef::MalformedConfig,
            format!("invalid checkpoint format: {e}"),
        )
    })?;

    if checkpoint.pending_indices.is_empty() {
        // Already finished, assemble document
        let total = checkpoint.completed_replications.len();
        let (mean_wait, std_error, completion_rate) =
            compute_batch_aggregates(&checkpoint.completed_replications);

        let doc = BatchRunDocument {
            schema_version: 1,
            batch_id: checkpoint.batch_id,
            scenario_id: checkpoint.scenario_id,
            status: "complete".to_string(),
            num_workers,
            total_replications: total,
            replications: checkpoint.completed_replications,
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

        return Ok(doc);
    }

    // Determine pending tasks
    let all_seeds = allocate_replication_seeds(checkpoint.base_seed, checkpoint.total_replications);
    let mut pending_tasks: Vec<(usize, u64)> = checkpoint
        .pending_indices
        .iter()
        .map(|&idx| (idx, all_seeds[idx]))
        .collect();
    pending_tasks.reverse();

    let task_queue = Arc::new(Mutex::new(pending_tasks));
    let newly_completed: Arc<Mutex<Vec<ReplicationResult>>> = Arc::new(Mutex::new(Vec::new()));
    let cancel_flag = Arc::new(AtomicBool::new(false));

    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..num_workers.max(1) {
            let q = Arc::clone(&task_queue);
            let results = Arc::clone(&newly_completed);
            let cancel = Arc::clone(&cancel_flag);
            let scenario_clone = scenario.clone();
            let horizon_override = checkpoint.horizon_override;

            let h = scope.spawn(move || -> Result<(), ValidationError> {
                loop {
                    if cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Some(t) = cancel_token {
                        if t.is_cancelled() {
                            cancel.store(true, Ordering::Relaxed);
                            break;
                        }
                    }

                    let task = {
                        let mut guard = q.lock().unwrap();
                        guard.pop()
                    };

                    let Some((idx, seed)) = task else {
                        break;
                    };

                    let rep =
                        execute_single_replication(&scenario_clone, idx, seed, horizon_override)?;
                    {
                        let mut guard = results.lock().unwrap();
                        guard.push(rep);
                    }
                }
                Ok(())
            });
            handles.push(h);
        }

        for h in handles {
            h.join().unwrap()?;
        }
        Ok(())
    })?;

    // Combine previous results and new results
    let mut all_results = checkpoint.completed_replications;
    let mut new_results = Arc::try_unwrap(newly_completed)
        .unwrap()
        .into_inner()
        .unwrap();
    all_results.append(&mut new_results);
    all_results.sort_by_key(|r| r.replication_index);

    if all_results.len() < checkpoint.total_replications {
        return Err(ValidationError::new(
            ExitCodeDef::Interrupted,
            format!(
                "resumed batch incomplete: {}/{} replications finished",
                all_results.len(),
                checkpoint.total_replications
            ),
        ));
    }

    let total_reps = all_results.len();
    let (mean_wait, std_error, completion_rate) = compute_batch_aggregates(&all_results);

    let doc = BatchRunDocument {
        schema_version: 1,
        batch_id: checkpoint.batch_id,
        scenario_id: checkpoint.scenario_id,
        status: "complete".to_string(),
        num_workers,
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
