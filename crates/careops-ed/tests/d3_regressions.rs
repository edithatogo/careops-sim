//! D3 failure detection for the implemented E0 supplied-work slice.
use careops_ed::{parse_scenario_json, run_scenario, PatientStatus, ScenarioConfig};
use serde_json::{json, Value};

fn scenario() -> ScenarioConfig {
    parse_scenario_json(include_str!("../examples/one_patient.json")).unwrap()
}

#[test]
fn malformed_inputs_never_enter_the_runner() {
    for source in ["", "null", "[]", "{", "{\"seed\":-1}"] {
        assert!(parse_scenario_json(source).is_err(), "accepted {source:?}");
    }
    let original: Value =
        serde_json::from_str(include_str!("../examples/one_patient.json")).unwrap();
    for (field, value) in [
        ("seed", json!(-1)),
        ("seed", json!(1.5)),
        ("horizon_ticks", json!(0)),
        ("horizon_ticks", json!("10")),
        ("patients", json!([])),
        ("schema_version", json!(2)),
    ] {
        let mut input = original.clone();
        input[field] = value;
        assert!(
            parse_scenario_json(&input.to_string()).is_err(),
            "accepted {field}"
        );
    }
    let mut invalid = scenario();
    invalid.patients[0].work_ticks = 0;
    assert!(
        run_scenario(&invalid).is_err(),
        "typed input bypassed validation"
    );
}

#[test]
fn fifo_and_metric_accounting_match_an_independent_recurrence() {
    // Exhaust a bounded input grid; the oracle uses single-server arithmetic,
    // independently of the scheduler and its event ordering.
    for count in 1..=12 {
        for spacing in 0..=3 {
            for work in 1..=4 {
                let mut input = scenario();
                input.horizon_ticks = 200;
                let template = input.patients[0].clone();
                input.patients = (0..count)
                    .rev()
                    .map(|index| {
                        let mut patient = template.clone();
                        patient.patient_id = format!("p{index:02}");
                        patient.arrival_tick = 1 + index * spacing;
                        patient.work_ticks = work;
                        patient
                    })
                    .collect();
                let output = run_scenario(&input).unwrap();
                assert_eq!(output.arrivals, count as usize);
                assert_eq!(output.completed, count as usize);
                assert_eq!(output.unfinished, 0);
                let mut available = 0;
                for (index, patient) in output.patients.iter().enumerate() {
                    let arrival = 1 + index as u64 * spacing;
                    let start = available.max(arrival);
                    available = start + work;
                    assert_eq!(patient.patient_id, format!("p{index:02}"));
                    assert_eq!(patient.start_tick, Some(start));
                    assert_eq!(patient.completion_tick, Some(available));
                    assert_eq!(patient.wait_ticks, Some(start - arrival));
                    assert_eq!(patient.elapsed_ticks, Some(available - arrival));
                    assert_eq!(patient.status, PatientStatus::Completed);
                }
                input.patients.reverse();
                assert_eq!(run_scenario(&input).unwrap(), output);
            }
        }
    }
}

#[test]
fn horizon_conserves_queued_and_running_work() {
    let mut input = scenario();
    input.horizon_ticks = 3;
    let mut second = input.patients[0].clone();
    second.patient_id = "z-second".into();
    input.patients.push(second);
    let output = run_scenario(&input).unwrap();
    assert_eq!(
        (
            output.arrivals,
            output.started,
            output.completed,
            output.unfinished
        ),
        (2, 2, 1, 1)
    );
    assert_eq!(output.patients[1].start_tick, Some(3));
    assert_eq!(output.patients[1].completion_tick, None);
    input.horizon_ticks = 2;
    let output = run_scenario(&input).unwrap();
    assert_eq!(
        (
            output.arrivals,
            output.started,
            output.completed,
            output.unfinished
        ),
        (2, 1, 0, 2)
    );
    assert_eq!(output.patients[1].start_tick, None);
}

#[test]
fn seed_change_is_reported_without_inventing_stochastic_behavior() {
    let mut input = scenario();
    let first = run_scenario(&input).unwrap();
    input.seed = u64::MAX;
    let second = run_scenario(&input).unwrap();
    assert_eq!(second.seed, u64::MAX);
    assert_eq!(first.patients, second.patients);
    assert_eq!(run_scenario(&input).unwrap(), second);
}
