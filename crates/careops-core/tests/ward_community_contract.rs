use careops_core::ward_community::{
    simulate, WardCommunityError, WardCommunityScenario, WardCommunityState,
};

fn scenario() -> WardCommunityScenario {
    WardCommunityScenario {
        ward_patient_ready_tick: 5,
        ed_patient_ready_tick: 7,
        community_place_available_tick: Some(10),
        transport_pickup_available_tick: Some(12),
        transport_duration_ticks: 4,
        handover_acceptance_delay_ticks: Some(2),
        horizon_tick: 20,
    }
}

#[test]
fn downstream_place_and_transport_delay_hold_the_claim_and_extend_ed_boarding() {
    let result = simulate(scenario()).unwrap();
    assert_eq!(result.state, WardCommunityState::HandoverComplete);
    assert_eq!(result.actual_ward_departure_tick, Some(12));
    assert_eq!(result.community_arrival_tick, Some(16));
    assert_eq!(result.handover_complete_tick, Some(18));
    assert_eq!(result.ed_admission_tick, Some(18));
    assert_eq!(result.ward_occupancy_after_ready_ticks, 7);
    assert_eq!(result.origin_claim_hold_after_ready_ticks, 13);
    assert_eq!(result.ed_boarding_ticks, 11);
    assert!(!result.ward_physically_occupied_at_horizon);
    assert!(!result.origin_claim_held_at_horizon);
}

#[test]
fn readiness_gates_departure_and_ed_readiness_gates_admission() {
    let result = simulate(WardCommunityScenario {
        ward_patient_ready_tick: 11,
        ed_patient_ready_tick: 12,
        community_place_available_tick: Some(4),
        transport_pickup_available_tick: Some(7),
        transport_duration_ticks: 0,
        handover_acceptance_delay_ticks: Some(0),
        horizon_tick: 20,
    })
    .unwrap();
    assert_eq!(result.actual_ward_departure_tick, Some(11));
    assert_eq!(result.community_arrival_tick, Some(11));
    assert_eq!(result.handover_complete_tick, Some(11));
    assert_eq!(result.ed_admission_tick, Some(12));
    assert_eq!(result.ward_occupancy_after_ready_ticks, 0);
    assert_eq!(result.origin_claim_hold_after_ready_ticks, 0);
    assert_eq!(result.ed_boarding_ticks, 0);
}

#[test]
fn missing_place_or_pickup_keeps_the_ward_bed_and_ed_encounter_boarded() {
    for (place, pickup, state) in [
        (None, Some(3), WardCommunityState::WaitingForCommunity),
        (Some(3), None, WardCommunityState::WaitingForTransport),
        (None, None, WardCommunityState::WaitingForBoth),
    ] {
        let result = simulate(WardCommunityScenario {
            ward_patient_ready_tick: 2,
            ed_patient_ready_tick: 4,
            community_place_available_tick: place,
            transport_pickup_available_tick: pickup,
            transport_duration_ticks: 1,
            handover_acceptance_delay_ticks: Some(0),
            horizon_tick: 10,
        })
        .unwrap();
        assert_eq!(result.state, state);
        assert_eq!(result.actual_ward_departure_tick, None);
        assert_eq!(result.ed_admission_tick, None);
        assert_eq!(result.ward_occupancy_after_ready_ticks, 8);
        assert_eq!(result.origin_claim_hold_after_ready_ticks, 8);
        assert_eq!(result.ed_boarding_ticks, 6);
        assert!(result.ward_physically_occupied_at_horizon);
        assert!(result.origin_claim_held_at_horizon);
    }
}

#[test]
fn future_place_or_pickup_is_unavailable_within_the_observed_horizon() {
    let mut case = scenario();
    case.community_place_available_tick = Some(21);
    case.transport_pickup_available_tick = Some(12);
    let result = simulate(case).unwrap();
    assert_eq!(result.state, WardCommunityState::WaitingForCommunity);
    assert_eq!(result.ed_boarding_ticks, 13);
    assert!(result.ward_physically_occupied_at_horizon);

    let mut case = scenario();
    case.community_place_available_tick = Some(10);
    case.transport_pickup_available_tick = Some(21);
    let result = simulate(case).unwrap();
    assert_eq!(result.state, WardCommunityState::WaitingForTransport);
    assert!(result.origin_claim_held_at_horizon);
}

#[test]
fn in_transit_horizon_keeps_origin_claim_until_handover() {
    let result = simulate(WardCommunityScenario {
        ward_patient_ready_tick: 2,
        ed_patient_ready_tick: 3,
        community_place_available_tick: Some(2),
        transport_pickup_available_tick: Some(2),
        transport_duration_ticks: 5,
        handover_acceptance_delay_ticks: Some(0),
        horizon_tick: 5,
    })
    .unwrap();
    assert_eq!(result.state, WardCommunityState::InTransit);
    assert_eq!(result.actual_ward_departure_tick, Some(2));
    assert_eq!(result.community_arrival_tick, None);
    assert_eq!(result.handover_complete_tick, None);
    assert_eq!(result.ed_admission_tick, None);
    assert_eq!(result.ward_occupancy_after_ready_ticks, 0);
    assert_eq!(result.origin_claim_hold_after_ready_ticks, 3);
    assert_eq!(result.ed_boarding_ticks, 2);
    assert!(!result.ward_physically_occupied_at_horizon);
    assert!(result.origin_claim_held_at_horizon);
}

#[test]
fn arrival_at_horizon_can_leave_handover_pending() {
    let mut case = WardCommunityScenario {
        ward_patient_ready_tick: 2,
        ed_patient_ready_tick: 3,
        community_place_available_tick: Some(2),
        transport_pickup_available_tick: Some(2),
        transport_duration_ticks: 3,
        handover_acceptance_delay_ticks: Some(1),
        horizon_tick: 5,
    };
    let result = simulate(case).unwrap();
    assert_eq!(result.state, WardCommunityState::HandoverPending);
    assert_eq!(result.community_arrival_tick, Some(5));
    assert_eq!(result.handover_complete_tick, None);
    assert_eq!(result.ed_admission_tick, None);
    assert!(result.origin_claim_held_at_horizon);

    case.handover_acceptance_delay_ticks = None;
    let no_ack = simulate(case).unwrap();
    assert_eq!(no_ack.state, WardCommunityState::HandoverPending);
    assert_eq!(no_ack.community_arrival_tick, Some(5));
    assert_eq!(no_ack.handover_complete_tick, None);

    case.handover_acceptance_delay_ticks = Some(0);
    let immediate_ack = simulate(case).unwrap();
    assert_eq!(immediate_ack.state, WardCommunityState::HandoverComplete);
    assert_eq!(immediate_ack.handover_complete_tick, Some(5));
    assert_eq!(immediate_ack.ed_admission_tick, Some(5));
    assert!(!immediate_ack.origin_claim_held_at_horizon);
}

#[test]
fn handover_at_inclusive_horizon_releases_claim_and_admits_ed_patient() {
    let result = simulate(WardCommunityScenario {
        ward_patient_ready_tick: 2,
        ed_patient_ready_tick: 3,
        community_place_available_tick: Some(2),
        transport_pickup_available_tick: Some(2),
        transport_duration_ticks: 2,
        handover_acceptance_delay_ticks: Some(1),
        horizon_tick: 5,
    })
    .unwrap();
    assert_eq!(result.state, WardCommunityState::HandoverComplete);
    assert_eq!(result.community_arrival_tick, Some(4));
    assert_eq!(result.handover_complete_tick, Some(5));
    assert_eq!(result.ed_admission_tick, Some(5));
    assert_eq!(result.ed_boarding_ticks, 2);
    assert!(!result.origin_claim_held_at_horizon);
}

#[test]
fn departure_at_inclusive_horizon_is_observed_and_keeps_origin_claim_held() {
    let result = simulate(WardCommunityScenario {
        ward_patient_ready_tick: 2,
        ed_patient_ready_tick: 3,
        community_place_available_tick: Some(5),
        transport_pickup_available_tick: Some(5),
        transport_duration_ticks: 0,
        handover_acceptance_delay_ticks: Some(1),
        horizon_tick: 5,
    })
    .unwrap();
    assert_eq!(result.actual_ward_departure_tick, Some(5));
    assert_eq!(result.community_arrival_tick, Some(5));
    assert_eq!(result.state, WardCommunityState::HandoverPending);
    assert_eq!(result.ward_occupancy_after_ready_ticks, 3);
    assert_eq!(result.origin_claim_hold_after_ready_ticks, 3);
    assert!(!result.ward_physically_occupied_at_horizon);
    assert!(result.origin_claim_held_at_horizon);
}

#[test]
fn readiness_after_horizon_and_tick_overflow_fail_closed() {
    let mut case = scenario();
    case.ed_patient_ready_tick = 21;
    assert_eq!(
        simulate(case),
        Err(WardCommunityError::ReadinessAfterHorizon)
    );

    let mut case = scenario();
    case.community_place_available_tick = Some(1);
    case.transport_pickup_available_tick = Some(1);
    case.transport_duration_ticks = u64::MAX;
    assert_eq!(simulate(case), Err(WardCommunityError::TimeOverflow));

    let mut case = scenario();
    case.community_place_available_tick = Some(1);
    case.transport_pickup_available_tick = Some(1);
    case.transport_duration_ticks = 0;
    case.handover_acceptance_delay_ticks = Some(u64::MAX);
    assert_eq!(simulate(case), Err(WardCommunityError::TimeOverflow));
}

#[test]
fn identical_scenario_replays_to_the_same_outcome() {
    let case = scenario();
    assert_eq!(simulate(case), simulate(case));
}
