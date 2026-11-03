#![cfg(feature = "native-data")]
use careops_ed::capture::{run_captured_batch, CaptureBatch, CaptureLimits, PartStore};
use careops_ed::disposition::api_runner::CancellationToken;
use careops_ed::disposition::arrow_export::generate_arrow_export_bundle;
use careops_ed::disposition::batch_runner::{
    load_completed_batch, resume_batch_from_checkpoint, run_batch_with_workers, BatchConfig,
};
use careops_ed::parquet_capture::{ParquetPartStore, DEFAULT_LIMITS};
use careops_ed::{run_scenario, ScenarioConfig};
use polars::prelude::*;
use std::fs;
use std::io::Cursor;
fn scenario() -> ScenarioConfig {
    serde_json::from_str(include_str!("../examples/nominal_mvp.json")).unwrap()
}
fn config() -> BatchConfig {
    BatchConfig {
        batch_id: "production-capture".into(),
        scenario_id: scenario().scenario_id,
        base_seed: 7,
        replications: 4,
        num_workers: 1,
        horizon_override: Some(8),
        ..Default::default()
    }
}
#[test]
fn maximum_tick_completion_exports_without_invented_spatial_addition() {
    let mut scenario = scenario();
    scenario.patients.truncate(1);
    scenario.horizon_ticks = u64::MAX;
    scenario.patients[0].arrival_tick = u64::MAX - 1;
    scenario.patients[0].work_ticks = 1;
    let mut config = config();
    config.replications = 1;
    config.horizon_override = None;
    let doc = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    assert_eq!(doc.replications[0].completed, 1);
    let bundle = generate_arrow_export_bundle(&scenario, &doc, None, None).unwrap();
    assert_eq!(bundle.patient_records[0].completion_tick, Some(u64::MAX));
    assert!(bundle.spatial_records.is_empty());
    assert_eq!(bundle.spatial_summary.capture_status, "not_captured");
}
#[test]
fn partial_marker_is_absent_when_durable_progress_cannot_be_validated() {
    let root = tempfile::tempdir().unwrap();
    let cancel = CancellationToken::new();
    let callback_cancel = cancel.clone();
    let checkpoint = root.path().join("capture/checkpoint.json");
    let callback_checkpoint = checkpoint.clone();
    let callback = move |p: careops_ed::disposition::batch_runner::BatchProgress| {
        if p.completed_replications == 2 {
            fs::write(&callback_checkpoint, b"{}").unwrap();
            callback_cancel.cancel();
        }
    };
    assert!(run_batch_with_workers(
        &scenario(),
        &config(),
        Some(root.path()),
        Some(&cancel),
        Some(&callback)
    )
    .is_err());
    assert_eq!(fs::read(checkpoint).unwrap(), b"{}");
    assert!(!root.path().join("batch_run.partial").exists());
    assert!(!root.path().join("batch_run.json").exists());
}
#[test]
fn parquet_is_zstd_typed_and_original_horizon_rows_match_across_workers() {
    let s = scenario();
    let mut baseline = None;
    for workers in [1, 2, 4] {
        let root = tempfile::tempdir().unwrap();
        let mut c = config();
        c.num_workers = workers;
        let doc = run_batch_with_workers(&s, &c, Some(root.path()), None, None).unwrap();
        let bundle = generate_arrow_export_bundle(&s, &doc, None, None).unwrap();
        assert_eq!(bundle.provenance.horizon_ticks, 8);
        assert_eq!(bundle.provenance.base_seed, 7);
        for rep in &doc.replications {
            let mut effective = s.clone();
            effective.seed = rep.seed;
            effective.horizon_ticks = 8;
            let original = run_scenario(&effective).unwrap();
            let rows: Vec<_> = bundle
                .patient_records
                .iter()
                .filter(|r| r.replication_index == rep.replication_index)
                .collect();
            assert_eq!(rows.len(), original.patients.len());
            for p in original.patients {
                let r = rows.iter().find(|r| r.patient_id == p.patient_id).unwrap();
                assert_eq!(r.completion_tick, p.completion_tick);
                assert_eq!(r.start_tick, p.start_tick);
                assert_eq!(r.wait_ticks, p.wait_ticks);
                assert_eq!(r.elapsed_ticks, p.elapsed_ticks);
            }
        }
        if let Some(ref expected) = baseline {
            assert_eq!(&bundle.patient_records, expected);
        } else {
            baseline = Some(bundle.patient_records.clone());
        }
        let src = doc.capture.as_ref().unwrap();
        let store = ParquetPartStore::open(&src.directory, src.limits).unwrap();
        let reps = store.completed_replications().unwrap();
        for part in &reps[0].parts {
            let bytes = store.read_part(&part.name, src.limits.max_bytes).unwrap();
            assert_eq!(&bytes[..4], b"PAR1");
            let mut reader = ParquetReader::new(Cursor::new(bytes));
            let md = reader.get_metadata().unwrap();
            for rg in &md.row_groups {
                for col in rg.columns_under_root_iter("patient_id").unwrap() {
                    assert_eq!(format!("{:?}", col.compression().unwrap()), "Zstd");
                }
            }
            let df = reader.finish().unwrap();
            assert_eq!(
                df.column("arrival_tick_lo").unwrap().dtype(),
                &DataType::UInt64
            );
            assert_eq!(df.column("patient_id").unwrap().dtype(), &DataType::String);
            assert_eq!(df.height(), part.rows);
        }
        drop(doc);
        let loaded = load_completed_batch(&root.path().join("batch_run.json")).unwrap();
        assert_eq!(
            generate_arrow_export_bundle(&s, &loaded, None, None)
                .unwrap()
                .patient_records,
            bundle.patient_records
        );
    }
}
#[test]
fn temporary_source_survives_clones_and_missing_or_corrupt_storage_never_falls_back() {
    let s = scenario();
    let doc = run_batch_with_workers(&s, &config(), None, None, None).unwrap();
    let clone = doc.clone();
    let dir = doc.capture.as_ref().unwrap().directory.clone();
    drop(doc);
    assert!(dir.exists());
    generate_arrow_export_bundle(&s, &clone, None, None).unwrap();
    let src = clone.capture.as_ref().unwrap();
    let store = ParquetPartStore::open(&dir, src.limits).unwrap();
    let part = store.part_names().unwrap().remove(0);
    fs::write(dir.join(&part), b"corrupt").unwrap();
    assert!(generate_arrow_export_bundle(&s, &clone, None, None).is_err());
    drop(clone);
    assert!(!dir.exists());
    let mut doc = run_batch_with_workers(&s, &config(), None, None, None).unwrap();
    doc.capture = None;
    assert!(generate_arrow_export_bundle(&s, &doc, None, None)
        .unwrap_err()
        .message
        .contains("cannot rerun"));
}
#[test]
fn production_limits_collision_and_input_drift_reject() {
    let root = tempfile::tempdir().unwrap();
    let limits = CaptureLimits {
        max_rows: 2,
        ..DEFAULT_LIMITS
    };
    let store = ParquetPartStore::open(root.path(), limits).unwrap();
    let s = scenario();
    let mut c = config();
    c.replications = 1;
    let reps = run_captured_batch(&s, &c, limits, root.path(), &store).unwrap();
    assert!(reps[0].parts.len() > 1);
    assert!(reps[0]
        .parts
        .iter()
        .all(|p| p.rows <= 2 && p.bytes <= limits.max_bytes));
    assert!(run_captured_batch(&s, &c, limits, root.path(), &store).is_err());
    let tiny = tempfile::tempdir().unwrap();
    let limits = CaptureLimits {
        max_bytes: 32,
        ..limits
    };
    let store = ParquetPartStore::open(tiny.path(), limits).unwrap();
    assert!(run_captured_batch(&s, &c, limits, tiny.path(), &store).is_err());
    assert!(!store.is_complete().unwrap());
    let mut doc = run_batch_with_workers(&s, &c, None, None, None).unwrap();
    let mut drift = s.clone();
    drift.horizon_ticks += 1;
    assert!(generate_arrow_export_bundle(&drift, &doc, None, None).is_err());
    doc.replications[0].seed += 1;
    assert!(generate_arrow_export_bundle(&s, &doc, None, None).is_err());
}
#[test]
fn actual_cancelled_checkpoint_resumes_and_exports_without_losing_completed_parts() {
    let root = tempfile::tempdir().unwrap();
    let s = scenario();
    let c = config();
    let cancel = CancellationToken::new();
    let cancel_cb = cancel.clone();
    let callback = move |p: careops_ed::disposition::batch_runner::BatchProgress| {
        if p.completed_replications == 2 {
            cancel_cb.cancel();
        }
    };
    assert!(
        run_batch_with_workers(&s, &c, Some(root.path()), Some(&cancel), Some(&callback)).is_err()
    );
    let capture_dir = root.path().join("capture");
    let checkpoint: careops_ed::capture::CaptureCheckpoint =
        serde_json::from_slice(&fs::read(capture_dir.join("checkpoint.json")).unwrap()).unwrap();
    let marker: serde_json::Value =
        serde_json::from_slice(&fs::read(root.path().join("batch_run.partial")).unwrap()).unwrap();
    assert!(checkpoint.completed_replications.len() >= 2);
    assert_eq!(
        marker["completed_replications"],
        checkpoint.completed_replications.len()
    );
    let store = ParquetPartStore::open(&capture_dir, DEFAULT_LIMITS).unwrap();
    let preserved: Vec<_> = store
        .part_names()
        .unwrap()
        .iter()
        .map(|p| (p.clone(), fs::read(capture_dir.join(p)).unwrap()))
        .collect();
    assert!(!preserved.is_empty());
    assert!(!store.is_complete().unwrap());
    let resumed = resume_batch_from_checkpoint(
        &capture_dir.join("checkpoint.json"),
        &s,
        4,
        root.path(),
        None,
    )
    .unwrap();
    for (p, b) in preserved {
        assert_eq!(fs::read(capture_dir.join(p)).unwrap(), b);
    }
    let baseline = run_batch_with_workers(&s, &c, None, None, None).unwrap();
    assert_eq!(resumed.replications, baseline.replications);
    assert_eq!(
        generate_arrow_export_bundle(&s, &resumed, None, None)
            .unwrap()
            .patient_records,
        generate_arrow_export_bundle(&s, &baseline, None, None)
            .unwrap()
            .patient_records
    );
}
#[test]
fn publication_failure_and_symlink_reject_without_overwrite() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("manifest.json")).unwrap();
    let store = ParquetPartStore::open(root.path(), DEFAULT_LIMITS).unwrap();
    assert!(
        run_captured_batch(&scenario(), &config(), DEFAULT_LIMITS, root.path(), &store).is_err()
    );
    #[cfg(unix)]
    {
        let external = tempfile::tempdir().unwrap();
        let link = root.path().join("evil.part");
        std::os::unix::fs::symlink(external.path(), &link).unwrap();
        assert!(store.read_part("evil.part", 1024).is_err());
        assert!(store.read_part("../escape", 1024).is_err());
    }
}
#[test]
fn full_u64_ticks_and_nullable_limbs_roundtrip() {
    let root = tempfile::tempdir().unwrap();
    let store = ParquetPartStore::open(root.path(), DEFAULT_LIMITS).unwrap();
    let b = CaptureBatch {
        model_contract_sha256: careops_ed::capture::MODEL_CONTRACT_SHA256.into(),
        time_unit: "ticks".into(),
        schema: "careops.ed.capture-boundary.v2".into(),
        run_id: "edge".into(),
        input_sha256: "0".repeat(64),
        replication_index: 0,
        seed: u64::MAX,
        first_sequence: 0,
        rows: vec![careops_ed::PatientResult {
            patient_id: "edge".into(),
            arrival_tick: u64::MAX - 2,
            start_tick: None,
            completion_tick: None,
            work_ticks: u64::MAX,
            wait_ticks: None,
            elapsed_ticks: None,
            status: careops_ed::PatientStatus::Unfinished,
        }],
    };
    let mut bytes = Vec::new();
    store.encode_part(&b, &mut bytes).unwrap();
    assert_eq!(store.decode_part(&bytes).unwrap(), b);
}

#[test]
fn partition_finalization_rejects_missing_duplicate_foreign_and_completed_sets() {
    use careops_ed::capture::{capture_replication, finalize_captured_replications};
    let root = tempfile::tempdir().unwrap();
    let s = scenario();
    let mut c = config();
    c.replications = 2;
    let store = ParquetPartStore::open(root.path(), DEFAULT_LIMITS).unwrap();
    let first = capture_replication(&s, &c, DEFAULT_LIMITS, &store, 0).unwrap();
    assert!(finalize_captured_replications(
        &s,
        &c,
        DEFAULT_LIMITS,
        root.path(),
        &store,
        vec![first.clone()]
    )
    .is_err());
    assert!(finalize_captured_replications(
        &s,
        &c,
        DEFAULT_LIMITS,
        root.path(),
        &store,
        vec![first.clone(), first.clone()]
    )
    .is_err());
    let second = capture_replication(&s, &c, DEFAULT_LIMITS, &store, 1).unwrap();
    let mut foreign = c.clone();
    foreign.batch_id = "foreign".into();
    assert!(finalize_captured_replications(
        &s,
        &foreign,
        DEFAULT_LIMITS,
        root.path(),
        &store,
        vec![first.clone(), second.clone()]
    )
    .is_err());
    assert!(!store.is_complete().unwrap());
    assert!(!root.path().join("checkpoint.json").exists());
    let reps = finalize_captured_replications(
        &s,
        &c,
        DEFAULT_LIMITS,
        root.path(),
        &store,
        vec![second, first],
    )
    .unwrap();
    assert!(
        finalize_captured_replications(&s, &c, DEFAULT_LIMITS, root.path(), &store, reps).is_err()
    );
}

#[test]
fn serialized_temporary_locator_does_not_claim_ownership() {
    let s = scenario();
    let doc = run_batch_with_workers(&s, &config(), None, None, None).unwrap();
    let restored: careops_ed::disposition::batch_runner::BatchRunDocument =
        serde_json::from_slice(&serde_json::to_vec(&doc).unwrap()).unwrap();
    generate_arrow_export_bundle(&s, &restored, None, None).unwrap();
    drop(doc);
    assert!(generate_arrow_export_bundle(&s, &restored, None, None).is_err());
}

#[test]
fn production_encoder_propagates_actual_partial_file_write_error() {
    use std::io::{self, Write};
    struct DiskError {
        file: fs::File,
        remaining: usize,
    }
    impl Write for DiskError {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.remaining == 0 {
                return Err(io::Error::other("injected disk exhaustion"));
            }
            let n = self.file.write(&bytes[..bytes.len().min(self.remaining)])?;
            self.remaining -= n;
            Ok(n)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.file.flush()
        }
    }
    let s = scenario();
    let summary = run_scenario(&s).unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = ParquetPartStore::open(root.path(), DEFAULT_LIMITS).unwrap();
    let batch = CaptureBatch {
        model_contract_sha256: careops_ed::capture::MODEL_CONTRACT_SHA256.into(),
        schema: "careops.ed.capture-boundary.v2".into(),
        time_unit: s.time_unit,
        run_id: "disk-error".into(),
        input_sha256: "0".repeat(64),
        replication_index: 0,
        seed: 7,
        first_sequence: 0,
        rows: summary.patients,
    };
    let mut writer = DiskError {
        file: fs::File::create(root.path().join("partial.tmp")).unwrap(),
        remaining: 32,
    };
    assert!(store.encode_part(&batch, &mut writer).is_err());
    assert!(root.path().join("partial.tmp").metadata().unwrap().len() > 0);
    assert!(!store.is_complete().unwrap());
}

#[test]
fn reordered_partitions_keep_rows_bound_to_their_own_replication() {
    use careops_ed::capture::CaptureCheckpoint;
    use careops_ed::disposition::api_runner::compute_sha256_hex;
    use careops_ed::disposition::batch_runner::compute_batch_aggregates;
    let mut s = scenario();
    s.patients.truncate(1);
    s.patients[0].arrival_tick = 0;
    s.patients[0].work_ticks = 1;
    s.horizon_ticks = 20;
    let mut c = config();
    c.replications = 2;
    c.horizon_override = None;
    let mut doc = run_batch_with_workers(&s, &c, None, None, None).unwrap();
    let src = doc.capture.as_ref().unwrap().clone();
    let store = ParquetPartStore::open(&src.directory, src.limits).unwrap();
    let path = src.directory.join("checkpoint.json");
    let mut cp: CaptureCheckpoint = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    // A valid stored partition with distinct timing catches accidental positional
    // association even though this deterministic scenario normally has equal rows.
    let part = &mut cp.completed_replications[1].parts[0];
    let mut batch = store
        .decode_part(&store.read_part(&part.name, src.limits.max_bytes).unwrap())
        .unwrap();
    let row = &mut batch.rows[0];
    row.start_tick = Some(1);
    row.completion_tick = Some(2);
    row.wait_ticks = Some(1);
    row.elapsed_ticks = Some(2);
    let mut bytes = Vec::new();
    store.encode_part(&batch, &mut bytes).unwrap();
    part.bytes = bytes.len();
    part.sha256 = compute_sha256_hex(&bytes);
    fs::write(src.directory.join(&part.name), bytes).unwrap();
    cp.completed_replications[1].summary.mean_wait_ticks = 1.0;
    cp.completed_replications.reverse();
    doc.replications = cp
        .completed_replications
        .iter()
        .map(|r| r.summary.clone())
        .collect();
    let (mean, se, rate) = compute_batch_aggregates(&doc.replications).unwrap();
    doc.aggregate_mean_wait = mean;
    doc.aggregate_std_error = se;
    doc.mean_completion_rate = rate;
    fs::write(&path, serde_json::to_vec(&cp).unwrap()).unwrap();
    fs::remove_file(src.directory.join("manifest.json")).unwrap();
    store.complete(&cp.completed_replications).unwrap();
    let bundle = generate_arrow_export_bundle(&s, &doc, None, None).unwrap();
    assert_eq!(
        bundle
            .patient_records
            .iter()
            .find(|r| r.replication_index == 0)
            .unwrap()
            .wait_ticks,
        Some(0)
    );
    assert_eq!(
        bundle
            .patient_records
            .iter()
            .find(|r| r.replication_index == 1)
            .unwrap()
            .wait_ticks,
        Some(1)
    );
}

#[test]
fn resume_rejects_occupied_destination_before_advancing_checkpoint() {
    let root = tempfile::tempdir().unwrap();
    let destination = tempfile::tempdir().unwrap();
    let s = scenario();
    let c = config();
    let cancel = CancellationToken::new();
    let cb_cancel = cancel.clone();
    let cb = move |p: careops_ed::disposition::batch_runner::BatchProgress| {
        if p.completed_replications == 1 {
            cb_cancel.cancel();
        }
    };
    assert!(run_batch_with_workers(&s, &c, Some(root.path()), Some(&cancel), Some(&cb)).is_err());
    let checkpoint = root.path().join("capture/checkpoint.json");
    let before = fs::read(&checkpoint).unwrap();
    let existing = b"protected unrelated completed batch";
    fs::write(destination.path().join("batch_run.json"), existing).unwrap();
    assert!(resume_batch_from_checkpoint(&checkpoint, &s, 2, destination.path(), None).is_err());
    assert_eq!(fs::read(&checkpoint).unwrap(), before);
    assert_eq!(
        fs::read(destination.path().join("batch_run.json")).unwrap(),
        existing
    );
    assert!(!root.path().join("capture/manifest.json").exists());
}
