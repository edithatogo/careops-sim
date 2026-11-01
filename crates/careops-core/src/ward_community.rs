//! Minimal deterministic ED-to-ward-to-community boarding boundary.
//!
//! This is a single-bed scenario oracle, not a capacity allocator, clinical
//! policy, transport fleet, or event scheduler. It keeps physical vacancy at
//! departure separate from origin-claim release at explicit handover.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WardCommunityState {
    WaitingForCommunity,
    WaitingForTransport,
    WaitingForBoth,
    InTransit,
    HandoverPending,
    HandoverComplete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WardCommunityError {
    ReadinessAfterHorizon,
    TimeOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WardCommunityScenario {
    pub ward_patient_ready_tick: u64,
    pub ed_patient_ready_tick: u64,
    pub community_place_available_tick: Option<u64>,
    pub transport_pickup_available_tick: Option<u64>,
    pub transport_duration_ticks: u64,
    /// `None` means no handover is observed within this modeled case.
    pub handover_acceptance_delay_ticks: Option<u64>,
    pub horizon_tick: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WardCommunityOutcome {
    pub state: WardCommunityState,
    pub actual_ward_departure_tick: Option<u64>,
    pub community_arrival_tick: Option<u64>,
    pub handover_complete_tick: Option<u64>,
    pub ed_admission_tick: Option<u64>,
    /// Physical-occupancy interval from ward clinical readiness to departure
    /// or horizon; total pre-readiness occupancy is outside this fixture.
    pub ward_occupancy_after_ready_ticks: u64,
    /// Origin-claim hold interval after readiness until handover or horizon.
    pub origin_claim_hold_after_ready_ticks: u64,
    pub ed_boarding_ticks: u64,
    pub ward_physically_occupied_at_horizon: bool,
    pub origin_claim_held_at_horizon: bool,
}

/// Evaluate one ward occupant and one ED patient against exogenous boundary
/// observations. All events at the inclusive horizon are observed.
pub fn simulate(
    scenario: WardCommunityScenario,
) -> Result<WardCommunityOutcome, WardCommunityError> {
    if scenario.ward_patient_ready_tick > scenario.horizon_tick
        || scenario.ed_patient_ready_tick > scenario.horizon_tick
    {
        return Err(WardCommunityError::ReadinessAfterHorizon);
    }

    let place_tick = scenario
        .community_place_available_tick
        .filter(|tick| *tick <= scenario.horizon_tick);
    let pickup_tick = scenario
        .transport_pickup_available_tick
        .filter(|tick| *tick <= scenario.horizon_tick);

    let departure_tick = match (place_tick, pickup_tick) {
        (Some(place), Some(pickup)) => {
            Some(scenario.ward_patient_ready_tick.max(place).max(pickup))
        }
        _ => None,
    }
    .filter(|tick| *tick <= scenario.horizon_tick);

    let Some(departure_tick) = departure_tick else {
        let state = match (place_tick.is_some(), pickup_tick.is_some()) {
            (false, false) => WardCommunityState::WaitingForBoth,
            (false, true) => WardCommunityState::WaitingForCommunity,
            (true, false) => WardCommunityState::WaitingForTransport,
            (true, true) => unreachable!("available prerequisites imply departure by horizon"),
        };
        return Ok(WardCommunityOutcome {
            state,
            actual_ward_departure_tick: None,
            community_arrival_tick: None,
            handover_complete_tick: None,
            ed_admission_tick: None,
            ward_occupancy_after_ready_ticks: scenario
                .horizon_tick
                .checked_sub(scenario.ward_patient_ready_tick)
                .expect("readiness was validated against the horizon"),
            origin_claim_hold_after_ready_ticks: scenario
                .horizon_tick
                .checked_sub(scenario.ward_patient_ready_tick)
                .expect("readiness was validated against the horizon"),
            ed_boarding_ticks: scenario
                .horizon_tick
                .checked_sub(scenario.ed_patient_ready_tick)
                .expect("readiness was validated against the horizon"),
            ward_physically_occupied_at_horizon: true,
            origin_claim_held_at_horizon: true,
        });
    };

    let arrival_tick = departure_tick
        .checked_add(scenario.transport_duration_ticks)
        .ok_or(WardCommunityError::TimeOverflow)?;
    let observed_arrival = (arrival_tick <= scenario.horizon_tick).then_some(arrival_tick);
    let handover_tick = match scenario.handover_acceptance_delay_ticks {
        Some(delay) => Some(
            arrival_tick
                .checked_add(delay)
                .ok_or(WardCommunityError::TimeOverflow)?,
        ),
        None => None,
    };
    let observed_handover = handover_tick.filter(|tick| *tick <= scenario.horizon_tick);

    let state = match (observed_arrival, observed_handover) {
        (None, _) => WardCommunityState::InTransit,
        (Some(_), None) => WardCommunityState::HandoverPending,
        (Some(_), Some(_)) => WardCommunityState::HandoverComplete,
    };
    let ed_admission_tick = observed_handover.map(|tick| tick.max(scenario.ed_patient_ready_tick));
    let ed_boarding_end = ed_admission_tick.unwrap_or(scenario.horizon_tick);

    Ok(WardCommunityOutcome {
        state,
        actual_ward_departure_tick: Some(departure_tick),
        community_arrival_tick: observed_arrival,
        handover_complete_tick: observed_handover,
        ed_admission_tick,
        ward_occupancy_after_ready_ticks: departure_tick
            .checked_sub(scenario.ward_patient_ready_tick)
            .expect("departure cannot precede clinical readiness"),
        origin_claim_hold_after_ready_ticks: observed_handover
            .unwrap_or(scenario.horizon_tick)
            .checked_sub(scenario.ward_patient_ready_tick)
            .expect("handover or horizon cannot precede clinical readiness"),
        ed_boarding_ticks: ed_boarding_end
            .checked_sub(scenario.ed_patient_ready_tick)
            .expect("admission or horizon cannot precede ED readiness"),
        ward_physically_occupied_at_horizon: false,
        origin_claim_held_at_horizon: observed_handover.is_none(),
    })
}
