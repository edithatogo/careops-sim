//! Public API, CLI runner, deterministic error codes, and recovery test suite for Phase E3.1.
//!
//! Covers:
//! - Validation of schema and parameter bounds (`validate`).
//! - Experiment execution, overrides, seed allocation, cancellation, and atomic writes (`run`).
//! - Overwrite protection, disk full simulation, and crash / partial artifact inspection.
//! - Comparative evaluation with units, denominators, uncertainty, and incompatible scenario diagnostics (`compare`).
//! - Structured multi-format exports (`export`).
//! - Fresh-user CLI walkthrough and help interfaces.

use std::fs;
use std::path::{Path, PathBuf};

use careops_ed::disposition::api_runner::{
    allocate_replication_seeds, compare_run_documents, compute_sha256_hex, execute_cli_args,
    export_document, inspect_artifact, run_experiment, validate_scenario_source, ArtifactStatus,
    CancellationToken, ExitCodeDef, ExportFormat, RunOptions, ValidationSuccess,
};
use careops_ed::parse_scenario_json;

struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "careops_test_{}_{}_{}",
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

fn nominal_path() -> PathBuf {
    repo_root().join("crates/careops-ed/examples/nominal_mvp.json")
}

fn constrained_path() -> PathBuf {
    repo_root().join("crates/careops-ed/examples/constrained_mvp.json")
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
fn test_run_experiment_nominal_success() {
    let bytes = fs::read(nominal_path()).unwrap();
    let source = std::str::from_utf8(&bytes).unwrap();
    let scenario = parse_scenario_json(source).unwrap();

    let options = RunOptions::default();
    let res = run_experiment(&scenario, &options, "nominal_mvp.json", &bytes, None);
    assert!(res.is_ok());

    let doc = res.unwrap();
    assert_eq!(doc.manifest.schema_version, 1);
    assert_eq!(doc.manifest.scenario_id, "nominal-ed-mvp");
    assert_eq!(doc.manifest.seed, 42);
    assert_eq!(doc.manifest.time_unit, "tick");
    assert_eq!(doc.summary.arrivals, 5);
    assert_eq!(doc.summary.completed, 5);
    assert_eq!(doc.summary.unfinished, 0);

    // Verify sha256 in manifest matches bytes
    let expected_sha = compute_sha256_hex(&bytes);
    assert_eq!(doc.manifest.input_sha256, expected_sha);
}

#[test]
fn test_run_experiment_seed_override() {
    let bytes = fs::read(nominal_path()).unwrap();
    let source = std::str::from_utf8(&bytes).unwrap();
    let scenario = parse_scenario_json(source).unwrap();

    let options = RunOptions {
        seed_override: Some(9999),
        ..Default::default()
    };

    let doc = run_experiment(&scenario, &options, "nominal_mvp.json", &bytes, None).unwrap();
    assert_eq!(doc.manifest.seed, 9999);
    assert_eq!(doc.summary.seed, 9999);
}

#[test]
fn test_run_experiment_horizon_override() {
    let bytes = fs::read(nominal_path()).unwrap();
    let source = std::str::from_utf8(&bytes).unwrap();
    let scenario = parse_scenario_json(source).unwrap();

    let options = RunOptions {
        horizon_override: Some(7),
        ..Default::default()
    };

    let doc = run_experiment(&scenario, &options, "nominal_mvp.json", &bytes, None).unwrap();
    assert_eq!(doc.manifest.horizon_ticks, 7);
    assert_eq!(doc.summary.horizon_ticks, 7);
    // With shortened horizon of 7 ticks, patient 5 arriving at tick 6 cannot finish 3 ticks of work
    assert!(doc.summary.completed < 5);
    assert!(doc.summary.unfinished > 0);
}

#[test]
fn test_run_experiment_overwrite_protection() {
    let bytes = fs::read(nominal_path()).unwrap();
    let source = std::str::from_utf8(&bytes).unwrap();
    let scenario = parse_scenario_json(source).unwrap();

    let temp_dir = TestTempDir::new("overwrite");
    let out_file = temp_dir.path().join("existing_run.json");

    // Create existing file
    fs::write(&out_file, b"existing content").unwrap();

    let mut options = RunOptions {
        output_path: Some(out_file.clone()),
        overwrite: false,
        ..Default::default()
    };

    let res = run_experiment(&scenario, &options, "nominal.json", &bytes, None);
    assert!(res.is_err(), "Must reject overwrite without explicit flag");
    let err = res.unwrap_err();
    assert_eq!(err.code, ExitCodeDef::FileAlreadyExists);

    // Now enable overwrite
    options.overwrite = true;
    let res_overwrite = run_experiment(&scenario, &options, "nominal.json", &bytes, None);
    assert!(
        res_overwrite.is_ok(),
        "Must permit overwrite with --overwrite"
    );
    let written = fs::read_to_string(&out_file).unwrap();
    assert!(written.contains("\"schema_version\": 1"));
}

#[test]
fn test_run_experiment_simulated_disk_full() {
    let bytes = fs::read(nominal_path()).unwrap();
    let source = std::str::from_utf8(&bytes).unwrap();
    let scenario = parse_scenario_json(source).unwrap();

    let temp_dir = TestTempDir::new("disk_full");
    let out_file = temp_dir.path().join("disk_full_run.json");

    let options = RunOptions {
        output_path: Some(out_file),
        simulate_disk_full: true,
        ..Default::default()
    };

    let res = run_experiment(&scenario, &options, "nominal.json", &bytes, None);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code, ExitCodeDef::OutputWriteFailed);
}

#[test]
fn test_run_experiment_cancellation_and_recovery_inspection() {
    let bytes = fs::read(nominal_path()).unwrap();
    let source = std::str::from_utf8(&bytes).unwrap();
    let scenario = parse_scenario_json(source).unwrap();

    let temp_dir = TestTempDir::new("cancel");
    let out_file = temp_dir.path().join("cancelled_run.json");

    let token = CancellationToken::new();
    token.cancel(); // Pre-cancelled

    let options = RunOptions {
        output_path: Some(out_file.clone()),
        ..Default::default()
    };

    let res = run_experiment(&scenario, &options, "nominal.json", &bytes, Some(&token));
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code, ExitCodeDef::Interrupted);

    // Verify inspect_artifact does NOT treat missing or partial as complete
    let status = inspect_artifact(&out_file);
    assert!(matches!(
        status,
        ArtifactStatus::NotFound | ArtifactStatus::IncompletePartial(_)
    ));
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

#[test]
fn test_compare_runs_nominal_vs_constrained() {
    let nom_bytes = fs::read(nominal_path()).unwrap();
    let nom_scen = parse_scenario_json(std::str::from_utf8(&nom_bytes).unwrap()).unwrap();
    let nom_doc = run_experiment(
        &nom_scen,
        &RunOptions::default(),
        "nominal.json",
        &nom_bytes,
        None,
    )
    .unwrap();

    let con_bytes = fs::read(constrained_path()).unwrap();
    let con_scen = parse_scenario_json(std::str::from_utf8(&con_bytes).unwrap()).unwrap();
    let con_doc = run_experiment(
        &con_scen,
        &RunOptions::default(),
        "constrained.json",
        &con_bytes,
        None,
    )
    .unwrap();

    let comp = compare_run_documents(&nom_doc, &con_doc).expect("comparison must succeed");
    assert_eq!(comp.base_scenario_id, "nominal-ed-mvp");
    assert_eq!(comp.candidate_scenario_id, "constrained-ed-mvp");
    assert_eq!(comp.time_unit, "tick");

    // Find mean_wait metric
    let wait_metric = comp
        .metrics
        .iter()
        .find(|m| m.metric_name == "mean_wait")
        .expect("mean_wait metric must be present");

    assert_eq!(wait_metric.unit, "tick");
    assert!((wait_metric.base_value - 0.4).abs() < 1e-4);
    assert!((wait_metric.candidate_value - 4.2).abs() < 1e-4);
    assert!(wait_metric.delta > 0.0);
    assert!(wait_metric.percent_change.unwrap() > 0.0);

    // Diagnostics should report increased wait
    assert!(comp
        .diagnostics
        .iter()
        .any(|d| d.contains("increased mean wait")));
}

#[test]
fn test_compare_runs_incompatible_time_units() {
    let nom_bytes = fs::read(nominal_path()).unwrap();
    let nom_scen = parse_scenario_json(std::str::from_utf8(&nom_bytes).unwrap()).unwrap();
    let mut base_doc = run_experiment(
        &nom_scen,
        &RunOptions::default(),
        "nominal.json",
        &nom_bytes,
        None,
    )
    .unwrap();
    base_doc.manifest.time_unit = "minute".to_string();

    let con_bytes = fs::read(constrained_path()).unwrap();
    let con_scen = parse_scenario_json(std::str::from_utf8(&con_bytes).unwrap()).unwrap();
    let cand_doc = run_experiment(
        &con_scen,
        &RunOptions::default(),
        "constrained.json",
        &con_bytes,
        None,
    )
    .unwrap();

    let res = compare_run_documents(&base_doc, &cand_doc);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.code, ExitCodeDef::IncompatibleScenario);
    assert!(err.message.contains("incompatible time units"));
}

#[test]
fn test_export_document_formats() {
    let bytes = fs::read(nominal_path()).unwrap();
    let scenario = parse_scenario_json(std::str::from_utf8(&bytes).unwrap()).unwrap();
    let doc = run_experiment(
        &scenario,
        &RunOptions::default(),
        "nominal.json",
        &bytes,
        None,
    )
    .unwrap();

    // Export JSON
    let json_str = export_document(&doc, ExportFormat::Json).unwrap();
    assert!(json_str.contains("\"scenario_id\": \"nominal-ed-mvp\""));

    // Export CSV
    let csv_str = export_document(&doc, ExportFormat::Csv).unwrap();
    assert!(csv_str.contains("patient_id,arrival_tick"));
    assert!(csv_str.contains("patient-001"));

    // Export Arrow schema metadata
    let arrow_str = export_document(&doc, ExportFormat::ArrowSchema).unwrap();
    assert!(arrow_str.contains("\"arrow_schema\""));
    assert!(arrow_str.contains("\"fields\""));
}

#[test]
fn test_cli_fresh_user_walkthrough_help() {
    let out = execute_cli_args(&["--help".to_string()]);
    assert_eq!(out.exit_code, ExitCodeDef::Success);
    assert!(out.stdout.contains("Usage: careops-ed <command>"));
    assert!(out.stdout.contains("validate"));
    assert!(out.stdout.contains("run"));
    assert!(out.stdout.contains("compare"));
    assert!(out.stdout.contains("export"));
    assert!(out.stdout.contains("Exit Codes:"));
}

#[test]
fn test_cli_missing_command_returns_usage_error() {
    let out = execute_cli_args(&[]);
    assert_eq!(out.exit_code, ExitCodeDef::UsageError);
    assert!(out.stderr.contains("missing command"));
}

#[test]
fn test_cli_validate_scenario() {
    let nom_str = nominal_path().to_str().unwrap().to_string();
    let out = execute_cli_args(&["validate".to_string(), nom_str]);
    assert_eq!(out.exit_code, ExitCodeDef::Success);
    assert!(out.stdout.contains("valid scenario 'nominal-ed-mvp'"));

    let non_existent = "/path/to/missing_scenario.json".to_string();
    let out_missing = execute_cli_args(&["validate".to_string(), non_existent]);
    assert_eq!(out_missing.exit_code, ExitCodeDef::FileNotFound);
}

#[test]
fn test_cli_run_and_compare_lifecycle() {
    let temp_dir = TestTempDir::new("lifecycle");
    let nom_out = temp_dir.path().join("nom_run.json");
    let con_out = temp_dir.path().join("con_run.json");

    // 1. Run nominal
    let run_nom_args = vec![
        "run".to_string(),
        nominal_path().to_str().unwrap().to_string(),
        "--output".to_string(),
        nom_out.to_str().unwrap().to_string(),
    ];
    let out_nom = execute_cli_args(&run_nom_args);
    assert_eq!(out_nom.exit_code, ExitCodeDef::Success);
    assert!(nom_out.exists());

    // 2. Run constrained
    let run_con_args = vec![
        "run".to_string(),
        constrained_path().to_str().unwrap().to_string(),
        "--output".to_string(),
        con_out.to_str().unwrap().to_string(),
    ];
    let out_con = execute_cli_args(&run_con_args);
    assert_eq!(out_con.exit_code, ExitCodeDef::Success);
    assert!(con_out.exists());

    // 3. Compare runs
    let compare_args = vec![
        "compare".to_string(),
        nom_out.to_str().unwrap().to_string(),
        con_out.to_str().unwrap().to_string(),
    ];
    let out_comp = execute_cli_args(&compare_args);
    assert_eq!(out_comp.exit_code, ExitCodeDef::Success);
    assert!(out_comp.stdout.contains("mean_wait"));
    assert!(out_comp.stdout.contains("nominal-ed-mvp"));

    // 4. Export run to CSV
    let csv_out = temp_dir.path().join("exported.csv");
    let export_args = vec![
        "export".to_string(),
        nom_out.to_str().unwrap().to_string(),
        "--format".to_string(),
        "csv".to_string(),
        "--output".to_string(),
        csv_out.to_str().unwrap().to_string(),
    ];
    let out_exp = execute_cli_args(&export_args);
    assert_eq!(out_exp.exit_code, ExitCodeDef::Success);
    assert!(csv_out.exists());
    let csv_text = fs::read_to_string(&csv_out).unwrap();
    assert!(csv_text.contains("patient_id,arrival_tick"));
}
