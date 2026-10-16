#![forbid(unsafe_code)]

//! Downstream consumer qualification, public API contract, and packaging tests for Task E4.3.
//!
//! # Task E4.3 Qualification Scope
//!
//! Verifies:
//! 1. Downstream consumption of `careops-ed` API from a clean consumer perspective:
//!    - `parse_scenario_json` validation on valid scenarios and rejection of invalid configs.
//!    - `run_experiment` execution with atomic file writes, manifest SHA-256 integrity,
//!      overwrite protection (`ExitCodeDef::FileAlreadyExists`), and disk-full simulation.
//!    - `compare_run_documents` metric comparison with units, denominators, uncertainty (SE),
//!      and incompatible scenario rejection (`ExitCodeDef::IncompatibleScenario`).
//!    - Export functions (Arrow schema metadata, CSV summaries, calibration traces).
//! 2. Package dry-run, dependency pin inspection, and license verification:
//!    - Clean file inventory via `cargo package --locked --list -p careops-ed`.
//!    - SPDX license declaration (`Apache-2.0`).
//!    - Packaging dry-run behavior and identification of unresolved path dependency release holds.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use careops_ed::disposition::api_runner::{
    allocate_replication_seeds, compare_run_documents, compute_sha256_hex, execute_cli_args,
    export_document, inspect_artifact, run_experiment, validate_scenario_source, ArtifactStatus,
    CancellationToken, ExitCodeDef, ExportFormat, RunOptions, ValidationSuccess,
};
use careops_ed::disposition::arrow_export::{
    build_metrics_arrow_schema, build_patient_arrow_schema, build_spatial_arrow_schema,
    compute_uncertainty, export_bundle_to_json, export_spatial_summary_to_csv,
    generate_arrow_export_bundle, generate_parquet_schema_ddl, LocationTransitRecord, PositionKind,
};
use careops_ed::disposition::composite_fixture::{
    calibration_traces_to_csv, export_patient_calibration_traces, ComposedPatient,
    CompositeEdFixture, PatientLifecycle,
};
use careops_ed::disposition::operations::ExecutionFidelity;
use careops_ed::{parse_scenario_json, run_scenario, Eligibility, RunSummary, ScenarioConfig};

struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "careops_e43_test_{}_{}_{}",
            prefix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("create test temp dir");
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
        .expect("parent of careops-ed")
        .parent()
        .expect("parent of crates")
        .to_path_buf()
}

fn nominal_path() -> PathBuf {
    repo_root().join("crates/careops-ed/examples/nominal_mvp.json")
}

fn constrained_path() -> PathBuf {
    repo_root().join("crates/careops-ed/examples/constrained_mvp.json")
}

#[test]
fn test_downstream_parse_scenario_json_nominal_and_rejections() {
    let nominal_source = fs::read_to_string(nominal_path()).expect("read nominal_mvp.json");

    // 1. Nominal scenario parsing must succeed with full structure
    let scenario: ScenarioConfig =
        parse_scenario_json(&nominal_source).expect("nominal scenario must parse cleanly");
    assert_eq!(scenario.schema_version, 1);
    assert_eq!(scenario.scenario_id, "nominal-ed-mvp");
    assert_eq!(scenario.seed, 42);
    assert_eq!(scenario.time_unit, "tick");
    assert_eq!(scenario.horizon_ticks, 20);
    assert_eq!(scenario.patients.len(), 5);

    // Validate using validate_scenario_source
    let val_report: ValidationSuccess =
        validate_scenario_source(&nominal_source).expect("validation report must succeed");
    assert_eq!(val_report.scenario_id, "nominal-ed-mvp");
    assert_eq!(val_report.patient_count, 5);
    assert_eq!(val_report.physical_count, Some(5));
    assert_eq!(val_report.open_count, Some(5));

    // Also verify low-level run_scenario produces correct RunSummary
    let low_level_summary: RunSummary =
        run_scenario(&scenario).expect("run_scenario should execute");
    assert_eq!(low_level_summary.arrivals, 5);
    assert_eq!(low_level_summary.completed, 5);

    // 2. Malformed JSON syntax must be rejected
    let malformed_err = parse_scenario_json("{ not valid json }");
    assert!(malformed_err.is_err(), "malformed json must be rejected");

    // 3. Nonpositive horizon must be rejected by validator
    let zero_horizon_source =
        nominal_source.replace("\"horizon_ticks\": 20", "\"horizon_ticks\": 0");
    let val_zero_horizon = validate_scenario_source(&zero_horizon_source);
    assert!(val_zero_horizon.is_err());
    let err = val_zero_horizon.unwrap_err();
    assert_eq!(err.code, ExitCodeDef::MalformedConfig);
    assert!(err.message.contains("horizon_ticks"));

    // 4. Unknown zone reference must be rejected by parse_scenario_json
    let unknown_zone_source = nominal_source.replace(
        "\"location_id\": \"loc.triage.1\", \"name\": \"Triage Desk 1\", \"zone_id\": \"zone.triage\"",
        "\"location_id\": \"loc.triage.1\", \"name\": \"Triage Desk 1\", \"zone_id\": \"zone.non_existent\"",
    );
    let parse_unknown_zone = parse_scenario_json(&unknown_zone_source);
    assert!(
        parse_unknown_zone.is_err(),
        "unresolved zone reference must fail parsing"
    );
    let err_str = parse_unknown_zone.unwrap_err().to_string();
    assert!(
        err_str.contains("unknown zone") || err_str.contains("unresolved"),
        "error must cite unknown zone: {err_str}"
    );

    // 5. Zero capacity (both physical and open counts 0) must be rejected
    let zero_capacity_source = nominal_source
        .replace("\"physical_count\": 5,", "\"physical_count\": 0,")
        .replace("\"open_count\": 5,", "\"open_count\": 0,")
        .replace("\"staffed_count\": 5,", "\"staffed_count\": 0,");
    let val_zero_cap = validate_scenario_source(&zero_capacity_source);
    assert!(val_zero_cap.is_err());
    let cap_err = val_zero_cap.unwrap_err();
    assert_eq!(cap_err.code, ExitCodeDef::MalformedConfig);
    assert!(cap_err
        .message
        .contains("capacity physical and open counts"));

    // 6. Unknown JSON field must be rejected by deny_unknown_fields
    let unknown_field_source =
        nominal_source.replace("\"seed\": 42,", "\"seed\": 42, \"unexpected_field\": 999,");
    let parse_unknown_field = parse_scenario_json(&unknown_field_source);
    assert!(
        parse_unknown_field.is_err(),
        "unknown field must be rejected"
    );
}

#[test]
fn test_downstream_run_experiment_atomic_and_overwrite() {
    let bytes = fs::read(nominal_path()).expect("read nominal_mvp.json");
    let source = std::str::from_utf8(&bytes).expect("valid utf8");
    let scenario = parse_scenario_json(source).expect("valid scenario");

    let temp_dir = TestTempDir::new("experiment");
    let out_file = temp_dir.path().join("nominal_run.json");

    // 1. Execute run and write output
    let options = RunOptions {
        output_path: Some(out_file.clone()),
        overwrite: false,
        ..Default::default()
    };

    let doc = run_experiment(&scenario, &options, "nominal_mvp.json", &bytes, None)
        .expect("run_experiment must succeed");

    assert_eq!(doc.manifest.scenario_id, "nominal-ed-mvp");
    assert_eq!(doc.manifest.seed, 42);
    assert_eq!(doc.summary.arrivals, 5);
    assert_eq!(doc.summary.completed, 5);
    assert_eq!(doc.summary.unfinished, 0);

    // Verify input SHA-256 matches input bytes
    let expected_sha = compute_sha256_hex(&bytes);
    assert_eq!(doc.manifest.input_sha256, expected_sha);

    // Verify file was written and is valid JSON
    assert!(out_file.exists());
    let file_content = fs::read_to_string(&out_file).expect("read written run file");
    assert!(file_content.contains("\"scenario_id\": \"nominal-ed-mvp\""));
    assert!(file_content.contains("\"schema_version\": 1"));

    // Verify artifact status
    let status = inspect_artifact(&out_file);
    assert!(
        matches!(status, ArtifactStatus::Complete),
        "written artifact must be complete"
    );

    // 2. Overwrite protection: running to same output with overwrite: false must fail
    let res_no_overwrite = run_experiment(&scenario, &options, "nominal_mvp.json", &bytes, None);
    assert!(
        res_no_overwrite.is_err(),
        "must fail when output already exists without overwrite flag"
    );
    let err = res_no_overwrite.unwrap_err();
    assert_eq!(err.code, ExitCodeDef::FileAlreadyExists);

    // 3. Overwrite allowed: running with overwrite: true succeeds
    let overwrite_options = RunOptions {
        output_path: Some(out_file.clone()),
        overwrite: true,
        ..Default::default()
    };
    let res_overwrite = run_experiment(
        &scenario,
        &overwrite_options,
        "nominal_mvp.json",
        &bytes,
        None,
    );
    assert!(
        res_overwrite.is_ok(),
        "must succeed when overwrite flag is true"
    );

    // 4. Simulated disk full returns OutputWriteFailed
    let disk_full_options = RunOptions {
        output_path: Some(temp_dir.path().join("disk_full.json")),
        simulate_disk_full: true,
        ..Default::default()
    };
    let res_disk_full = run_experiment(
        &scenario,
        &disk_full_options,
        "nominal_mvp.json",
        &bytes,
        None,
    );
    assert!(res_disk_full.is_err());
    assert_eq!(
        res_disk_full.unwrap_err().code,
        ExitCodeDef::OutputWriteFailed
    );

    // 5. Cancellation returns Interrupted and does not leave valid artifact
    let cancel_token = CancellationToken::new();
    cancel_token.cancel();
    let cancel_path = temp_dir.path().join("cancelled.json");
    let cancel_options = RunOptions {
        output_path: Some(cancel_path.clone()),
        ..Default::default()
    };
    let res_cancel = run_experiment(
        &scenario,
        &cancel_options,
        "nominal_mvp.json",
        &bytes,
        Some(&cancel_token),
    );
    assert!(res_cancel.is_err());
    assert_eq!(res_cancel.unwrap_err().code, ExitCodeDef::Interrupted);
    let cancel_status = inspect_artifact(&cancel_path);
    assert!(matches!(
        cancel_status,
        ArtifactStatus::NotFound | ArtifactStatus::IncompletePartial(_)
    ));

    // 6. Replication seed allocation determinism
    let seeds = allocate_replication_seeds(100, 5);
    assert_eq!(seeds.len(), 5);
    let distinct_seeds: std::collections::HashSet<_> = seeds.into_iter().collect();
    assert_eq!(distinct_seeds.len(), 5);

    // 7. CLI dispatch execution test
    let help_res = execute_cli_args(&["--help".to_string()]);
    assert_eq!(help_res.exit_code, ExitCodeDef::Success);
    assert!(help_res.stdout.contains("Usage: careops-ed"));
}

#[test]
fn test_downstream_compare_runs_units_denominators_uncertainty() {
    let nom_bytes = fs::read(nominal_path()).expect("read nominal_mvp.json");
    let nom_scen = parse_scenario_json(std::str::from_utf8(&nom_bytes).unwrap()).unwrap();
    let nom_doc = run_experiment(
        &nom_scen,
        &RunOptions::default(),
        "nominal.json",
        &nom_bytes,
        None,
    )
    .expect("run nominal");

    let con_bytes = fs::read(constrained_path()).expect("read constrained_mvp.json");
    let con_scen = parse_scenario_json(std::str::from_utf8(&con_bytes).unwrap()).unwrap();
    let con_doc = run_experiment(
        &con_scen,
        &RunOptions::default(),
        "constrained.json",
        &con_bytes,
        None,
    )
    .expect("run constrained");

    let report = compare_run_documents(&nom_doc, &con_doc).expect("comparison must succeed");
    assert_eq!(report.base_scenario_id, "nominal-ed-mvp");
    assert_eq!(report.candidate_scenario_id, "constrained-ed-mvp");
    assert_eq!(report.time_unit, "tick");

    // Metric: mean_wait
    let mean_wait = report
        .metrics
        .iter()
        .find(|m| m.metric_name == "mean_wait")
        .expect("mean_wait metric required");
    assert_eq!(mean_wait.unit, "tick");
    assert_eq!(
        mean_wait.denominator_description,
        "completed patients with wait intervals"
    );
    assert!((mean_wait.base_value - 0.4).abs() < 1e-4);
    assert!((mean_wait.candidate_value - 4.2).abs() < 1e-4);
    assert!(mean_wait.delta > 0.0, "delta must be positive");
    assert!(mean_wait.percent_change.unwrap() > 0.0);
    assert!(
        mean_wait.base_se.is_some(),
        "base standard error must be present for n > 1"
    );
    assert!(
        mean_wait.candidate_se.is_some(),
        "candidate standard error must be present for n > 1"
    );

    // Metric: arrivals
    let arrivals = report
        .metrics
        .iter()
        .find(|m| m.metric_name == "arrivals")
        .expect("arrivals metric required");
    assert_eq!(arrivals.unit, "patient");
    assert_eq!(arrivals.denominator_description, "total presentation count");
    assert_eq!(arrivals.base_value, 5.0);
    assert_eq!(arrivals.candidate_value, 5.0);
    assert_eq!(arrivals.delta, 0.0);

    // Metric: completed
    let completed = report
        .metrics
        .iter()
        .find(|m| m.metric_name == "completed")
        .expect("completed metric required");
    assert_eq!(completed.unit, "patient");
    assert_eq!(completed.denominator_description, "arrivals");

    // Diagnostics report increased wait
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.contains("increased mean wait")));

    // Rejection of incompatible scenarios: mismatched time units
    let mut incompatible_doc = con_doc.clone();
    incompatible_doc.manifest.time_unit = "second".to_string();
    let comp_err = compare_run_documents(&nom_doc, &incompatible_doc);
    assert!(comp_err.is_err());
    let err = comp_err.unwrap_err();
    assert_eq!(err.code, ExitCodeDef::IncompatibleScenario);
    assert!(err.message.contains("incompatible time units"));

    // Rejection of incompatible schema versions
    let mut bad_version_doc = con_doc;
    bad_version_doc.manifest.schema_version = 99;
    let version_err = compare_run_documents(&nom_doc, &bad_version_doc);
    assert!(version_err.is_err());
    assert_eq!(
        version_err.unwrap_err().code,
        ExitCodeDef::IncompatibleScenario
    );
}

#[test]
fn test_downstream_exports_arrow_csv_and_calibration_traces() {
    let bytes = fs::read(nominal_path()).expect("read nominal_mvp.json");
    let scenario = parse_scenario_json(std::str::from_utf8(&bytes).unwrap()).unwrap();
    let doc = run_experiment(
        &scenario,
        &RunOptions::default(),
        "nominal.json",
        &bytes,
        None,
    )
    .expect("run experiment");

    // 1. Export JSON
    let json_text = export_document(&doc, ExportFormat::Json).expect("export json");
    assert!(json_text.contains("\"scenario_id\": \"nominal-ed-mvp\""));

    // 2. Export CSV
    let csv_text = export_document(&doc, ExportFormat::Csv).expect("export csv");
    assert!(csv_text.contains("patient_id,arrival_tick"));
    assert!(csv_text.contains("patient-001"));

    // 3. Export Arrow schema metadata
    let arrow_schema_text =
        export_document(&doc, ExportFormat::ArrowSchema).expect("export arrow schema");
    assert!(arrow_schema_text.contains("\"arrow_schema\""));
    assert!(arrow_schema_text.contains("\"fields\""));
    assert!(arrow_schema_text.contains("started_patients"));

    // 4. Test Arrow schemas and Parquet DDL generators
    let patient_schema = build_patient_arrow_schema("test-scenario", "tick");
    assert_eq!(patient_schema.schema_name, "careops.ed.patient_records.v1");
    assert!(patient_schema
        .fields
        .iter()
        .any(|f| f.name == "wait_ticks" && f.denominator == Some("started_patients".to_string())));

    let metrics_schema = build_metrics_arrow_schema();
    assert_eq!(metrics_schema.schema_name, "careops.ed.metrics.v1");
    assert!(metrics_schema
        .fields
        .iter()
        .any(|f| f.name == "standard_error"));
    assert!(metrics_schema
        .fields
        .iter()
        .any(|f| f.name == "denominator_name"));

    let spatial_schema = build_spatial_arrow_schema();
    assert_eq!(spatial_schema.schema_name, "careops.ed.spatial_transit.v1");

    let ddl = generate_parquet_schema_ddl();
    assert!(ddl.contains("message careops_ed_patient_record"));

    // Arrow Export Bundle generation from batch run
    let batch_cfg = careops_ed::disposition::batch_runner::BatchConfig {
        schema_version: 1,
        batch_id: "arrow-batch-e43".to_string(),
        scenario_id: "nominal-ed-mvp".to_string(),
        base_seed: 42,
        replications: 2,
        num_workers: 1,
        horizon_override: None,
        checkpoint_interval: None,
    };
    let batch_doc = careops_ed::disposition::batch_runner::run_batch_with_workers(
        &scenario, &batch_cfg, None, None, None,
    )
    .expect("run batch");
    let bundle = generate_arrow_export_bundle(&scenario, &batch_doc, None, None).expect("bundle");
    let bundle_json = export_bundle_to_json(&bundle).expect("bundle json");
    assert!(bundle_json.contains("\"schema_version\""));
    assert!(bundle_json.contains("\"patient_schema\""));
    let spatial_csv = export_spatial_summary_to_csv(&bundle.spatial_summary);
    assert!(spatial_csv.contains("route,traversals,mean_duration_ticks,spatial_revision"));

    // Location transit record inspection
    let transit_record = LocationTransitRecord {
        patient_id: "patient-test".to_string(),
        replication_index: 0,
        step_index: 0,
        from_location: "loc_waiting".to_string(),
        to_location: "loc_bay_1".to_string(),
        distance_metres: 15.0,
        departure_tick: 2,
        arrival_tick: 5,
        transit_ticks: 3,
        position_kind: PositionKind::Observed,
        spatial_revision: "spatial.v1.synthetic-generic-ed".to_string(),
    };
    assert_eq!(transit_record.position_kind, PositionKind::Observed);

    // Uncertainty computation test
    let uncert = compute_uncertainty(&[10.0, 12.0, 14.0, 16.0, 18.0]);
    assert_eq!(uncert.replications_count, 5);
    assert!(uncert.standard_error > 0.0);
    assert!(uncert.ci_95_lower < uncert.ci_95_upper);

    // 5. Calibration traces export
    let test_patients = vec![
        ComposedPatient {
            id: 201,
            name: "Patient_201".to_string(),
            acuity: 2,
            arrival_tick: 2,
            work_required_ticks: 10,
            work_elapsed_ticks: 10,
            start_tick: Some(3),
            completion_tick: Some(13),
            assigned_bay: None,
            lifecycle: PatientLifecycle::Discharged,
            task_class_id: "task.treatment".to_string(),
            zone_id: "zone.acute".to_string(),
            location_id: "loc.bay.1".to_string(),
        },
        ComposedPatient {
            id: 202,
            name: "Patient_202".to_string(),
            acuity: 3,
            arrival_tick: 4,
            work_required_ticks: 6,
            work_elapsed_ticks: 6,
            start_tick: Some(5),
            completion_tick: Some(11),
            assigned_bay: None,
            lifecycle: PatientLifecycle::Discharged,
            task_class_id: "task.treatment".to_string(),
            zone_id: "zone.acute".to_string(),
            location_id: "loc.bay.2".to_string(),
        },
    ];

    let traces = export_patient_calibration_traces(&test_patients, 1, 15);
    assert_eq!(traces.len(), 2);
    assert_eq!(traces[0].case_id, "case_201");
    assert_eq!(traces[0].day, 1);
    assert_eq!(traces[0].start_tick, 3);
    assert_eq!(traces[0].end_tick, 13);
    assert_eq!(traces[0].transit_ticks, 15);
    assert_eq!(traces[0].work_ticks, 10);

    let traces_csv = calibration_traces_to_csv(&traces);
    assert!(traces_csv.contains("case_id,day,start_tick,end_tick,transit_ticks,work_ticks"));
    assert!(traces_csv.contains("case_201,1,3,13,15,10"));
    assert!(traces_csv.contains("case_202,1,5,11,15,6"));

    // Also verify CompositeEdFixture export_calibration_traces
    let eligibilities = vec![
        Eligibility {
            staff_role_id: "staff.ed_nurse".to_string(),
            task_class_id: "task.treatment".to_string(),
            zone_id: "zone.acute".to_string(),
            location_id: Some("loc.bay.1".to_string()),
        },
        Eligibility {
            staff_role_id: "staff.ed_nurse".to_string(),
            task_class_id: "task.treatment".to_string(),
            zone_id: "zone.acute".to_string(),
            location_id: Some("loc.bay.2".to_string()),
        },
    ];
    let mut fixture = CompositeEdFixture::new(
        2,
        2,
        "staff.ed_nurse",
        eligibilities,
        ExecutionFidelity::Macro,
    );
    let mut admitted_p1 = test_patients[0].clone();
    admitted_p1.lifecycle = PatientLifecycle::Arrived;
    admitted_p1.work_elapsed_ticks = 0;
    admitted_p1.start_tick = None;
    admitted_p1.completion_tick = None;

    let mut admitted_p2 = test_patients[1].clone();
    admitted_p2.lifecycle = PatientLifecycle::Arrived;
    admitted_p2.work_elapsed_ticks = 0;
    admitted_p2.start_tick = None;
    admitted_p2.completion_tick = None;

    fixture.admit_patient(admitted_p1);
    fixture.admit_patient(admitted_p2);
    for _ in 0..50 {
        fixture.dispatch_cycle();
        fixture.advance_tick();
        if fixture.completed.len() >= 2 {
            break;
        }
    }
    let fixture_traces = fixture
        .export_calibration_traces(1, &[])
        .expect("fixture trace export must succeed");
    assert_eq!(fixture_traces.len(), 2);
}

#[test]
fn test_downstream_package_dry_run_and_license_verification() {
    let root = repo_root();

    // 1. Inspect package manifest licensing metadata
    let crate_manifest_path = root.join("crates/careops-ed/Cargo.toml");
    let crate_manifest_text =
        fs::read_to_string(&crate_manifest_path).expect("read careops-ed Cargo.toml");
    assert!(
        crate_manifest_text.contains("license.workspace = true"),
        "careops-ed must inherit workspace license"
    );

    let workspace_manifest_path = root.join("Cargo.toml");
    let workspace_manifest_text =
        fs::read_to_string(&workspace_manifest_path).expect("read workspace Cargo.toml");
    assert!(
        workspace_manifest_text.contains("license = \"Apache-2.0\""),
        "workspace must declare Apache-2.0 license"
    );
    assert!(
        workspace_manifest_text.contains("rust-version = \"1.99\""),
        "workspace must declare rust-version 1.99"
    );

    // 2. Test package file inventory via `cargo package --locked --list --allow-dirty -p careops-ed`
    let list_output = Command::new("cargo")
        .args([
            "package",
            "--locked",
            "--list",
            "--allow-dirty",
            "-p",
            "careops-ed",
        ])
        .current_dir(&root)
        .output()
        .expect("execute cargo package --list");

    assert!(
        list_output.status.success(),
        "cargo package --list must succeed. stderr: {}",
        String::from_utf8_lossy(&list_output.stderr)
    );

    let file_list = String::from_utf8_lossy(&list_output.stdout);
    assert!(file_list.contains("Cargo.toml"));
    assert!(file_list.contains("README.md"));
    assert!(file_list.contains("src/lib.rs"));
    assert!(file_list.contains("src/api_runner.rs"));
    assert!(file_list.contains("examples/nominal_mvp.json"));
    assert!(file_list.contains("examples/constrained_mvp.json"));

    // 3. Test package dry-run / release blocker behavior:
    // Calling `cargo publish --locked --dry-run --allow-dirty -p careops-ed` catches that `kairo-ecs-core`
    // is a path dependency without an external crates.io release, formally confirming the
    // documented Kairos release blocker.
    let publish_dry_run = Command::new("cargo")
        .args([
            "publish",
            "--locked",
            "--dry-run",
            "--allow-dirty",
            "-p",
            "careops-ed",
        ])
        .current_dir(&root)
        .output()
        .expect("execute cargo publish --dry-run");

    assert!(
        !publish_dry_run.status.success(),
        "publishing dry-run must halt due to path dependency release blocker"
    );
    let stderr = String::from_utf8_lossy(&publish_dry_run.stderr);
    assert!(
        stderr.contains("kairo-ecs-core") || stderr.contains("version requirement"),
        "dry-run must diagnose unversioned/unreleased path dependency: {stderr}"
    );

    // 4. Verify that cargo package rejects `--dry-run` flag with an argument error
    // (since `--dry-run` is valid on publish, not package)
    let package_dry_run = Command::new("cargo")
        .args(["package", "--locked", "--dry-run", "-p", "careops-ed"])
        .current_dir(&root)
        .output()
        .expect("execute cargo package --dry-run");

    assert!(
        !package_dry_run.status.success(),
        "cargo package does not accept --dry-run"
    );
    let pkg_stderr = String::from_utf8_lossy(&package_dry_run.stderr);
    assert!(
        pkg_stderr.contains("unexpected argument '--dry-run'"),
        "cargo package must report unexpected --dry-run argument: {pkg_stderr}"
    );
}
