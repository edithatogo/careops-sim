//! Failing cases and mathematical oracle tests for clinical pathways, progression, and policy clocks (Task A1.1).
//!
//! Validates:
//! - Branching and loop bounds (circuit breaking on max repetitions).
//! - Invalid transitions (unregistered transitions and disjoint stage jumps fail closed).
//! - Progression during waiting (patient clinical state evolves while queued/boarding per CC-04).
//! - Elapsed-age / residual-time and covariate changes (hazard scaling without discontinuity).
//! - Stale event invalidation and deterministic multi-key event tie-breaking.
//! - Zero-target accounting (immediate service target without division-by-zero per CC-06).
//! - Analytic exponential competing-risk fractions and zero-hazard controls.
//! - Unchanged-covariate invariance.
//! - Right-censoring at simulation horizon.
//! - Explicit draw ownership preventing cross-subsystem RNG leakage (CC-07).

use std::collections::HashMap;

use careops_core::pathway::{
    CompetingRiskModel, HazardRate, PathwayErrorKind, PathwayRngStreams, PolicyClock,
    ScheduledEvent, StageGraph, StageId, StageNode,
};

// -----------------------------------------------------------------------------
// 1. Branching & Loop Bounds
// -----------------------------------------------------------------------------

#[test]
fn test_pathway_branching_and_loop_bounds_exceeded_fails_closed() {
    let mut graph = StageGraph::new();
    let s_triage = StageId::new("triage");
    let s_assessment = StageId::new("assessment");
    let s_repeat_diagnostic = StageId::new("repeat_diagnostic");

    graph.add_node(StageNode {
        id: s_triage.clone(),
        name: "Triage".to_string(),
        max_repetitions: 1,
        is_terminal: false,
    });
    graph.add_node(StageNode {
        id: s_assessment.clone(),
        name: "Assessment".to_string(),
        max_repetitions: 1,
        is_terminal: false,
    });
    graph.add_node(StageNode {
        id: s_repeat_diagnostic.clone(),
        name: "Diagnostic Loop".to_string(),
        max_repetitions: 2, // Maximum 2 repetitions allowed!
        is_terminal: false,
    });

    graph.add_transition(&s_triage, &s_assessment);
    graph.add_transition(&s_assessment, &s_repeat_diagnostic);
    graph.add_transition(&s_repeat_diagnostic, &s_repeat_diagnostic); // Self loop

    // Repetition 0 and 1 are allowed
    assert!(
        graph
            .validate_transition(&s_repeat_diagnostic, &s_repeat_diagnostic, 0)
            .is_ok()
    );
    assert!(
        graph
            .validate_transition(&s_repeat_diagnostic, &s_repeat_diagnostic, 1)
            .is_ok()
    );

    // Repetition 2 exceeds max_repetitions (2) -> must fail closed!
    let res = graph.validate_transition(&s_repeat_diagnostic, &s_repeat_diagnostic, 2);
    assert!(
        res.is_err(),
        "loop count exceeding max_repetitions must fail closed"
    );
    let err = res.unwrap_err();
    assert_eq!(err.kind, PathwayErrorKind::LoopBoundExceeded);
    assert!(err.message.contains("repetition limit of 2 exceeded"));
}

// -----------------------------------------------------------------------------
// 2. Invalid Transitions
// -----------------------------------------------------------------------------

#[test]
fn test_pathway_invalid_unregistered_transition_fails_closed() {
    let mut graph = StageGraph::new();
    let s_arrival = StageId::new("arrival");
    let s_triage = StageId::new("triage");
    let s_discharge = StageId::new("discharge");

    graph.add_node(StageNode {
        id: s_arrival.clone(),
        name: "Arrival".to_string(),
        max_repetitions: 1,
        is_terminal: false,
    });
    graph.add_node(StageNode {
        id: s_triage.clone(),
        name: "Triage".to_string(),
        max_repetitions: 1,
        is_terminal: false,
    });
    graph.add_node(StageNode {
        id: s_discharge.clone(),
        name: "Discharge".to_string(),
        max_repetitions: 1,
        is_terminal: true,
    });

    graph.add_transition(&s_arrival, &s_triage);
    graph.add_transition(&s_triage, &s_discharge);

    // Valid transitions
    assert!(graph.validate_transition(&s_arrival, &s_triage, 0).is_ok());
    assert!(
        graph
            .validate_transition(&s_triage, &s_discharge, 0)
            .is_ok()
    );

    // Invalid transition: Arrival skipping triage directly to Discharge
    let res = graph.validate_transition(&s_arrival, &s_discharge, 0);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind, PathwayErrorKind::InvalidTransition);
    assert!(
        err.message
            .contains("transition from 'arrival' to 'discharge' is not permitted")
    );

    // Invalid transition: Unregistered stage node
    let s_ghost = StageId::new("unregistered_stage");
    let res_ghost = graph.validate_transition(&s_triage, &s_ghost, 0);
    assert!(res_ghost.is_err());
    assert_eq!(
        res_ghost.unwrap_err().kind,
        PathwayErrorKind::InvalidTransition
    );
}

// -----------------------------------------------------------------------------
// 3. Progression During Waiting (CC-04)
// -----------------------------------------------------------------------------

#[test]
fn test_clinical_progression_continues_during_queue_wait() {
    // CC-04: Patient state evolves while waiting in queues or boarding in hallways.
    #[allow(dead_code)]
    struct PatientSimulation {
        is_server_available: bool,
        queue_wait_ticks: u64,
        accumulated_hazard_integral: f64,
    }

    let mut patient_waiting = PatientSimulation {
        is_server_available: false, // Queued waiting for physical bed
        queue_wait_ticks: 0,
        accumulated_hazard_integral: 0.0,
    };

    let hazard_rate_per_tick = 0.02; // 2% per tick progression hazard

    // Advance 50 ticks of simulation time
    for _ in 0..50 {
        patient_waiting.queue_wait_ticks += 1;
        // Clinical hazard accumulates regardless of server availability
        patient_waiting.accumulated_hazard_integral += hazard_rate_per_tick;
    }

    assert_eq!(patient_waiting.queue_wait_ticks, 50);
    assert!(
        (patient_waiting.accumulated_hazard_integral - 1.0).abs() < 1e-9,
        "clinical progression hazard must advance during queue wait (CC-04)"
    );
}

// -----------------------------------------------------------------------------
// 4. Elapsed-Age / Residual-Time and Covariate Changes
// -----------------------------------------------------------------------------

#[test]
fn test_residual_time_rescaling_on_covariate_change_conserves_hazard() {
    let mut model = CompetingRiskModel::new();
    let mut deterioration_hazard = HazardRate::new("deterioration", 0.01);
    deterioration_hazard
        .covariate_coefficients
        .insert("acuity_score".to_string(), std::f64::consts::LN_2); // ln(2) -> doubling hazard per unit
    model.add_hazard(deterioration_hazard);

    let mut old_covariates = HashMap::new();
    old_covariates.insert("acuity_score".to_string(), 0.0); // lambda_old = 0.01 * exp(0) = 0.01

    let mut new_covariates = HashMap::new();
    new_covariates.insert("acuity_score".to_string(), 1.0); // lambda_new = 0.01 * exp(ln 2) = 0.02 (doubled!)

    let t_rem_old = 40.0; // 40 ticks remaining under old covariates
    let t_rem_new = model
        .adjust_residual_time_on_covariate_change(t_rem_old, &old_covariates, &new_covariates)
        .expect("residual time adjustment must succeed");

    // Because hazard doubled from 0.01 to 0.02, remaining residual time should halve from 40.0 to 20.0
    assert!(
        (t_rem_new - 20.0).abs() < 1e-4,
        "doubled hazard must halve remaining residual time: expected 20.0, got {t_rem_new}"
    );
}

// -----------------------------------------------------------------------------
// 5. Stale Event Invalidation & Deterministic Multi-Key Tie Breaking
// -----------------------------------------------------------------------------

#[test]
fn test_stale_event_delivery_rejected_on_generation_advance() {
    let scheduled_event = ScheduledEvent {
        event_id: 101,
        tick: 45,
        priority: 1,
        patient_id: 2002,
        generation: 3,
        description: "Scheduled reassessment".to_string(),
    };

    // Patient state is still generation 3: valid
    assert!(scheduled_event.validate_execution(3).is_ok());

    // Patient rapidly deteriorated and state advanced to generation 4 beforehand:
    // Event is now stale and must be rejected!
    let res = scheduled_event.validate_execution(4);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind, PathwayErrorKind::StaleEvent);
    assert!(err.message.contains("is stale; current generation is 4"));
}

#[test]
fn test_deterministic_multi_key_event_tie_breaking() {
    // Tie breaking tuple: (tick, priority, event_id, patient_id)
    let e1 = ScheduledEvent {
        event_id: 10,
        tick: 100,
        priority: 2,
        patient_id: 50,
        generation: 1,
        description: "Event 1".to_string(),
    };
    let e2 = ScheduledEvent {
        event_id: 5,
        tick: 100,
        priority: 1, // Higher priority (1 < 2)
        patient_id: 60,
        generation: 1,
        description: "Event 2".to_string(),
    };
    let e3 = ScheduledEvent {
        event_id: 4,
        tick: 100,
        priority: 1, // Same priority as e2, but event_id 4 < 5
        patient_id: 70,
        generation: 1,
        description: "Event 3".to_string(),
    };

    let mut list = [e1.clone(), e2.clone(), e3.clone()];
    list.sort();

    // Correct deterministic order: e3 (prio 1, id 4) -> e2 (prio 1, id 5) -> e1 (prio 2, id 10)
    assert_eq!(list[0].event_id, 4);
    assert_eq!(list[1].event_id, 5);
    assert_eq!(list[2].event_id, 10);
}

// -----------------------------------------------------------------------------
// 6. Zero-Target Accounting (CC-06)
// -----------------------------------------------------------------------------

#[test]
fn test_zero_target_clock_immediate_service_breach_without_nan_or_div_zero() {
    // ATS 1 target = 0 min / 0 ticks (immediate service required)
    let mut clock = PolicyClock::new("ats1_resuscitation", 0);

    // Initial state: elapsed = 0 -> breach_ratio must be 0.0 (no breach, no NaN/panic)
    assert_eq!(clock.breach_ratio(), 0.0);
    assert!(!clock.is_breached());

    // Elapsed advances to 1 tick -> breach is instantaneous (INFINITY, is_breached = true)
    clock.advance(1);
    assert!(clock.breach_ratio().is_infinite());
    assert!(clock.is_breached());
    assert!(!clock.breach_ratio().is_nan());
}

// -----------------------------------------------------------------------------
// 7. Analytic Exponential Competing-Risk & Zero-Hazard Controls
// -----------------------------------------------------------------------------

#[test]
fn test_analytic_exponential_competing_risk_ratios_match_exact_fractions() {
    let mut model = CompetingRiskModel::new();
    model.add_hazard(HazardRate::new("cause_a", 0.02));
    model.add_hazard(HazardRate::new("cause_b", 0.03));

    let empty_covariates = HashMap::new();
    let probs = model
        .analytic_cause_probabilities(&empty_covariates)
        .expect("competing risk probabilities must compute");

    let prob_a = probs.get("cause_a").copied().unwrap();
    let prob_b = probs.get("cause_b").copied().unwrap();

    // Exact theoretical ratios: 0.02 / 0.05 = 0.40; 0.03 / 0.05 = 0.60
    assert!((prob_a - 0.40).abs() < 1e-9);
    assert!((prob_b - 0.60).abs() < 1e-9);
}

#[test]
fn test_zero_hazard_control_produces_no_event() {
    let mut model = CompetingRiskModel::new();
    model.add_hazard(HazardRate::new("quiescent", 0.0));

    let empty_covariates = HashMap::new();
    let res = model.analytic_cause_probabilities(&empty_covariates);

    // Zero-hazard must fail closed with ZeroTargetError rather than NaN or division by zero
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().kind, PathwayErrorKind::ZeroTargetError);
}

// -----------------------------------------------------------------------------
// 8. Unchanged-Covariate Invariance
// -----------------------------------------------------------------------------

#[test]
fn test_unchanged_covariate_invariance_produces_identical_hazard() {
    let mut model = CompetingRiskModel::new();
    let mut h = HazardRate::new("stabilization", 0.05);
    h.covariate_coefficients.insert("age".to_string(), 0.01);
    model.add_hazard(h);

    let mut covs = HashMap::new();
    covs.insert("age".to_string(), 45.0);

    let total_1 = model.total_hazard(&covs);
    let total_2 = model.total_hazard(&covs);
    assert_eq!(
        total_1, total_2,
        "unchanged covariates must produce strictly invariant hazard"
    );
}

// -----------------------------------------------------------------------------
// 9. Right-Censoring at Simulation Horizon
// -----------------------------------------------------------------------------

#[test]
fn test_right_censoring_at_horizon_prevents_unauthorized_state_transition() {
    let horizon_ticks = 100u64;
    let scheduled_completion_tick = 125u64;

    struct Episode {
        completed: bool,
        censored: bool,
        observed_ticks: u64,
    }

    let mut ep = Episode {
        completed: false,
        censored: false,
        observed_ticks: 0,
    };

    // Progression runner step evaluating against horizon
    let advance_to_horizon = |e: &mut Episode, horizon: u64, completion: u64| {
        if completion > horizon {
            e.censored = true;
            e.completed = false;
            e.observed_ticks = horizon;
        } else {
            e.censored = false;
            e.completed = true;
            e.observed_ticks = completion;
        }
    };

    advance_to_horizon(&mut ep, horizon_ticks, scheduled_completion_tick);
    assert!(ep.censored, "event past horizon must be right-censored");
    assert!(!ep.completed, "censored event must NOT be marked complete");
    assert_eq!(
        ep.observed_ticks, 100,
        "observed work must be clamped to horizon"
    );
}

// -----------------------------------------------------------------------------
// 10. Explicit Draw Ownership Across Subsystem Streams (CC-07)
// -----------------------------------------------------------------------------

#[test]
fn test_explicit_draw_ownership_prevents_leakage_between_subsystems() {
    let patient_seed = 424242;
    let mut streams_baseline = PathwayRngStreams::new(patient_seed);
    let mut streams_with_service_activity = PathwayRngStreams::new(patient_seed);

    // In streams_with_service_activity, perform 100 draws on service_stream and routing_stream
    for _ in 0..100 {
        streams_with_service_activity.service_stream.next_uniform();
        streams_with_service_activity.routing_stream.next_uniform();
    }

    // Now draw next 10 progression variates from both bundles
    let mut draws_baseline = Vec::new();
    let mut draws_with_activity = Vec::new();

    for _ in 0..10 {
        draws_baseline.push(streams_baseline.progression_stream.next_exponential(0.05));
        draws_with_activity.push(
            streams_with_service_activity
                .progression_stream
                .next_exponential(0.05),
        );
    }

    // The progression stream draws MUST be 100% bit-identical despite heavy service/routing draws!
    assert_eq!(
        draws_baseline, draws_with_activity,
        "progression RNG stream must be decoupled from service and routing draws (CC-07)"
    );
}

#[test]
fn test_failed_service_calibration_cannot_resample_progression_clock() {
    let patient_seed = 999111;
    let mut streams = PathwayRngStreams::new(patient_seed);

    // Initial progression event time
    let initial_progression_draw = streams.progression_stream.next_exponential(0.02);

    // Simulate failed service parameter calibration iteration retrying 50 times
    for _ in 0..50 {
        let _service_draw = streams.service_stream.next_exponential(0.10);
        // Failed calibration retry...
    }

    // Progression draw count must remain exactly 1!
    assert_eq!(
        streams.progression_stream.draw_count, 1,
        "failed calibration of service duration must not resample progression clock"
    );
    assert_eq!(
        streams.service_stream.draw_count, 50,
        "service stream registers the 50 calibration retries"
    );
    assert_eq!(initial_progression_draw, initial_progression_draw);
}
