use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use careops_core::experiment::{
    allocate_replication_seeds, resume_generic_batch, run_generic_batch, ExperimentError,
    ExperimentErrorKind, GenericBatchCheckpoint, IndexedReplication,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TestReplication {
    replication_index: usize,
    seed: u64,
}

impl IndexedReplication for TestReplication {
    fn replication_index(&self) -> usize {
        self.replication_index
    }

    fn seed(&self) -> u64 {
        self.seed
    }
}

fn temp_dir(label: &str) -> PathBuf {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "careops_experiment_contract_{label}_{}_{}",
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}

#[test]
fn run_rejects_result_with_wrong_replication_identity() {
    let wrong_index = run_generic_batch::<TestReplication, (), _>(
        "wrong-index",
        17,
        1,
        1,
        None,
        (),
        None,
        None,
        None,
        |_, seed, _| {
            Ok(TestReplication {
                replication_index: 1,
                seed,
            })
        },
    )
    .expect_err("a callback must not change the assigned index");
    assert_eq!(wrong_index.kind, ExperimentErrorKind::ExecutionFailed);
    assert!(wrong_index.message.contains("identity mismatch"));

    let wrong_seed = run_generic_batch::<TestReplication, (), _>(
        "wrong-seed",
        17,
        1,
        1,
        None,
        (),
        None,
        None,
        None,
        |index, seed, _| {
            Ok(TestReplication {
                replication_index: index,
                seed: seed.wrapping_add(1),
            })
        },
    )
    .expect_err("a callback must not change the assigned seed");
    assert_eq!(wrong_seed.kind, ExperimentErrorKind::ExecutionFailed);
    assert!(wrong_seed.message.contains("identity mismatch"));
}

#[test]
fn resume_rejects_malformed_checkpoint_identity_and_index_sets() {
    let output_dir = temp_dir("malformed-checkpoint");
    let checkpoint_path = output_dir.join("checkpoint.json");
    let base_seed = 29;
    let seeds = allocate_replication_seeds(base_seed, 2);
    let checkpoint = GenericBatchCheckpoint {
        schema_version: 1,
        batch_id: "malformed".to_string(),
        base_seed,
        total_replications: 2,
        completed_replications: vec![TestReplication {
            replication_index: 0,
            seed: seeds[0].wrapping_add(1),
        }],
        pending_indices: vec![1],
        status: "in_progress".to_string(),
        checkpoint_epoch_ms: 0,
        metadata: (),
    };
    fs::write(&checkpoint_path, serde_json::to_vec(&checkpoint).unwrap())
        .expect("checkpoint should be written");

    let callback_calls = AtomicUsize::new(0);
    let result = resume_generic_batch::<TestReplication, (), _>(
        &checkpoint_path,
        1,
        &output_dir,
        None,
        None,
        |index, seed, _| {
            callback_calls.fetch_add(1, Ordering::SeqCst);
            Ok(TestReplication {
                replication_index: index,
                seed,
            })
        },
    );

    let error: ExperimentError = result.expect_err("invalid checkpoint identity must be rejected");
    assert_eq!(error.kind, ExperimentErrorKind::MalformedConfig);
    assert!(error.message.contains("identity mismatch"));
    assert_eq!(callback_calls.load(Ordering::SeqCst), 0);

    let duplicate_checkpoint = GenericBatchCheckpoint {
        completed_replications: vec![TestReplication {
            replication_index: 0,
            seed: seeds[0],
        }],
        pending_indices: vec![0, 1],
        ..checkpoint
    };
    fs::write(
        &checkpoint_path,
        serde_json::to_vec(&duplicate_checkpoint).unwrap(),
    )
    .expect("duplicate checkpoint should be written");
    let error = resume_generic_batch::<TestReplication, (), _>(
        &checkpoint_path,
        1,
        &output_dir,
        None,
        None,
        |index, seed, _| {
            Ok(TestReplication {
                replication_index: index,
                seed,
            })
        },
    )
    .expect_err("overlapping completed and pending indices must be rejected");
    assert_eq!(error.kind, ExperimentErrorKind::MalformedConfig);
    assert!(error.message.contains("both completed and pending"));

    let out_of_range_checkpoint = GenericBatchCheckpoint {
        pending_indices: vec![2],
        ..duplicate_checkpoint
    };
    fs::write(
        &checkpoint_path,
        serde_json::to_vec(&out_of_range_checkpoint).unwrap(),
    )
    .expect("out-of-range checkpoint should be written");
    let error = resume_generic_batch::<TestReplication, (), _>(
        &checkpoint_path,
        1,
        &output_dir,
        None,
        None,
        |index, seed, _| {
            Ok(TestReplication {
                replication_index: index,
                seed,
            })
        },
    )
    .expect_err("out-of-range pending indices must be rejected before seed lookup");
    assert_eq!(error.kind, ExperimentErrorKind::MalformedConfig);
    assert!(error
        .message
        .contains("pending replication index 2 is out of range"));
}
