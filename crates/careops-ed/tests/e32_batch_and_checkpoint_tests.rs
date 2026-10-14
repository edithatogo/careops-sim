//! Integration tests for independent CPU worker pool, checkpointing, progress tracking,
//! and fail-safe batch execution for Phase E3.2.
//!
//! Verifies:
//! - Strict deterministic equivalence across 1, 2, and 4 CPU workers.
//! - Monotonic progress callback tracking.
//! - Checkpoint creation and atomic serialization.
//! - Cancellation handling and `.partial` run artifact protection.
//! - Checkpoint resumption completing pending work and matching uninterrupted baselines.
//! - Safe loading rejecting incomplete artifacts.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use careops_ed::disposition::api_runner::{CancellationToken, ExitCodeDef};
use careops_ed::disposition::batch_runner::{
    atomic_write_file, compute_batch_aggregates, load_completed_batch,
    resume_batch_from_checkpoint, run_batch_with_workers, BatchCheckpoint, BatchConfig,
    BatchProgress, PartialBatchMarker,
};
use careops_ed::parse_scenario_json;

struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "careops_batch_test_{}_{}_{}",
            prefix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).unwrap();
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

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn load_nominal_scenario() -> careops_ed::ScenarioConfig {
    let p = repo_root().join("crates/careops-ed/examples/nominal_mvp.json");
    let content = fs::read_to_string(&p).expect("read nominal_mvp.json");
    parse_scenario_json(&content).expect("parse nominal_mvp.json")
}

#[test]
fn test_batch_config_default() {
    let cfg = BatchConfig::default();
    assert_eq!(cfg.schema_version, 1);
    assert_eq!(cfg.batch_id, "default-batch");
    assert_eq!(cfg.replications, 4);
    assert_eq!(cfg.num_workers, 2);
    assert_eq!(cfg.checkpoint_interval, Some(2));
}

#[test]
fn test_batch_worker_deterministic_equivalence() {
    let scenario = load_nominal_scenario();

    let cfg_1 = BatchConfig {
        batch_id: "det-eq-1".to_string(),
        scenario_id: "nominal".to_string(),
        base_seed: 12345,
        replications: 4,
        num_workers: 1,
        horizon_override: Some(500),
        checkpoint_interval: None,
        ..Default::default()
    };

    let cfg_2 = BatchConfig {
        batch_id: "det-eq-2".to_string(),
        num_workers: 2,
        ..cfg_1.clone()
    };

    let cfg_4 = BatchConfig {
        batch_id: "det-eq-4".to_string(),
        num_workers: 4,
        ..cfg_1.clone()
    };

    let res_1 = run_batch_with_workers(&scenario, &cfg_1, None, None, None)
        .expect("1 worker batch must succeed");
    let res_2 = run_batch_with_workers(&scenario, &cfg_2, None, None, None)
        .expect("2 worker batch must succeed");
    let res_4 = run_batch_with_workers(&scenario, &cfg_4, None, None, None)
        .expect("4 worker batch must succeed");

    assert_eq!(res_1.total_replications, 4);
    assert_eq!(res_2.total_replications, 4);
    assert_eq!(res_4.total_replications, 4);

    for i in 0..4 {
        let r1 = &res_1.replications[i];
        let r2 = &res_2.replications[i];
        let r4 = &res_4.replications[i];

        assert_eq!(r1.replication_index, i);
        assert_eq!(r2.replication_index, i);
        assert_eq!(r4.replication_index, i);

        assert_eq!(r1.seed, r2.seed);
        assert_eq!(r1.seed, r4.seed);

        assert_eq!(r1.arrivals, r2.arrivals);
        assert_eq!(r1.arrivals, r4.arrivals);

        assert_eq!(r1.started, r2.started);
        assert_eq!(r1.started, r4.started);

        assert_eq!(r1.completed, r2.completed);
        assert_eq!(r1.completed, r4.completed);

        assert_eq!(r1.unfinished, r2.unfinished);
        assert_eq!(r1.unfinished, r4.unfinished);

        assert!(
            (r1.mean_wait_ticks - r2.mean_wait_ticks).abs() < 1e-9,
            "mean wait mismatch rep {i}: {} vs {}",
            r1.mean_wait_ticks,
            r2.mean_wait_ticks
        );
        assert!(
            (r1.mean_wait_ticks - r4.mean_wait_ticks).abs() < 1e-9,
            "mean wait mismatch rep {i}: {} vs {}",
            r1.mean_wait_ticks,
            r4.mean_wait_ticks
        );
    }

    assert!(
        (res_1.aggregate_mean_wait - res_2.aggregate_mean_wait).abs() < 1e-9,
        "aggregate mean wait mismatch 1w vs 2w"
    );
    assert!(
        (res_1.aggregate_mean_wait - res_4.aggregate_mean_wait).abs() < 1e-9,
        "aggregate mean wait mismatch 1w vs 4w"
    );

    assert!(
        (res_1.aggregate_std_error - res_2.aggregate_std_error).abs() < 1e-9,
        "aggregate std error mismatch 1w vs 2w"
    );
    assert!(
        (res_1.aggregate_std_error - res_4.aggregate_std_error).abs() < 1e-9,
        "aggregate std error mismatch 1w vs 4w"
    );

    assert!(
        (res_1.mean_completion_rate - res_2.mean_completion_rate).abs() < 1e-9,
        "mean completion rate mismatch 1w vs 2w"
    );
    assert!(
        (res_1.mean_completion_rate - res_4.mean_completion_rate).abs() < 1e-9,
        "mean completion rate mismatch 1w vs 4w"
    );
}

#[test]
fn test_batch_progress_tracking() {
    let scenario = load_nominal_scenario();
    let recorded_progress = Arc::new(Mutex::new(Vec::new()));
    let rec_clone = Arc::clone(&recorded_progress);

    let callback = move |p: BatchProgress| {
        let mut guard = rec_clone.lock().unwrap();
        guard.push(p);
    };

    let cfg = BatchConfig {
        batch_id: "progress-test".to_string(),
        replications: 4,
        num_workers: 2,
        horizon_override: Some(300),
        ..Default::default()
    };

    let res = run_batch_with_workers(&scenario, &cfg, None, None, Some(&callback))
        .expect("batch with progress must succeed");

    assert_eq!(res.total_replications, 4);

    let progress_list = recorded_progress.lock().unwrap().clone();
    assert_eq!(progress_list.len(), 4);

    let last = progress_list.last().unwrap();
    assert_eq!(last.completed_replications, 4);
    assert!((last.percent_complete - 100.0).abs() < 1e-5);
}

#[test]
fn test_batch_checkpointing_interval() {
    let temp_dir = TestTempDir::new("checkpoint_int");
    let scenario = load_nominal_scenario();

    let cfg = BatchConfig {
        batch_id: "checkpoint-test".to_string(),
        replications: 4,
        num_workers: 2,
        horizon_override: Some(300),
        checkpoint_interval: Some(2),
        ..Default::default()
    };

    let res = run_batch_with_workers(&scenario, &cfg, Some(temp_dir.path()), None, None)
        .expect("batch with checkpoint must succeed");

    assert_eq!(res.status, "complete");

    // Both checkpoint.json and batch_run.json must exist
    let chk_path = temp_dir.path().join("checkpoint.json");
    let run_path = temp_dir.path().join("batch_run.json");

    assert!(chk_path.exists(), "checkpoint.json must exist");
    assert!(run_path.exists(), "batch_run.json must exist");

    // Partial marker must NOT exist on success
    let partial_path = temp_dir.path().join("batch_run.partial");
    assert!(
        !partial_path.exists(),
        "batch_run.partial must not exist on success"
    );

    let chk_bytes = fs::read(&chk_path).unwrap();
    let checkpoint: BatchCheckpoint = serde_json::from_slice(&chk_bytes).unwrap();
    assert_eq!(checkpoint.batch_id, "checkpoint-test");
    assert_eq!(checkpoint.total_replications, 4);
}

#[test]
fn test_batch_cancellation_and_partial_artifact() {
    let temp_dir = TestTempDir::new("cancellation");
    let scenario = load_nominal_scenario();
    let cancel = CancellationToken::new();

    let cfg = BatchConfig {
        batch_id: "cancel-test".to_string(),
        replications: 4,
        num_workers: 1,
        horizon_override: Some(300),
        checkpoint_interval: Some(1),
        ..Default::default()
    };

    // Cancel before or during execution
    cancel.cancel();

    let res = run_batch_with_workers(&scenario, &cfg, Some(temp_dir.path()), Some(&cancel), None);
    assert!(res.is_err(), "cancelled batch must return Err");
    let err = res.unwrap_err();
    assert_eq!(err.code, ExitCodeDef::Interrupted);

    // If pre-cancelled before dispatch, verify output or cancel mid-flight
    let cancel2 = CancellationToken::new();
    let cancel2_clone = cancel2.clone();

    // Cancel on first progress callback
    let callback = move |_p: BatchProgress| {
        cancel2_clone.cancel();
    };

    let res2 = run_batch_with_workers(
        &scenario,
        &cfg,
        Some(temp_dir.path()),
        Some(&cancel2),
        Some(&callback),
    );

    assert!(res2.is_err(), "cancelled batch mid-flight must return Err");
    let err2 = res2.unwrap_err();
    assert_eq!(err2.code, ExitCodeDef::Interrupted);

    let partial_path = temp_dir.path().join("batch_run.partial");
    assert!(
        partial_path.exists(),
        "batch_run.partial must be written on cancellation"
    );

    // Verify loading partial fails with explicit message
    let load_res = load_completed_batch(&partial_path);
    assert!(load_res.is_err());
    let load_err = load_res.unwrap_err();
    assert_eq!(load_err.code, ExitCodeDef::MalformedConfig);
    assert!(load_err
        .message
        .contains("a partial run is clearly incomplete"));
}

#[test]
fn test_checkpoint_resumption_produces_identical_baseline() {
    let temp_dir = TestTempDir::new("resumption");
    let scenario = load_nominal_scenario();

    // 1. Run uninterrupted baseline
    let cfg_baseline = BatchConfig {
        batch_id: "resume-compare".to_string(),
        base_seed: 9999,
        replications: 4,
        num_workers: 2,
        horizon_override: Some(400),
        checkpoint_interval: None,
        ..Default::default()
    };

    let baseline_doc = run_batch_with_workers(&scenario, &cfg_baseline, None, None, None)
        .expect("baseline batch must succeed");

    // 2. Synthesize checkpoint with first 2 replications done
    let checkpoint = BatchCheckpoint {
        schema_version: 1,
        batch_id: "resume-compare".to_string(),
        scenario_id: "nominal-ed-mvp".to_string(),
        base_seed: 9999,
        total_replications: 4,
        horizon_override: Some(400),
        completed_replications: vec![
            baseline_doc.replications[0].clone(),
            baseline_doc.replications[1].clone(),
        ],
        pending_indices: vec![2, 3],
        status: "in_progress".to_string(),
        checkpoint_epoch_ms: 1000,
    };

    let chk_path = temp_dir.path().join("checkpoint.json");
    let chk_bytes = serde_json::to_vec_pretty(&checkpoint).unwrap();
    atomic_write_file(&chk_path, &chk_bytes).unwrap();

    // Write a dummy partial file to verify it gets cleaned up upon resumption success
    let partial_marker = PartialBatchMarker {
        status: "incomplete".to_string(),
        batch_id: "resume-compare".to_string(),
        completed_replications: 2,
        total_replications: 4,
        error: "interrupted".to_string(),
    };
    let partial_path = temp_dir.path().join("batch_run.partial");
    atomic_write_file(
        &partial_path,
        &serde_json::to_vec_pretty(&partial_marker).unwrap(),
    )
    .unwrap();

    // 3. Resume from checkpoint
    let resumed_doc = resume_batch_from_checkpoint(&chk_path, &scenario, 2, temp_dir.path(), None)
        .expect("resumption must succeed");

    assert_eq!(resumed_doc.total_replications, 4);
    assert_eq!(resumed_doc.replications.len(), 4);

    // Verify partial was cleaned up
    assert!(
        !partial_path.exists(),
        "batch_run.partial must be cleaned up on completion"
    );

    // Verify final batch_run.json exists and loads cleanly
    let run_path = temp_dir.path().join("batch_run.json");
    assert!(run_path.exists());
    let loaded = load_completed_batch(&run_path).expect("loaded batch must succeed");
    assert_eq!(loaded.status, "complete");

    // Verify exact equivalence between resumed document and baseline document
    for i in 0..4 {
        assert_eq!(resumed_doc.replications[i], baseline_doc.replications[i]);
    }
    assert!((resumed_doc.aggregate_mean_wait - baseline_doc.aggregate_mean_wait).abs() < 1e-9);
    assert!((resumed_doc.aggregate_std_error - baseline_doc.aggregate_std_error).abs() < 1e-9);
    assert!((resumed_doc.mean_completion_rate - baseline_doc.mean_completion_rate).abs() < 1e-9);
}

#[test]
fn test_resume_already_finished_checkpoint() {
    let temp_dir = TestTempDir::new("resume_finished");
    let scenario = load_nominal_scenario();

    let cfg = BatchConfig {
        batch_id: "already-finished".to_string(),
        base_seed: 555,
        replications: 2,
        num_workers: 1,
        horizon_override: Some(200),
        checkpoint_interval: None,
        ..Default::default()
    };

    let doc = run_batch_with_workers(&scenario, &cfg, None, None, None).unwrap();

    let checkpoint = BatchCheckpoint {
        schema_version: 1,
        batch_id: "already-finished".to_string(),
        scenario_id: "nominal-ed-mvp".to_string(),
        base_seed: 555,
        total_replications: 2,
        horizon_override: Some(200),
        completed_replications: doc.replications.clone(),
        pending_indices: vec![],
        status: "in_progress".to_string(),
        checkpoint_epoch_ms: 2000,
    };

    let chk_path = temp_dir.path().join("checkpoint.json");
    atomic_write_file(&chk_path, &serde_json::to_vec_pretty(&checkpoint).unwrap()).unwrap();

    let resumed = resume_batch_from_checkpoint(&chk_path, &scenario, 1, temp_dir.path(), None)
        .expect("resuming finished checkpoint must succeed");

    assert_eq!(resumed.status, "complete");
    assert_eq!(resumed.total_replications, 2);
    assert_eq!(resumed.replications, doc.replications);
}

#[test]
fn test_load_completed_batch_rejections() {
    let temp_dir = TestTempDir::new("load_rejections");

    // Non-existent path
    let non_existent = temp_dir.path().join("non_existent.json");
    let err = load_completed_batch(&non_existent).unwrap_err();
    assert_eq!(err.code, ExitCodeDef::FileNotFound);

    // Corrupt JSON
    let corrupt_path = temp_dir.path().join("corrupt.json");
    fs::write(&corrupt_path, "not json content").unwrap();
    let err = load_completed_batch(&corrupt_path).unwrap_err();
    assert_eq!(err.code, ExitCodeDef::MalformedConfig);

    // Partial marker masquerading as batch artifact
    let partial_marker = PartialBatchMarker {
        status: "incomplete".to_string(),
        batch_id: "bad".to_string(),
        completed_replications: 1,
        total_replications: 4,
        error: "crashed".to_string(),
    };
    let marker_path = temp_dir.path().join("marked.json");
    fs::write(&marker_path, serde_json::to_vec(&partial_marker).unwrap()).unwrap();
    let err = load_completed_batch(&marker_path).unwrap_err();
    assert_eq!(err.code, ExitCodeDef::MalformedConfig);
    assert!(err.message.contains("a partial run is clearly incomplete"));
}

#[test]
fn test_compute_batch_aggregates_empty() {
    let (mean, se, rate) = compute_batch_aggregates(&[]);
    assert_eq!(mean, 0.0);
    assert_eq!(se, 0.0);
    assert_eq!(rate, 0.0);
}
