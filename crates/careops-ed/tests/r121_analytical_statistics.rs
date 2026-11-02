//! Compiling baseline controls for intentional R12 method corrections.
//! These assertions expose actual old behavior; R12.2 adds typed native-table
//! error assertions using the unchanged independent vectors.
#![cfg(feature = "native-data")] // Native export/capture is the capability under test.

use careops_ed::disposition::arrow_export::{compute_uncertainty, generate_arrow_export_bundle};
use careops_ed::disposition::batch_runner::{
    compute_batch_aggregates, run_batch_with_workers, BatchConfig, ReplicationResult,
};
use careops_ed::parse_scenario_json;
use serde_json::Value;

fn vectors() -> Value {
    serde_json::from_str(include_str!("fixtures/r121_analytical.json")).unwrap()
}

#[test]
fn pinned_tick_helper_preserves_borrow_full_width_and_underflow() {
    for case in vectors()["tick_subtraction"].as_array().unwrap() {
        let result = kairo_ecs_arrow::polars_interop::subtract_tick_limbs(
            case["end"][0].as_u64().unwrap(),
            case["end"][1].as_u64().unwrap(),
            case["start"][0].as_u64().unwrap(),
            case["start"][1].as_u64().unwrap(),
        );
        if case.get("expected_error").is_some() {
            assert!(result.is_err());
        } else {
            assert_eq!(
                result.unwrap(),
                (
                    case["difference"][0].as_u64().unwrap(),
                    case["difference"][1].as_u64().unwrap()
                )
            );
        }
    }
}

fn row(index: usize, arrivals: usize, completed: usize, mean: f64) -> ReplicationResult {
    ReplicationResult {
        replication_index: index,
        seed: index as u64 + 7,
        arrivals,
        started: arrivals,
        completed,
        unfinished: arrivals - completed,
        mean_wait_ticks: mean,
        horizon_ticks: 100,
    }
}

#[test]
fn unequal_replications_preserve_wait_weighting_and_pooled_completion() {
    let expected = vectors();
    let expected = &expected["unequal_replications"];
    let (mean, se, completion) = compute_batch_aggregates(&[row(0, 1, 1, 0.0), row(1, 9, 0, 10.0)]);
    assert_eq!(mean, expected["mean_wait"].as_f64().unwrap());
    assert_eq!(se, expected["standard_error"].as_f64().unwrap());
    assert_eq!(completion, expected["pooled_completion"].as_f64().unwrap());
}

#[test]
fn skew_uses_sample_ddof_one() {
    let expected = vectors();
    let expected = &expected["skew"];
    let values: Vec<_> = expected["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    let uncertainty = serde_json::to_value(compute_uncertainty(&values)).unwrap();
    assert_eq!(uncertainty["variance"], expected["sample_variance"]);
    assert_eq!(uncertainty["standard_error"], expected["standard_error"]);
}

#[test]
fn empty_uncertainty_is_undefined() {
    let value = serde_json::to_value(compute_uncertainty(&[])).unwrap();
    assert_eq!(value["variance"], Value::Null);
    assert_eq!(value["standard_error"], Value::Null);
    assert_eq!(value["ci_95_lower"], Value::Null);
}

#[test]
fn singleton_uncertainty_is_undefined() {
    let value = serde_json::to_value(compute_uncertainty(&[7.0])).unwrap();
    assert_eq!(value["variance"], Value::Null);
    assert_eq!(value["standard_error"], Value::Null);
    assert_eq!(value["ci_95_lower"], Value::Null);
}

#[test]
fn identical_pair_has_defined_zero_variance() {
    let value = serde_json::to_value(compute_uncertainty(&[7.0, 7.0])).unwrap();
    assert_eq!(value["variance"], 0.0);
    assert_eq!(value["standard_error"], 0.0);
}

#[test]
fn empty_and_zero_arrival_scalar_conventions_are_separate() {
    assert_eq!(compute_batch_aggregates(&[]), (0.0, 0.0, 0.0));
    let (mean, _, completion) =
        compute_batch_aggregates(&[row(0, 0, 0, 0.0), row(1, 10, 10, 10.0)]);
    assert_eq!(mean, 5.0);
    assert_eq!(completion, 1.0);
}

fn nonfinite_cannot_be_success(value: f64) {
    let (mean, se, _) = compute_batch_aggregates(&[row(0, 1, 1, value), row(1, 1, 1, 1.0)]);
    assert!(
        mean.is_finite() && se.is_finite(),
        "R12.2 must return a typed nonfinite-input error"
    );
}

#[test]
fn nan_cannot_be_a_successful_statistic() {
    nonfinite_cannot_be_success(f64::NAN);
}
#[test]
fn positive_infinity_cannot_be_a_successful_statistic() {
    nonfinite_cannot_be_success(f64::INFINITY);
}
#[test]
fn negative_infinity_cannot_be_a_successful_statistic() {
    nonfinite_cannot_be_success(f64::NEG_INFINITY);
}

#[test]
fn count_overflow_cannot_panic() {
    let result = std::panic::catch_unwind(|| {
        compute_batch_aggregates(&[row(0, usize::MAX, 0, 0.0), row(1, 1, 0, 0.0)])
    });
    assert!(
        result.is_ok(),
        "R12.2 must return a typed overflow error before aggregation"
    );
}

fn two_replication_bundle() -> Value {
    let scenario = parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let config = BatchConfig {
        scenario_id: scenario.scenario_id.clone(),
        replications: 2,
        num_workers: 1,
        base_seed: 7,
        ..BatchConfig::default()
    };
    let batch = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    serde_json::to_value(generate_arrow_export_bundle(&scenario, &batch, None, None).unwrap())
        .unwrap()
}

#[test]
fn pooled_completion_has_no_unqualified_unweighted_rate_interval() {
    let bundle = two_replication_bundle();
    let metric = bundle["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "completion_rate")
        .unwrap();
    assert_eq!(metric["uncertainty"]["ci_95_lower"], Value::Null);
}

#[test]
fn pooled_elapsed_has_no_unqualified_patient_independence_interval() {
    let bundle = two_replication_bundle();
    let metric = bundle["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "mean_elapsed_ticks")
        .unwrap();
    assert_eq!(metric["uncertainty"]["ci_95_lower"], Value::Null);
}

#[test]
fn patient_work_capture_cannot_manufacture_spatial_movements() {
    let bundle = two_replication_bundle();
    assert_eq!(bundle["spatial_records"].as_array().unwrap().len(), 0);
}
