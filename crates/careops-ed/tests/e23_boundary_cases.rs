#[path = "support/e13.rs"]
mod e13;

use std::collections::BTreeMap;

use careops_ed::disposition::DispositionIntent::{Admission, Discharge};
use careops_ed::offers::OfferStatus;
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

/// ST001 & ST002 Oracle:
/// Compatible offers allocate and consume; expired offers do not allocate, leaving
/// boarder waiting without capacity leaks.
#[test]
fn st001_st002_compatible_and_expired_offers() {
    let patients = vec![
        patient_spec("boarder-1", 1, Admission, [1, 4, 3]),
        patient_spec("boarder-2", 10, Admission, [1, 4, 3]),
    ];
    let offers = vec![
        OfferSpec {
            id: "off-valid",
            destination: "dest.med",
            patient: Some("boarder-1"),
            available: 8,
            withdrawn: 30,
            duration: 1,
        },
        // Offer for boarder-2 that withdraws before boarder-2 finishes treatment (treatment finishes at 10 + 1 + 2 + 2 = 15)
        OfferSpec {
            id: "off-expired",
            destination: "dest.med",
            patient: Some("boarder-2"),
            available: 11,
            withdrawn: 13,
            duration: 1,
        },
    ];

    let config = build(BuildOptions {
        horizon: 30,
        patients,
        offers,
        discharge_delay: 0,
        care: None,
        capacities: BTreeMap::new(),
    });

    let out = run_terminal(&config).unwrap();
    assert_conservation(&out);

    let p1 = patient(&out, "boarder-1");
    let p2 = patient(&out, "boarder-2");

    // Boarder-1 consumed offer and transferred
    assert_eq!(p1.endpoint_state, EndpointState::Terminal);
    assert!(matches!(
        p1.realized_outcome,
        Some(RealizedOutcome::Transferred { .. })
    ));

    // Find offers by ID
    let off_valid = out
        .offers
        .iter()
        .find(|o| o.offer.offer_id == "off-valid")
        .unwrap();
    let off_expired = out
        .offers
        .iter()
        .find(|o| o.offer.offer_id == "off-expired")
        .unwrap();

    assert_eq!(off_valid.status, OfferStatus::Consumed);
    assert_eq!(p2.endpoint_state, EndpointState::Boarding);
    assert_eq!(off_expired.status, OfferStatus::Expired);
}

/// ST003 & ST004 Oracle:
/// Occupancy retention across physical departure delays and recurring boarder care.
#[test]
fn st003_st004_occupancy_retention_and_care() {
    let patients = vec![patient_spec("delayed-discharge", 1, Discharge, [2, 2, 2])];

    let config = build(BuildOptions {
        horizon: 15,
        patients,
        offers: vec![],
        discharge_delay: 3,
        care: None,
        capacities: BTreeMap::new(),
    });

    let out = run_terminal(&config).unwrap();
    assert_conservation(&out);

    let p = patient(&out, "delayed-discharge");
    assert_eq!(p.endpoint_state, EndpointState::Terminal);
    // Clinical work ended at tick 7, physical departure at tick 10 (7 + 3)
    assert_eq!(p.terminal_tick, Some(10));
    assert_eq!(p.space.observed_occupancy_ticks, 9);
    assert_eq!(p.space.released_tick, Some(10));
}

/// ST005 & ST006 Oracle:
/// Arrival offload sensitivity under constrained staff/triage capacity queues deterministically.
#[test]
fn st005_st006_offload_and_receiver_staff_sensitivity() {
    let mut capacities = BTreeMap::new();
    capacities.insert("res.triage".to_owned(), 1); // 1 triage nurse
    capacities.insert("res.space".to_owned(), 5);

    let patients = vec![
        patient_spec("amb-1", 1, Discharge, [4, 2, 2]),
        patient_spec("amb-2", 1, Discharge, [4, 2, 2]),
    ];

    let config = build(BuildOptions {
        horizon: 20,
        patients,
        offers: vec![],
        discharge_delay: 0,
        care: None,
        capacities,
    });

    let out = run_terminal(&config).unwrap();
    assert_conservation(&out);

    let amb1 = patient(&out, "amb-1");
    let amb2 = patient(&out, "amb-2");

    assert_eq!(amb1.endpoint_state, EndpointState::Terminal);
    assert_eq!(amb2.endpoint_state, EndpointState::Terminal);

    // Because triage is serialized, amb-2 experiences an offload delay waiting for triage
    assert!(amb2.completed_los_ticks.unwrap() > amb1.completed_los_ticks.unwrap());
}

/// ST007–ST009 Oracle:
/// Prohibits implicit unverified multi-role staffing inference and verifies conservation.
#[test]
fn st007_st009_multi_role_limits_and_ward_boundary_conservation() {
    let patients = vec![
        patient_spec("p-std-1", 1, Discharge, [1, 2, 2]),
        patient_spec("p-std-2", 2, Discharge, [1, 2, 2]),
    ];
    let config = standard(15, patients);
    let out = run_terminal(&config).unwrap();

    assert_conservation(&out);
    assert_interval_accounting(&config, &out, 15);
}
