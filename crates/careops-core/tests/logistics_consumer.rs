//! Non-clinical logistics consumer demonstrating generic experiment lifecycle reuse:
//! package sorting and delivery dock routing hub simulation.
//!
//! Verifies:
//! - Worker invariance: 1, 2, and 4 workers produce identical outputs for matching seeds.
//! - Cancellation safety: cancelled batch writes `batch_run.partial` and never writes `batch_run.json`.
//! - Checkpointing and crash recovery: resuming an interrupted batch finishes remaining replications
//!   without duplicates and matches unbroken baseline.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use careops_core::experiment::{
    atomic_write_file, current_epoch_ms, resume_generic_batch, run_generic_batch,
    CancellationToken, ExperimentError, ExperimentErrorKind, GenericBatchRunDocument,
    IndexedReplication,
};

/// Configuration for package sorting hub simulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogisticsConfig {
    pub warehouse_id: String,
    pub num_packages: usize,
    pub num_docks: usize,
    pub dock_rate_per_tick: usize,
}

impl Default for LogisticsConfig {
    fn default() -> Self {
        Self {
            warehouse_id: "hub-alpha-syd".to_string(),
            num_packages: 120,
            num_docks: 4,
            dock_rate_per_tick: 2,
        }
    }
}

/// Replication result for the logistics package sorting simulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogisticsReplicationResult {
    pub replication_index: usize,
    pub seed: u64,
    pub packages_processed: usize,
    pub mean_sort_latency_ms: f64,
    pub dock_utilization: f64,
    pub peak_queue_depth: usize,
}

impl IndexedReplication for LogisticsReplicationResult {
    fn replication_index(&self) -> usize {
        self.replication_index
    }
    fn seed(&self) -> u64 {
        self.seed
    }
    fn primary_metric(&self) -> f64 {
        self.mean_sort_latency_ms
    }
}

/// Aggregate metrics across replications for the logistics experiment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogisticsAggregates {
    pub aggregate_mean_sort_latency_ms: f64,
    pub aggregate_mean_utilization: f64,
}

/// Deterministic pseudo-random number generator for package routing.
struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x5DEECE66D)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.0 >> 32) as u32
    }

    fn next_range(&mut self, min: usize, max: usize) -> usize {
        if max <= min {
            return min;
        }
        let span = (max - min) as u32;
        min + (self.next_u32() % span) as usize
    }
}

/// Simulates package sorting and dock routing for one replication.
pub fn simulate_logistics_hub(
    config: &LogisticsConfig,
    replication_index: usize,
    seed: u64,
) -> Result<LogisticsReplicationResult, ExperimentError> {
    let mut rng = SimpleRng::new(seed);
    let mut queue: Vec<usize> = Vec::new(); // package arrival ticks
    let mut latencies: Vec<f64> = Vec::with_capacity(config.num_packages);
    let mut peak_queue = 0;
    let mut dock_busy_ticks = 0usize;

    let mut current_tick = 0usize;
    let mut packages_arrived = 0usize;
    let mut packages_departed = 0usize;

    while packages_departed < config.num_packages || !queue.is_empty() {
        current_tick += 1;

        // Arrival generation
        if packages_arrived < config.num_packages {
            let arrivals_this_tick = rng.next_range(0, 3);
            for _ in 0..arrivals_this_tick {
                if packages_arrived < config.num_packages {
                    queue.push(current_tick);
                    packages_arrived += 1;
                }
            }
        }

        if queue.len() > peak_queue {
            peak_queue = queue.len();
        }

        // Service at docks
        let capacity = config.num_docks * config.dock_rate_per_tick;
        let can_process = queue.len().min(capacity);
        if can_process > 0 {
            dock_busy_ticks += can_process.div_ceil(config.dock_rate_per_tick);
            for _ in 0..can_process {
                let arrival_tick = queue.remove(0);
                let latency =
                    (current_tick - arrival_tick) as f64 * 10.0 + (rng.next_range(5, 20) as f64);
                latencies.push(latency);
                packages_departed += 1;
            }
        }
    }

    let mean_latency = if !latencies.is_empty() {
        latencies.iter().sum::<f64>() / latencies.len() as f64
    } else {
        0.0
    };

    let total_dock_ticks = current_tick * config.num_docks;
    let utilization = if total_dock_ticks > 0 {
        dock_busy_ticks as f64 / total_dock_ticks as f64
    } else {
        0.0
    };

    Ok(LogisticsReplicationResult {
        replication_index,
        seed,
        packages_processed: packages_departed,
        mean_sort_latency_ms: mean_latency,
        dock_utilization: utilization,
        peak_queue_depth: peak_queue,
    })
}

/// Computes aggregates for a batch of logistics replications.
pub fn compute_logistics_aggregates(results: &[LogisticsReplicationResult]) -> LogisticsAggregates {
    if results.is_empty() {
        return LogisticsAggregates {
            aggregate_mean_sort_latency_ms: 0.0,
            aggregate_mean_utilization: 0.0,
        };
    }
    let n = results.len() as f64;
    let mean_lat = results.iter().map(|r| r.mean_sort_latency_ms).sum::<f64>() / n;
    let mean_util = results.iter().map(|r| r.dock_utilization).sum::<f64>() / n;
    LogisticsAggregates {
        aggregate_mean_sort_latency_ms: mean_lat,
        aggregate_mean_utilization: mean_util,
    }
}

struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "careops_logistics_{}_{}_{}",
            prefix,
            std::process::id(),
            current_epoch_ms()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("failed to create temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_logistics_single_replication_determinism() {
    let cfg = LogisticsConfig::default();
    let r1 = simulate_logistics_hub(&cfg, 0, 42).expect("run 1 must succeed");
    let r2 = simulate_logistics_hub(&cfg, 0, 42).expect("run 2 must succeed");
    assert_eq!(r1, r2);
}

#[test]
fn test_logistics_worker_invariance_1_2_4() {
    let cfg = LogisticsConfig::default();
    let base_seed = 1001;
    let replications = 8;

    let runner_fn =
        |idx: usize, seed: u64, meta: &LogisticsConfig| simulate_logistics_hub(meta, idx, seed);

    // Run with 1 worker
    let res_1 = run_generic_batch::<LogisticsReplicationResult, LogisticsConfig, _>(
        "batch-logistics-1w",
        base_seed,
        replications,
        1,
        None,
        cfg.clone(),
        None,
        None,
        None,
        runner_fn,
    )
    .expect("1 worker run must succeed");

    // Run with 2 workers
    let res_2 = run_generic_batch::<LogisticsReplicationResult, LogisticsConfig, _>(
        "batch-logistics-2w",
        base_seed,
        replications,
        2,
        None,
        cfg.clone(),
        None,
        None,
        None,
        runner_fn,
    )
    .expect("2 worker run must succeed");

    // Run with 4 workers
    let res_4 = run_generic_batch::<LogisticsReplicationResult, LogisticsConfig, _>(
        "batch-logistics-4w",
        base_seed,
        replications,
        4,
        None,
        cfg.clone(),
        None,
        None,
        None,
        runner_fn,
    )
    .expect("4 worker run must succeed");

    assert_eq!(res_1.len(), replications);
    assert_eq!(
        res_1, res_2,
        "1-worker and 2-worker results must be bit-identical"
    );
    assert_eq!(
        res_1, res_4,
        "1-worker and 4-worker results must be bit-identical"
    );

    let agg_1 = compute_logistics_aggregates(&res_1);
    let agg_4 = compute_logistics_aggregates(&res_4);
    assert_eq!(agg_1, agg_4);
}

#[test]
fn test_logistics_cancellation_safety() {
    let temp_dir = TestTempDir::new("cancel");
    let cfg = LogisticsConfig::default();
    let token = CancellationToken::new();

    // Cancel prior to dispatch
    token.cancel();

    let runner_fn =
        |idx: usize, seed: u64, meta: &LogisticsConfig| simulate_logistics_hub(meta, idx, seed);

    let res = run_generic_batch::<LogisticsReplicationResult, LogisticsConfig, _>(
        "batch-cancelled",
        42,
        4,
        2,
        None,
        cfg,
        Some(temp_dir.path()),
        Some(&token),
        None,
        runner_fn,
    );

    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind, ExperimentErrorKind::Interrupted);

    // Verify batch_run.json was NOT published
    let final_path = temp_dir.path().join("batch_run.json");
    assert!(
        !final_path.exists(),
        "batch_run.json must never be published on cancellation"
    );
}

#[test]
fn test_logistics_checkpoint_and_resume_recovery() {
    let temp_dir = TestTempDir::new("recovery");
    let cfg = LogisticsConfig::default();
    let base_seed = 5005;
    let replications = 6;

    // First: Run unbroken batch to obtain authoritative baseline
    let runner_fn =
        |idx: usize, seed: u64, meta: &LogisticsConfig| simulate_logistics_hub(meta, idx, seed);

    let baseline = run_generic_batch::<LogisticsReplicationResult, LogisticsConfig, _>(
        "batch-baseline",
        base_seed,
        replications,
        2,
        None,
        cfg.clone(),
        None,
        None,
        None,
        runner_fn,
    )
    .expect("baseline batch must succeed");

    // Second: Run with checkpoint interval 2, but simulate crash/interruption after 2 replications
    let cancel_token = CancellationToken::new();
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let cnt_clone = Arc::clone(&counter);
    let tok_clone = cancel_token.clone();

    let crashing_runner = move |idx: usize, seed: u64, meta: &LogisticsConfig| {
        let n = cnt_clone.fetch_add(1, Ordering::SeqCst);
        if n >= 2 {
            tok_clone.cancel();
        }
        simulate_logistics_hub(meta, idx, seed)
    };

    let partial_res = run_generic_batch::<LogisticsReplicationResult, LogisticsConfig, _>(
        "batch-crashed",
        base_seed,
        replications,
        1,
        Some(2),
        cfg.clone(),
        Some(temp_dir.path()),
        Some(&cancel_token),
        None,
        crashing_runner,
    );

    assert!(partial_res.is_err(), "simulated crashed run must fail");
    let checkpoint_path = temp_dir.path().join("checkpoint.json");
    assert!(
        checkpoint_path.exists(),
        "checkpoint.json must have been written"
    );
    let partial_marker = temp_dir.path().join("batch_run.partial");
    assert!(partial_marker.exists(), "batch_run.partial must be present");
    assert!(
        !temp_dir.path().join("batch_run.json").exists(),
        "batch_run.json must not exist yet"
    );

    // Third: Resume from checkpoint
    let (chk, resumed_results) = resume_generic_batch::<
        LogisticsReplicationResult,
        LogisticsConfig,
        _,
    >(&checkpoint_path, 2, temp_dir.path(), None, None, runner_fn)
    .expect("resumption must succeed");

    assert_eq!(resumed_results.len(), replications);
    assert_eq!(
        resumed_results, baseline,
        "resumed batch must produce identical results to baseline with zero duplicate work"
    );
    assert_eq!(chk.metadata, cfg, "metadata must be restored accurately");

    // Now write final document and verify partial marker is removed
    let aggregates = compute_logistics_aggregates(&resumed_results);
    let doc = GenericBatchRunDocument {
        schema_version: 1,
        batch_id: chk.batch_id,
        status: "complete".to_string(),
        num_workers: 2,
        total_replications: replications,
        replications: resumed_results,
        aggregates,
        metadata: chk.metadata,
        completed_at_epoch_ms: current_epoch_ms(),
    };

    let doc_bytes = serde_json::to_vec_pretty(&doc).unwrap();
    let final_path = temp_dir.path().join("batch_run.json");
    atomic_write_file(&final_path, &doc_bytes).expect("atomic write must succeed");

    assert!(final_path.exists());
    assert!(
        !temp_dir.path().join("batch_run.partial").exists(),
        "batch_run.partial must be removed on clean completion"
    );
}
