use careops_birthing::{spaces::*, team::*, theatre::*};
use careops_core::capacity::{Calendar, CapabilitySet, UtcInterval};
use careops_core::pathway::{ObservedPolicyState, VersionedUrgencyAssessment};
use careops_core::staffing::{BookingId, StaffId};
use std::collections::BTreeMap;

fn calendar(a: i64, b: i64) -> Calendar {
    Calendar::new(vec![UtcInterval::new(a, b).unwrap()]).unwrap()
}
fn observed(category: u8, tick: u64) -> ObservedPolicyState {
    ObservedPolicyState {
        observed_at_tick: tick,
        elapsed_wait_ticks: tick,
        urgency: VersionedUrgencyAssessment {
            jurisdiction_id: "synthetic".into(),
            scheme_id: "fixture".into(),
            scheme_version: "v1".into(),
            category,
            assessed_at_tick: tick,
            effective_date: "2026-11-03".into(),
            source_authority: "synthetic-test".into(),
            citation: "fixture:B5".into(),
            source_sha256: None,
        },
    }
}
fn policy() -> QueuePolicy {
    QueuePolicy {
        revision: "policy:v1".into(),
        jurisdiction_id: "synthetic".into(),
        scheme_id: "fixture".into(),
        scheme_version: "v1".into(),
        source_authority: "synthetic-test".into(),
        citation: "fixture:B5".into(),
        rules: BTreeMap::from([
            (
                0,
                QueueRule {
                    priority_rank: -1,
                    targets: Targets {
                        transfer: Some(0),
                        incision: Some(3),
                        birth: Some(5),
                    },
                },
            ),
            (
                1,
                QueueRule {
                    priority_rank: 0,
                    targets: Targets {
                        transfer: Some(0),
                        incision: Some(3),
                        birth: Some(5),
                    },
                },
            ),
            (
                2,
                QueueRule {
                    priority_rank: 10,
                    targets: Targets {
                        transfer: Some(0),
                        incision: Some(3),
                        birth: Some(5),
                    },
                },
            ),
        ]),
    }
}
fn request(id: u64, category: u8) -> TheatreRequest {
    TheatreRequest {
        id,
        episode: 100 + id,
        booking: BookingId::new(1000 + id),
        decision_tick: 0,
        cohort_size: 1,
        observed: observed(category, 0),
        policy: policy(),
    }
}
fn fixture() -> (TheatreService, SpacePool, TeamAllocator) {
    let staffing = RatioProfile {
        revision: "staff:v1".into(),
        cohort_unit: "synthetic_case".into(),
        patients_per_staff: BTreeMap::from([("operator".into(), 1)]),
    };
    let theatre = TheatreService::new(TheatreProfile {
        revision: "service:v1".into(),
        preparation_ticks: 2,
        birth_after_incision_ticks: 3,
        completion_after_birth_ticks: 1,
        cleaning_ticks: 0,
        required_capabilities: vec!["theatre".into()],
        staffing,
    })
    .unwrap();
    let rooms = SpacePool::new(vec![SpaceProfile {
        id: 1,
        revision: "room:v1".into(),
        kind: SpaceKind::Theatre,
        capabilities: CapabilitySet::new(["theatre"]).unwrap(),
        open: calendar(0, 100),
        staffed: Some(calendar(0, 100)),
        equipped: Some(true),
    }])
    .unwrap();
    let team = TeamAllocator::new(vec![MemberProfile {
        staff: StaffId::new(1),
        revision: "staff:v1".into(),
        roles: CapabilitySet::new(["operator"]).unwrap(),
        roster: calendar(0, 100),
        unavailable: Calendar::default(),
        overtime_per_shift_seconds: 0,
        call_in_latency_seconds: 0,
        call_windows: None,
    }])
    .unwrap();
    (theatre, rooms, team)
}
fn ready(theatre: &mut TheatreService, id: u64) {
    theatre.accept(id, 0).unwrap();
    theatre.transfer_departed(id, 0).unwrap();
    theatre.transfer_arrived(id, 0).unwrap();
}

#[test]
fn transfer_is_not_birth_and_actual_callbacks_preserve_target_breaches() {
    let (mut theatre, mut rooms, mut team) = fixture();
    theatre.submit(request(1, 2)).unwrap();
    theatre.accept(1, 0).unwrap();
    theatre.transfer_departed(1, 1).unwrap();
    theatre.transfer_arrived(1, 2).unwrap();
    assert!(theatre.case(1).unwrap().timings.birth.is_none());
    assert!(!theatre
        .events()
        .iter()
        .any(|e| e.kind == CallbackKind::Birth));
    assert_eq!(
        theatre.start_next(2, &mut rooms, &mut team).unwrap(),
        Some(1)
    );
    assert_eq!(theatre.next_event_tick(), Some(4));
    let incision = theatre.advance_to(4, &mut rooms, &team).unwrap();
    assert_eq!(
        incision.iter().map(|e| e.kind).collect::<Vec<_>>(),
        vec![CallbackKind::ProcedureStarted]
    );
    assert!(theatre.case(1).unwrap().timings.birth.is_none());
    theatre.advance_to(8, &mut rooms, &team).unwrap();
    team.audit_responsibility(&rooms, 8).unwrap();
    let case = theatre.case(1).unwrap();
    assert_eq!(
        (
            case.timings.incision,
            case.timings.birth,
            case.timings.completed
        ),
        (Some(4), Some(7), Some(8))
    );
    assert_eq!(case.metrics.transfer.as_ref().unwrap().breached, Some(true));
    assert_eq!(case.metrics.incision.as_ref().unwrap().elapsed, 4);
    assert_eq!(case.metrics.birth.as_ref().unwrap().elapsed, 7);
    assert_eq!(case.metrics.birth.as_ref().unwrap().breached, Some(true));
    assert_eq!(rooms.snapshot(1).unwrap().occupied, 0);
}

#[test]
fn reassessment_reorders_only_queued_cases_and_never_interrupts_active_procedure() {
    let (mut theatre, mut rooms, mut team) = fixture();
    for (id, category) in [(1, 2), (2, 2), (3, 1)] {
        theatre.submit(request(id, category)).unwrap();
        ready(&mut theatre, id);
    }
    assert_eq!(
        theatre.start_next(0, &mut rooms, &mut team).unwrap(),
        Some(3)
    );
    theatre.reassess(1, observed(0, 1), policy()).unwrap();
    assert!(theatre.cancel(3, 1, "urgent queued case".into()).is_err());
    assert_eq!(theatre.start_next(1, &mut rooms, &mut team).unwrap(), None);
    theatre.advance_to(6, &mut rooms, &team).unwrap();
    assert_eq!(theatre.case(3).unwrap().timings.birth, Some(5));
    assert_eq!(
        theatre.start_next(6, &mut rooms, &mut team).unwrap(),
        Some(1)
    );
    assert_eq!(theatre.case(1).unwrap().initial.decision_tick, 0);
    assert!(theatre.case(2).unwrap().timings.started.is_none());
}

#[test]
fn denied_resource_attempt_is_a_wait_cause_not_a_partial_grant() {
    let (mut theatre, mut rooms, _) = fixture();
    let mut team = TeamAllocator::new(vec![MemberProfile {
        staff: StaffId::new(1),
        revision: "staff:v1".into(),
        roles: CapabilitySet::new(["operator"]).unwrap(),
        roster: calendar(0, 100),
        unavailable: calendar(0, 5),
        overtime_per_shift_seconds: 0,
        call_in_latency_seconds: 0,
        call_windows: None,
    }])
    .unwrap();
    theatre.submit(request(1, 2)).unwrap();
    ready(&mut theatre, 1);
    assert_eq!(theatre.start_next(0, &mut rooms, &mut team).unwrap(), None);
    assert_eq!(team.registry().assignment_count(), 0);
    assert_eq!(rooms.snapshot(1).unwrap().occupied, 0);
    assert_eq!(theatre.case(1).unwrap().resource_failures.len(), 1);
    theatre.advance_to(5, &mut rooms, &team).unwrap();
    assert_eq!(
        theatre.start_next(5, &mut rooms, &mut team).unwrap(),
        Some(1)
    );
}

#[test]
fn rejected_dispatch_advances_observation_time_without_new_resource_claims() {
    let (mut theatre, mut rooms, mut team) = fixture();
    // Occupy the sole qualified worker with an unrelated static booking.
    team.allocate(TeamRequest {
        booking: BookingId::new(9),
        patients: 1,
        profile: theatre.profile().staffing.clone(),
        interval: UtcInterval::new(0, 10).unwrap(),
        kind: careops_core::staffing::DutyKind::Service,
    })
    .unwrap();
    theatre.submit(request(1, 2)).unwrap();
    theatre.accept(1, 0).unwrap();
    theatre.transfer_departed(1, 1).unwrap();
    theatre.transfer_arrived(1, 2).unwrap();
    assert_eq!(theatre.start_next(2, &mut rooms, &mut team).unwrap(), None);
    assert_eq!(rooms.current_tick(), 2);
    team.audit_responsibility(&rooms, 2).unwrap();
    assert_eq!(team.registry().assignment_count(), 1);
    assert_eq!(rooms.snapshot(1).unwrap().occupied, 0);
}

#[test]
fn a_policy_revision_cannot_change_contents_between_cases_or_reassessments() {
    let (mut theatre, _, _) = fixture();
    theatre.submit(request(1, 2)).unwrap();
    let mut conflicting = request(2, 2);
    conflicting.policy.rules.get_mut(&2).unwrap().targets.birth = Some(999);
    assert!(theatre.submit(conflicting).is_err());
    assert!(theatre.case(2).is_none());
    let mut conflict = policy();
    conflict.rules.get_mut(&2).unwrap().priority_rank = -99;
    assert!(theatre.reassess(1, observed(2, 1), conflict).is_err());
    assert_eq!(theatre.case(1).unwrap().assessments.len(), 1);
}

#[test]
fn nonzero_decision_time_remains_the_target_and_observed_wait_anchor() {
    let (mut theatre, mut rooms, mut team) = fixture();
    let mut req = request(1, 2);
    req.decision_tick = 10;
    req.observed = observed(2, 10);
    req.observed.elapsed_wait_ticks = 0;
    theatre.submit(req).unwrap();
    theatre.accept(1, 10).unwrap();
    theatre.transfer_departed(1, 12).unwrap();
    theatre.transfer_arrived(1, 13).unwrap();
    assert_eq!(
        theatre
            .case(1)
            .unwrap()
            .metrics
            .transfer
            .as_ref()
            .unwrap()
            .elapsed,
        2
    );
    assert!(theatre.reassess(1, observed(1, 14), policy()).is_err());
    let mut obs = observed(1, 14);
    obs.elapsed_wait_ticks = 4;
    theatre.reassess(1, obs, policy()).unwrap();
    theatre.start_next(14, &mut rooms, &mut team).unwrap();
    theatre.advance_to(20, &mut rooms, &team).unwrap();
    let metric = theatre.case(1).unwrap().metrics.birth.as_ref().unwrap();
    assert_eq!(metric.elapsed, 9);
    assert_eq!(metric.assessment_tick, 14);
}

#[test]
fn queued_cancellation_is_idempotent_and_never_allocates_or_delivers() {
    let (mut theatre, mut rooms, mut team) = fixture();
    let req = request(1, 2);
    theatre.submit(req.clone()).unwrap();
    ready(&mut theatre, 1);
    theatre.cancel(1, 1, "synthetic withdrawal".into()).unwrap();
    theatre.cancel(1, 1, "synthetic withdrawal".into()).unwrap();
    assert!(theatre.cancel(1, 1, "changed reason".into()).is_err());
    theatre.submit(req).unwrap();
    assert_eq!(theatre.start_next(1, &mut rooms, &mut team).unwrap(), None);
    theatre.advance_to(10, &mut rooms, &team).unwrap();
    assert_eq!(
        theatre
            .events()
            .iter()
            .filter(|e| e.kind == CallbackKind::Cancelled)
            .count(),
        1
    );
    assert!(!theatre
        .events()
        .iter()
        .any(|e| e.kind == CallbackKind::Birth));
    assert_eq!(team.registry().assignment_count(), 0);
    assert_eq!(rooms.snapshot(1).unwrap().occupied, 0);
}

#[test]
fn clinical_callbacks_reject_wrong_room_history_or_premature_departure_atomically() {
    for birth_stage in [false, true] {
        for wrong_pool in [false, true] {
            let (mut theatre, mut rooms, mut team) = fixture();
            theatre.submit(request(1, 2)).unwrap();
            ready(&mut theatre, 1);
            theatre.start_next(0, &mut rooms, &mut team).unwrap();
            if birth_stage {
                theatre.advance_to(2, &mut rooms, &team).unwrap();
            }
            let at = if birth_stage { 5 } else { 2 };
            let mut unrelated = SpacePool::new(Vec::new()).unwrap();
            let supplied = if wrong_pool {
                &mut unrelated
            } else {
                rooms.depart(101, if birth_stage { 3 } else { 1 }).unwrap();
                &mut rooms
            };
            let before_tick = supplied.current_tick();
            let before_events = theatre.events().len();
            assert!(theatre.advance_to(at, supplied, &team).is_err());
            assert_eq!(supplied.current_tick(), before_tick);
            assert_eq!(theatre.events().len(), before_events);
            assert!(theatre.case(1).unwrap().timings.birth.is_none());
            if !birth_stage {
                assert!(theatre.case(1).unwrap().timings.incision.is_none());
            }
        }
    }
}

#[test]
fn clinical_callbacks_require_the_paired_team_admission_not_just_a_room() {
    let (mut theatre, mut rooms, mut team) = fixture();
    theatre.submit(request(1, 2)).unwrap();
    ready(&mut theatre, 1);
    theatre.start_next(0, &mut rooms, &mut team).unwrap();
    let (_, _, mut unrelated) = fixture();
    let before = theatre.events().len();
    assert!(theatre.advance_to(2, &mut rooms, &unrelated).is_err());
    assert_eq!(theatre.events().len(), before);
    assert_eq!(rooms.current_tick(), 0);
    // Even a matching standalone booking is not a paired service admission.
    unrelated
        .allocate(TeamRequest {
            booking: BookingId::new(1001),
            patients: 1,
            profile: theatre.profile().staffing.clone(),
            interval: UtcInterval::new(0, 6).unwrap(),
            kind: careops_core::staffing::DutyKind::Procedure {
                interruptible: false,
            },
        })
        .unwrap();
    assert!(theatre.advance_to(2, &mut rooms, &unrelated).is_err());
    assert_eq!(theatre.events().len(), before);
    assert_eq!(rooms.current_tick(), 0);
    theatre.advance_to(2, &mut rooms, &team).unwrap();
}

#[test]
fn retries_are_idempotent_and_conflicts_or_future_observations_are_atomic() {
    let (mut theatre, mut rooms, mut team) = fixture();
    let initial = request(1, 2);
    theatre.submit(initial.clone()).unwrap();
    theatre.submit(initial.clone()).unwrap();
    let mut changed = initial;
    changed.cohort_size = 2;
    assert!(theatre.submit(changed).is_err());
    let mut future = request(2, 2);
    future.observed.urgency.assessed_at_tick = 1;
    assert!(theatre.submit(future).is_err());
    assert!(theatre.case(2).is_none());
    ready(&mut theatre, 1);
    theatre.accept(1, 0).unwrap();
    let boundary_count = theatre.events().len();
    assert!(theatre.accept(1, 1).is_err());
    assert_eq!(theatre.events().len(), boundary_count);
    theatre.start_next(0, &mut rooms, &mut team).unwrap();
    theatre.advance_to(6, &mut rooms, &team).unwrap();
    let count = theatre.events().len();
    assert!(theatre.advance_to(6, &mut rooms, &team).unwrap().is_empty());
    theatre.transfer_arrived(1, 0).unwrap();
    assert_eq!(theatre.events().len(), count);
    assert_eq!(
        theatre
            .events()
            .iter()
            .filter(|e| e.kind == CallbackKind::Birth)
            .count(),
        1
    );
    assert!(theatre.cancel(1, 7, "completed".into()).is_err());
}

#[test]
fn zero_unknown_and_changed_policy_targets_never_rewrite_finished_metrics() {
    let (mut theatre, mut rooms, mut team) = fixture();
    let mut req = request(1, 2);
    req.policy.rules.get_mut(&2).unwrap().targets = Targets {
        transfer: Some(0),
        incision: Some(0),
        birth: None,
    };
    theatre.submit(req).unwrap();
    ready(&mut theatre, 1);
    theatre.start_next(0, &mut rooms, &mut team).unwrap();
    theatre.advance_to(2, &mut rooms, &team).unwrap();
    let mut obs = observed(2, 3);
    obs.urgency.scheme_version = "v2".into();
    let mut new_policy = policy();
    new_policy.revision = "policy:v2".into();
    new_policy.scheme_version = "v2".into();
    new_policy.rules.get_mut(&2).unwrap().targets = Targets {
        transfer: Some(999),
        incision: Some(999),
        birth: Some(0),
    };
    theatre.reassess(1, obs, new_policy).unwrap();
    theatre.advance_to(6, &mut rooms, &team).unwrap();
    let case = theatre.case(1).unwrap();
    assert_eq!(
        case.metrics.transfer.as_ref().unwrap().breached,
        Some(false)
    );
    assert_eq!(
        case.metrics.incision.as_ref().unwrap().policy_revision,
        "policy:v1"
    );
    assert_eq!(case.metrics.incision.as_ref().unwrap().breached, Some(true));
    assert_eq!(
        case.metrics.birth.as_ref().unwrap().policy_revision,
        "policy:v2"
    );
    assert_eq!(case.metrics.birth.as_ref().unwrap().breached, Some(true));
    let (mut unknown, mut rooms, mut team) = fixture();
    let mut req = request(1, 2);
    req.policy.rules.get_mut(&2).unwrap().targets.birth = None;
    unknown.submit(req).unwrap();
    ready(&mut unknown, 1);
    unknown.start_next(0, &mut rooms, &mut team).unwrap();
    unknown.advance_to(6, &mut rooms, &team).unwrap();
    assert_eq!(
        unknown
            .case(1)
            .unwrap()
            .metrics
            .birth
            .as_ref()
            .unwrap()
            .breached,
        None
    );
}

#[test]
fn due_callbacks_precede_external_events_and_zero_time_stages_have_stable_order() {
    let (mut theatre, mut rooms, mut team) = fixture();
    theatre.submit(request(1, 2)).unwrap();
    ready(&mut theatre, 1);
    theatre.start_next(0, &mut rooms, &mut team).unwrap();
    assert!(theatre.reassess(1, observed(1, 2), policy()).is_err());
    theatre.advance_to(2, &mut rooms, &team).unwrap();
    theatre.reassess(1, observed(1, 2), policy()).unwrap();
    let (base, mut rooms, mut team) = fixture();
    let mut profile = base.profile().clone();
    profile.preparation_ticks = 0;
    profile.birth_after_incision_ticks = 0;
    let mut instant = TheatreService::new(profile.clone()).unwrap();
    instant.submit(request(1, 2)).unwrap();
    ready(&mut instant, 1);
    instant.start_next(0, &mut rooms, &mut team).unwrap();
    assert_eq!(
        instant
            .advance_to(0, &mut rooms, &team)
            .unwrap()
            .iter()
            .map(|e| e.kind)
            .collect::<Vec<_>>(),
        vec![CallbackKind::ProcedureStarted, CallbackKind::Birth]
    );
    instant.advance_to(1, &mut rooms, &team).unwrap();
    team.audit_responsibility(&rooms, 1).unwrap();
    profile.completion_after_birth_ticks = 0;
    assert!(TheatreService::new(profile).is_err());
}
