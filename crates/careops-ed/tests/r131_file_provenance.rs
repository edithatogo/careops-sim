#![cfg(feature = "native-data")]
use careops_ed::capture::{CaptureBatch, PartStore, MODEL_CONTRACT_SHA256, SCHEMA};
use careops_ed::disposition::batch_runner::{run_batch_with_workers, BatchConfig};
use careops_ed::parquet_capture::{ParquetPartStore, CODEC, DEFAULT_LIMITS, PATIENT_SCHEMA};
use careops_ed::{parse_scenario_json, PatientResult, PatientStatus};
use polars::prelude::*;
use std::io::Cursor;

fn batch() -> CaptureBatch {
    CaptureBatch {
        schema: SCHEMA.into(),
        run_id: "metadata-native".into(),
        input_sha256: "ab".repeat(32),
        model_contract_sha256: MODEL_CONTRACT_SHA256.into(),
        time_unit: "tick".into(),
        replication_index: 2,
        seed: u64::MAX,
        first_sequence: 0,
        rows: vec![PatientResult {
            patient_id: "p,\"q\nline".into(),
            arrival_tick: u64::MAX - 1,
            start_tick: None,
            completion_tick: None,
            work_ticks: 1,
            wait_ticks: None,
            elapsed_ticks: None,
            status: PatientStatus::Unfinished,
        }],
    }
}
fn encoded(store: &ParquetPartStore, b: &CaptureBatch) -> Vec<u8> {
    let mut bytes = Vec::new();
    store.encode_part(b, &mut bytes).unwrap();
    bytes
}
fn metadata(bytes: &[u8]) -> Vec<(String, String)> {
    ParquetReader::new(Cursor::new(bytes))
        .get_metadata()
        .unwrap()
        .key_value_metadata
        .as_ref()
        .unwrap()
        .iter()
        .filter(|kv| kv.key.starts_with("careops."))
        .map(|kv| (kv.key.clone(), kv.value.clone().unwrap()))
        .collect()
}
fn rewrite(bytes: &[u8], kv: Vec<(String, String)>) -> Vec<u8> {
    let mut df = ParquetReader::new(Cursor::new(bytes)).finish().unwrap();
    let mut out = Vec::new();
    ParquetWriter::new(&mut out)
        .with_compression(ParquetCompression::Zstd(None))
        .with_key_value_metadata(Some(KeyValueMetadata::from_static(kv)))
        .finish(&mut df)
        .unwrap();
    out
}
#[test]
fn populated_file_metadata_binds_native_columns_and_limb_nulls() {
    let root = tempfile::tempdir().unwrap();
    let store = ParquetPartStore::open(root.path(), DEFAULT_LIMITS).unwrap();
    let b = batch();
    let bytes = encoded(&store, &b);
    let kv = metadata(&bytes);
    for (key, value) in [
        ("schema", PATIENT_SCHEMA.to_owned()),
        ("codec", CODEC.to_owned()),
        ("run_id", b.run_id.clone()),
        ("effective_input_sha256", b.input_sha256.clone()),
        ("model_contract_sha256", b.model_contract_sha256.clone()),
        ("time_unit", b.time_unit.clone()),
        ("seed", b.seed.to_string()),
        ("replication_index", "2".into()),
        ("first_sequence", "0".into()),
        ("rows", "1".into()),
        (
            "polars_revision",
            "96439685eb2f95108f51087d2b76091917f2593c".into(),
        ),
    ] {
        assert!(
            kv.contains(&(format!("careops.{key}"), value)),
            "missing or wrong {key}"
        );
    }
    assert!(!kv.iter().any(|(k, _)| k.contains("raw_input")));
    assert_eq!(store.decode_part(&bytes).unwrap(), b);
}
#[test]
fn altered_missing_duplicate_and_unknown_reserved_metadata_refuse() {
    let root = tempfile::tempdir().unwrap();
    let store = ParquetPartStore::open(root.path(), DEFAULT_LIMITS).unwrap();
    let bytes = encoded(&store, &batch());
    let kv = metadata(&bytes);
    assert!(!kv.is_empty(), "baseline has no file provenance");
    for i in 0..kv.len() {
        let mut changed = kv.clone();
        changed[i].1.push('x');
        assert!(
            store.decode_part(&rewrite(&bytes, changed)).is_err(),
            "altered {}",
            kv[i].0
        );
        let mut missing = kv.clone();
        missing.remove(i);
        assert!(
            store.decode_part(&rewrite(&bytes, missing)).is_err(),
            "missing {}",
            kv[i].0
        );
        let mut duplicate = kv.clone();
        duplicate.push(kv[i].clone());
        assert!(
            store.decode_part(&rewrite(&bytes, duplicate)).is_err(),
            "duplicate {}",
            kv[i].0
        );
    }
    let mut unknown = kv;
    unknown.push(("careops.unrecognized".into(), "value".into()));
    assert!(store.decode_part(&rewrite(&bytes, unknown)).is_err());
}
#[test]
fn historical_model_identity_reads_but_is_not_relabelled_current() {
    let root = tempfile::tempdir().unwrap();
    let store = ParquetPartStore::open(root.path(), DEFAULT_LIMITS).unwrap();
    let mut b = batch();
    b.model_contract_sha256 = "cd".repeat(32);
    assert_ne!(b.model_contract_sha256, MODEL_CONTRACT_SHA256);
    assert_eq!(store.decode_part(&encoded(&store, &b)).unwrap(), b);
}
#[test]
fn codec_producer_is_frozen_and_unsupported_producer_refuses() {
    let root = tempfile::tempdir().unwrap();
    let store = ParquetPartStore::open(root.path(), DEFAULT_LIMITS).unwrap();
    let b = batch();
    let bytes = encoded(&store, &b);
    let mut declared = metadata(&bytes);
    let producer = declared
        .iter_mut()
        .find(|(key, _)| key == "careops.producer")
        .unwrap();
    // A recorded qualified identity, independent of a future reader package.
    assert_eq!(producer.1, "careops-ed/0.1.0");
    assert_eq!(
        store
            .decode_part(&rewrite(&bytes, declared.clone()))
            .unwrap(),
        b
    );
    declared
        .iter_mut()
        .find(|(key, _)| key == "careops.producer")
        .unwrap()
        .1 = "careops-ed/99.0.0".into();
    assert!(store.decode_part(&rewrite(&bytes, declared)).is_err());
}
#[test]
fn original_run_independent_reader_fixture() {
    let owned = tempfile::tempdir().unwrap();
    let output = std::env::var_os("CAREOPS_R131_READER_DESTINATION").map(std::path::PathBuf::from);
    let root = output.as_deref().unwrap_or(owned.path());
    if output.is_some() {
        std::fs::create_dir(root).expect("fresh explicit fixture destination");
    }
    let scenario = parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let cfg = BatchConfig {
        batch_id: "r131-independent".into(),
        scenario_id: scenario.scenario_id.clone(),
        base_seed: 7,
        replications: 4,
        num_workers: 2,
        horizon_override: Some(8),
        ..Default::default()
    };
    let doc = run_batch_with_workers(&scenario, &cfg, Some(root), None, None).unwrap();
    if output.is_some() {
        let mut effective = scenario.clone();
        effective.seed = 7;
        effective.horizon_ticks = 8;
        std::fs::write(
            root.join("effective-input.json"),
            serde_json::to_vec(&effective).unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("parsed-input.json"),
            serde_json::to_vec(&scenario).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(doc.replications.len(), 4);
    let source = doc.capture.as_ref().unwrap();
    let store = ParquetPartStore::open(&source.directory, source.limits).unwrap();
    assert_eq!(store.completed_replications().unwrap().len(), 4);
}
