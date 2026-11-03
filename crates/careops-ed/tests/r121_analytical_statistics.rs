//! Independent R12 statistical and typed-error oracles.
//! Frozen vectors and original behavioral failures are retained in R12.1 history;
//! native and minimal execution now verify the corrected production paths.

use careops_ed::disposition::api_runner::ExitCodeDef;
use careops_ed::disposition::arrow_export::compute_uncertainty;
use careops_ed::disposition::batch_runner::{
    compute_batch_aggregates, execute_single_replication, ReplicationResult,
};
#[cfg(feature = "native-data")]
use careops_ed::disposition::{
    arrow_export::summarize_replications,
    batch_runner::{run_batch_with_workers, BatchConfig},
    native_output::{
        export_native_batch, read_native_dataset, ExportLimits, NativeDataset, NativeFormat,
    },
};
use careops_ed::parse_scenario_json;
use serde_json::Value;

fn vectors() -> Value {
    serde_json::from_str(include_str!("fixtures/r121_analytical.json")).unwrap()
}

#[cfg(feature = "native-data")] // Pinned native provider is absent in the minimal graph.
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
    let (mean, se, completion) =
        compute_batch_aggregates(&[row(0, 1, 1, 0.0), row(1, 9, 0, 10.0)]).unwrap();
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
    let uncertainty = serde_json::to_value(compute_uncertainty(&values).unwrap()).unwrap();
    assert_eq!(uncertainty["variance"], expected["sample_variance"]);
    assert_eq!(uncertainty["standard_error"], expected["standard_error"]);
}

#[test]
fn empty_uncertainty_is_undefined() {
    let value = serde_json::to_value(compute_uncertainty(&[]).unwrap()).unwrap();
    assert_eq!(value["variance"], Value::Null);
    assert_eq!(value["standard_error"], Value::Null);
    assert_eq!(value["ci_95_lower"], Value::Null);
}

#[test]
fn singleton_uncertainty_is_undefined() {
    let value = serde_json::to_value(compute_uncertainty(&[7.0]).unwrap()).unwrap();
    assert_eq!(value["variance"], Value::Null);
    assert_eq!(value["standard_error"], Value::Null);
    assert_eq!(value["ci_95_lower"], Value::Null);
}

#[test]
fn identical_pair_has_defined_zero_variance() {
    let value = serde_json::to_value(compute_uncertainty(&[7.0, 7.0]).unwrap()).unwrap();
    assert_eq!(value["variance"], 0.0);
    assert_eq!(value["standard_error"], 0.0);
}

#[test]
fn empty_and_zero_arrival_scalar_conventions_are_separate() {
    assert_eq!(compute_batch_aggregates(&[]).unwrap(), (0.0, 0.0, 0.0));
    let (mean, _, completion) =
        compute_batch_aggregates(&[row(0, 0, 0, 0.0), row(1, 10, 10, 10.0)]).unwrap();
    assert_eq!(mean, 5.0);
    assert_eq!(completion, 1.0);
}

fn nonfinite_cannot_be_success(value: f64) {
    let error = compute_batch_aggregates(&[row(0, 1, 1, value), row(1, 1, 1, 1.0)]).unwrap_err();
    assert_eq!(error.code, ExitCodeDef::MalformedConfig);
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
    let error =
        compute_batch_aggregates(&[row(0, usize::MAX, 0, 0.0), row(1, 1, 0, 0.0)]).unwrap_err();
    assert_eq!(error.code, ExitCodeDef::MalformedConfig);
}

#[test]
fn finite_input_with_nonfinite_derived_variance_is_a_typed_error() {
    let error = compute_uncertainty(&[0.0, 1e308]).unwrap_err();
    assert_eq!(error.code, ExitCodeDef::MalformedConfig);
}

#[cfg(feature = "native-data")] // Actual native capture/export capability.
fn two_replication_dataset() -> NativeDataset {
    let scenario = parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let config = BatchConfig {
        scenario_id: scenario.scenario_id.clone(),
        replications: 2,
        num_workers: 1,
        base_seed: 7,
        ..BatchConfig::default()
    };
    let batch = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    native_dataset(&scenario, &batch)
}

#[cfg(feature = "native-data")]
fn native_dataset(
    scenario: &careops_ed::ScenarioConfig,
    batch: &careops_ed::disposition::batch_runner::BatchRunDocument,
) -> NativeDataset {
    let destination = tempfile::tempdir().unwrap();
    export_native_batch(
        scenario,
        batch,
        destination.path(),
        NativeFormat::Parquet,
        ExportLimits::default(),
    )
    .unwrap();
    read_native_dataset(destination.path()).unwrap()
}

#[cfg(feature = "native-data")] // Actual native capture/export capability.
#[test]
fn pooled_completion_has_no_unqualified_unweighted_rate_interval() {
    let data = two_replication_dataset();
    let metrics = data.metrics();
    let row = (0..metrics.height())
        .find(|&i| metrics.column("name").unwrap().str().unwrap().get(i) == Some("completion_rate"))
        .unwrap();
    assert_eq!(
        metrics
            .column("ci_95_lower")
            .unwrap()
            .f64()
            .unwrap()
            .get(row),
        None
    );
}

#[cfg(feature = "native-data")] // Actual native capture/export capability.
#[test]
fn pooled_elapsed_has_no_unqualified_patient_independence_interval() {
    let data = two_replication_dataset();
    let metrics = data.metrics();
    let row = (0..metrics.height())
        .find(|&i| {
            metrics.column("name").unwrap().str().unwrap().get(i) == Some("mean_elapsed_ticks")
        })
        .unwrap();
    assert_eq!(
        metrics
            .column("ci_95_lower")
            .unwrap()
            .f64()
            .unwrap()
            .get(row),
        None
    );
}

#[cfg(feature = "native-data")] // Actual native capture/export capability.
#[test]
fn patient_work_capture_cannot_manufacture_spatial_movements() {
    let data = two_replication_dataset();
    assert_eq!(
        data.manifest()
            .tables
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>(),
        ["patients", "replications", "metrics"]
    );
}

#[cfg(feature = "native-data")]
#[test]
fn native_all_empty_records_match_frozen_points_uncertainty_and_counts() {
    let fixture = vectors();
    let case = &fixture["all_empty_replications"];
    let rows: Vec<_> = case["replications"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mut result = row(
                i,
                r["arrivals"].as_u64().unwrap() as usize,
                r["completed"].as_u64().unwrap() as usize,
                r["mean_wait_ticks"].as_f64().unwrap(),
            );
            result.started = r["started"].as_u64().unwrap() as usize;
            result.horizon_ticks = r["horizon_ticks"].as_u64().unwrap();
            result
        })
        .collect();
    let records = summarize_replications(&rows, &[], "tick").unwrap();
    for record in records {
        let actual = serde_json::to_value(&record).unwrap();
        let expected = &case["expected_metrics"][&record.name];
        assert_eq!(actual["value"], expected["point"], "{} point", record.name);
        assert_eq!(actual["uncertainty"]["variance"], expected["variance"]);
        assert_eq!(
            actual["uncertainty"]["status"],
            expected["uncertainty_status"]
        );
        for count in [
            "total_replications",
            "eligible_replications",
            "included_replications",
        ] {
            assert_eq!(actual[count], expected[count], "{} {count}", record.name);
        }
    }
}

#[cfg(feature = "native-data")]
#[test]
fn native_weighted_points_and_metadata_match_independent_estimands() {
    let records =
        summarize_replications(&[row(0, 1, 1, 0.0), row(1, 9, 0, 10.0)], &[1], "tick").unwrap();
    let metric = |name: &str| records.iter().find(|r| r.name == name).unwrap();
    assert_eq!(metric("mean_wait_ticks").value, Some(5.0));
    assert_eq!(metric("mean_wait_ticks").uncertainty.variance, Some(50.0));
    assert_eq!(
        metric("mean_wait_ticks").uncertainty.standard_error,
        Some(5.0)
    );
    let completion = metric("completion_rate");
    assert_eq!(completion.value, Some(0.1));
    assert_eq!(completion.numerator_count, Some(1));
    assert_eq!(completion.denominator_count, Some(10));
    assert_eq!(completion.uncertainty.status, "ratio_method_unqualified");
    assert_eq!(completion.uncertainty.ci_95_lower, None);
    // Equal replication rates: (0/1 + 9/9)/2, not pooled 9/10.
    assert_eq!(metric("censored_presentation_rate").value, Some(0.5));
}

#[cfg(feature = "native-data")]
#[test]
fn native_pooled_elapsed_matches_frozen_vector_without_independence_interval() {
    let fixture = vectors();
    let case = &fixture["pooled_elapsed"];
    let elapsed: Vec<_> = case["completed_durations_by_replication"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|rep| rep.as_array().unwrap())
        .map(|v| u128::from(v.as_u64().unwrap()))
        .collect();
    let records =
        summarize_replications(&[row(0, 2, 2, 0.0), row(1, 1, 1, 0.0)], &elapsed, "tick").unwrap();
    let metric = records
        .iter()
        .find(|r| r.name == "mean_elapsed_ticks")
        .unwrap();
    assert_eq!(metric.value, case["point"].as_f64());
    assert_eq!(
        metric.uncertainty.status,
        case["uncertainty_status"].as_str().unwrap()
    );
    assert_eq!(metric.uncertainty.variance, None);
    assert_eq!(metric.uncertainty.ci_95_lower, None);
}

#[test]
fn inconsistent_empty_replication_mean_is_a_typed_error() {
    let error = compute_batch_aggregates(&[row(0, 0, 0, 100.0), row(1, 10, 10, 10.0)]).unwrap_err();
    assert_eq!(error.code, ExitCodeDef::MalformedConfig);
}

#[cfg(feature = "native-data")]
#[test]
fn native_records_reject_inconsistent_empty_replication_mean() {
    let error = summarize_replications(
        &[row(0, 0, 0, 100.0), row(1, 10, 10, 10.0)],
        &[1; 10],
        "tick",
    )
    .unwrap_err();
    assert_eq!(error.code, ExitCodeDef::MalformedConfig);
}

#[cfg(feature = "native-data")]
#[test]
fn native_throughput_keeps_equal_replication_weights_for_different_horizons() {
    let mut first = row(0, 1, 1, 0.0);
    first.horizon_ticks = 1;
    let mut second = row(1, 1, 1, 0.0);
    second.horizon_ticks = 100;
    let records = summarize_replications(&[first, second], &[1, 1], "tick").unwrap();
    let metric = records
        .iter()
        .find(|r| r.name == "throughput_per_horizon_tick")
        .unwrap();
    assert_eq!(metric.value, Some(0.505)); // (1/1 + 1/100)/2, not2/101.
    assert_eq!(metric.denominator_value, None);
    assert_eq!(metric.denominator_count, None);
    assert_eq!(metric.weighting, "equal_replication");
}

#[cfg(feature = "native-data")]
#[test]
fn native_no_replications_has_no_point_or_interval() {
    let records = summarize_replications(&[], &[], "tick").unwrap();
    assert_eq!(records.len(), 5);
    for record in records {
        assert_eq!(record.value, None);
        assert_eq!(record.uncertainty.variance, None);
        assert_eq!(record.uncertainty.ci_95_lower, None);
        assert_eq!(record.uncertainty.status, "no_samples");
        assert_eq!(record.total_replications, 0);
        assert_eq!(record.included_replications, 0);
    }
}

#[cfg(feature = "native-data")]
#[test]
fn native_capture_configured_location_is_association_not_movement() {
    let scenario = parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let config = BatchConfig {
        scenario_id: scenario.scenario_id.clone(),
        replications: 1,
        num_workers: 1,
        ..BatchConfig::default()
    };
    let batch = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    let data = native_dataset(&scenario, &batch);
    let patients = data.patients();
    assert!(patients.height() > 0);
    for row in 0..patients.height() {
        let resource_id = patients
            .column("resource_id")
            .unwrap()
            .str()
            .unwrap()
            .get(row)
            .unwrap();
        let resource = scenario
            .capacity_location
            .resource_buckets
            .iter()
            .find(|r| r.resource_id == resource_id)
            .unwrap();
        assert_eq!(
            patients
                .column("location_id")
                .unwrap()
                .str()
                .unwrap()
                .get(row),
            Some(resource.location_id.as_str())
        );
    }
    // Native output carries configured associations, never fabricated movement observations.
    assert_eq!(
        data.manifest()
            .tables
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>(),
        ["patients", "replications", "metrics"]
    );
}

#[cfg(feature = "native-data")]
#[test]
fn native_missing_mandatory_resource_buckets_is_a_domain_refusal() {
    let mut scenario = parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    scenario.capacity_location.resource_buckets.clear();
    let config = BatchConfig {
        scenario_id: scenario.scenario_id.clone(),
        replications: 1,
        num_workers: 1,
        ..BatchConfig::default()
    };
    let error = run_batch_with_workers(&scenario, &config, None, None, None).unwrap_err();
    assert_eq!(error.code, ExitCodeDef::SimulationFailed); // Existing domain/API taxonomy.
    assert!(error.message.contains("resource_buckets must not be empty"));
}

#[test]
fn valid_large_waits_do_not_overflow_the_replication_mean() {
    let mut scenario = parse_scenario_json(include_str!("../examples/one_patient.json")).unwrap();
    scenario.horizon_ticks = u64::MAX;
    let template = scenario.patients[0].clone();
    scenario.patients = (0..4)
        .map(|i| {
            let mut patient = template.clone();
            patient.patient_id = format!("large-wait-{i}");
            patient.arrival_tick = 0;
            patient.work_ticks = if i == 0 { u64::MAX / 2 } else { 1 };
            patient
        })
        .collect();
    let result = execute_single_replication(&scenario, 0, 7, None).unwrap();
    assert_eq!(result.completed, 4);
    // Exact waits:0,h,h+1,h+2 with h=2^63-1. Mean=3*2^61.
    assert_eq!(result.mean_wait_ticks, 3.0 * (1u64 << 61) as f64);
}
