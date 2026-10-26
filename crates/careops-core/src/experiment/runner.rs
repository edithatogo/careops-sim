//! Generic multi-worker experiment runner, progress tracking, checkpointing, and crash recovery.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::cancellation::CancellationToken;
use super::error::{ExperimentError, ExperimentErrorKind};
use super::io::atomic_write_file;
use super::seed::allocate_replication_seeds;

/// Returns the current Unix epoch in milliseconds.
pub fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Contract for replication results that have an index and seed.
pub trait IndexedReplication: Clone + Send + 'static {
    fn replication_index(&self) -> usize;
    fn seed(&self) -> u64;
    fn primary_metric(&self) -> f64 {
        0.0
    }
}

fn validate_replication_identity<R: IndexedReplication>(
    replication: &R,
    expected_index: usize,
    expected_seed: u64,
) -> Result<(), ExperimentError> {
    if replication.replication_index() != expected_index || replication.seed() != expected_seed {
        return Err(ExperimentError::new(
            ExperimentErrorKind::ExecutionFailed,
            format!(
                "replication result identity mismatch: expected index {expected_index} and seed {expected_seed}, received index {} and seed {}",
                replication.replication_index(),
                replication.seed()
            ),
        ));
    }
    Ok(())
}

fn validate_checkpoint<R: IndexedReplication, M>(
    checkpoint: &GenericBatchCheckpoint<R, M>,
) -> Result<(), ExperimentError> {
    let malformed = |message: String| {
        ExperimentError::new(
            ExperimentErrorKind::MalformedConfig,
            format!("invalid checkpoint: {message}"),
        )
    };

    if checkpoint.schema_version != 1 {
        return Err(malformed(format!(
            "unsupported schema version {}",
            checkpoint.schema_version
        )));
    }
    if checkpoint.total_replications == 0 {
        return Err(malformed(
            "total_replications must be greater than zero".to_string(),
        ));
    }

    let seeds = allocate_replication_seeds(checkpoint.base_seed, checkpoint.total_replications);
    let mut assigned_indices = HashSet::with_capacity(checkpoint.total_replications);
    for replication in &checkpoint.completed_replications {
        let index = replication.replication_index();
        if index >= checkpoint.total_replications {
            return Err(malformed(format!(
                "completed replication index {index} is out of range"
            )));
        }
        validate_replication_identity(replication, index, seeds[index])
            .map_err(|error| malformed(error.message))?;
        if !assigned_indices.insert(index) {
            return Err(malformed(format!(
                "completed replication index {index} is duplicated"
            )));
        }
    }

    for &index in &checkpoint.pending_indices {
        if index >= checkpoint.total_replications {
            return Err(malformed(format!(
                "pending replication index {index} is out of range"
            )));
        }
        if !assigned_indices.insert(index) {
            return Err(malformed(format!(
                "replication index {index} occurs more than once or is both completed and pending"
            )));
        }
    }

    if assigned_indices.len() != checkpoint.total_replications {
        return Err(malformed(format!(
            "completed and pending indices cover {} of {} replications",
            assigned_indices.len(),
            checkpoint.total_replications
        )));
    }

    Ok(())
}

/// Progress snapshot of a running batch experiment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchProgress {
    pub batch_id: String,
    pub total_replications: usize,
    pub completed_replications: usize,
    pub percent_complete: f64,
    pub running_metric: f64,
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

/// Generic checkpoint state persisted during a running batch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericBatchCheckpoint<R, M = ()> {
    pub schema_version: u32,
    pub batch_id: String,
    pub base_seed: u64,
    pub total_replications: usize,
    pub completed_replications: Vec<R>,
    pub pending_indices: Vec<usize>,
    pub status: String,
    pub checkpoint_epoch_ms: u64,
    #[serde(flatten)]
    pub metadata: M,
}

/// Generic completed batch experiment document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericBatchRunDocument<R, A, M = ()> {
    pub schema_version: u32,
    pub batch_id: String,
    pub status: String,
    pub num_workers: usize,
    pub total_replications: usize,
    pub replications: Vec<R>,
    #[serde(flatten)]
    pub aggregates: A,
    #[serde(flatten)]
    pub metadata: M,
    pub completed_at_epoch_ms: u64,
}

/// Executes a batch of replications across independent CPU worker threads.
#[allow(clippy::too_many_arguments)] // Distinct controls preserve the runner's public lifecycle contract.
pub fn run_generic_batch<R, M, F>(
    batch_id: &str,
    base_seed: u64,
    replications: usize,
    num_workers: usize,
    checkpoint_interval: Option<usize>,
    metadata: M,
    output_dir: Option<&Path>,
    cancel_token: Option<&CancellationToken>,
    progress_callback: Option<&(dyn Fn(BatchProgress) + Send + Sync)>,
    run_replication: F,
) -> Result<Vec<R>, ExperimentError>
where
    R: IndexedReplication + Serialize + for<'de> Deserialize<'de>,
    M: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
    F: Fn(usize, u64, &M) -> Result<R, ExperimentError> + Sync + Send,
{
    if replications == 0 {
        return Err(ExperimentError::new(
            ExperimentErrorKind::MalformedConfig,
            "replications count must be greater than zero",
        ));
    }

    let effective_workers = num_workers.max(1);
    let all_seeds = allocate_replication_seeds(base_seed, replications);

    // Queue of tasks (index, seed), popped in ascending index order
    let mut tasks: Vec<(usize, u64)> = all_seeds.into_iter().enumerate().collect();
    tasks.reverse();
    let task_queue = Arc::new(Mutex::new(tasks));
    let completed_results: Arc<Mutex<Vec<R>>> = Arc::new(Mutex::new(Vec::new()));
    let cancel_flag = Arc::new(AtomicBool::new(false));

    if let Some(token) = cancel_token {
        if token.is_cancelled() {
            return Err(ExperimentError::new(
                ExperimentErrorKind::Interrupted,
                "batch run cancelled before dispatch",
            ));
        }
    }

    let interval = checkpoint_interval.unwrap_or(usize::MAX);
    let checkpoint_counter = Arc::new(AtomicUsize::new(0));

    // Spawn workers with scoped threads
    let worker_result = std::thread::scope(|scope| {
        let mut handles = Vec::new();

        for _ in 0..effective_workers {
            let q = Arc::clone(&task_queue);
            let results = Arc::clone(&completed_results);
            let cancel = Arc::clone(&cancel_flag);
            let chk_counter = Arc::clone(&checkpoint_counter);
            let metadata_clone = metadata.clone();
            let runner_ref = &run_replication;

            let handle = scope.spawn(move || -> Result<(), ExperimentError> {
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

                    let rep_res = runner_ref(idx, seed, &metadata_clone)?;
                    validate_replication_identity(&rep_res, idx, seed)?;

                    {
                        let mut guard = results.lock().unwrap();
                        guard.push(rep_res);
                    }

                    // Progress reporting
                    if let Some(cb) = progress_callback {
                        let guard = results.lock().unwrap();
                        let count = guard.len();
                        let sum_metric: f64 = guard.iter().map(|r| r.primary_metric()).sum();
                        let running_metric = if count > 0 {
                            sum_metric / count as f64
                        } else {
                            0.0
                        };
                        cb(BatchProgress {
                            batch_id: batch_id.to_string(),
                            total_replications: replications,
                            completed_replications: count,
                            percent_complete: (count as f64 / replications as f64) * 100.0,
                            running_metric,
                        });
                    }

                    // Checkpointing
                    let completed_now = chk_counter.fetch_add(1, Ordering::SeqCst) + 1;
                    if completed_now.is_multiple_of(interval) {
                        if let Some(dir) = output_dir {
                            let guard = results.lock().unwrap();
                            let done_indices: std::collections::HashSet<usize> =
                                guard.iter().map(|r| r.replication_index()).collect();
                            let pending: Vec<usize> = (0..replications)
                                .filter(|i| !done_indices.contains(i))
                                .collect();

                            let mut done_reps = guard.clone();
                            done_reps.sort_by_key(|r| r.replication_index());

                            let checkpoint = GenericBatchCheckpoint {
                                schema_version: 1,
                                batch_id: batch_id.to_string(),
                                base_seed,
                                total_replications: replications,
                                completed_replications: done_reps,
                                pending_indices: pending,
                                status: "in_progress".to_string(),
                                checkpoint_epoch_ms: current_epoch_ms(),
                                metadata: metadata_clone.clone(),
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

            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap()?;
        }
        Ok(())
    });

    if let Err(e) = worker_result {
        // Record partial marker if directory provided
        if let Some(dir) = output_dir {
            let guard = completed_results.lock().unwrap();
            let marker = PartialBatchMarker {
                status: "incomplete".to_string(),
                batch_id: batch_id.to_string(),
                completed_replications: guard.len(),
                total_replications: replications,
                error: format!("worker error: {e}"),
            };
            let marker_path = dir.join("batch_run.partial");
            if let Ok(bytes) = serde_json::to_vec_pretty(&marker) {
                let _ = atomic_write_file(&marker_path, &bytes);
            }
        }
        return Err(e);
    }

    let mut results = Arc::try_unwrap(completed_results)
        .ok()
        .expect("all worker threads have exited")
        .into_inner()
        .unwrap();

    // Deterministic sorting by replication index guarantees worker invariance
    results.sort_by_key(|r| r.replication_index());

    let is_cancelled = cancel_flag.load(Ordering::Relaxed)
        || cancel_token.map(|t| t.is_cancelled()).unwrap_or(false);

    if is_cancelled || results.len() < replications {
        if let Some(dir) = output_dir {
            let partial_marker = PartialBatchMarker {
                status: "incomplete".to_string(),
                batch_id: batch_id.to_string(),
                completed_replications: results.len(),
                total_replications: replications,
                error: "batch execution was cancelled or interrupted".to_string(),
            };
            let partial_path = dir.join("batch_run.partial");
            if let Ok(bytes) = serde_json::to_vec_pretty(&partial_marker) {
                let _ = atomic_write_file(&partial_path, &bytes);
            }
        }
        return Err(ExperimentError::new(
            ExperimentErrorKind::Interrupted,
            format!(
                "batch run incomplete: {}/{} replications completed",
                results.len(),
                replications
            ),
        ));
    }

    // Success cleanup: remove partial marker if present
    if let Some(dir) = output_dir {
        let partial_path = dir.join("batch_run.partial");
        if partial_path.exists() {
            let _ = fs::remove_file(partial_path);
        }
    }

    Ok(results)
}

/// Resumes a previously interrupted batch from its checkpoint file.
pub fn resume_generic_batch<R, M, F>(
    checkpoint_path: &Path,
    num_workers: usize,
    output_dir: &Path,
    cancel_token: Option<&CancellationToken>,
    progress_callback: Option<&(dyn Fn(BatchProgress) + Send + Sync)>,
    run_replication: F,
) -> Result<(GenericBatchCheckpoint<R, M>, Vec<R>), ExperimentError>
where
    R: IndexedReplication + Serialize + for<'de> Deserialize<'de>,
    M: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
    F: Fn(usize, u64, &M) -> Result<R, ExperimentError> + Sync + Send,
{
    if !checkpoint_path.exists() {
        return Err(ExperimentError::new(
            ExperimentErrorKind::FileNotFound,
            format!("checkpoint file not found: '{}'", checkpoint_path.display()),
        ));
    }

    let bytes = fs::read(checkpoint_path)
        .map_err(|e| ExperimentError::new(ExperimentErrorKind::FileNotFound, e.to_string()))?;

    let checkpoint: GenericBatchCheckpoint<R, M> = serde_json::from_slice(&bytes).map_err(|e| {
        ExperimentError::new(
            ExperimentErrorKind::MalformedConfig,
            format!("invalid checkpoint format: {e}"),
        )
    })?;

    validate_checkpoint(&checkpoint)?;

    if checkpoint.pending_indices.is_empty() {
        // Already finished
        let partial_path = output_dir.join("batch_run.partial");
        if partial_path.exists() {
            let _ = fs::remove_file(partial_path);
        }
        let completed = checkpoint.completed_replications.clone();
        return Ok((checkpoint, completed));
    }

    let all_seeds = allocate_replication_seeds(checkpoint.base_seed, checkpoint.total_replications);
    let mut pending_tasks: Vec<(usize, u64)> = checkpoint
        .pending_indices
        .iter()
        .map(|&idx| (idx, all_seeds[idx]))
        .collect();
    pending_tasks.reverse();

    let task_queue = Arc::new(Mutex::new(pending_tasks));
    let newly_completed: Arc<Mutex<Vec<R>>> = Arc::new(Mutex::new(Vec::new()));
    let cancel_flag = Arc::new(AtomicBool::new(false));

    let effective_workers = num_workers.max(1);
    let prev_completed_count = checkpoint.completed_replications.len();
    let prev_sum: f64 = checkpoint
        .completed_replications
        .iter()
        .map(|r| r.primary_metric())
        .sum();
    let batch_id_ref = &checkpoint.batch_id;
    let metadata_ref = &checkpoint.metadata;

    let worker_result = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..effective_workers {
            let q = Arc::clone(&task_queue);
            let results = Arc::clone(&newly_completed);
            let cancel = Arc::clone(&cancel_flag);
            let runner_ref = &run_replication;

            let h = scope.spawn(move || -> Result<(), ExperimentError> {
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

                    let rep_res = runner_ref(idx, seed, metadata_ref)?;
                    validate_replication_identity(&rep_res, idx, seed)?;

                    {
                        let mut guard = results.lock().unwrap();
                        guard.push(rep_res);
                    }

                    if let Some(cb) = progress_callback {
                        let guard = results.lock().unwrap();
                        let newly_done = guard.len();
                        let total_done = prev_completed_count + newly_done;
                        let new_sum: f64 = guard.iter().map(|r| r.primary_metric()).sum();
                        let running_metric = if total_done > 0 {
                            (prev_sum + new_sum) / total_done as f64
                        } else {
                            0.0
                        };
                        cb(BatchProgress {
                            batch_id: batch_id_ref.clone(),
                            total_replications: checkpoint.total_replications,
                            completed_replications: total_done,
                            percent_complete: (total_done as f64
                                / checkpoint.total_replications as f64)
                                * 100.0,
                            running_metric,
                        });
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
    });

    worker_result?;

    let mut new_results = Arc::try_unwrap(newly_completed)
        .ok()
        .expect("all worker threads have exited")
        .into_inner()
        .unwrap();

    let mut all_results = checkpoint.completed_replications.clone();
    all_results.append(&mut new_results);
    all_results.sort_by_key(|r| r.replication_index());

    let is_cancelled = cancel_flag.load(Ordering::Relaxed)
        || cancel_token.map(|t| t.is_cancelled()).unwrap_or(false);

    if is_cancelled || all_results.len() < checkpoint.total_replications {
        let partial_marker = PartialBatchMarker {
            status: "incomplete".to_string(),
            batch_id: checkpoint.batch_id.clone(),
            completed_replications: all_results.len(),
            total_replications: checkpoint.total_replications,
            error: "resumed batch execution was cancelled or interrupted".to_string(),
        };
        let partial_path = output_dir.join("batch_run.partial");
        if let Ok(bytes) = serde_json::to_vec_pretty(&partial_marker) {
            let _ = atomic_write_file(&partial_path, &bytes);
        }
        return Err(ExperimentError::new(
            ExperimentErrorKind::Interrupted,
            format!(
                "resumed batch incomplete: {}/{} replications completed",
                all_results.len(),
                checkpoint.total_replications
            ),
        ));
    }

    let partial_path = output_dir.join("batch_run.partial");
    if partial_path.exists() {
        let _ = fs::remove_file(partial_path);
    }

    Ok((checkpoint, all_results))
}
