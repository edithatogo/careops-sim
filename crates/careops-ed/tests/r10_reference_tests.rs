//! R10 reference oracles for ED statistics and synthetic patient export rows.
//!
//! These assertions freeze current observable definitions for later Polars
//! replacement. They are software compatibility checks, not clinical evidence.

#[cfg(feature = "native-data")]
use std::collections::BTreeSet;
#[cfg(feature = "native-data")]
use std::fs;
#[cfg(feature = "native-data")]
use std::path::PathBuf;

use careops_ed::disposition::arrow_export::compute_uncertainty;
#[cfg(feature = "native-data")]
use careops_ed::disposition::batch_runner::{run_batch_with_workers, BatchConfig};
#[cfg(feature = "native-data")]
use careops_ed::disposition::native_output::{
    export_native_batch, read_native_dataset, ExportLimits, NativeFormat,
};
#[cfg(feature = "native-data")]
use careops_ed::parse_scenario_json;

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
fn one_patient_scenario() -> careops_ed::ScenarioConfig {
    let path = repo_root().join("crates/careops-ed/examples/one_patient.json");
    let source = fs::read_to_string(path).expect("read synthetic one-patient fixture");
    parse_scenario_json(&source).expect("parse synthetic one-patient fixture")
}

#[test]
fn empty_and_sample_uncertainty_match_independent_reference() {
    let empty = compute_uncertainty(&[]).unwrap();
    assert_eq!(empty.replications_count, 0);
    assert_eq!(empty.variance, None);
    assert_eq!(empty.standard_error, None);
    assert_eq!(empty.ci_95_lower, None);
    assert_eq!(empty.ci_95_upper, None);

    let singleton = compute_uncertainty(&[42.0]).unwrap();
    assert_eq!(singleton.replications_count, 1);
    assert_eq!(singleton.variance, None);
    assert_eq!(singleton.standard_error, None);
    assert_eq!(singleton.ci_95_lower, None);
    assert_eq!(singleton.ci_95_upper, None);

    // Independent hand calculation: mean=4, sample variance=20/3,
    // standard error=sqrt((20/3)/4), with the current normal 95% interval.
    let observed = compute_uncertainty(&[1.0, 3.0, 5.0, 7.0]).unwrap();
    let variance = 20.0 / 3.0;
    let standard_error = (variance / 4.0_f64).sqrt();
    let margin = 1.959964 * standard_error;
    assert_eq!(observed.replications_count, 4);
    assert!((observed.variance.unwrap() - variance).abs() < 1e-12);
    assert!((observed.standard_error.unwrap() - standard_error).abs() < 1e-12);
    assert!((observed.ci_95_lower.unwrap() - (4.0 - margin)).abs() < 1e-12);
    assert!((observed.ci_95_upper.unwrap() - (4.0 + margin)).abs() < 1e-12);
}

#[cfg(feature = "native-data")]
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
    let destination = tempfile::tempdir().unwrap();
    export_native_batch(
        &scenario,
        &document,
        destination.path(),
        NativeFormat::Parquet,
        ExportLimits::default(),
    )
    .unwrap();
    let data = read_native_dataset(destination.path()).unwrap();
    let patients = data.patients();
    assert_eq!(patients.height(), 3);
    let mut identities = BTreeSet::new();
    for row in 0..patients.height() {
        identities.insert((
            patients
                .column("replication_index")
                .unwrap()
                .u64()
                .unwrap()
                .get(row)
                .unwrap(),
            patients
                .column("patient_id")
                .unwrap()
                .str()
                .unwrap()
                .get(row)
                .unwrap(),
        ));
    }
    assert_eq!(
        identities.len(),
        3,
        "each replication must have exactly one row for the synthetic patient"
    );
    for (index, replication) in document.replications.iter().enumerate() {
        let row = (0..patients.height())
            .find(|&i| {
                patients
                    .column("replication_index")
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(i)
                    == Some(replication.replication_index as u64)
            })
            .unwrap();
        assert_eq!(replication.replication_index, index);
        assert_eq!(
            patients
                .column("patient_id")
                .unwrap()
                .str()
                .unwrap()
                .get(row),
            Some("synthetic-patient-1")
        );
        assert_eq!(
            patients.column("seed").unwrap().u64().unwrap().get(row),
            Some(replication.seed)
        );
        for (name, expected) in [
            ("arrival_tick", 1),
            ("start_tick", 1),
            ("completion_tick", 3),
            ("work_ticks", 2),
            ("wait_ticks", 0),
            ("elapsed_ticks", 2),
        ] {
            assert_eq!(
                patients
                    .column(&format!("{name}_hi"))
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(row),
                Some(0)
            );
            assert_eq!(
                patients
                    .column(&format!("{name}_lo"))
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(row),
                Some(expected)
            );
        }
        assert_eq!(
            patients.column("status").unwrap().str().unwrap().get(row),
            Some("completed")
        );
        // The native schema preserves censoring through real endpoint nulls and
        // status, rather than the retired DTO's derived censor_status string.
        assert_eq!(replication.unfinished, 0);
        assert_eq!(
            patients
                .column("completion_tick_hi")
                .unwrap()
                .u64()
                .unwrap()
                .get(row),
            Some(0)
        );
        assert_eq!(
            patients
                .column("completion_tick_lo")
                .unwrap()
                .u64()
                .unwrap()
                .get(row),
            Some(3)
        );
    }
    let metrics = data.metrics();
    for (name, denominator) in [
        ("mean_wait_ticks", "included_replications"),
        ("completion_rate", "total_arrivals"),
    ] {
        let row = (0..metrics.height())
            .find(|&i| metrics.column("name").unwrap().str().unwrap().get(i) == Some(name))
            .unwrap();
        assert_eq!(
            metrics
                .column("denominator_name")
                .unwrap()
                .str()
                .unwrap()
                .get(row),
            Some(denominator)
        );
        assert_eq!(
            metrics
                .column("denominator_value")
                .unwrap()
                .f64()
                .unwrap()
                .get(row),
            Some(3.0)
        );
        assert_eq!(
            metrics
                .column("uncertainty_replications")
                .unwrap()
                .u64()
                .unwrap()
                .get(row),
            Some(3)
        );
    }
}
