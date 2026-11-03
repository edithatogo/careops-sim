//! Actual native files and typed frames replace retired descriptor DDL and JSON bundles.
//! No movement is captured by this model: only patients, replications and metrics are published.
use careops_ed::disposition::arrow_export::{
    compute_uncertainty, interpolate_patient_position, LocationTransitRecord, PositionKind,
};
#[cfg(feature = "native-data")]
use careops_ed::disposition::batch_runner::{
    resume_batch_from_checkpoint, run_batch_with_workers, BatchConfig, BatchRunDocument,
};
#[cfg(feature = "native-data")]
use careops_ed::disposition::native_output::{
    export_native_batch, read_native_dataset, ExportLimits, NativeDataset, NativeFormat,
};
#[cfg(feature = "native-data")]
use careops_ed::parse_scenario_json;
#[cfg(feature = "native-data")]
use std::fs;
#[cfg(feature = "native-data")]
use std::path::{Path, PathBuf};
#[cfg(feature = "native-data")]
fn publish(
    scenario: &careops_ed::ScenarioConfig,
    document: &BatchRunDocument,
    root: &Path,
) -> NativeDataset {
    let destination = root.join("data");
    fs::create_dir(&destination).unwrap();
    export_native_batch(
        scenario,
        document,
        &destination,
        NativeFormat::Parquet,
        ExportLimits::default(),
    )
    .unwrap();
    read_native_dataset(&destination).unwrap()
}
#[cfg(feature = "native-data")]
fn snapshot(scenario: &careops_ed::ScenarioConfig, config: &BatchConfig) -> NativeDataset {
    let root = tempdir("snapshot");
    let doc = run_batch_with_workers(scenario, config, Some(root.path()), None, None).unwrap();
    publish(scenario, &doc, root.path())
}
#[cfg(feature = "native-data")]
fn equal_frames(a: &NativeDataset, b: &NativeDataset) {
    assert!(a.patients().equals_missing(b.patients()));
    assert!(a.replications().equals_missing(b.replications()));
    assert!(a.metrics().equals_missing(b.metrics()));
}
#[cfg(feature = "native-data")]
fn metric(data: &NativeDataset, name: &str) -> usize {
    let names = data.metrics().column("name").unwrap().str().unwrap();
    (0..data.metrics().height())
        .find(|&row| names.get(row) == Some(name))
        .unwrap()
}

#[cfg(feature = "native-data")]
fn tempdir(prefix: &str) -> tempfile::TempDir {
    tempfile::Builder::new().prefix(prefix).tempdir().unwrap()
}

#[cfg(feature = "native-data")]
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[cfg(feature = "native-data")]
fn load_nominal_scenario() -> careops_ed::ScenarioConfig {
    let p = repo_root().join("crates/careops-ed/examples/nominal_mvp.json");
    let content = fs::read_to_string(&p).expect("read nominal_mvp.json");
    parse_scenario_json(&content).expect("parse nominal_mvp.json")
}

#[test]
fn test_statistical_uncertainty_calculator() {
    // Single sample
    let b1 = compute_uncertainty(&[42.0]).unwrap();
    assert_eq!(b1.variance, None);
    assert_eq!(b1.standard_error, None);
    assert_eq!(b1.ci_95_lower, None);
    assert_eq!(b1.ci_95_upper, None);

    // Identical samples
    let b_const = compute_uncertainty(&[10.0, 10.0, 10.0]).unwrap();
    assert_eq!(b_const.variance, Some(0.0));
    assert_eq!(b_const.standard_error, Some(0.0));
    assert_eq!(b_const.ci_95_lower, Some(10.0));
    assert_eq!(b_const.ci_95_upper, Some(10.0));

    // Multi sample: [10.0, 20.0, 30.0] -> mean = 20.0, var = 100.0, SE = sqrt(100/3) ~= 5.7735
    let b_calc = compute_uncertainty(&[10.0, 20.0, 30.0]).unwrap();
    assert!((b_calc.variance.unwrap() - 100.0).abs() < 1e-6);
    let se = b_calc.standard_error.unwrap();
    assert!((se - (100.0 / 3.0_f64).sqrt()).abs() < 1e-6);
    let lower = b_calc.ci_95_lower.unwrap();
    let upper = b_calc.ci_95_upper.unwrap();
    let mean = 20.0;
    assert!((lower - (mean - 1.959964 * se)).abs() < 1e-6);
    assert!((upper - (mean + 1.959964 * se)).abs() < 1e-6);
}
#[test]
fn test_trajectory_interpolation_and_position_kinds() {
    let transit = LocationTransitRecord {
        patient_id: "P1".to_string(),
        replication_index: 0,
        step_index: 0,
        from_location: "loc_arrival".to_string(),
        to_location: "loc_bed".to_string(),
        distance_metres: 25.0,
        departure_tick: 10,
        arrival_tick: 20,
        transit_ticks: 10,
        position_kind: PositionKind::Simulated,
        spatial_revision: "rev.test".to_string(),
    };

    let transits = vec![transit];

    // Before departure: Observed at arrival
    let pos_before = interpolate_patient_position(&transits, 5).expect("pos before");
    assert_eq!(pos_before.position_kind, PositionKind::Observed);
    assert_eq!(pos_before.transit_progress, 0.0);
    assert_eq!(pos_before.current_location, "loc_arrival");

    // At departure: Observed at arrival
    let pos_start = interpolate_patient_position(&transits, 10).expect("pos start");
    assert_eq!(pos_start.position_kind, PositionKind::Observed);
    assert_eq!(pos_start.transit_progress, 0.0);
    assert_eq!(pos_start.current_location, "loc_arrival");

    // At arrival: Observed at destination
    let pos_end = interpolate_patient_position(&transits, 20).expect("pos end");
    assert_eq!(pos_end.position_kind, PositionKind::Observed);
    assert_eq!(pos_end.transit_progress, 1.0);
    assert_eq!(pos_end.current_location, "loc_bed");

    // Mid-transit at tick 15: Interpolated with progress 0.5
    let pos_mid = interpolate_patient_position(&transits, 15).expect("pos mid");
    assert_eq!(pos_mid.position_kind, PositionKind::Interpolated);
    assert!((pos_mid.transit_progress - 0.5).abs() < 1e-6);
    assert_eq!(pos_mid.current_location, "loc_arrival");
    assert_eq!(pos_mid.destination_location, Some("loc_bed".to_string()));
}

#[cfg(feature = "native-data")]
#[test]
fn test_actual_native_schemas_and_no_manufactured_spatial_capture() {
    let data = snapshot(&load_nominal_scenario(), &BatchConfig::default());
    let tables = &data.manifest().tables;
    assert_eq!(
        tables.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        ["patients", "replications", "metrics"]
    );
    for (name, schema) in [
        ("patients", "careops.ed.patients.v2"),
        ("replications", "careops.ed.replications.v1"),
        ("metrics", "careops.ed.metrics.v1"),
    ] {
        let table = tables.iter().find(|t| t.name == name).unwrap();
        assert_eq!(table.schema, schema);
        assert!(!table.parts.is_empty());
    }
    let patient = tables.iter().find(|t| t.name == "patients").unwrap();
    assert!(patient
        .columns
        .iter()
        .any(|c| c.name == "wait_ticks_lo" && c.nullable));
    let metrics = tables.iter().find(|t| t.name == "metrics").unwrap();
    for name in [
        "value",
        "variance",
        "standard_error",
        "ci_95_lower",
        "ci_95_upper",
        "denominator_value",
    ] {
        assert!(
            metrics
                .columns
                .iter()
                .find(|c| c.name == name)
                .unwrap()
                .nullable
        );
    }
    assert_eq!(data.patients().height(), 20);
}
#[cfg(feature = "native-data")]
#[test]
fn test_worker_equivalence_for_native_frames() {
    let scenario = load_nominal_scenario();
    let config = BatchConfig {
        batch_id: "native-workers".into(),
        base_seed: 54321,
        replications: 4,
        num_workers: 1,
        horizon_override: Some(500),
        checkpoint_interval: None,
        ..Default::default()
    };
    let baseline = snapshot(&scenario, &config);
    for workers in [2, 4] {
        // Same identity is required because run_id is authoritative table data.
        equal_frames(
            &baseline,
            &snapshot(
                &scenario,
                &BatchConfig {
                    num_workers: workers,
                    ..config.clone()
                },
            ),
        );
    }
}
#[cfg(feature = "native-data")]
#[test]
fn test_checkpoint_resumption_native_equivalence() {
    let tmp = tempdir("resume");
    let scenario = load_nominal_scenario();
    let config = BatchConfig {
        batch_id: "native-resume".into(),
        base_seed: 8888,
        replications: 4,
        num_workers: 2,
        horizon_override: Some(500),
        checkpoint_interval: None,
        ..Default::default()
    };
    let baseline = snapshot(&scenario, &config);
    let cancel = careops_ed::disposition::api_runner::CancellationToken::new();
    let cancel_cb = cancel.clone();
    let acknowledged = std::sync::atomic::AtomicUsize::new(0);
    let progress = |p: careops_ed::disposition::batch_runner::BatchProgress| {
        acknowledged.store(
            p.completed_replications,
            std::sync::atomic::Ordering::SeqCst,
        );
        if p.completed_replications == 2 {
            cancel_cb.cancel();
        }
    };
    let interrupted = run_batch_with_workers(
        &scenario,
        &BatchConfig {
            num_workers: 1,
            ..config.clone()
        },
        Some(tmp.path()),
        Some(&cancel),
        Some(&progress),
    );
    assert_eq!(
        interrupted.unwrap_err().code,
        careops_ed::disposition::api_runner::ExitCodeDef::Interrupted
    );
    assert_eq!(acknowledged.load(std::sync::atomic::Ordering::SeqCst), 2);
    let checkpoint = tmp.path().join("capture/checkpoint.json");
    assert!(checkpoint.is_file());
    let resumed =
        resume_batch_from_checkpoint(&checkpoint, &scenario, 2, tmp.path(), None).unwrap();
    equal_frames(&baseline, &publish(&scenario, &resumed, tmp.path()));
}
#[cfg(feature = "native-data")]
#[test]
fn test_censored_and_unfinished_native_denominators() {
    let mut scenario = load_nominal_scenario();
    scenario.horizon_ticks = 10;
    let data = snapshot(
        &scenario,
        &BatchConfig {
            replications: 2,
            num_workers: 1,
            horizon_override: Some(10),
            checkpoint_interval: None,
            ..Default::default()
        },
    );
    for (name, denominator) in [
        ("mean_wait_ticks", "included_replications"),
        ("completion_rate", "total_arrivals"),
        (
            "throughput_per_horizon_tick",
            "per_replication_horizon_ticks",
        ),
        ("censored_presentation_rate", "included_replications"),
    ] {
        let row = metric(&data, name);
        assert_eq!(
            data.metrics()
                .column("denominator_name")
                .unwrap()
                .str()
                .unwrap()
                .get(row),
            Some(denominator)
        );
    }
    assert_eq!(
        data.metrics()
            .column("denominator_value")
            .unwrap()
            .f64()
            .unwrap()
            .get(metric(&data, "throughput_per_horizon_tick")),
        None
    );
    assert!(
        data.metrics()
            .column("value")
            .unwrap()
            .f64()
            .unwrap()
            .get(metric(&data, "censored_presentation_rate"))
            .unwrap()
            >= 0.0
    );
}
#[cfg(feature = "native-data")]
#[test]
fn test_export_binds_actual_compiler_identity() {
    let observed = std::process::Command::new(env!("CAREOPS_ED_BUILD_COMPILER"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(observed.status.success());
    assert_eq!(
        String::from_utf8(observed.stdout).unwrap().trim(),
        env!("CAREOPS_ED_BUILD_RUSTC")
    );
    let data = snapshot(
        &load_nominal_scenario(),
        &BatchConfig {
            replications: 1,
            num_workers: 1,
            ..Default::default()
        },
    );
    assert_eq!(
        data.manifest().model_contract_sha256,
        careops_ed::capture::MODEL_CONTRACT_SHA256
    );
}

#[cfg(feature = "native-data")]
#[test]
fn test_native_uncertainty_uses_replication_sample_unit() {
    for replications in [1, 4] {
        let data = snapshot(
            &load_nominal_scenario(),
            &BatchConfig {
                replications,
                num_workers: 1,
                ..Default::default()
            },
        );
        let row = metric(&data, "mean_wait_ticks");
        assert_eq!(
            data.metrics()
                .column("unit")
                .unwrap()
                .str()
                .unwrap()
                .get(row),
            Some("tick")
        );
        assert_eq!(
            data.metrics()
                .column("included_replications")
                .unwrap()
                .u64()
                .unwrap()
                .get(row),
            Some(replications as u64)
        );
        let se = data
            .metrics()
            .column("standard_error")
            .unwrap()
            .f64()
            .unwrap()
            .get(row);
        assert_eq!(se, if replications == 1 { None } else { Some(0.0) });
    }
}
