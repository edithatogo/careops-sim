#[path = "support/e13.rs"]
mod e13;

use std::collections::BTreeMap;

use careops_ed::disposition::DispositionIntent::{Admission, Discharge};
use careops_ed::offers::OfferStatus;
use careops_ed::pathway::StageKind;
use careops_ed::terminal::{run_terminal, EndpointState, RealizedOutcome, UnfinishedReason};
use e13::{
    assert_conservation, assert_interval_accounting, build, patient, standard, BuildOptions,
    OfferSpec, PatientSpec,
};

fn person(
    id: &'static str,
    arrival: u64,
    intent: careops_ed::disposition::DispositionIntent,
) -> PatientSpec {
    PatientSpec {
        id,
        arrival,
        intent,
        stages: [2, 3, 5],
        resources: ["res.triage", "res.assessment", "res.treatment"],
    }
}

#[test]
fn immediate_delayed_and_horizon_censored_discharge_are_distinct() {
    let immediate = standard(11, vec![person("a", 1, Discharge)]);
    let out = run_terminal(&immediate).unwrap();
    let a = patient(&out, "a");
    assert_eq!(a.endpoint_state, EndpointState::Terminal);
    assert_eq!(a.terminal_tick, Some(11));
    assert_eq!(a.completed_los_ticks, Some(10));
    assert_eq!(a.space.acquired_tick, Some(1));
    assert_eq!(a.space.released_tick, Some(11));
    assert_eq!(a.space.observed_occupancy_ticks, 10);
    assert_conservation(&out);
    assert_interval_accounting(&immediate, &out, 11);

    let delayed = build(BuildOptions {
        horizon: 13,
        patients: vec![person("a", 1, Discharge)],
        offers: vec![],
        discharge_delay: 2,
        care: None,
        capacities: BTreeMap::new(),
    });
    let out = run_terminal(&delayed).unwrap();
    let a = patient(&out, "a");
    assert_eq!(a.terminal_tick, Some(13));
    assert_eq!(a.completed_los_ticks, Some(12));
    assert_eq!(a.space.observed_occupancy_ticks, 12);
    assert!(a.transfer.is_none() && a.care.is_empty());
    assert_conservation(&out);
    assert_interval_accounting(&delayed, &out, 13);

    let pending = build(BuildOptions {
        horizon: 12,
        patients: vec![person("a", 1, Discharge)],
        offers: vec![],
        discharge_delay: 2,
        care: None,
        capacities: BTreeMap::new(),
    });
    let out = run_terminal(&pending).unwrap();
    let a = patient(&out, "a");
    assert_eq!(a.endpoint_state, EndpointState::DischargePending);
    assert_eq!(
        a.unfinished_reason,
        Some(UnfinishedReason::DischargePending)
    );
    assert_eq!(a.terminal_tick, None);
    assert_eq!(a.realized_outcome, None);
    assert_eq!(a.completed_los_ticks, None);
    assert_eq!(a.observed_elapsed_ticks, 11);
    assert_eq!(a.space.released_tick, None);
    assert_eq!(a.space.observed_occupancy_ticks, 11);
    assert_conservation(&out);
    assert_interval_accounting(&pending, &out, 12);
}

#[test]
fn disabled_abandonment_leaves_boarder_open_and_wrong_or_expired_offers_unconsumed() {
    let no_offer = build(BuildOptions {
        horizon: 30,
        patients: vec![person("a", 1, Admission)],
        offers: vec![],
        discharge_delay: 0,
        care: None,
        capacities: BTreeMap::new(),
    });
    let out = run_terminal(&no_offer).unwrap();
    let a = patient(&out, "a");
    assert_eq!(a.endpoint_state, EndpointState::Boarding);
    assert_eq!(
        a.unfinished_reason,
        Some(UnfinishedReason::AwaitingOfferAllocation)
    );
    assert_eq!(a.decision.as_ref().unwrap().observed_tick, 11);
    assert_eq!(a.boarding.as_ref().unwrap().start_tick, 11);
    assert_eq!(a.boarding.as_ref().unwrap().observed_ticks, 19);
    assert_eq!(a.observed_boarding_ticks, Some(19));
    assert_eq!(a.space.observed_occupancy_ticks, 29);
    assert_eq!(a.realized_outcome, None);
    assert_eq!(a.terminal_tick, None);
    assert_eq!(a.completed_los_ticks, None);
    assert_conservation(&out);
    assert_interval_accounting(&no_offer, &out, 30);

    let incompatible = build(BuildOptions {
        horizon: 16,
        patients: vec![person("a", 1, Admission)],
        offers: vec![OfferSpec {
            id: "wrong",
            destination: "dest.other",
            patient: None,
            available: 11,
            withdrawn: 15,
            duration: 2,
        }],
        discharge_delay: 0,
        care: None,
        capacities: BTreeMap::new(),
    });
    let out = run_terminal(&incompatible).unwrap();
    assert_eq!(patient(&out, "a").endpoint_state, EndpointState::Boarding);
    assert_eq!(out.offers[0].status, OfferStatus::Expired);
    assert!(out.offers[0].allocation.is_none());
    assert_conservation(&out);
}

#[test]
fn transfer_at_horizon_is_censored_then_terminal_at_exact_completion() {
    let config = |horizon| {
        build(BuildOptions {
            horizon,
            patients: vec![person("a", 1, Admission)],
            offers: vec![OfferSpec {
                id: "med-1",
                destination: "dest.med",
                patient: None,
                available: 13,
                withdrawn: 14,
                duration: 2,
            }],
            discharge_delay: 0,
            care: None,
            capacities: BTreeMap::new(),
        })
    };
    let at14 = config(14);
    let out = run_terminal(&at14).unwrap();
    let a = patient(&out, "a");
    assert_eq!(a.endpoint_state, EndpointState::InTransfer);
    assert_eq!(a.unfinished_reason, Some(UnfinishedReason::TransferActive));
    assert_eq!(a.transfer.as_ref().unwrap().start_tick, Some(13));
    assert_eq!(a.transfer.as_ref().unwrap().completion_tick, None);
    assert_eq!(a.transfer.as_ref().unwrap().observed_work_ticks, 1);
    assert_eq!(a.boarding_time_ticks, None);
    assert_eq!(a.observed_boarding_ticks, Some(3));
    assert_eq!(a.space.observed_occupancy_ticks, 13);
    assert_conservation(&out);
    assert_interval_accounting(&at14, &out, 14);

    let at15 = config(15);
    let out = run_terminal(&at15).unwrap();
    let a = patient(&out, "a");
    assert_eq!(a.endpoint_state, EndpointState::Terminal);
    assert_eq!(a.terminal_tick, Some(15));
    assert_eq!(a.completed_los_ticks, Some(14));
    assert_eq!(a.boarding_time_ticks, Some(4));
    assert_eq!(a.observed_boarding_ticks, Some(4));
    assert_eq!(a.space.observed_occupancy_ticks, 14);
    assert_eq!(
        a.realized_outcome,
        Some(RealizedOutcome::Transferred {
            destination: "dest.med".into(),
            offer_id: "med-1".into()
        })
    );
    assert_eq!(out.offers[0].status, OfferStatus::Consumed);
    assert_conservation(&out);
    assert_interval_accounting(&at15, &out, 15);
}

#[test]
fn care_due_tie_cancels_without_work_and_active_care_blocks_offer_until_completion() {
    let canceled = build(BuildOptions {
        horizon: 15,
        patients: vec![person("a", 1, Admission)],
        offers: vec![OfferSpec {
            id: "med",
            destination: "dest.med",
            patient: None,
            available: 13,
            withdrawn: 20,
            duration: 2,
        }],
        discharge_delay: 0,
        care: Some(("res.care".into(), "task.care".into(), 2, 3)),
        capacities: BTreeMap::new(),
    });
    let out = run_terminal(&canceled).unwrap();
    let a = patient(&out, "a");
    assert_eq!(a.care.len(), 1);
    assert_eq!(a.care[0].queue_entry_tick, 13);
    assert_eq!(a.care[0].start_tick, None);
    assert_eq!(a.care[0].cancelled_tick, Some(13));
    assert_eq!(a.care[0].observed_work_ticks, 0);
    assert_eq!(
        out.resources
            .iter()
            .find(|r| r.resource_id == "res.care")
            .unwrap()
            .queued_at_horizon,
        0
    );
    assert_eq!(a.endpoint_state, EndpointState::Terminal);
    assert_eq!(a.terminal_tick, Some(15));
    assert_conservation(&out);
    assert_interval_accounting(&canceled, &out, 15);

    let expires_during_care = build(BuildOptions {
        horizon: 18,
        patients: vec![person("a", 1, Admission)],
        offers: vec![OfferSpec {
            id: "short",
            destination: "dest.med",
            patient: None,
            available: 14,
            withdrawn: 15,
            duration: 2,
        }],
        discharge_delay: 0,
        care: Some(("res.care".into(), "task.care".into(), 2, 3)),
        capacities: BTreeMap::new(),
    });
    let out = run_terminal(&expires_during_care).unwrap();
    let a = patient(&out, "a");
    assert_eq!(
        (a.care[0].start_tick, a.care[0].completion_tick),
        (Some(13), Some(16))
    );
    assert_eq!(a.endpoint_state, EndpointState::Boarding);
    assert_eq!(out.offers[0].status, OfferStatus::Expired);
    assert!(out.offers[0].allocation.is_none());
    assert_eq!(
        a.care.len(),
        2,
        "care is scheduled after completion plus interval"
    );
    assert_eq!(a.care[1].queue_entry_tick, 18);
    assert_eq!(a.care[1].start_tick, Some(18));
    assert_eq!(a.care[1].observed_work_ticks, 0);
    assert_eq!(a.care[1].completion_tick, None);
    assert_conservation(&out);
    assert_interval_accounting(&expires_during_care, &out, 18);
}

#[test]
fn space_is_retained_until_departure_and_outside_patient_starts_at_release() {
    let config = build(BuildOptions {
        horizon: 21,
        patients: vec![person("a", 1, Discharge), person("b", 1, Discharge)],
        offers: vec![],
        discharge_delay: 0,
        care: None,
        capacities: BTreeMap::from([
            ("res.space".into(), 1),
            ("res.triage".into(), 1),
            ("res.assessment".into(), 1),
            ("res.treatment".into(), 1),
        ]),
    });
    let out = run_terminal(&config).unwrap();
    let a = patient(&out, "a");
    let b = patient(&out, "b");
    assert_eq!(a.pathway.episodes[0].start_tick, Some(1));
    assert_eq!(a.terminal_tick, Some(11));
    assert_eq!(b.space.queue_entry_tick, 1);
    assert_eq!(b.space.acquired_tick, Some(11));
    assert_eq!(b.pathway.episodes[0].start_tick, Some(11));
    assert_eq!(b.pathway.episodes[1].start_tick, Some(13));
    assert_eq!(b.pathway.episodes[2].start_tick, Some(16));
    assert_eq!(b.terminal_tick, Some(21));
    assert_conservation(&out);
    assert_interval_accounting(&config, &out, 21);
}

#[test]
fn zero_delay_departure_releases_space_before_same_tick_grants() {
    let config = build(BuildOptions {
        horizon: 11,
        patients: vec![person("a", 1, Discharge), person("b", 2, Discharge)],
        offers: vec![OfferSpec {
            id: "not-for-discharge",
            destination: "dest.med",
            patient: None,
            available: 11,
            withdrawn: 12,
            duration: 1,
        }],
        discharge_delay: 0,
        care: None,
        capacities: BTreeMap::from([
            ("res.space".into(), 1),
            ("res.triage".into(), 1),
            ("res.assessment".into(), 1),
            ("res.treatment".into(), 1),
        ]),
    });
    let out = run_terminal(&config).unwrap();
    assert_eq!(patient(&out, "a").terminal_tick, Some(11));
    let b = patient(&out, "b");
    assert_eq!(b.space.acquired_tick, Some(11));
    assert_eq!(b.pathway.episodes[0].start_tick, Some(11));
    assert_eq!(b.pathway.episodes[0].observed_work_ticks, 0);
    assert_eq!(b.endpoint_state, EndpointState::Clinical);
    assert_eq!(
        b.unfinished_reason,
        Some(UnfinishedReason::ClinicalActive {
            stage: StageKind::Triage
        })
    );
    let triage = out
        .resources
        .iter()
        .find(|r| r.resource_id == "res.triage")
        .unwrap();
    assert_eq!(triage.busy_at_horizon, 1);
    assert_eq!(triage.peak_busy, 1);
    assert_eq!(triage.queued_at_horizon, 0);
    assert_conservation(&out);
    assert_interval_accounting(&config, &out, 11);
}

#[test]
fn transfer_capacity_delays_physical_departure_and_conserves_space() {
    let config = build(BuildOptions {
        horizon: 15,
        patients: vec![person("a", 1, Admission), person("b", 1, Admission)],
        offers: vec![
            OfferSpec {
                id: "offer-a",
                destination: "dest.med",
                patient: Some("a"),
                available: 11,
                withdrawn: 20,
                duration: 2,
            },
            OfferSpec {
                id: "offer-b",
                destination: "dest.med",
                patient: Some("b"),
                available: 11,
                withdrawn: 20,
                duration: 2,
            },
        ],
        discharge_delay: 0,
        care: None,
        capacities: BTreeMap::from([
            ("res.space".into(), 2),
            ("res.triage".into(), 2),
            ("res.assessment".into(), 2),
            ("res.treatment".into(), 2),
            ("res.transfer".into(), 1),
        ]),
    });
    let out = run_terminal(&config).unwrap();
    let a = patient(&out, "a");
    let b = patient(&out, "b");
    assert_eq!(a.transfer.as_ref().unwrap().start_tick, Some(11));
    assert_eq!(a.terminal_tick, Some(13));
    assert_eq!(b.transfer.as_ref().unwrap().queue_entry_tick, 11);
    assert_eq!(b.transfer.as_ref().unwrap().start_tick, Some(13));
    assert_eq!(b.transfer.as_ref().unwrap().completion_tick, Some(15));
    assert_eq!(b.terminal_tick, Some(15));
    assert_eq!(b.space.released_tick, Some(15));
    assert_eq!(b.completed_los_ticks, Some(14));
    assert_conservation(&out);
    assert_interval_accounting(&config, &out, 15);

    let horizon12 = build(BuildOptions {
        horizon: 12,
        patients: vec![person("a", 1, Admission), person("b", 1, Admission)],
        offers: vec![
            OfferSpec {
                id: "offer-a",
                destination: "dest.med",
                patient: Some("a"),
                available: 11,
                withdrawn: 20,
                duration: 2,
            },
            OfferSpec {
                id: "offer-b",
                destination: "dest.med",
                patient: Some("b"),
                available: 11,
                withdrawn: 20,
                duration: 2,
            },
        ],
        discharge_delay: 0,
        care: None,
        capacities: BTreeMap::from([
            ("res.space".into(), 2),
            ("res.triage".into(), 2),
            ("res.assessment".into(), 2),
            ("res.treatment".into(), 2),
        ]),
    });
    let out = run_terminal(&horizon12).unwrap();
    assert_eq!(
        patient(&out, "b").transfer.as_ref().unwrap().start_tick,
        None
    );
    let transfer = out
        .resources
        .iter()
        .find(|r| r.resource_id == "res.transfer")
        .unwrap();
    assert_eq!(transfer.busy_at_horizon, 1);
    assert_eq!(transfer.queued_at_horizon, 1);
    assert_interval_accounting(&horizon12, &out, 12);
}

#[test]
fn care_and_clinical_work_share_one_fcfs_pool() {
    let mut clinical = person("a-clinical", 11, Discharge);
    clinical.stages = [2, 3, 5];
    let mut boarder = person("z-boarder", 1, Admission);
    boarder.resources = ["res.triage", "res.assessment", "res.treatment"];
    let config = build(BuildOptions {
        horizon: 21,
        patients: vec![boarder, clinical],
        offers: vec![],
        discharge_delay: 0,
        care: Some(("res.assessment".into(), "task.care".into(), 2, 3)),
        capacities: BTreeMap::from([
            ("res.space".into(), 2),
            ("res.triage".into(), 1),
            ("res.assessment".into(), 1),
            ("res.treatment".into(), 1),
        ]),
    });
    let out = run_terminal(&config).unwrap();
    let boarder = patient(&out, "z-boarder");
    let clinical = patient(&out, "a-clinical");
    assert_eq!(boarder.care[0].queue_entry_tick, 13);
    assert_eq!(clinical.pathway.episodes[1].queue_entry_tick, 13);
    assert_eq!(clinical.pathway.episodes[1].start_tick, Some(13));
    assert_eq!(clinical.pathway.episodes[1].completion_tick, Some(16));
    assert_eq!(boarder.care[0].start_tick, Some(16));
    assert_eq!(boarder.care[0].completion_tick, Some(19));
    assert_conservation(&out);
    assert_interval_accounting(&config, &out, 21);
}

#[test]
fn reversing_configuration_vectors_preserves_canonical_result() {
    let base = build(BuildOptions {
        horizon: 17,
        patients: vec![person("b", 1, Admission), person("a", 1, Admission)],
        offers: vec![
            OfferSpec {
                id: "offer-b",
                destination: "dest.med",
                patient: Some("b"),
                available: 11,
                withdrawn: 20,
                duration: 2,
            },
            OfferSpec {
                id: "offer-a",
                destination: "dest.med",
                patient: Some("a"),
                available: 11,
                withdrawn: 20,
                duration: 2,
            },
        ],
        discharge_delay: 0,
        care: None,
        capacities: BTreeMap::from([
            ("res.space".into(), 2),
            ("res.triage".into(), 2),
            ("res.assessment".into(), 2),
            ("res.treatment".into(), 2),
        ]),
    });
    let expected = run_terminal(&base).unwrap();
    let mut permuted = base;
    permuted.pathway.patients.reverse();
    permuted.services.reverse();
    permuted.decisions.reverse();
    permuted.admissions.reverse();
    permuted.offers.reverse();
    permuted
        .pathway
        .capacity_location
        .resource_buckets
        .reverse();
    permuted.pathway.capacity_location.task_classes.reverse();
    permuted.pathway.capacity_location.eligibility.reverse();
    let actual = run_terminal(&permuted).unwrap();
    assert_eq!(actual, expected);
    assert_conservation(&actual);
    assert_interval_accounting(&permuted, &actual, 17);
}

#[test]
fn wide_completion_arithmetic_censors_without_wrapping() {
    let config = build(BuildOptions {
        horizon: u64::MAX,
        patients: vec![PatientSpec {
            id: "a",
            arrival: u64::MAX - 20,
            intent: Discharge,
            stages: [5, 5, 5],
            resources: ["res.triage", "res.assessment", "res.treatment"],
        }],
        offers: vec![],
        discharge_delay: 10,
        care: None,
        capacities: BTreeMap::new(),
    });
    let out = run_terminal(&config).unwrap();
    let a = patient(&out, "a");
    assert_eq!(a.endpoint_state, EndpointState::DischargePending);
    assert_eq!(a.terminal_tick, None);
    assert_eq!(a.completed_los_ticks, None);
    assert_eq!(a.pathway.episodes[2].completion_tick, Some(u64::MAX - 5));
    assert_eq!(a.observed_elapsed_ticks, 20);
    assert_conservation(&out);
    assert_interval_accounting(&config, &out, u64::MAX);
}

#[test]
fn malformed_terminal_policy_rejects_before_any_lifecycle_result() {
    let mut config = build(BuildOptions {
        horizon: 20,
        patients: vec![person("a", 1, Admission)],
        offers: vec![],
        discharge_delay: 0,
        care: Some(("res.care".into(), "task.care".into(), 2, 3)),
        capacities: BTreeMap::new(),
    });
    config.policy.boarder_care.as_mut().unwrap().interval_ticks = 0;
    let error = run_terminal(&config).unwrap_err();
    assert!(error.to_string().contains("positive interval/duration"));
}
