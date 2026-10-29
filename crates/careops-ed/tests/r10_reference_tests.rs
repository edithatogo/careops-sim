//! R10 reference oracles for ED statistics and synthetic patient export rows.
//!
//! These assertions freeze current observable definitions for later Polars
//! replacement. They are software compatibility checks, not clinical evidence.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use careops_ed::disposition::arrow_export::{compute_uncertainty, generate_arrow_export_bundle};
use careops_ed::disposition::batch_runner::{run_batch_with_workers, BatchConfig};
use careops_ed::parse_scenario_json;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn one_patient_scenario() -> careops_ed::ScenarioConfig {
    let path = repo_root().join("crates/careops-ed/examples/one_patient.json");
    let source = fs::read_to_string(path).expect("read synthetic one-patient fixture");
    parse_scenario_json(&source).expect("parse synthetic one-patient fixture")
}

#[test]
fn empty_and_sample_uncertainty_match_independent_reference() {
    let empty = compute_uncertainty(&[]);
    assert_eq!(empty.replications_count, 0);
    assert_eq!(empty.variance, 0.0);
    assert_eq!(empty.standard_error, 0.0);
    assert_eq!(empty.ci_95_lower, 0.0);
    assert_eq!(empty.ci_95_upper, 0.0);

    let singleton = compute_uncertainty(&[42.0]);
    assert_eq!(singleton.replications_count, 1);
    assert_eq!(singleton.variance, 0.0);
    assert_eq!(singleton.standard_error, 0.0);
    assert_eq!(singleton.ci_95_lower, 42.0);
    assert_eq!(singleton.ci_95_upper, 42.0);

    // Independent hand calculation: mean=4, sample variance=20/3,
    // standard error=sqrt((20/3)/4), with the current normal 95% interval.
    let observed = compute_uncertainty(&[1.0, 3.0, 5.0, 7.0]);
    let variance = 20.0 / 3.0;
    let standard_error = (variance / 4.0_f64).sqrt();
    let margin = 1.959964 * standard_error;
    assert_eq!(observed.replications_count, 4);
    assert!((observed.variance - variance).abs() < 1e-12);
    assert!((observed.standard_error - standard_error).abs() < 1e-12);
    assert!((observed.ci_95_lower - (4.0 - margin)).abs() < 1e-12);
    assert!((observed.ci_95_upper - (4.0 + margin)).abs() < 1e-12);
}

#[test]
fn synthetic_export_rows_match_patient_reference_per_replication() {
    let scenario = one_patient_scenario();
    let config = BatchConfig {
        schema_version: 1,
        batch_id: "r10-reference-rows".to_string(),
        scenario_id: scenario.scenario_id.clone(),
        base_seed: 7,
        replications: 3,
        num_workers: 2,
        horizon_override: Some(10),
        checkpoint_interval: None,
    };

    let document = run_batch_with_workers(&scenario, &config, None, None, None)
        .expect("run synthetic reference batch");
    let bundle = generate_arrow_export_bundle(&scenario, &document, None, None)
        .expect("export synthetic reference rows");

    assert_eq!(bundle.patient_records.len(), 3);
    assert_eq!(
        bundle
            .patient_records
            .iter()
            .map(|row| (row.replication_index, row.patient_id.clone()))
            .collect::<BTreeSet<_>>()
            .len(),
        3,
        "each replication must have exactly one row for the synthetic patient"
    );

    for (index, replication) in document.replications.iter().enumerate() {
        let row = bundle
            .patient_records
            .iter()
            .find(|row| row.replication_index == replication.replication_index)
            .expect("one exported patient row for each replication");
        assert_eq!(replication.replication_index, index);
        assert_eq!(row.patient_id, "synthetic-patient-1");
        assert_eq!(row.seed, replication.seed);
        assert_eq!(row.arrival_tick, 1);
        assert_eq!(row.start_tick, Some(1));
        assert_eq!(row.completion_tick, Some(3));
        assert_eq!(row.work_ticks, 2);
        assert_eq!(row.wait_ticks, Some(0));
        assert_eq!(row.elapsed_ticks, Some(2));
        assert_eq!(row.status, "completed");
        assert_eq!(row.censor_status, "uncensored");
    }

    let wait = bundle
        .metrics
        .iter()
        .find(|metric| metric.name == "mean_wait_ticks")
        .expect("mean-wait metric");
    let completion = bundle
        .metrics
        .iter()
        .find(|metric| metric.name == "completion_rate")
        .expect("completion metric");
    assert_eq!(wait.denominator_name, "started_patients");
    assert_eq!(wait.denominator_value, 3.0);
    assert_eq!(completion.denominator_name, "total_arrivals");
    assert_eq!(completion.denominator_value, 3.0);
    assert_eq!(wait.uncertainty.replications_count, 3);
    assert_eq!(completion.uncertainty.replications_count, 3);
}
