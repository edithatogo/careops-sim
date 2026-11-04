#![cfg(feature = "native-data")]
use careops_ed::disposition::api_runner::ExitCodeDef;
use careops_ed::disposition::arrow_export::native_tables::{
    associate_patients, patient_replication_summary, patient_table,
};
use careops_ed::disposition::batch_runner::{run_batch_with_workers, BatchConfig};
use careops_ed::parse_scenario_json;
use polars::prelude::*;
use serde_json::Value;
fn vectors() -> Value {
    serde_json::from_str(include_str!("fixtures/r121_analytical.json")).unwrap()
}
fn facts(key: &str) -> DataFrame {
    let v = vectors();
    let rows = v["joins"][key].as_array().unwrap();
    df!("run_id"=> rows.iter().map(|r|r[0].as_str().unwrap()).collect::<Vec<_>>(),
        "replication_index"=>rows.iter().map(|r|r[1].as_u64().unwrap()).collect::<Vec<_>>(),
        "patient_id"=>rows.iter().map(|r|r[2].as_str().unwrap()).collect::<Vec<_>>(),
        "sequence"=>vec![0u64,0], "scenario_id"=>vec!["s","s"])
    .unwrap()
}
fn dimensions(resource: &str) -> (DataFrame, DataFrame) {
    (
        df!("scenario_id"=>["s"],"patient_id"=>["p0"],"resource_id"=>[resource]).unwrap(),
        df!("scenario_id"=>["s"],"resource_id"=>["a"],"location_id"=>["bay"]).unwrap(),
    )
}
#[test]
fn frozen_cross_replication_identity_is_valid() {
    let (p, r) = dimensions("a");
    let result = associate_patients(facts("valid_cross_replication"), p, r).unwrap();
    assert_eq!(result.height(), 2);
    assert_eq!(
        result.column("location_id").unwrap().str().unwrap().get(1),
        Some("bay")
    );
    assert!(result.column("_patient_present").is_err());
}
#[test]
fn frozen_duplicate_fact_is_typed_error() {
    let (p, r) = dimensions("a");
    assert_eq!(
        associate_patients(facts("duplicate_fact"), p, r)
            .unwrap_err()
            .code,
        ExitCodeDef::MalformedConfig
    );
}
#[test]
fn frozen_duplicate_dimension_is_typed_error() {
    let v = vectors();
    let rows = v["joins"]["duplicate_dimension"].as_array().unwrap();
    let p=df!("scenario_id"=>rows.iter().map(|r|r[0].as_str().unwrap()).collect::<Vec<_>>(),"patient_id"=>rows.iter().map(|r|r[1].as_str().unwrap()).collect::<Vec<_>>(),"resource_id"=>rows.iter().map(|r|r[2].as_str().unwrap()).collect::<Vec<_>>()).unwrap();
    let (_, r) = dimensions("a");
    assert_eq!(
        associate_patients(facts("valid_cross_replication"), p, r)
            .unwrap_err()
            .code,
        ExitCodeDef::MalformedConfig
    );
}
#[test]
fn frozen_missing_resource_is_typed_error() {
    let v = vectors();
    let (p, r) = dimensions(
        v["joins"]["missing_resource"]["patient_resource"]
            .as_str()
            .unwrap(),
    );
    assert_eq!(
        associate_patients(facts("valid_cross_replication"), p, r)
            .unwrap_err()
            .code,
        ExitCodeDef::MalformedConfig
    );
}
#[test]
fn duplicate_resources_and_missing_patients_reject() {
    let (p, r) = dimensions("a");
    let doubled = r.vstack(&r).unwrap();
    assert!(associate_patients(facts("valid_cross_replication"), p.clone(), doubled).is_err());
    assert!(associate_patients(facts("valid_cross_replication"), p.clear(), r).is_err());
}
#[test]
fn null_keys_and_reserved_columns_reject() {
    let (p, r) = dimensions("a");
    let mut f = facts("valid_cross_replication");
    f.with_column(Series::new("patient_id".into(), [None::<&str>, Some("p0")]).into())
        .unwrap();
    assert!(associate_patients(f, p.clone(), r.clone()).is_err());
    let mut f = facts("valid_cross_replication");
    f.with_column(Series::new("location_id".into(), ["fake", "fake"]).into())
        .unwrap();
    assert!(associate_patients(f, p, r).is_err());
}
#[test]
fn captured_sequence_is_canonical_not_patient_or_tick_order() {
    let (p, r) = dimensions("a");
    let mut f = facts("valid_cross_replication");
    f.with_column(Series::new("replication_index".into(), [0u64, 0]).into())
        .unwrap();
    f.with_column(Series::new("sequence".into(), [1u64, 0]).into())
        .unwrap();
    f.with_column(Series::new("patient_id".into(), ["p1", "p0"]).into())
        .unwrap();
    let p = p
        .vstack(&df!("scenario_id"=>["s"],"patient_id"=>["p1"],"resource_id"=>["a"]).unwrap())
        .unwrap();
    let joined = associate_patients(f, p, r).unwrap();
    let ids = joined.column("patient_id").unwrap().str().unwrap();
    assert_eq!([ids.get(0), ids.get(1)], [Some("p0"), Some("p1")]);
}
#[test]
fn actual_capture_tables_preserve_groups_nulls_and_raw_limbs() {
    let scenario = parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let config = BatchConfig {
        scenario_id: scenario.scenario_id.clone(),
        replications: 2,
        num_workers: 2,
        base_seed: 7,
        ..BatchConfig::default()
    };
    let doc = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    let table = patient_table(&scenario, &doc).unwrap();
    let summary = patient_replication_summary(table.clone()).unwrap();
    assert_eq!(
        table.height(),
        doc.replications.iter().map(|r| r.arrivals).sum::<usize>()
    );
    assert_eq!(summary.height(), 2);
    for (i, rep) in doc.replications.iter().enumerate() {
        assert_eq!(
            summary.column("arrivals").unwrap().u64().unwrap().get(i),
            Some(rep.arrivals as u64)
        );
        assert_eq!(
            summary.column("completed").unwrap().u64().unwrap().get(i),
            Some(rep.completed as u64)
        );
    }
    assert_eq!(
        table
            .column("arrival_tick_hi")
            .unwrap()
            .u64()
            .unwrap()
            .max(),
        Some(0)
    );
    assert_eq!(
        table.column("start_tick_lo").unwrap().null_count(),
        table.column("start_tick_hi").unwrap().null_count()
    );
    assert!(table.column("location_id").is_ok());
    assert!(table.column("transit").is_err());
}
#[test]
fn typed_empty_analytical_projection_has_no_fake_rows() {
    let scenario = parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let config = BatchConfig {
        scenario_id: scenario.scenario_id.clone(),
        replications: 2,
        num_workers: 1,
        base_seed: 7,
        ..BatchConfig::default()
    };
    let doc = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    let table = patient_table(&scenario, &doc).unwrap().clear();
    // A real runner cohort cannot be empty under the current scenario contract.
    // Exercise typed no-data analytical projection without claiming such a run.
    assert_eq!(table.height(), 0);
    assert_eq!(
        table.column("start_tick_lo").unwrap().dtype(),
        &DataType::UInt64
    );
    assert_eq!(patient_replication_summary(table).unwrap().height(), 0);
}

#[test]
fn sequence_gaps_and_duplicate_ordinals_reject() {
    for ordinals in [[0u64, 0], [0, 2], [1, 2]] {
        let (p, r) = dimensions("a");
        let mut f = facts("valid_cross_replication");
        f.with_column(Series::new("replication_index".into(), [0u64, 0]).into())
            .unwrap();
        f.with_column(Series::new("patient_id".into(), ["p0", "p1"]).into())
            .unwrap();
        f.with_column(Series::new("sequence".into(), ordinals).into())
            .unwrap();
        let p = p
            .vstack(&df!("scenario_id"=>["s"],"patient_id"=>["p1"],"resource_id"=>["a"]).unwrap())
            .unwrap();
        assert_eq!(
            associate_patients(f, p, r).unwrap_err().code,
            ExitCodeDef::MalformedConfig
        );
    }
}
#[test]
fn grouped_wait_uses_frozen_checked_full_width_subtraction() {
    let scenario = parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let config = BatchConfig {
        scenario_id: scenario.scenario_id.clone(),
        replications: 1,
        num_workers: 1,
        ..BatchConfig::default()
    };
    let doc = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    let table = patient_table(&scenario, &doc).unwrap().slice(0, 1);
    for vector in vectors()["tick_subtraction"].as_array().unwrap() {
        let mut f = table.clone();
        for name in [
            "completion_tick_hi",
            "completion_tick_lo",
            "elapsed_ticks_hi",
            "elapsed_ticks_lo",
        ] {
            f.with_column(Series::new(name.into(), [None::<u64>]).into())
                .unwrap();
        }
        for (name, field) in [("arrival_tick", "start"), ("start_tick", "end")] {
            for (limb, i) in [("hi", 0), ("lo", 1)] {
                f.with_column(
                    Series::new(
                        format!("{name}_{limb}").into(),
                        [vector[field][i].as_u64().unwrap()],
                    )
                    .into(),
                )
                .unwrap();
            }
        }
        if vector.get("expected_error").is_some() {
            assert!(patient_replication_summary(f).is_err());
            continue;
        }
        let hi = vector["difference"][0].as_u64().unwrap();
        let lo = vector["difference"][1].as_u64().unwrap();
        f.with_column(Series::new("wait_ticks_hi".into(), [hi]).into())
            .unwrap();
        f.with_column(Series::new("wait_ticks_lo".into(), [lo]).into())
            .unwrap();
        let group = patient_replication_summary(f.clone()).unwrap();
        assert_eq!(
            group
                .column("mean_wait_ticks")
                .unwrap()
                .f64()
                .unwrap()
                .get(0),
            Some((((hi as u128) << 64) | (lo as u128)) as f64)
        );
        f.with_column(Series::new("start_tick_hi".into(), [None::<u64>]).into())
            .unwrap();
        assert!(patient_replication_summary(f).is_err());
    }
}
#[test]
fn all_null_start_columns_preserve_no_sample_mean() {
    let scenario = parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let config = BatchConfig {
        scenario_id: scenario.scenario_id.clone(),
        replications: 1,
        num_workers: 1,
        ..BatchConfig::default()
    };
    let doc = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    let mut table = patient_table(&scenario, &doc).unwrap();
    for name in [
        "start_tick_hi",
        "start_tick_lo",
        "wait_ticks_hi",
        "wait_ticks_lo",
        "completion_tick_hi",
        "completion_tick_lo",
        "elapsed_ticks_hi",
        "elapsed_ticks_lo",
    ] {
        table
            .with_column(Series::new(name.into(), vec![None::<u64>; table.height()]).into())
            .unwrap();
    }
    let group = patient_replication_summary(table).unwrap();
    assert_eq!(
        group.column("started").unwrap().u64().unwrap().get(0),
        Some(0)
    );
    assert_eq!(
        group
            .column("mean_wait_ticks")
            .unwrap()
            .f64()
            .unwrap()
            .get(0),
        None
    );
}

#[test]
fn public_summary_rejects_duplicate_facts_and_half_null_completion() {
    let scenario = parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let config = BatchConfig {
        scenario_id: scenario.scenario_id.clone(),
        replications: 1,
        num_workers: 1,
        ..BatchConfig::default()
    };
    let doc = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    let table = patient_table(&scenario, &doc).unwrap();
    assert_eq!(
        patient_replication_summary(table.vstack(&table).unwrap())
            .unwrap_err()
            .code,
        ExitCodeDef::MalformedConfig
    );
    let mut f = table.slice(0, 1);
    assert!(f
        .column("completion_tick_lo")
        .unwrap()
        .u64()
        .unwrap()
        .get(0)
        .is_some());
    f.with_column(Series::new("completion_tick_hi".into(), [None::<u64>]).into())
        .unwrap();
    assert_eq!(
        patient_replication_summary(f).unwrap_err().code,
        ExitCodeDef::MalformedConfig
    );
}
#[test]
fn exact_dimension_schema_is_required() {
    let (p, _) = dimensions("a");
    let r = df!("scenario_id"=>["s"],"resource_id"=>["a"],"unexpected"=>["fake"]).unwrap();
    assert_eq!(
        associate_patients(facts("valid_cross_replication"), p, r)
            .unwrap_err()
            .code,
        ExitCodeDef::MalformedConfig
    );
}

#[test]
fn public_summary_rejects_completion_before_or_without_start() {
    let scenario = parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let config = BatchConfig {
        scenario_id: scenario.scenario_id.clone(),
        replications: 1,
        num_workers: 1,
        ..BatchConfig::default()
    };
    let doc = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    let table = patient_table(&scenario, &doc).unwrap().slice(0, 1);
    let mut rejected = Vec::new();
    for start in [Some(10u64), None] {
        let mut f = table.clone();
        for (name, value) in [
            ("arrival_tick", Some(0u64)),
            ("start_tick", start),
            ("wait_ticks", start),
            ("completion_tick", Some(5)),
            ("elapsed_ticks", Some(5)),
        ] {
            f.with_column(Series::new(format!("{name}_hi").into(), [value.map(|_| 0u64)]).into())
                .unwrap();
            f.with_column(Series::new(format!("{name}_lo").into(), [value]).into())
                .unwrap();
        }
        rejected.push(patient_replication_summary(f).is_err());
    }
    assert_eq!(
        rejected,
        [true, true],
        "completion-before-start and no-start must both reject"
    );
}
