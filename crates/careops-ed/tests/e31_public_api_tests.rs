//! Native public API qualification. CLI lifecycle and retired export flags are covered in
//! careops-ed-cli/tests/consumer_contract.rs; native tables replace full-table JSON.
use careops_ed::disposition::api_runner::{
    allocate_replication_seeds, compute_sha256_hex, validate_scenario_source, ExitCodeDef,
    ValidationSuccess,
};
#[cfg(feature = "native-data")]
use careops_ed::disposition::batch_runner::{run_batch_with_workers, BatchConfig};
#[cfg(feature = "native-data")]
use careops_ed::disposition::native_output::{
    export_native_batch, read_native_dataset, ExportLimits, NativeDataset, NativeFormat,
};
#[cfg(feature = "native-data")]
use careops_ed::parse_scenario_json;
use std::fs;
use std::path::PathBuf;
#[cfg(feature = "native-data")]
fn tempdir(prefix: &str) -> tempfile::TempDir {
    tempfile::Builder::new().prefix(prefix).tempdir().unwrap()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn nominal_path() -> PathBuf {
    repo_root().join("crates/careops-ed/examples/nominal_mvp.json")
}

fn malformed_path() -> PathBuf {
    repo_root().join("crates/careops-ed/examples/malformed_mvp.json")
}
#[test]
fn test_sha256_hex_matches_known_standard() {
    let empty_sha = compute_sha256_hex(b"");
    assert_eq!(
        empty_sha,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );

    let test_sha = compute_sha256_hex(b"careops-sim-deterministic-test");
    assert_eq!(test_sha.len(), 64);
}

#[test]
fn test_validate_scenario_source_nominal() {
    let content = fs::read_to_string(nominal_path()).expect("read nominal_mvp.json");
    let res = validate_scenario_source(&content);
    assert!(res.is_ok(), "nominal scenario must be valid");
    let report: ValidationSuccess = res.unwrap();
    assert_eq!(report.scenario_id, "nominal-ed-mvp");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.horizon_ticks, 20);
    assert_eq!(report.time_unit, "tick");
    assert_eq!(report.patient_count, 5);
    assert_eq!(report.physical_count, Some(5));
}

#[test]
fn test_validate_scenario_source_malformed_rejected() {
    let content = fs::read_to_string(malformed_path()).expect("read malformed_mvp.json");
    let res = validate_scenario_source(&content);
    assert!(res.is_err(), "malformed scenario must be rejected");
    let err = res.unwrap_err();
    assert_eq!(err.code, ExitCodeDef::MalformedConfig);
}

#[test]
fn test_validate_scenario_invalid_json() {
    let res = validate_scenario_source("{ invalid json }");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code, ExitCodeDef::MalformedConfig);
}

#[test]
fn test_validate_scenario_zero_horizon() {
    let content = fs::read_to_string(nominal_path()).unwrap();
    let zero_horizon = content.replace("\"horizon_ticks\": 20", "\"horizon_ticks\": 0");
    let res = validate_scenario_source(&zero_horizon);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code, ExitCodeDef::MalformedConfig);
}

#[test]
fn test_replication_seed_allocation_uniqueness() {
    let base_seed = 12345;
    let count = 20;
    let seeds = allocate_replication_seeds(base_seed, count);
    assert_eq!(seeds.len(), count);

    let mut set = std::collections::HashSet::new();
    for s in seeds {
        assert!(set.insert(s), "all allocated seeds must be distinct");
    }
}

#[cfg(feature = "native-data")]
fn native(seed: u64, horizon: Option<u64>) -> NativeDataset {
    let scenario = parse_scenario_json(&fs::read_to_string(nominal_path()).unwrap()).unwrap();
    let tmp = tempdir("native-public");
    let config = BatchConfig {
        base_seed: seed,
        replications: 1,
        num_workers: 1,
        horizon_override: horizon,
        checkpoint_interval: None,
        ..Default::default()
    };
    let document =
        run_batch_with_workers(&scenario, &config, Some(tmp.path()), None, None).unwrap();
    let destination = tmp.path().join("data");
    fs::create_dir(&destination).unwrap();
    export_native_batch(
        &scenario,
        &document,
        &destination,
        NativeFormat::Parquet,
        ExportLimits::default(),
    )
    .unwrap();
    read_native_dataset(&destination).unwrap()
}
#[cfg(feature = "native-data")]
#[test]
fn test_native_nominal_success_and_parsed_input_hash() {
    let data = native(42, None);
    let manifest = data.manifest();
    assert_eq!(manifest.schema, "careops.ed.native-output.v1");
    assert_eq!(manifest.scenario_id, "nominal-ed-mvp");
    assert_eq!(manifest.base_seed, 42);
    assert_eq!(manifest.time_unit, "tick");
    assert_eq!(data.patients().height(), 5);
    for (name, expected) in [("arrivals", 5), ("completed", 5), ("unfinished", 0)] {
        assert_eq!(
            data.replications()
                .column(name)
                .unwrap()
                .u64()
                .unwrap()
                .get(0),
            Some(expected)
        );
    }
    let scenario = parse_scenario_json(&fs::read_to_string(nominal_path()).unwrap()).unwrap();
    assert_eq!(
        manifest.parsed_input_sha256,
        compute_sha256_hex(&serde_json::to_vec(&scenario).unwrap())
    );
    // Raw-file identity belongs to the CLI invocation, not a fabricated native manifest field.
    assert_eq!(manifest.raw_input_sha256, None);
}
#[cfg(feature = "native-data")]
#[test]
fn test_native_seed_override() {
    let data = native(9999, None);
    assert_eq!(data.manifest().base_seed, 9999);
    assert_eq!(
        data.replications()
            .column("seed")
            .unwrap()
            .u64()
            .unwrap()
            .get(0),
        Some(9999)
    );
}
#[cfg(feature = "native-data")]
#[test]
fn test_native_horizon_override_preserves_censoring() {
    let data = native(42, Some(7));
    assert_eq!(data.manifest().horizon_ticks, 7);
    assert!(
        data.patients()
            .column("completion_tick_lo")
            .unwrap()
            .null_count()
            > 0
    );
    let reps = data.replications();
    assert_eq!(
        reps.column("horizon_ticks").unwrap().u64().unwrap().get(0),
        Some(7)
    );
    assert!(
        reps.column("completed")
            .unwrap()
            .u64()
            .unwrap()
            .get(0)
            .unwrap()
            < 5
    );
    assert!(
        reps.column("unfinished")
            .unwrap()
            .u64()
            .unwrap()
            .get(0)
            .unwrap()
            > 0
    );
}
