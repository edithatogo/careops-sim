use careops_ed::disposition::composite_fixture::{
    export_patient_calibration_traces, ComposedPatient, PatientLifecycle,
};
fn patient() -> ComposedPatient {
    ComposedPatient {
        id: 7,
        name: "synthetic-7".into(),
        acuity: 2,
        arrival_tick: 3,
        work_required_ticks: 5,
        work_elapsed_ticks: 5,
        start_tick: None,
        completion_tick: None,
        assigned_bay: None,
        lifecycle: PatientLifecycle::Discharged,
        task_class_id: "t".into(),
        zone_id: "z".into(),
        location_id: "l".into(),
    }
}
#[test]
fn missing_calibration_endpoints_are_null_not_imputed() {
    let records = export_patient_calibration_traces(&[patient()], 1, 0);
    let records = serde_json::to_value(records).unwrap();
    assert!(records[0]["start_tick"].is_null());
    assert!(records[0]["end_tick"].is_null());
}
#[test]
fn missing_completion_does_not_add_work_or_overflow() {
    let mut p = patient();
    p.start_tick = Some(u64::MAX);
    let outcome = std::panic::catch_unwind(|| export_patient_calibration_traces(&[p], 1, 0));
    assert!(
        outcome.is_ok(),
        "absent completion must not trigger timestamp arithmetic"
    );
    let records = serde_json::to_value(outcome.unwrap()).unwrap();
    assert!(records[0]["end_tick"].is_null());
}

#[cfg(feature = "native-data")]
mod native {
    use super::*;
    use careops_ed::disposition::composite_fixture::{
        native_calibration::calibration_table, CalibrationTransitSource, CompositeEdFixture,
    };
    use careops_ed::disposition::operations::{ExecutionFidelity, MovementEdge, SyntheticGraph};
    use polars::prelude::*;
    #[test]
    fn native_null_pairs_max_ticks_and_declared_source_survive() {
        let mut p = patient();
        p.start_tick = Some(u64::MAX);
        p.completion_tick = Some(u64::MAX);
        let records = export_patient_calibration_traces(&[p, patient()], 1, 0);
        // Distinct patient keys, including a missing endpoint pair.
        let mut records = records;
        records[1].case_id = "case_8".into();
        let frame = calibration_table(&records, "fixture-1", "tick").unwrap();
        assert_eq!(frame.height(), 2);
        assert_eq!(
            frame
                .column("interval_basis")
                .unwrap()
                .str()
                .unwrap()
                .get(0),
            Some("end_minus_start")
        );
        assert_eq!(
            frame.column("start_tick_lo").unwrap().u64().unwrap().get(0),
            Some(u64::MAX)
        );
        assert_eq!(
            frame.column("start_tick_hi").unwrap().u64().unwrap().get(0),
            Some(0)
        );
        assert_eq!(frame.column("start_tick_lo").unwrap().null_count(), 1);
        assert_eq!(frame.column("start_tick_hi").unwrap().null_count(), 1);
        assert_eq!(
            frame
                .column("interval_ticks_lo")
                .unwrap()
                .u64()
                .unwrap()
                .get(0),
            Some(0)
        );
        assert_eq!(
            frame
                .column("interval_ticks_lo")
                .unwrap()
                .u64()
                .unwrap()
                .get(1),
            None
        );
        assert_eq!(
            frame
                .column("transit_source")
                .unwrap()
                .str()
                .unwrap()
                .get(0),
            Some("caller_supplied")
        );
        assert_eq!(
            frame.column("source_kind").unwrap().str().unwrap().get(0),
            Some("synthetic_fixture")
        );
        assert_eq!(
            frame.column("schema").unwrap().str().unwrap().get(0),
            Some("careops.ed.synthetic-fixture-calibration.v2")
        );
    }
    #[test]
    fn independently_missing_start_and_end_are_not_reconstructed() {
        for (start, end) in [(None, Some(8)), (Some(3), None), (None, None)] {
            let mut p = patient();
            p.start_tick = start;
            p.completion_tick = end;
            let frame = calibration_table(
                &export_patient_calibration_traces(&[p], 1, 0),
                "fixture-1",
                "tick",
            )
            .unwrap();
            assert_eq!(
                frame.column("start_tick_lo").unwrap().u64().unwrap().get(0),
                start
            );
            assert_eq!(
                frame.column("end_tick_lo").unwrap().u64().unwrap().get(0),
                end
            );
            assert_eq!(
                frame
                    .column("interval_ticks_lo")
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(0),
                None
            );
        }
    }
    #[test]
    fn duplicate_keys_reject_but_same_case_on_distinct_days_is_valid() {
        let record = export_patient_calibration_traces(&[patient()], 1, 0).remove(0);
        assert!(calibration_table(&[record.clone(), record.clone()], "fixture-1", "tick").is_err());
        let mut next = record.clone();
        next.day = 2;
        let table = calibration_table(&[next, record], "fixture-1", "tick").unwrap();
        assert_eq!(table.column("day").unwrap().u32().unwrap().get(0), Some(1));
        assert_eq!(table.height(), 2);
    }
    #[test]
    fn empty_native_table_is_typed_without_placeholder_records() {
        let table = calibration_table(&[], "fixture-1", "tick").unwrap();
        assert_eq!(table.height(), 0);
        assert_eq!(
            table.column("start_tick_lo").unwrap().dtype(),
            &DataType::UInt64
        );
        assert!(calibration_table(&[], "", "tick").is_err());
        assert!(calibration_table(&[], "fixture-1", "").is_err());
    }
    #[test]
    fn negative_service_elapsed_is_typed_refusal() {
        let mut p = patient();
        p.start_tick = Some(8);
        p.completion_tick = Some(3);
        assert!(calibration_table(
            &export_patient_calibration_traces(&[p], 1, 0),
            "fixture-1",
            "tick"
        )
        .is_err());
    }
    fn micro(seconds: f64) -> CompositeEdFixture {
        let mut fixture = CompositeEdFixture::new(1, 1, "r", vec![], ExecutionFidelity::Micro);
        fixture.completed.push(patient());
        fixture.set_graph(SyntheticGraph::new(vec![MovementEdge {
            edge_id: "edge".into(),
            from: "a".into(),
            to: "b".into(),
            distance_m: seconds,
            speed_m_per_s: 1.0,
            accessible: true,
        }]));
        fixture
    }
    #[test]
    fn route_source_seconds_and_floor_mapping_are_explicit() {
        let fixture = micro(3.75);
        let records = fixture.export_calibration_traces(1, &["edge"]).unwrap();
        assert_eq!(records[0].transit_ticks, 3);
        assert_eq!(records[0].transit_source_seconds, Some(3.75));
        assert!(fixture
            .calibration_table(1, &["edge"], "fixture-1", "tick")
            .is_err());
        let table = fixture
            .calibration_table(1, &["edge"], "fixture-1", "second")
            .unwrap();
        assert_eq!(
            table
                .column("transit_source_seconds")
                .unwrap()
                .f64()
                .unwrap()
                .get(0),
            Some(3.75)
        );
        assert_eq!(
            table
                .column("transit_conversion")
                .unwrap()
                .str()
                .unwrap()
                .get(0),
            Some("floor_seconds")
        );
    }
    #[test]
    fn invalid_or_unavailable_route_projection_never_becomes_zero() {
        for seconds in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            -1.0,
            u64::MAX as f64,
        ] {
            assert!(micro(seconds)
                .export_calibration_traces(1, &["edge"])
                .is_err());
        }
        assert!(micro(0.0).export_calibration_traces(1, &[]).is_err());
        let fixture = CompositeEdFixture::new(1, 1, "r", vec![], ExecutionFidelity::Micro);
        assert!(fixture.export_calibration_traces(1, &["edge"]).is_err());
    }
    #[test]
    fn macro_zero_and_caller_zero_have_distinct_provenance() {
        let mut fixture = CompositeEdFixture::new(1, 1, "r", vec![], ExecutionFidelity::Macro);
        fixture.completed.push(patient());
        let macro_records = fixture.export_calibration_traces(1, &[]).unwrap();
        let caller = export_patient_calibration_traces(&[patient()], 1, 0);
        assert_eq!(macro_records[0].transit_ticks, caller[0].transit_ticks);
        assert_ne!(macro_records[0].transit_source, caller[0].transit_source);
        let mut inconsistent = macro_records[0].clone();
        inconsistent.transit_ticks = 1;
        assert!(calibration_table(&[inconsistent], "fixture-1", "tick").is_err());
        let mut inconsistent = caller[0].clone();
        inconsistent.transit_source = CalibrationTransitSource::RouteModel;
        assert!(calibration_table(&[inconsistent], "fixture-1", "second").is_err());
    }
}
