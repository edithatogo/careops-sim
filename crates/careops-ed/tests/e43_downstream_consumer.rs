#![forbid(unsafe_code)]
//! Downstream qualification uses native typed datasets; CLI lifecycle is qualified in the CLI crate.
//! Descriptor-only DDL, full-table JSON and uncaptured spatial CSV are retired.
use careops_ed::disposition::api_runner::{
    allocate_replication_seeds, validate_scenario_source, ExitCodeDef, ValidationSuccess,
};
use careops_ed::disposition::arrow_export::{
    compute_uncertainty, LocationTransitRecord, PositionKind,
};
#[cfg(feature = "native-data")]
use careops_ed::disposition::batch_runner::{run_batch_with_workers, BatchConfig};
#[cfg(feature = "native-data")]
use careops_ed::disposition::composite_fixture::native_calibration::calibration_table;
use careops_ed::disposition::composite_fixture::{
    export_patient_calibration_traces, ComposedPatient, CompositeEdFixture, PatientLifecycle,
};
#[cfg(feature = "native-data")]
use careops_ed::disposition::native_output::{
    export_native_batch, read_native_dataset, ExportLimits, NativeFormat,
};
use careops_ed::disposition::operations::ExecutionFidelity;
use careops_ed::{parse_scenario_json, run_scenario, Eligibility, RunSummary, ScenarioConfig};
use std::fs;
#[cfg(feature = "native-data")]
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
#[cfg(feature = "native-data")]
struct TestTempDir {
    path: PathBuf,
}

#[cfg(feature = "native-data")]
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

#[cfg(feature = "native-data")]
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

#[cfg(feature = "native-data")]
#[test]
fn test_downstream_native_files_and_seed_allocation() {
    let scenario = parse_scenario_json(&fs::read_to_string(nominal_path()).unwrap()).unwrap();
    let root = TestTempDir::new("native-downstream");
    let config = BatchConfig {
        replications: 1,
        num_workers: 1,
        ..Default::default()
    };
    let batch = run_batch_with_workers(&scenario, &config, Some(root.path()), None, None).unwrap();
    let output = root.path().join("data");
    fs::create_dir(&output).unwrap();
    export_native_batch(
        &scenario,
        &batch,
        &output,
        NativeFormat::Parquet,
        ExportLimits::default(),
    )
    .unwrap();
    let data = read_native_dataset(&output).unwrap();
    assert_eq!(data.manifest().scenario_id, "nominal-ed-mvp");
    assert_eq!(data.patients().height(), 5);
    for (name, value) in [("arrivals", 5), ("completed", 5), ("unfinished", 0)] {
        assert_eq!(
            data.replications()
                .column(name)
                .unwrap()
                .u64()
                .unwrap()
                .get(0),
            Some(value)
        );
    }
}
#[test]
fn test_downstream_typed_calibration_and_domain_fixtures() {
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
    let uncert = compute_uncertainty(&[10.0, 12.0, 14.0, 16.0, 18.0]).unwrap();
    assert_eq!(uncert.replications_count, 5);
    assert!(uncert.standard_error.unwrap() > 0.0);
    assert!(uncert.ci_95_lower.unwrap() < uncert.ci_95_upper.unwrap());

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
    assert_eq!(traces[0].start_tick, Some(3));
    assert_eq!(traces[0].end_tick, Some(13));
    assert_eq!(traces[0].transit_ticks, 15);
    assert_eq!(traces[0].work_ticks, 10);

    #[cfg(feature = "native-data")]
    {
        let table = calibration_table(&traces, "downstream-e43", "tick").unwrap();
        assert_eq!(table.height(), 2);
        for (row, case, start, end, work) in [(0, "case_201", 3, 13, 10), (1, "case_202", 5, 11, 6)]
        {
            assert_eq!(
                table.column("day").unwrap().u32().unwrap().get(row),
                Some(1)
            );
            assert_eq!(
                table.column("case_id").unwrap().str().unwrap().get(row),
                Some(case)
            );
            for (name, value) in [
                ("start_tick", start),
                ("end_tick", end),
                ("transit_ticks", 15),
                ("work_ticks", work),
                ("interval_ticks", work),
            ] {
                assert_eq!(
                    table
                        .column(&format!("{name}_hi"))
                        .unwrap()
                        .u64()
                        .unwrap()
                        .get(row),
                    Some(0)
                );
                assert_eq!(
                    table
                        .column(&format!("{name}_lo"))
                        .unwrap()
                        .u64()
                        .unwrap()
                        .get(row),
                    Some(value)
                );
            }
            for (name, value) in [
                ("fixture_id", "downstream-e43"),
                ("source_kind", "synthetic_fixture"),
                ("time_unit", "tick"),
                ("interval_basis", "end_minus_start"),
                ("transit_source", "caller_supplied"),
                ("transit_conversion", "not_applicable"),
            ] {
                assert_eq!(
                    table.column(name).unwrap().str().unwrap().get(row),
                    Some(value)
                );
            }
            assert_eq!(
                table
                    .column("transit_source_seconds")
                    .unwrap()
                    .f64()
                    .unwrap()
                    .get(row),
                None
            );
        }
    }

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

#[test]
fn test_downstream_seed_allocation() {
    let distinct: std::collections::HashSet<_> =
        allocate_replication_seeds(100, 5).into_iter().collect();
    assert_eq!(distinct.len(), 5);
}
