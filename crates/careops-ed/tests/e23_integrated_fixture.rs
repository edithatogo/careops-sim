#[path = "support/e13.rs"]
mod e13;

use careops_ed::disposition::DispositionIntent::{Admission, Discharge};
use careops_ed::terminal::{run_terminal, EndpointState, RealizedOutcome};
use e13::{
    assert_conservation, assert_interval_accounting, build, patient, standard, BuildOptions,
    OfferSpec, PatientSpec,
};

fn patient_spec(
    id: &'static str,
    arrival: u64,
    intent: careops_ed::disposition::DispositionIntent,
    stages: [u64; 3],
) -> PatientSpec {
    PatientSpec {
        id,
        arrival,
        intent,
        stages,
        resources: ["res.triage", "res.assessment", "res.treatment"],
    }
}

/// E2.3.fixture oracle:
/// Integrated nominal ED scenario with simultaneous arrivals, mixed discharge and admission pathways.
/// Proves strict patient and capacity conservation across the full lifecycle.
#[test]
fn nominal_integrated_ed_conserves_patients_and_capacity() {
    let patients = vec![
        patient_spec("p1", 1, Discharge, [2, 3, 4]),
        patient_spec("p2", 1, Admission, [1, 4, 3]),
        patient_spec("p3", 2, Discharge, [2, 2, 2]),
        patient_spec("p4", 3, Admission, [3, 2, 4]),
    ];
    let offers = vec![
        OfferSpec {
            id: "off-1",
            destination: "dest.med",
            patient: Some("p2"),
            available: 8,
            withdrawn: 30,
            duration: 1,
        },
        OfferSpec {
            id: "off-2",
            destination: "dest.med",
            patient: Some("p4"),
            available: 15,
            withdrawn: 30,
            duration: 1,
        },
    ];

    let config = build(BuildOptions {
        horizon: 30,
        patients,
        offers,
        discharge_delay: 1,
        care: None,
        capacities: std::collections::BTreeMap::new(),
    });

    let out = run_terminal(&config).unwrap();
    assert_conservation(&out);
    assert_interval_accounting(&config, &out, 30);

    // Verify all patients completed normally before horizon
    let p1 = patient(&out, "p1");
    let p2 = patient(&out, "p2");
    let p3 = patient(&out, "p3");
    let p4 = patient(&out, "p4");

    assert_eq!(p1.endpoint_state, EndpointState::Terminal);
    assert_eq!(p1.realized_outcome, Some(RealizedOutcome::Discharged));

    assert_eq!(p2.endpoint_state, EndpointState::Terminal);
    assert!(matches!(
        p2.realized_outcome,
        Some(RealizedOutcome::Transferred { .. })
    ));

    assert_eq!(p3.endpoint_state, EndpointState::Terminal);
    assert_eq!(p3.realized_outcome, Some(RealizedOutcome::Discharged));

    assert_eq!(p4.endpoint_state, EndpointState::Terminal);
    assert!(matches!(
        p4.realized_outcome,
        Some(RealizedOutcome::Transferred { .. })
    ));
}

/// E2.3.fixture oracle:
/// Simultaneous arrivals competing for the same single triage resource queue deterministically.
#[test]
fn simultaneous_arrivals_compete_deterministically() {
    let patients = vec![
        patient_spec("pat-alpha", 1, Discharge, [3, 2, 2]),
        patient_spec("pat-beta", 1, Discharge, [3, 2, 2]),
    ];

    let config = standard(25, patients);
    let out = run_terminal(&config).unwrap();

    assert_conservation(&out);
    assert_interval_accounting(&config, &out, 25);

    let alpha = patient(&out, "pat-alpha");
    let beta = patient(&out, "pat-beta");

    // Both reached terminal state without deadlock or leaking capacity
    assert_eq!(alpha.endpoint_state, EndpointState::Terminal);
    assert_eq!(beta.endpoint_state, EndpointState::Terminal);

    // One finishes before the other due to single triage capacity (res.triage capacity = 1)
    assert!(alpha.terminal_tick != beta.terminal_tick);
}

/// E2.3.fixture oracle:
/// Horizon censoring correctly accounts for active in-flight patients without state corruption.
#[test]
fn horizon_censoring_preserves_in_flight_state_and_accounting() {
    let patients = vec![
        patient_spec("fast", 1, Discharge, [1, 1, 1]),
        patient_spec("slow", 1, Discharge, [10, 10, 10]),
    ];

    // Horizon is 8 ticks: fast completes, slow is in flight
    let config = standard(8, patients);
    let out = run_terminal(&config).unwrap();

    assert_conservation(&out);
    assert_interval_accounting(&config, &out, 8);

    let fast = patient(&out, "fast");
    let slow = patient(&out, "slow");

    assert_eq!(fast.endpoint_state, EndpointState::Terminal);
    assert_ne!(slow.endpoint_state, EndpointState::Terminal);
    assert_eq!(slow.terminal_tick, None);
    assert!(slow.space.acquired_tick.is_some());
    assert_eq!(slow.space.released_tick, None);
}

/// E2.3.fixture oracle:
/// Execution is repeatable and deterministic across independent runs with identical inputs.
#[test]
fn repeatable_execution_under_identical_inputs() {
    let patients1 = vec![
        patient_spec("p1", 1, Discharge, [2, 3, 4]),
        patient_spec("p2", 2, Discharge, [2, 2, 2]),
    ];
    let patients2 = vec![
        patient_spec("p1", 1, Discharge, [2, 3, 4]),
        patient_spec("p2", 2, Discharge, [2, 2, 2]),
    ];

    let config1 = standard(20, patients1);
    let config2 = standard(20, patients2);

    let out1 = run_terminal(&config1).unwrap();
    let out2 = run_terminal(&config2).unwrap();

    assert_eq!(
        patient(&out1, "p1").terminal_tick,
        patient(&out2, "p1").terminal_tick
    );
    assert_eq!(
        patient(&out1, "p2").terminal_tick,
        patient(&out2, "p2").terminal_tick
    );
    assert_eq!(
        patient(&out1, "p1").completed_los_ticks,
        patient(&out2, "p1").completed_los_ticks
    );
}
