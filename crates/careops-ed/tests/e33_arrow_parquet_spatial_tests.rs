//! Integration tests for Arrow/Parquet schemas, metric denominators, statistical uncertainty,
//! spatial summaries, and trajectory replay for Phase E3.3.
//!
//! Verifies:
//! - Arrow/Parquet schemas for patients, metrics, and spatial transit records.
//! - Denominators and statistical uncertainty (mean, variance, SE, 95% CI).
//! - Deterministic equivalence across 1, 2, and 4 workers for export bundles.
//! - Checkpoint resumption equivalence matching uninterrupted baselines.
//! - Position kinds: Observed, Simulated, and Interpolated.
//! - Continuous replay trajectory interpolation.
//! - Spatial summary and CSV export.
//! - CLI integration with `--export-arrow` and `--export-spatial`.

use std::fs;
use std::path::{Path, PathBuf};

use careops_ed::disposition::api_runner::execute_cli_args;
use careops_ed::disposition::arrow_export::{
    build_metrics_arrow_schema, build_patient_arrow_schema, build_spatial_arrow_schema,
    compute_uncertainty, export_bundle_to_json, export_spatial_summary_to_csv,
    generate_arrow_export_bundle, generate_parquet_schema_ddl, interpolate_patient_position,
    LocationTransitRecord, PositionKind,
};
use careops_ed::disposition::batch_runner::{
    resume_batch_from_checkpoint, run_batch_with_workers, BatchConfig,
};
use careops_ed::parse_scenario_json;

struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "careops_arrow_test_{}_{}_{}",
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
fn test_arrow_schemas_and_parquet_ddl() {
    let patient_schema = build_patient_arrow_schema("scenario-1", "tick");
    assert_eq!(patient_schema.schema_name, "careops.ed.patient_records.v1");
    assert!(patient_schema.fields.iter().any(|f| f.name == "patient_id"));
    assert!(patient_schema
        .fields
        .iter()
        .any(|f| f.name == "wait_ticks" && f.denominator == Some("started_patients".to_string())));
    assert!(patient_schema
        .fields
        .iter()
        .any(|f| f.name == "elapsed_ticks"
            && f.denominator == Some("completed_patients".to_string())));

    let metrics_schema = build_metrics_arrow_schema();
    assert_eq!(metrics_schema.schema_name, "careops.ed.metrics.v1");
    assert!(metrics_schema
        .fields
        .iter()
        .any(|f| f.name == "standard_error"));
    assert!(metrics_schema
        .fields
        .iter()
        .any(|f| f.name == "ci_95_lower"));
    assert!(metrics_schema
        .fields
        .iter()
        .any(|f| f.name == "ci_95_upper"));
    assert!(metrics_schema
        .fields
        .iter()
        .any(|f| f.name == "censored_count"));

    let spatial_schema = build_spatial_arrow_schema();
    assert_eq!(spatial_schema.schema_name, "careops.ed.spatial_transit.v1");
    assert!(spatial_schema
        .fields
        .iter()
        .any(|f| f.name == "position_kind"));
    assert!(spatial_schema
        .fields
        .iter()
        .any(|f| f.name == "spatial_revision"));
    assert!(spatial_schema
        .fields
        .iter()
        .any(|f| f.name == "distance_metres"));

    let parquet_ddl = generate_parquet_schema_ddl();
    assert!(parquet_ddl.contains("careops_ed_patient_record"));
    assert!(parquet_ddl.contains("careops_ed_metric_summary"));
    assert!(parquet_ddl.contains("careops_ed_spatial_transit"));
}

#[test]
fn test_statistical_uncertainty_calculator() {
    // Single sample
    let b1 = compute_uncertainty(&[42.0]);
    assert_eq!(b1.variance, 0.0);
    assert_eq!(b1.standard_error, 0.0);
    assert_eq!(b1.ci_95_lower, 42.0);
    assert_eq!(b1.ci_95_upper, 42.0);

    // Identical samples
    let b_const = compute_uncertainty(&[10.0, 10.0, 10.0]);
    assert_eq!(b_const.variance, 0.0);
    assert_eq!(b_const.standard_error, 0.0);
    assert_eq!(b_const.ci_95_lower, 10.0);
    assert_eq!(b_const.ci_95_upper, 10.0);

    // Multi sample: [10.0, 20.0, 30.0] -> mean = 20.0, var = 100.0, SE = sqrt(100/3) ~= 5.7735
    let b_calc = compute_uncertainty(&[10.0, 20.0, 30.0]);
    assert!((b_calc.variance - 100.0).abs() < 1e-6);
    let se = b_calc.standard_error;
    assert!((se - (100.0 / 3.0_f64).sqrt()).abs() < 1e-6);
    let lower = b_calc.ci_95_lower;
    let upper = b_calc.ci_95_upper;
    let mean = 20.0;
    assert!((lower - (mean - 1.959964 * se)).abs() < 1e-6);
    assert!((upper - (mean + 1.959964 * se)).abs() < 1e-6);
}

#[test]
fn test_worker_equivalence_for_arrow_bundles() {
    let scenario = load_nominal_scenario();

    let cfg_1 = BatchConfig {
        batch_id: "arrow-worker-1".to_string(),
        scenario_id: "nominal".to_string(),
        base_seed: 54321,
        replications: 4,
        num_workers: 1,
        horizon_override: Some(500),
        checkpoint_interval: None,
        ..Default::default()
    };

    let cfg_2 = BatchConfig {
        batch_id: "arrow-worker-2".to_string(),
        num_workers: 2,
        ..cfg_1.clone()
    };

    let cfg_4 = BatchConfig {
        batch_id: "arrow-worker-4".to_string(),
        num_workers: 4,
        ..cfg_1.clone()
    };

    let doc_1 = run_batch_with_workers(&scenario, &cfg_1, None, None, None).expect("1-worker");
    let doc_2 = run_batch_with_workers(&scenario, &cfg_2, None, None, None).expect("2-worker");
    let doc_4 = run_batch_with_workers(&scenario, &cfg_4, None, None, None).expect("4-worker");

    let bundle_1 = generate_arrow_export_bundle(&scenario, &doc_1, None, None).expect("bundle 1");
    let bundle_2 = generate_arrow_export_bundle(&scenario, &doc_2, None, None).expect("bundle 2");
    let bundle_4 = generate_arrow_export_bundle(&scenario, &doc_4, None, None).expect("bundle 4");

    // All patient records must match exactly
    assert_eq!(bundle_1.patient_records, bundle_2.patient_records);
    assert_eq!(bundle_1.patient_records, bundle_4.patient_records);

    // All spatial records must match exactly
    assert_eq!(bundle_1.spatial_records, bundle_2.spatial_records);
    assert_eq!(bundle_1.spatial_records, bundle_4.spatial_records);

    // Spatial summaries must match exactly
    assert_eq!(bundle_1.spatial_summary, bundle_2.spatial_summary);
    assert_eq!(bundle_1.spatial_summary, bundle_4.spatial_summary);

    // Metrics with denominators and uncertainty must match exactly
    assert_eq!(bundle_1.metrics, bundle_2.metrics);
    assert_eq!(bundle_1.metrics, bundle_4.metrics);
}

#[test]
fn test_checkpoint_resumption_bundle_equivalence() {
    let tmp = TestTempDir::new("chk_arrow");
    let scenario = load_nominal_scenario();

    let cfg_baseline = BatchConfig {
        batch_id: "chk-arrow-base".to_string(),
        scenario_id: "nominal".to_string(),
        base_seed: 8888,
        replications: 4,
        num_workers: 2,
        horizon_override: Some(500),
        checkpoint_interval: None,
        ..Default::default()
    };

    let doc_base =
        run_batch_with_workers(&scenario, &cfg_baseline, None, None, None).expect("baseline");
    let bundle_base =
        generate_arrow_export_bundle(&scenario, &doc_base, None, None).expect("base bundle");

    // Synthesize checkpoint with first 2 replications done out of 4 total
    let checkpoint = careops_ed::disposition::batch_runner::BatchCheckpoint {
        schema_version: 1,
        batch_id: "chk-arrow-base".to_string(),
        scenario_id: "nominal-ed-mvp".to_string(),
        base_seed: 8888,
        total_replications: 4,
        horizon_override: Some(500),
        completed_replications: doc_base.replications[..2].to_vec(),
        pending_indices: vec![2, 3],
        status: "in_progress".to_string(),
        checkpoint_epoch_ms: 1000,
    };
    let chk_path = tmp.path().join("checkpoint.json");
    careops_ed::disposition::batch_runner::atomic_write_file(
        &chk_path,
        &serde_json::to_vec_pretty(&checkpoint).unwrap(),
    )
    .unwrap();

    // Resume to 4 replications
    let mut resumed_scenario = scenario.clone();
    resumed_scenario.horizon_ticks = 500;
    let doc_resumed =
        resume_batch_from_checkpoint(&chk_path, &resumed_scenario, 2, tmp.path(), None)
            .expect("resumed");
    let bundle_resumed =
        generate_arrow_export_bundle(&scenario, &doc_resumed, None, Some((true, Some(1000))))
            .expect("resumed bundle");

    assert_eq!(bundle_base.patient_records, bundle_resumed.patient_records);
    assert_eq!(bundle_base.spatial_records, bundle_resumed.spatial_records);
    assert_eq!(bundle_base.spatial_summary, bundle_resumed.spatial_summary);
    assert_eq!(bundle_base.metrics, bundle_resumed.metrics);
    assert!(bundle_resumed.provenance.resumed_from_checkpoint);
    assert_eq!(bundle_resumed.provenance.checkpoint_epoch_ms, Some(1000));
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

#[test]
fn test_spatial_summary_and_csv_export() {
    let scenario = load_nominal_scenario();
    let cfg = BatchConfig {
        batch_id: "spatial-summary-test".to_string(),
        scenario_id: "nominal".to_string(),
        base_seed: 777,
        replications: 2,
        num_workers: 1,
        horizon_override: Some(500),
        checkpoint_interval: None,
        ..Default::default()
    };

    let doc = run_batch_with_workers(&scenario, &cfg, None, None, None).expect("batch");
    let bundle = generate_arrow_export_bundle(&scenario, &doc, Some("spatial.v2.custom"), None)
        .expect("bundle");

    assert_eq!(bundle.provenance.spatial_revision, "spatial.v2.custom");
    assert_eq!(bundle.spatial_summary.spatial_revision, "spatial.v2.custom");

    let csv = export_spatial_summary_to_csv(&bundle.spatial_summary);
    assert!(csv.starts_with("route,traversals,mean_duration_ticks,spatial_revision\n"));
    assert!(csv.contains("spatial.v2.custom"));

    let json = export_bundle_to_json(&bundle).expect("json");
    assert!(json.contains("spatial.v2.custom"));
    assert!(json.contains("careops.ed.patient_records.v1"));
}

#[test]
fn test_censored_and_unfinished_counts() {
    let mut scenario = load_nominal_scenario();
    // Constrain horizon so patients remain unfinished / censored
    scenario.horizon_ticks = 10;

    let cfg = BatchConfig {
        batch_id: "censored-test".to_string(),
        scenario_id: scenario.scenario_id.clone(),
        base_seed: 42,
        replications: 2,
        num_workers: 1,
        horizon_override: Some(10),
        checkpoint_interval: None,
        ..Default::default()
    };

    let doc = run_batch_with_workers(&scenario, &cfg, None, None, None).expect("batch");
    let bundle = generate_arrow_export_bundle(&scenario, &doc, None, None).expect("bundle");

    // Verify denominator tracking in metrics
    let wait_m = bundle
        .metrics
        .iter()
        .find(|m| m.name == "mean_wait_ticks")
        .expect("wait metric");
    assert_eq!(wait_m.denominator_name, "started_patients");

    let comp_m = bundle
        .metrics
        .iter()
        .find(|m| m.name == "completion_rate")
        .expect("comp metric");
    assert_eq!(comp_m.denominator_name, "total_arrivals");

    let tp_m = bundle
        .metrics
        .iter()
        .find(|m| m.name == "throughput_per_horizon_tick")
        .expect("throughput metric");
    assert_eq!(tp_m.denominator_name, "horizon_ticks");
    assert_eq!(tp_m.denominator_value, 10.0);

    let censor_m = bundle
        .metrics
        .iter()
        .find(|m| m.name == "censored_presentation_rate")
        .expect("censor metric");
    assert_eq!(censor_m.denominator_name, "total_arrivals");
    assert!(censor_m.value >= 0.0);
}

#[test]
fn test_api_runner_batch_cli_arrow_export() {
    let tmp = TestTempDir::new("cli_arrow");
    let scenario_path = repo_root().join("crates/careops-ed/examples/nominal_mvp.json");
    let arrow_out = tmp.path().join("exports").join("bundle.json");
    let spatial_out = tmp.path().join("exports").join("spatial.csv");

    let args = vec![
        "batch".to_string(),
        scenario_path.display().to_string(),
        "--replications".to_string(),
        "2".to_string(),
        "--workers".to_string(),
        "1".to_string(),
        "--horizon".to_string(),
        "300".to_string(),
        "--export-arrow".to_string(),
        arrow_out.display().to_string(),
        "--export-spatial".to_string(),
        spatial_out.display().to_string(),
        "--spatial-rev".to_string(),
        "spatial.test.rev1".to_string(),
    ];

    let cli_res = execute_cli_args(&args);
    assert_eq!(
        cli_res.exit_code,
        careops_ed::disposition::api_runner::ExitCodeDef::Success,
        "CLI failed with stderr: {}",
        cli_res.stderr
    );
    assert!(arrow_out.exists());
    assert!(spatial_out.exists());

    let arrow_content = fs::read_to_string(&arrow_out).expect("read arrow out");
    assert!(arrow_content.contains("spatial.test.rev1"));
    assert!(arrow_content.contains("careops.ed.patient_records.v1"));
    assert!(arrow_content.contains("careops.ed.metrics.v1"));

    let spatial_content = fs::read_to_string(&spatial_out).expect("read spatial out");
    assert!(spatial_content.contains("spatial.test.rev1"));
    assert!(spatial_content.starts_with("route,traversals,mean_duration_ticks,spatial_revision\n"));
}

#[test]
fn test_export_records_actual_compiler_identity() {
    let scenario = load_nominal_scenario();
    let config = BatchConfig {
        batch_id: "compiler-provenance".to_string(),
        base_seed: 54321,
        replications: 1,
        num_workers: 1,
        ..Default::default()
    };
    let document = run_batch_with_workers(&scenario, &config, None, None, None).expect("batch");
    let bundle = generate_arrow_export_bundle(&scenario, &document, None, None).expect("export");
    let compiler = env!("CAREOPS_ED_BUILD_COMPILER");
    let output = std::process::Command::new(compiler)
        .arg("--version")
        .output()
        .expect("observe compiler independently");
    assert!(output.status.success());
    let observed = String::from_utf8(output.stdout).expect("compiler version text");
    assert_eq!(bundle.provenance.toolchain, observed.trim());
}
