use careops_ed::{parse_scenario_json, run_scenario};
use kairo_ecs_core::Scheduler;
use kairo_ecs_types::{EntityId, EventKind, ScheduleRequest, SimTime, StepOutcome};

fn request(at_ticks: u64, entity_index: u64) -> ScheduleRequest {
    ScheduleRequest {
        at: SimTime::from_ticks(u128::from(at_ticks)),
        priority: 0,
        entity: Some(EntityId::new(entity_index, 0)),
        kind: EventKind::custom(entity_index as u32),
    }
}

#[test]
fn cancel_removes_a_scheduled_event_once_and_keeps_live_order() {
    let mut scheduler = Scheduler::new();
    let cancelled = scheduler.schedule(request(2, 1));
    let live = scheduler.schedule(request(1, 2));

    assert!(scheduler.cancel(cancelled));
    assert!(!scheduler.cancel(cancelled));
    assert_eq!(scheduler.pending_events(), 1);

    match scheduler.step() {
        StepOutcome::Dispatched(event) => {
            assert_eq!(event.id, live);
            assert_eq!(event.at, SimTime::from_ticks(1));
        }
        outcome => panic!("expected the live event to dispatch, got {outcome:?}"),
    }
    assert_eq!(scheduler.pending_events(), 0);
    assert_eq!(scheduler.step(), StepOutcome::Empty);
}

#[test]
fn same_tick_work_stops_at_budget_and_continues_on_the_next_call() {
    const TOTAL: u64 = 10_000;
    const FIRST_BUDGET: u64 = 1_000;

    let mut scheduler = Scheduler::new();
    for entity_index in 0..TOTAL {
        scheduler.schedule(request(11, entity_index));
    }

    assert_eq!(scheduler.run_for(0), StepOutcome::LimitReached);
    assert_eq!(scheduler.pending_events(), TOTAL as usize);
    assert_eq!(scheduler.stats().dispatched_events, 0);
    assert_eq!(scheduler.now(), SimTime::from_ticks(0));

    assert_eq!(scheduler.run_for(FIRST_BUDGET), StepOutcome::LimitReached);
    assert_eq!(scheduler.pending_events(), (TOTAL - FIRST_BUDGET) as usize);
    assert_eq!(scheduler.stats().dispatched_events, FIRST_BUDGET);
    assert_eq!(scheduler.now(), SimTime::from_ticks(11));

    match scheduler.run_for(TOTAL - FIRST_BUDGET) {
        StepOutcome::Dispatched(event) => assert_eq!(event.sequence, TOTAL - 1),
        outcome => panic!("expected the remaining bounded batch to dispatch, got {outcome:?}"),
    }
    assert_eq!(scheduler.pending_events(), 0);
    assert_eq!(scheduler.stats().dispatched_events, TOTAL);
    assert_eq!(scheduler.now(), SimTime::from_ticks(11));
}

#[test]
fn ed_rejects_zero_duration_work_without_claiming_graceful_cancellation() {
    let source = include_str!("../examples/one_patient.json");
    let mut scenario = parse_scenario_json(source).expect("synthetic fixture should validate");
    scenario.patients[0].work_ticks = 0;

    assert!(scenario.validate().is_err());
    assert!(run_scenario(&scenario).is_err());
}

// These are scheduler bounds and ED input-validation checks. The ED run API has
// no cancellation handle; this test makes no claim about graceful ED cancellation.
