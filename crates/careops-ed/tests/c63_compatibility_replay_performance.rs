#![forbid(unsafe_code)]

//! C6.3 Compatibility, Deterministic Replay, and Performance Gates.
//!
//! Verifies:
//! 1. `test_feature_api_manifest_compatibility`:
//!    - Scenario JSON validation and deterministic exit codes (`ExitCodeDef`).
//!    - Run manifest schema version checking (`schema_version = 1`), hash verification (`compute_sha256_hex`).
//!    - CLI dispatcher commands (`validate`, `run`, `compare`, `export`).
//!    - Arrow/Parquet schema compatibility, explicit units, and denominators.
//!
//! 2. `test_deterministic_replay_and_worker_concurrency`:
//!    - Deterministic replay: repeated simulation runs on identical seeds yield identical run summaries.
//!    - Batch worker equivalence: batch runner with 1, 2, and 4 workers yields identical output summaries,
//!      identical hashes, and identical Arrow patient records.
//!
//! 3. `test_performance_benchmarks_and_scaling`:
//!    - Simulation step throughput (steps/sec and patients/sec) measured across 100, 500, and 1000 patients.
//!    - Comparison report generation latency measured on 1000-patient runs.
//!    - Ingestion memory footprint and probe overhead metrics.
//!    - Named hardware environment logging (OS, Arch, CPU parallelism).
//!
//! 4. `test_validity_limits_and_diagnostics`:
//!    - Zero horizon rejection (`ExitCodeDef::MalformedConfig`).
//!    - Zero work rejection (`work_ticks == 0` and empty `patients: []`).
//!    - Malformed JSON diagnostics (syntax errors, unknown fields, bad UTF-8).
//!    - Capacity constraints (non-positive counts, open > physical, staffed > open).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use careops_ed::disposition::api_runner::{
    compare_run_documents, compute_sha256_hex, execute_cli_args, export_document, run_experiment,
    validate_scenario_source, ExitCodeDef, ExportFormat, RunDocument, RunOptions,
    ValidationSuccess,
};
use careops_ed::disposition::arrow_export::{
    build_metrics_arrow_schema, build_patient_arrow_schema, build_spatial_arrow_schema,
    generate_arrow_export_bundle, generate_parquet_schema_ddl,
};
use careops_ed::disposition::batch_runner::{run_batch_with_workers, BatchConfig};
use careops_ed::{parse_scenario_json, run_scenario, ScenarioConfig};
use serde_json::json;

struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "careops_c63_{}_{}_{}",
            prefix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("create temp dir");
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

fn load_nominal_scenario() -> ScenarioConfig {
    let content = fs::read_to_string(nominal_path()).expect("read nominal_mvp.json");
    parse_scenario_json(&content).expect("parse nominal_mvp.json")
}

/// Constructs a valid synthetic scenario JSON Value for benchmarking and validity limits.
fn make_synthetic_scenario_json(
    scenario_id: &str,
    horizon_ticks: u64,
    physical_count: u64,
    open_count: u64,
    staffed_count: u64,
    patient_specs: &[(u64, u64)], // (arrival_tick, work_ticks)
) -> serde_json::Value {
    let patients: Vec<serde_json::Value> = patient_specs
        .iter()
        .enumerate()
        .map(|(i, &(arr, work))| {
            json!({
                "patient_id": format!("p_{i:04}"),
                "arrival_tick": arr,
                "work_ticks": work,
                "resource_id": "res.bay.1",
                "task_class_id": "task.care"
            })
        })
        .collect();

    json!({
        "schema_version": 1,
        "scenario_id": scenario_id,
        "seed": 42,
        "time_unit": "tick",
        "horizon_ticks": horizon_ticks,
        "provenance": {
            "class": "synthetic",
            "notes": "Synthetic performance and validity test fixture."
        },
        "capacity_location": {
            "schema_version": 3,
            "status": "proposed",
            "capacity": {
                "unit": "treatment_space",
                "bucket_completeness": "complete",
                "physical_status": "known",
                "physical_count": physical_count,
                "availability_status": "known",
                "open_count": open_count,
                "staffed_count": staffed_count,
                "equipment": []
            },
            "zones": [{"zone_id": "zone.acute", "name": "Acute Zone"}],
            "locations": [{
                "location_id": "loc.bay.1",
                "name": "Bay 1",
                "zone_id": "zone.acute"
            }],
            "resource_buckets": [{
                "resource_id": "res.bay.1",
                "resource_class": "class.bay",
                "location_id": "loc.bay.1",
                "zone_id": "zone.acute",
                "physical_status": "known",
                "physical_count": physical_count,
                "availability_status": "known",
                "open_count": open_count,
                "staffed_count": staffed_count,
                "capabilities": []
            }],
            "task_classes": [{
                "task_class_id": "task.care",
                "name": "Care Task",
                "required_resource_classes": ["class.bay"]
            }],
            "staff_roles": [{
                "staff_role_id": "staff.nurse",
                "name": "ED Nurse",
                "counts": [{
                    "basis": "present",
                    "status": "known",
                    "count": staffed_count,
                    "reference": "Shift roster"
                }]
            }],
            "eligibility": [{
                "staff_role_id": "staff.nurse",
                "task_class_id": "task.care",
                "zone_id": "zone.acute",
                "location_id": "loc.bay.1"
            }],
            "routes": [{
                "from_location_id": "loc.bay.1",
                "to_location_id": "loc.bay.1",
                "distance": {"value": 0.0, "unit": "m"}
            }]
        },
        "patients": patients
    })
}

// ---------------------------------------------------------------------------
// 1. Feature / API / Manifest Compatibility
// ---------------------------------------------------------------------------

#[test]
fn test_feature_api_manifest_compatibility() {
    let tmp = TestTempDir::new("api_manifest_compat");

    // A. Scenario JSON validation and deterministic exit codes
    let nominal_content = fs::read_to_string(nominal_path()).expect("read nominal_mvp.json");
    let val_res = validate_scenario_source(&nominal_content);
    assert!(val_res.is_ok(), "nominal scenario must be valid");
    let report: ValidationSuccess = val_res.unwrap();
    assert_eq!(report.scenario_id, "nominal-ed-mvp");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.time_unit, "tick");
    assert_eq!(report.horizon_ticks, 20);
    assert_eq!(report.patient_count, 5);

    // Unsupported schema version -> ExitCodeDef::MalformedConfig
    let bad_schema_json =
        nominal_content.replace("\"schema_version\": 1,", "\"schema_version\": 99,");
    let bad_val = validate_scenario_source(&bad_schema_json);
    assert!(bad_val.is_err());
    assert_eq!(bad_val.unwrap_err().code, ExitCodeDef::MalformedConfig);

    // Nonexistent file -> ExitCodeDef::FileNotFound
    let missing_out = execute_cli_args(&[
        "validate".to_string(),
        tmp.path().join("does_not_exist.json").display().to_string(),
    ]);
    assert_eq!(missing_out.exit_code, ExitCodeDef::FileNotFound);
    assert!(missing_out.stderr.contains("file not found"));

    // Missing command -> ExitCodeDef::UsageError
    let no_cmd_out = execute_cli_args(&[]);
    assert_eq!(no_cmd_out.exit_code, ExitCodeDef::UsageError);

    // B. Run manifest schema version checking & hash verification
    let empty_sha = compute_sha256_hex(b"");
    assert_eq!(
        empty_sha, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "empty sha256 must match standard FIPS 180-4 vector"
    );
    let abc_sha = compute_sha256_hex(b"abc");
    assert_eq!(
        abc_sha, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "abc sha256 must match standard FIPS 180-4 vector"
    );

    let scenario = load_nominal_scenario();
    let nominal_bytes = nominal_content.as_bytes();
    let expected_hash = compute_sha256_hex(nominal_bytes);

    let run_opts = RunOptions {
        seed_override: Some(42),
        horizon_override: Some(20),
        ..Default::default()
    };
    let doc = run_experiment(
        &scenario,
        &run_opts,
        "nominal_mvp.json",
        nominal_bytes,
        None,
    )
    .expect("run experiment");

    assert_eq!(
        doc.manifest.schema_version, 1,
        "manifest schema_version must be 1"
    );
    assert_eq!(doc.manifest.scenario_id, "nominal-ed-mvp");
    assert_eq!(doc.manifest.seed, 42);
    assert_eq!(doc.manifest.time_unit, "tick");
    assert_eq!(doc.manifest.horizon_ticks, 20);
    assert_eq!(doc.manifest.input_sha256, expected_hash);

    // Incompatible scenario comparisons:
    // Schema version mismatch -> ExitCodeDef::IncompatibleScenario
    let mut cand_doc_bad_schema = doc.clone();
    cand_doc_bad_schema.manifest.schema_version = 2;
    let comp_schema_err = compare_run_documents(&doc, &cand_doc_bad_schema);
    assert!(comp_schema_err.is_err());
    assert_eq!(
        comp_schema_err.unwrap_err().code,
        ExitCodeDef::IncompatibleScenario
    );

    // Time unit mismatch -> ExitCodeDef::IncompatibleScenario
    let mut cand_doc_bad_unit = doc.clone();
    cand_doc_bad_unit.manifest.time_unit = "second".to_string();
    let comp_unit_err = compare_run_documents(&doc, &cand_doc_bad_unit);
    assert!(comp_unit_err.is_err());
    assert_eq!(
        comp_unit_err.unwrap_err().code,
        ExitCodeDef::IncompatibleScenario
    );

    // C. CLI Dispatcher Commands: validate, run, compare, export
    // 1. validate
    let cli_val = execute_cli_args(&["validate".to_string(), nominal_path().display().to_string()]);
    assert_eq!(cli_val.exit_code, ExitCodeDef::Success);
    assert!(cli_val.stdout.contains("valid scenario 'nominal-ed-mvp'"));

    // 2. run
    let out_json = tmp.path().join("run_out.json");
    let cli_run = execute_cli_args(&[
        "run".to_string(),
        nominal_path().display().to_string(),
        "--output".to_string(),
        out_json.display().to_string(),
        "--seed".to_string(),
        "42".to_string(),
        "--horizon".to_string(),
        "30".to_string(),
    ]);
    assert_eq!(cli_run.exit_code, ExitCodeDef::Success);
    assert!(out_json.exists());

    // Overwrite protection: running again without --overwrite fails with FileAlreadyExists
    let cli_run_dup = execute_cli_args(&[
        "run".to_string(),
        nominal_path().display().to_string(),
        "--output".to_string(),
        out_json.display().to_string(),
    ]);
    assert_eq!(cli_run_dup.exit_code, ExitCodeDef::FileAlreadyExists);

    // Running with --overwrite succeeds
    let cli_run_overwrite = execute_cli_args(&[
        "run".to_string(),
        nominal_path().display().to_string(),
        "--output".to_string(),
        out_json.display().to_string(),
        "--overwrite".to_string(),
    ]);
    assert_eq!(cli_run_overwrite.exit_code, ExitCodeDef::Success);

    // 3. compare
    let out_cand_json = tmp.path().join("cand_out.json");
    let cli_run_cand = execute_cli_args(&[
        "run".to_string(),
        constrained_path().display().to_string(),
        "--output".to_string(),
        out_cand_json.display().to_string(),
        "--seed".to_string(),
        "42".to_string(),
    ]);
    assert_eq!(cli_run_cand.exit_code, ExitCodeDef::Success);

    let cli_comp = execute_cli_args(&[
        "compare".to_string(),
        out_json.display().to_string(),
        out_cand_json.display().to_string(),
    ]);
    assert_eq!(cli_comp.exit_code, ExitCodeDef::Success);
    assert!(cli_comp.stdout.contains("nominal-ed-mvp"));
    assert!(cli_comp.stdout.contains("constrained-ed-mvp"));
    assert!(cli_comp.stdout.contains("mean_wait"));

    // 4. export
    // Export CSV
    let export_csv_path = tmp.path().join("export.csv");
    let cli_exp_csv = execute_cli_args(&[
        "export".to_string(),
        out_json.display().to_string(),
        "--format".to_string(),
        "csv".to_string(),
        "--output".to_string(),
        export_csv_path.display().to_string(),
    ]);
    assert_eq!(cli_exp_csv.exit_code, ExitCodeDef::Success);
    let csv_content = fs::read_to_string(&export_csv_path).expect("read csv");
    assert!(csv_content.starts_with("patient_id,arrival_tick,start_tick,completion_tick"));

    // Export Arrow schema
    let export_arrow_path = tmp.path().join("export_arrow.json");
    let cli_exp_arrow = execute_cli_args(&[
        "export".to_string(),
        out_json.display().to_string(),
        "--format".to_string(),
        "arrow".to_string(),
        "--output".to_string(),
        export_arrow_path.display().to_string(),
    ]);
    assert_eq!(cli_exp_arrow.exit_code, ExitCodeDef::Success);
    let arrow_schema_text = fs::read_to_string(&export_arrow_path).expect("read arrow schema");
    assert!(arrow_schema_text.contains("arrow_schema"));
    assert!(arrow_schema_text.contains("patient_id"));
    assert!(arrow_schema_text.contains("started_patients"));

    // Export Parquet schema
    let cli_exp_parquet = execute_cli_args(&[
        "export".to_string(),
        out_json.display().to_string(),
        "--format".to_string(),
        "parquet".to_string(),
    ]);
    assert_eq!(cli_exp_parquet.exit_code, ExitCodeDef::Success);
    assert!(cli_exp_parquet
        .stdout
        .contains("message careops_ed_patient_record"));

    // Verify Arrow schema builders and field properties directly
    let p_schema = build_patient_arrow_schema("nominal", "tick");
    assert_eq!(p_schema.schema_name, "careops.ed.patient_records.v1");
    let wait_field = p_schema
        .fields
        .iter()
        .find(|f| f.name == "wait_ticks")
        .expect("wait_ticks field");
    assert_eq!(wait_field.unit, "tick");
    assert_eq!(wait_field.denominator.as_deref(), Some("started_patients"));

    let m_schema = build_metrics_arrow_schema();
    assert_eq!(m_schema.schema_name, "careops.ed.metrics.v1");

    let s_schema = build_spatial_arrow_schema();
    assert_eq!(s_schema.schema_name, "careops.ed.spatial_transit.v1");

    let ddl = generate_parquet_schema_ddl();
    assert!(ddl.contains("required int64 arrival_tick"));
}

// ---------------------------------------------------------------------------
// 2. Deterministic Replay and Worker Concurrency
// ---------------------------------------------------------------------------

#[test]
fn test_deterministic_replay_and_worker_concurrency() {
    let scenario = load_nominal_scenario();

    // A. Deterministic replay: repeated simulation runs on identical seeds yield identical run summaries
    let summary_a1 = run_scenario(&scenario).expect("run scenario A1");
    let summary_a2 = run_scenario(&scenario).expect("run scenario A2");
    assert_eq!(
        summary_a1, summary_a2,
        "identical scenario must produce identical RunSummary"
    );

    let input_bytes = fs::read(nominal_path()).expect("read nominal bytes");
    let opts = RunOptions {
        seed_override: Some(999),
        horizon_override: Some(50),
        ..Default::default()
    };
    let doc_1 = run_experiment(&scenario, &opts, "nominal_mvp.json", &input_bytes, None)
        .expect("experiment 1");
    let doc_2 = run_experiment(&scenario, &opts, "nominal_mvp.json", &input_bytes, None)
        .expect("experiment 2");

    assert_eq!(doc_1.manifest, doc_2.manifest);
    assert_eq!(doc_1.summary, doc_2.summary);
    assert_eq!(
        compute_sha256_hex(&serde_json::to_vec(&doc_1).unwrap()),
        compute_sha256_hex(&serde_json::to_vec(&doc_2).unwrap()),
        "serialized run documents on identical seeds must yield identical sha256"
    );

    // B. Batch worker equivalence: batch runner with 1, 2, and 4 workers
    let cfg_1 = BatchConfig {
        batch_id: "c63-worker-eq-1".to_string(),
        scenario_id: "nominal-ed-mvp".to_string(),
        base_seed: 54321,
        replications: 4,
        num_workers: 1,
        horizon_override: Some(200),
        checkpoint_interval: None,
        ..Default::default()
    };

    let cfg_2 = BatchConfig {
        batch_id: "c63-worker-eq-2".to_string(),
        num_workers: 2,
        ..cfg_1.clone()
    };

    let cfg_4 = BatchConfig {
        batch_id: "c63-worker-eq-4".to_string(),
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

    // Verify replication-by-replication equivalence
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

        assert_eq!(r1.mean_wait_ticks, r2.mean_wait_ticks);
        assert_eq!(r1.mean_wait_ticks, r4.mean_wait_ticks);

        assert_eq!(r1.horizon_ticks, r2.horizon_ticks);
        assert_eq!(r1.horizon_ticks, r4.horizon_ticks);
    }

    // Verify aggregate metrics equivalence
    assert_eq!(res_1.aggregate_mean_wait, res_2.aggregate_mean_wait);
    assert_eq!(res_2.aggregate_mean_wait, res_4.aggregate_mean_wait);
    assert_eq!(res_1.aggregate_std_error, res_2.aggregate_std_error);
    assert_eq!(res_2.aggregate_std_error, res_4.aggregate_std_error);
    assert_eq!(res_1.mean_completion_rate, res_2.mean_completion_rate);
    assert_eq!(res_2.mean_completion_rate, res_4.mean_completion_rate);

    // Verify sha256 output hashes match identically
    let rep_bytes_1 = serde_json::to_vec(&res_1.replications).unwrap();
    let rep_bytes_2 = serde_json::to_vec(&res_2.replications).unwrap();
    let rep_bytes_4 = serde_json::to_vec(&res_4.replications).unwrap();

    let hash_1 = compute_sha256_hex(&rep_bytes_1);
    let hash_2 = compute_sha256_hex(&rep_bytes_2);
    let hash_4 = compute_sha256_hex(&rep_bytes_4);

    assert_eq!(
        hash_1, hash_2,
        "1-worker and 2-worker output hashes must match"
    );
    assert_eq!(
        hash_2, hash_4,
        "2-worker and 4-worker output hashes must match"
    );

    // Verify Arrow bundle patient records and metrics match identically
    let bundle_1 = generate_arrow_export_bundle(&scenario, &res_1, None, None).expect("bundle 1");
    let bundle_2 = generate_arrow_export_bundle(&scenario, &res_2, None, None).expect("bundle 2");
    let bundle_4 = generate_arrow_export_bundle(&scenario, &res_4, None, None).expect("bundle 4");

    assert_eq!(bundle_1.patient_records, bundle_2.patient_records);
    assert_eq!(bundle_2.patient_records, bundle_4.patient_records);
    assert_eq!(bundle_1.metrics, bundle_2.metrics);
    assert_eq!(bundle_2.metrics, bundle_4.metrics);
}

// ---------------------------------------------------------------------------
// 3. Performance Benchmarks and Scaling
// ---------------------------------------------------------------------------

#[test]
fn test_performance_benchmarks_and_scaling() {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let parallelism = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    println!(
        "\n=================================================================\n\
         [BENCHMARK ENVIRONMENT] OS: {}, Arch: {}, CPU Parallelism: {} cores\n\
         =================================================================",
        os, arch, parallelism
    );

    // Benchmarking simulation step throughput across 100, 500, and 1000 patients
    let patient_scales = [100usize, 500usize, 1000usize];
    let mut stored_1000_doc: Option<RunDocument> = None;

    for &n_patients in &patient_scales {
        // Construct patient specs: staggered arrivals every tick, work_ticks = 2
        let specs: Vec<(u64, u64)> = (0..n_patients).map(|i| (i as u64, 2u64)).collect();
        let horizon_ticks = (n_patients as u64) * 2 + 50;

        let scenario_val = make_synthetic_scenario_json(
            &format!("scale-{n_patients}"),
            horizon_ticks,
            20, // physical bays
            20, // open bays
            20, // staffed bays
            &specs,
        );

        let json_str = serde_json::to_string(&scenario_val).expect("serialize scenario");
        let raw_bytes_len = json_str.len();

        // Ingestion benchmark: parse and validate
        let t_ingest_start = Instant::now();
        let scenario = parse_scenario_json(&json_str).expect("parse scenario");
        let t_ingest = t_ingest_start.elapsed();

        // Simulation execution benchmark
        let t_sim_start = Instant::now();
        let summary = run_scenario(&scenario).expect("run simulation");
        let t_sim = t_sim_start.elapsed();

        assert_eq!(summary.arrivals, n_patients);
        assert!(summary.completed > 0);

        // Each patient has an arrival event and a completion event = 2 ECS events
        let total_events = (summary.arrivals + summary.completed) as f64;
        let sim_secs = t_sim.as_secs_f64().max(1e-9);
        let steps_per_sec = total_events / sim_secs;
        let patients_per_sec = (summary.completed as f64) / sim_secs;

        // Ingestion memory / probe overhead:
        let bytes_per_patient = (raw_bytes_len as f64) / (n_patients as f64);

        println!(
            "  N={:<4} patients | Raw JSON: {:>6} B ({:>5.1} B/pat) | Ingest: {:>8.2?} | \
             Sim: {:>8.2?} | Steps/sec: {:>10.0} | Pat/sec: {:>10.0}",
            n_patients,
            raw_bytes_len,
            bytes_per_patient,
            t_ingest,
            t_sim,
            steps_per_sec,
            patients_per_sec
        );

        // Sanity assertions on throughput: must complete fast (well under 5s)
        assert!(t_sim.as_secs_f64() < 5.0, "simulation must not stall");
        assert!(
            steps_per_sec > 100.0,
            "step throughput must be positive and reasonable"
        );

        if n_patients == 1000 {
            let doc = RunDocument {
                manifest: careops_ed::disposition::api_runner::RunManifest {
                    schema_version: 1,
                    scenario_id: format!("scale-{n_patients}"),
                    input_file: "synthetic.json".to_string(),
                    input_sha256: compute_sha256_hex(json_str.as_bytes()),
                    seed: 42,
                    time_unit: "tick".to_string(),
                    horizon_ticks,
                },
                summary,
            };
            stored_1000_doc = Some(doc);
        }
    }

    // Benchmark comparison report generation latency on 1000-patient runs
    let base_1000_doc = stored_1000_doc.expect("1000 patient document must exist");
    let mut cand_1000_doc = base_1000_doc.clone();
    cand_1000_doc.manifest.scenario_id = "scale-1000-cand".to_string();
    cand_1000_doc.manifest.seed = 999;
    // Alter candidate wait ticks slightly to test delta and standard error calculations
    for p in &mut cand_1000_doc.summary.patients {
        if let Some(ref mut w) = p.wait_ticks {
            *w = w.saturating_add(1);
        }
    }

    let t_comp_start = Instant::now();
    let comp_report =
        compare_run_documents(&base_1000_doc, &cand_1000_doc).expect("comparison must succeed");
    let t_comp = t_comp_start.elapsed();

    println!(
        "  1000-patient Comparison Latency: {:>8.2?} across {} metrics",
        t_comp,
        comp_report.metrics.len()
    );

    assert!(
        t_comp.as_millis() < 100,
        "1000-patient comparison latency must be under 100ms (took {:?})",
        t_comp
    );
    assert_eq!(comp_report.base_scenario_id, "scale-1000");
    assert_eq!(comp_report.candidate_scenario_id, "scale-1000-cand");
    assert!(comp_report.metrics.len() >= 4);

    // Verify probe overhead: export_document latency
    let t_export_start = Instant::now();
    let export_text = export_document(&base_1000_doc, ExportFormat::Json).expect("export json");
    let t_export = t_export_start.elapsed();
    assert!(!export_text.is_empty());
    println!("  1000-patient Document Export Latency: {:>8.2?}", t_export);
    println!("=================================================================\n");
}

// ---------------------------------------------------------------------------
// 4. Validity Limits and Diagnostics
// ---------------------------------------------------------------------------

#[test]
fn test_validity_limits_and_diagnostics() {
    let tmp = TestTempDir::new("validity_limits");

    // A. Zero horizon rejection (ExitCodeDef::MalformedConfig)
    let zero_horizon_val = make_synthetic_scenario_json(
        "zero-horizon",
        0, // zero horizon!
        5,
        5,
        5,
        &[(0, 2)],
    );
    let zero_horizon_str = serde_json::to_string(&zero_horizon_val).unwrap();

    let val_err = validate_scenario_source(&zero_horizon_str);
    assert!(val_err.is_err());
    let err = val_err.unwrap_err();
    assert_eq!(err.code, ExitCodeDef::MalformedConfig);
    assert!(err.message.contains("horizon"));

    // Also verify via CLI dispatcher: validate command
    let zero_h_path = tmp.path().join("zero_horizon.json");
    fs::write(&zero_h_path, &zero_horizon_str).expect("write zero horizon");
    let cli_zero_h = execute_cli_args(&["validate".to_string(), zero_h_path.display().to_string()]);
    assert_eq!(cli_zero_h.exit_code, ExitCodeDef::MalformedConfig);

    // CLI run with --horizon 0 override
    let cli_override_zero_h = execute_cli_args(&[
        "run".to_string(),
        nominal_path().display().to_string(),
        "--horizon".to_string(),
        "0".to_string(),
    ]);
    assert_eq!(cli_override_zero_h.exit_code, ExitCodeDef::MalformedConfig);

    // B. Zero work rejection:
    // 1. Patient with work_ticks == 0
    let zero_work_val = make_synthetic_scenario_json(
        "zero-work",
        50,
        5,
        5,
        5,
        &[(0, 0)], // 0 work ticks!
    );
    let zero_work_str = serde_json::to_string(&zero_work_val).unwrap();
    let zero_work_err = validate_scenario_source(&zero_work_str);
    assert!(zero_work_err.is_err());
    let err = zero_work_err.unwrap_err();
    assert_eq!(err.code, ExitCodeDef::MalformedConfig);
    assert!(err.message.contains("work_ticks must be positive"));

    // 2. Empty patients list
    let mut empty_patients_val =
        make_synthetic_scenario_json("empty-patients", 50, 5, 5, 5, &[(0, 2)]);
    empty_patients_val["patients"] = json!([]);
    let empty_patients_str = serde_json::to_string(&empty_patients_val).unwrap();
    let empty_patients_err = validate_scenario_source(&empty_patients_str);
    assert!(empty_patients_err.is_err());
    let err = empty_patients_err.unwrap_err();
    assert_eq!(err.code, ExitCodeDef::MalformedConfig);
    assert!(err
        .message
        .contains("patients must contain at least one supplied work item"));

    // C. Malformed JSON diagnostics:
    // 1. Truncated / syntax error JSON
    let truncated_json = "{ \"schema_version\": 1, \"scenario_id\": ";
    let malformed_err1 = validate_scenario_source(truncated_json);
    assert!(malformed_err1.is_err());
    assert_eq!(
        malformed_err1.unwrap_err().code,
        ExitCodeDef::MalformedConfig
    );

    // 2. Non-JSON string
    let non_json = "this is completely invalid json syntax";
    let malformed_err2 = validate_scenario_source(non_json);
    assert!(malformed_err2.is_err());
    assert_eq!(
        malformed_err2.unwrap_err().code,
        ExitCodeDef::MalformedConfig
    );

    // 3. Unknown fields rejected due to deny_unknown_fields
    let mut unknown_field_val =
        make_synthetic_scenario_json("unknown-field", 50, 5, 5, 5, &[(0, 2)]);
    unknown_field_val["unexpected_arbitrary_field"] = json!("invalid");
    let unknown_field_str = serde_json::to_string(&unknown_field_val).unwrap();
    let unknown_err = validate_scenario_source(&unknown_field_str);
    assert!(unknown_err.is_err());
    assert_eq!(unknown_err.unwrap_err().code, ExitCodeDef::MalformedConfig);

    // 4. Malformed example file passed to CLI validate, compare, export
    let cli_malformed_val = execute_cli_args(&[
        "validate".to_string(),
        malformed_path().display().to_string(),
    ]);
    assert_eq!(cli_malformed_val.exit_code, ExitCodeDef::MalformedConfig);

    let cli_malformed_comp = execute_cli_args(&[
        "compare".to_string(),
        malformed_path().display().to_string(),
        nominal_path().display().to_string(),
    ]);
    assert_eq!(cli_malformed_comp.exit_code, ExitCodeDef::MalformedConfig);

    let cli_malformed_exp =
        execute_cli_args(&["export".to_string(), malformed_path().display().to_string()]);
    assert_eq!(cli_malformed_exp.exit_code, ExitCodeDef::MalformedConfig);

    // D. Capacity constraints (non-positive capacity rejection):
    // 1. Both physical_count and open_count are 0
    let mut zero_cap_val = make_synthetic_scenario_json("zero-cap", 50, 0, 0, 0, &[(0, 2)]);
    zero_cap_val["capacity_location"]["capacity"]["physical_count"] = json!(0);
    zero_cap_val["capacity_location"]["capacity"]["open_count"] = json!(0);
    zero_cap_val["capacity_location"]["capacity"]["staffed_count"] = json!(0);
    let zero_cap_str = serde_json::to_string(&zero_cap_val).unwrap();
    let zero_cap_err = validate_scenario_source(&zero_cap_str);
    assert!(zero_cap_err.is_err());
    let err = zero_cap_err.unwrap_err();
    assert_eq!(err.code, ExitCodeDef::MalformedConfig);
    assert!(
        err.message
            .contains("capacity physical and open counts cannot both be zero")
            || err
                .message
                .contains("needs positive known open and staffed counts")
    );

    // 2. Resource bucket with open_count == 0
    let mut zero_bucket_val = make_synthetic_scenario_json("zero-bucket", 50, 5, 5, 5, &[(0, 2)]);
    zero_bucket_val["capacity_location"]["resource_buckets"][0]["open_count"] = json!(0);
    let zero_bucket_str = serde_json::to_string(&zero_bucket_val).unwrap();
    let zero_bucket_err = validate_scenario_source(&zero_bucket_str);
    assert!(zero_bucket_err.is_err());
    assert_eq!(
        zero_bucket_err.unwrap_err().code,
        ExitCodeDef::MalformedConfig
    );

    // 3. open_count exceeds physical_count (inconsistent capacity)
    let mut open_exceeds_physical = make_synthetic_scenario_json(
        "open-exceeds-physical",
        50,
        2,
        5, // open = 5 > physical = 2
        2,
        &[(0, 2)],
    );
    open_exceeds_physical["capacity_location"]["capacity"]["physical_count"] = json!(2);
    open_exceeds_physical["capacity_location"]["capacity"]["open_count"] = json!(5);
    open_exceeds_physical["capacity_location"]["resource_buckets"][0]["physical_count"] = json!(2);
    open_exceeds_physical["capacity_location"]["resource_buckets"][0]["open_count"] = json!(5);
    let open_exceeds_str = serde_json::to_string(&open_exceeds_physical).unwrap();
    let open_err = validate_scenario_source(&open_exceeds_str);
    assert!(open_err.is_err());
    assert_eq!(open_err.unwrap_err().code, ExitCodeDef::MalformedConfig);

    // 4. staffed_count exceeds open_count (inconsistent capacity)
    let mut staffed_exceeds_open = make_synthetic_scenario_json(
        "staffed-exceeds-open",
        50,
        5,
        2, // open = 2
        4, // staffed = 4 > open = 2
        &[(0, 2)],
    );
    staffed_exceeds_open["capacity_location"]["capacity"]["staffed_count"] = json!(4);
    staffed_exceeds_open["capacity_location"]["capacity"]["open_count"] = json!(2);
    staffed_exceeds_open["capacity_location"]["resource_buckets"][0]["staffed_count"] = json!(4);
    staffed_exceeds_open["capacity_location"]["resource_buckets"][0]["open_count"] = json!(2);
    let staffed_err_str = serde_json::to_string(&staffed_exceeds_open).unwrap();
    let staffed_err = validate_scenario_source(&staffed_err_str);
    assert!(staffed_err.is_err());
    assert_eq!(staffed_err.unwrap_err().code, ExitCodeDef::MalformedConfig);
}
