#![forbid(unsafe_code)]

//! C6.3 domain compatibility, independent historical facts, replay and scaling.
//! CLI lifecycle/capability contracts live in careops-ed-cli tests, avoiding a
//! reverse dependency. Alpha row JSON/schema DDL is retired, not reproduced.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use careops_ed::disposition::api_runner::{
    compute_sha256_hex, validate_scenario_source, ExitCodeDef, ValidationSuccess,
};
use careops_ed::disposition::batch_runner::{run_batch_with_workers, BatchConfig};
#[cfg(feature = "native-data")]
use careops_ed::disposition::native_output::{
    export_native_batch, read_native_dataset, ExportLimits, NativeFormat,
};
use careops_ed::{parse_scenario_json, run_scenario, RunSummary, ScenarioConfig};
use serde_json::json;

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
fn test_feature_api_validation_and_standard_hash_vectors() {
    let nominal_content = fs::read_to_string(nominal_path()).expect("read nominal_mvp.json");
    let report: ValidationSuccess = validate_scenario_source(&nominal_content).unwrap();
    assert_eq!(report.scenario_id, "nominal-ed-mvp");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.time_unit, "tick");
    assert_eq!(report.horizon_ticks, 20);
    assert_eq!(report.patient_count, 5);
    let bad = nominal_content.replace("\"schema_version\": 1,", "\"schema_version\": 99,");
    assert_eq!(
        validate_scenario_source(&bad).unwrap_err().code,
        ExitCodeDef::MalformedConfig
    );
    assert_eq!(
        compute_sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        compute_sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

// ---------------------------------------------------------------------------
// 2. Deterministic Replay and Worker Concurrency
// ---------------------------------------------------------------------------

#[test]
fn historical_nominal_fixture_preserves_exact_independent_domain_facts() {
    let bytes =
        fs::read(repo_root().join("conductor/evidence/r1-native-baseline-20261021/ed/first.json"))
            .unwrap();
    assert_eq!(
        compute_sha256_hex(&bytes),
        "0988abef23d353a1178d2ebefbcde4debec97a4b352307aeabbf1647bab6fda7"
    );
    // This reads immutable historical evidence only; no alpha output is produced.
    let baseline: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let expected: RunSummary = serde_json::from_value(baseline["summary"].clone()).unwrap();
    let input = fs::read(nominal_path()).unwrap();
    assert_eq!(
        compute_sha256_hex(&input),
        baseline["manifest"]["input_sha256"].as_str().unwrap()
    );
    let mut scenario = parse_scenario_json(std::str::from_utf8(&input).unwrap()).unwrap();
    scenario.seed = baseline["manifest"]["seed"].as_u64().unwrap();
    scenario.horizon_ticks = baseline["manifest"]["horizon_ticks"].as_u64().unwrap();
    assert_eq!(run_scenario(&scenario).unwrap(), expected);
}

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

    let mut overridden = scenario.clone();
    overridden.seed = 999;
    overridden.horizon_ticks = 50;
    let first = run_scenario(&overridden).unwrap();
    let second = run_scenario(&overridden).unwrap();
    assert_eq!(first.seed, 999);
    assert_eq!(first.horizon_ticks, 50);
    assert_eq!(first, second);
    assert_eq!(
        compute_sha256_hex(&serde_json::to_vec(&first).unwrap()),
        compute_sha256_hex(&serde_json::to_vec(&second).unwrap())
    );

    // B. Batch worker equivalence: batch runner with 1, 2, and 4 workers
    let cfg_1 = BatchConfig {
        batch_id: "c63-worker-eq".to_string(),
        scenario_id: "nominal-ed-mvp".to_string(),
        base_seed: 54321,
        replications: 4,
        num_workers: 1,
        horizon_override: Some(200),
        checkpoint_interval: None,
        ..Default::default()
    };

    let cfg_2 = BatchConfig {
        batch_id: "c63-worker-eq".to_string(),
        num_workers: 2,
        ..cfg_1.clone()
    };

    let cfg_4 = BatchConfig {
        batch_id: "c63-worker-eq".to_string(),
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

    #[cfg(feature = "native-data")]
    {
        let mut datasets = Vec::new();
        for batch in [&res_1, &res_2, &res_4] {
            let root = tempfile::tempdir().unwrap();
            let manifest = export_native_batch(
                &scenario,
                batch,
                root.path(),
                NativeFormat::Parquet,
                ExportLimits::default(),
            )
            .unwrap();
            assert_eq!(
                manifest
                    .tables
                    .iter()
                    .map(|t| t.name.as_str())
                    .collect::<Vec<_>>(),
                ["patients", "replications", "metrics"]
            );
            datasets.push(read_native_dataset(root.path()).unwrap());
        }
        for other in &datasets[1..] {
            assert!(datasets[0].patients().equals_missing(other.patients()));
            assert!(datasets[0]
                .replications()
                .equals_missing(other.replications()));
            assert!(datasets[0].metrics().equals_missing(other.metrics()));
        }
    }
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
    }
    // Actual CLI comparison/export timing moved to c63_native_contract in the CLI crate.
}

// ---------------------------------------------------------------------------
// 4. Validity Limits and Diagnostics
// ---------------------------------------------------------------------------

#[test]
fn test_validity_limits_and_diagnostics() {
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
