use careops_ed::flow_adapter::{
    AdapterError, AdapterPlan, Admission, EdFlowAdapter, FoundationBounds, FoundationObservation,
    FrozenService, QueueKey, ResourceSpec, SampleOrigin, ServiceIdentity, TransitBounds,
    TransitInput,
};
use kairo_ecs_abm::spatial::{
    EdgeId, MovementModeId, MovementProfile, NodeId, RoutePlan, TransitEdge, TransitGraphV1,
};
use kairo_ecs_abm::TransitContextCheckpointLimitsV1;
use kairo_ecs_des::{FlowDomainControl, WorkState};
use kairo_ecs_types::SimTime;
use std::sync::Arc;

fn bounds() -> FoundationBounds {
    FoundationBounds {
        patient_owners: 4,
        services: 4,
        requests: 4,
        preempt_capable_requests: 0,
        releases_or_cancels: 0,
        parent_markers: 2,
    }
}
fn graph_route(zero: bool) -> (Arc<TransitGraphV1>, RoutePlan) {
    let mode = MovementModeId::new("walk").unwrap();
    let nodes = vec![NodeId::new(1), NodeId::new(2), NodeId::new(3)];
    let length = if zero { 0 } else { 2_000 };
    let edges = vec![
        TransitEdge {
            id: EdgeId::new(1),
            from: NodeId::new(1),
            to: NodeId::new(2),
            length_mm: length,
            allowed_modes: vec![mode.clone()],
        },
        TransitEdge {
            id: EdgeId::new(2),
            from: NodeId::new(2),
            to: NodeId::new(3),
            length_mm: length,
            allowed_modes: vec![mode.clone()],
        },
    ];
    let graph = Arc::new(TransitGraphV1::new(1, nodes, edges).unwrap());
    let profile = MovementProfile::new("walk", 1_000).unwrap();
    let destination = if zero { NodeId::new(1) } else { NodeId::new(3) };
    let route = graph
        .route(NodeId::new(1), destination, &profile, 1)
        .unwrap();
    (graph, route)
}
fn service(id: &str, ticks: u64) -> FrozenService {
    FrozenService {
        identity: ServiceIdentity {
            patient_id: id.into(),
            episode_id: format!("{id}-ep"),
            stage_index: 0,
            segment_index: 0,
        },
        original_ticks: ticks,
        useful_before_ticks: 0,
        remaining_ticks: ticks,
        origin: SampleOrigin::Fixed,
    }
}
fn adapter(horizon: u64, m: u64, l: u64, c: u64, resources: Vec<ResourceSpec>) -> EdFlowAdapter {
    let plan = AdapterPlan::validate(resources, vec![], horizon, bounds()).unwrap();
    EdFlowAdapter::new_with_transit(
        plan,
        TransitBounds {
            transits: m,
            total_route_segments: l,
            controls: c,
            native_snapshot_limits: TransitContextCheckpointLimitsV1::new(
                32, 65_536, 1_024, 131_072,
            ),
        },
    )
    .unwrap()
}
fn adapter_with_native_limits(limits: TransitContextCheckpointLimitsV1) -> EdFlowAdapter {
    let plan = AdapterPlan::validate(vec![], vec![], 8, bounds()).unwrap();
    EdFlowAdapter::new_with_transit(
        plan,
        TransitBounds {
            transits: 1,
            total_route_segments: 2,
            controls: 0,
            native_snapshot_limits: limits,
        },
    )
    .unwrap()
}

fn assert_stale_projection_unchanged(
    before: &careops_ed::flow_adapter::FoundationSnapshot,
    after: &careops_ed::flow_adapter::FoundationSnapshot,
) {
    assert_eq!(after.now, before.now);
    assert_eq!(after.services, before.services);
    assert_eq!(after.requests, before.requests);
    assert_eq!(after.resources, before.resources);
    assert_eq!(after.counters, before.counters);
    assert_eq!(after.native_budget.tick, before.native_budget.tick);
    assert_eq!(after.native_budget.limit, before.native_budget.limit);
    assert_eq!(after.native_budget.halted, before.native_budget.halted);
    assert_eq!(
        after.native_budget.scheduler.now,
        before.native_budget.scheduler.now
    );
    assert_eq!(
        after.native_budget.scheduler.scheduled_events,
        before.native_budget.scheduler.scheduled_events
    );
    assert_eq!(
        after.native_budget.scheduler.cancelled_events,
        before.native_budget.scheduler.cancelled_events
    );
    assert_eq!(
        after.native_budget.scheduler.dispatched_events,
        before.native_budget.scheduler.dispatched_events + 1
    );
    assert_eq!(
        after.native_budget.scheduler.pending_events + 1,
        before.native_budget.scheduler.pending_events
    );
    assert_eq!(
        after.native_budget.consumed,
        before.native_budget.consumed + 1
    );
}
fn start(
    a: &mut EdFlowAdapter,
    patient: &str,
    ticks: u64,
    route: (Arc<TransitGraphV1>, RoutePlan),
    at: u64,
) -> (
    careops_ed::flow_adapter::ServiceHandle,
    careops_ed::flow_adapter::TransitHandle,
) {
    let s = a.create_service(service(patient, ticks)).unwrap();
    let t = a
        .begin_transit(
            &s,
            TransitInput {
                route_id: format!("route-{patient}"),
                graph: route.0,
                route: route.1,
                start_tick: at,
            },
        )
        .unwrap();
    (s, t)
}
fn until_arrival(a: &mut EdFlowAdapter) -> Vec<careops_ed::flow_adapter::AdapterDispatch> {
    let mut out = Vec::new();
    while let Some(d) = a.step().unwrap() {
        let arrived = d.observations.iter().any(|o| {
            matches!(
                o,
                FoundationObservation::Transit {
                    arrival: Some(_),
                    ..
                }
            )
        });
        out.push(d);
        if arrived {
            break;
        }
    }
    out
}
fn drain(a: &mut EdFlowAdapter) -> Vec<careops_ed::flow_adapter::AdapterDispatch> {
    let mut out = Vec::new();
    while let Some(d) = a.step().unwrap() {
        out.push(d);
    }
    out
}

#[test]
fn adapter_transit_actual_moving_and_service_unconsumed() {
    let mut a = adapter(8, 1, 2, 0, vec![]);
    let (service, transit) = start(&mut a, "p1", 3, graph_route(false), 0);
    let initial = a.transit_snapshot(&transit).unwrap();
    let initial_service = a
        .snapshot()
        .services
        .into_iter()
        .find(|row| row.identity == *service.identity())
        .unwrap();
    let assert_unconsumed = |adapter: &EdFlowAdapter| {
        let snapshot = adapter.snapshot();
        assert_eq!(
            snapshot
                .services
                .iter()
                .find(|row| row.identity == *service.identity()),
            Some(&initial_service)
        );
        assert!(snapshot.resources.is_empty());
        assert!(snapshot.requests.is_empty());
    };
    assert_eq!(initial.route.segments.len(), 2);
    assert_eq!(initial.route.duration_ticks, 4);
    assert!(a.transit_arrival(&transit).unwrap().is_none());
    let mut events = Vec::new();
    let start = a.step().unwrap().unwrap();
    assert_eq!(start.native.at, SimTime::ZERO);
    assert_unconsumed(&a);
    events.push(start);
    let progress = a.step().unwrap().unwrap();
    assert_eq!(progress.native.at, SimTime::from_ticks(2));
    assert_unconsumed(&a);
    events.push(progress);
    let arrival_dispatch = a.step().unwrap().unwrap();
    assert_eq!(arrival_dispatch.native.at, SimTime::from_ticks(4));
    assert_unconsumed(&a);
    events.push(arrival_dispatch);
    let transit_rows: Vec<_> = events
        .iter()
        .flat_map(|d| d.observations.iter())
        .filter_map(|o| match o {
            FoundationObservation::Transit {
                carrier,
                event,
                phase,
                ..
            } => Some((*carrier, *event, *phase)),
            _ => None,
        })
        .collect();
    assert_eq!(transit_rows.len(), 3);
    assert!(transit_rows.windows(2).all(|pair| pair[0].0 == pair[1].0));
    assert_eq!(transit_rows[0].2, kairo_ecs_abm::TransitPhase::Moving);
    assert_eq!(transit_rows[1].2, kairo_ecs_abm::TransitPhase::Moving);
    assert_eq!(transit_rows[2].2, kairo_ecs_abm::TransitPhase::Arrived);
    for dispatch in &events {
        for observation in &dispatch.observations {
            if let FoundationObservation::Transit { event, .. } = observation {
                assert_eq!(*event, dispatch.native.event);
            }
        }
    }
    let arrival = a.transit_arrival(&transit).unwrap().unwrap();
    assert_eq!(arrival.service_work(), service.work_id());
    assert_eq!(
        arrival.owner(),
        a.transit_snapshot(&transit).unwrap().service_owner
    );
    assert!(events
        .iter()
        .flat_map(|d| &d.native.callback_batches)
        .all(|b| matches!(b, kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.len() <= 1)));
    assert_eq!(
        a.transit_snapshot(&transit).unwrap().route.duration_ticks,
        initial.route.duration_ticks
    );
    assert_unconsumed(&a);
    assert!(events
        .iter()
        .all(|dispatch| dispatch.native.records.is_empty()));
    assert!(events
        .iter()
        .flat_map(|dispatch| &dispatch.native.callback_batches)
        .all(|batch| matches!(batch, kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.len() <= 1)));
}

#[test]
fn adapter_transit_pause_retained_and_consumed_resume() {
    let mut a = adapter(10, 1, 2, 2, vec![]);
    let (consumed_service, t) = start(&mut a, "p2", 2, graph_route(false), 0);
    let consumed_service_before = a
        .snapshot()
        .services
        .into_iter()
        .find(|row| row.identity == *consumed_service.identity())
        .unwrap();
    let consumed_start = a.step().unwrap().unwrap();
    assert_eq!(consumed_start.native.at, SimTime::ZERO);
    let old_domain = a.peek_next().unwrap().id;
    a.schedule_transit_control(&t, FlowDomainControl::Pause, 2)
        .unwrap();
    let pause = a.step().unwrap().unwrap();
    assert_eq!(pause.native.at, SimTime::from_ticks(2));
    let paused = a.transit_snapshot(&t).unwrap();
    assert_eq!(paused.phase, kairo_ecs_abm::TransitPhase::Paused);
    assert_eq!(paused.segment_index, 1);
    assert_eq!(paused.elapsed_in_segment_ticks, 0);
    assert_eq!(
        a.snapshot()
            .services
            .iter()
            .find(|row| row.identity == *consumed_service.identity()),
        Some(&consumed_service_before)
    );
    assert_eq!(a.peek_next().unwrap().id, old_domain);
    let stale = a.step().unwrap().unwrap();
    assert_eq!(stale.native.event, old_domain);
    assert_eq!(stale.native.at, SimTime::from_ticks(2));
    assert!(stale
        .native
        .callback_batches
        .iter()
        .any(|b| matches!(b, kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.is_empty())));
    assert_eq!(stale.native.error, None);
    assert!(stale.native.records.is_empty());
    let after_stale = a.transit_snapshot(&t).unwrap();
    let mut expected_after_stale = paused.clone();
    expected_after_stale.expected_event = None;
    expected_after_stale.expected_due_ticks = None;
    expected_after_stale.next_progress_ticket = None;
    assert_eq!(after_stale, expected_after_stale);
    assert_eq!(
        a.snapshot()
            .services
            .iter()
            .find(|row| row.identity == *consumed_service.identity()),
        Some(&consumed_service_before)
    );
    let resume_event = a
        .schedule_transit_control(&t, FlowDomainControl::Resume, 5)
        .unwrap();
    let ds = drain(&mut a);
    assert!(ds
        .iter()
        .any(|d| { d.native.event == resume_event && d.native.at == SimTime::from_ticks(5) }));
    assert!(ds
        .iter()
        .flat_map(|dispatch| &dispatch.native.callback_batches)
        .all(|batch| matches!(batch, kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.len() <= 1)));
    for dispatch in std::iter::once(&consumed_start)
        .chain(std::iter::once(&pause))
        .chain(std::iter::once(&stale))
        .chain(ds.iter())
    {
        assert!(dispatch
            .native
            .callback_batches
            .iter()
            .all(|batch| matches!(batch, kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.len() <= 1)));
    }
    assert_eq!(
        a.transit_arrival(&t).unwrap().unwrap().arrived_at(),
        SimTime::from_ticks(7)
    );

    let mut retained = adapter(10, 1, 2, 2, vec![]);
    let (retained_service, retained_transit) = start(&mut retained, "p2", 2, graph_route(false), 0);
    let retained_service_before = retained
        .snapshot()
        .services
        .into_iter()
        .find(|row| row.identity == *retained_service.identity())
        .unwrap();
    let retained_start = retained.step().unwrap().unwrap();
    assert_eq!(retained_start.native.at, SimTime::ZERO);
    let retained_old_domain = retained.peek_next().unwrap().id;
    retained
        .schedule_transit_control(&retained_transit, FlowDomainControl::Pause, 2)
        .unwrap();
    let retained_pause = retained.step().unwrap().unwrap();
    assert_eq!(retained_pause.native.at, SimTime::from_ticks(2));
    let retained_paused = retained.transit_snapshot(&retained_transit).unwrap();
    assert_eq!(retained_paused.phase, kairo_ecs_abm::TransitPhase::Paused);
    assert_eq!(retained.peek_next().unwrap().id, retained_old_domain);
    assert_eq!(
        retained_paused.expected_event,
        Some((retained_old_domain.index, retained_old_domain.generation))
    );
    assert_eq!(
        retained
            .snapshot()
            .services
            .iter()
            .find(|row| row.identity == *retained_service.identity()),
        Some(&retained_service_before)
    );
    let retained_resume = retained
        .schedule_transit_control(&retained_transit, FlowDomainControl::Resume, 5)
        .unwrap();
    let retained_cut = retained.transit_snapshot(&retained_transit).unwrap();
    assert_eq!(
        retained_cut.expected_event,
        Some((retained_old_domain.index, retained_old_domain.generation))
    );
    let retained_stale = retained.step().unwrap().unwrap();
    assert_eq!(retained_stale.native.event, retained_old_domain);
    assert_eq!(retained_stale.native.at, SimTime::from_ticks(2));
    assert_eq!(retained_stale.native.error, None);
    assert!(retained_stale.native.records.is_empty());
    assert!(retained_stale
        .native
        .callback_batches
        .iter()
        .any(|batch| matches!(
            batch,
            kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.is_empty()
        )));
    let retained_after_stale = retained.transit_snapshot(&retained_transit).unwrap();
    assert_eq!(retained_after_stale.expected_event, None);
    assert_eq!(retained_after_stale.expected_due_ticks, None);
    assert_eq!(retained_after_stale.next_progress_ticket, None);
    assert_eq!(
        retained
            .snapshot()
            .services
            .iter()
            .find(|row| row.identity == *retained_service.identity()),
        Some(&retained_service_before)
    );
    let mut retained_suffix = vec![retained_start, retained_pause, retained_stale];
    retained_suffix.extend(until_arrival(&mut retained));
    assert!(retained_suffix.iter().any(|dispatch| {
        dispatch.native.event == retained_old_domain
            && dispatch.native.at == SimTime::from_ticks(2)
            && dispatch.native.error.is_none()
            && dispatch.native.records.is_empty()
            && dispatch.native.callback_batches.iter().any(|batch| {
                matches!(
                    batch,
                    kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.is_empty()
                )
            })
    }));
    assert!(retained_suffix.iter().any(|dispatch| {
        dispatch.native.event == retained_resume && dispatch.native.at == SimTime::from_ticks(5)
    }));
    assert!(retained_suffix
        .iter()
        .flat_map(|dispatch| &dispatch.native.callback_batches)
        .all(|batch| matches!(batch, kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.len() <= 1)));
    let retained_arrival = retained
        .transit_arrival(&retained_transit)
        .unwrap()
        .unwrap();
    assert_eq!(retained_arrival.arrived_at(), SimTime::from_ticks(7));
    assert_eq!(retained.snapshot().services, a.snapshot().services);
    assert_eq!(retained.snapshot().services[0], retained_service_before);
    let retained_final = retained.transit_snapshot(&retained_transit).unwrap();
    let consumed_final = a.transit_snapshot(&t).unwrap();
    assert_eq!(retained_final.route, consumed_final.route);
    assert_eq!(retained_final.segment_index, consumed_final.segment_index);
    assert_eq!(
        retained_final.elapsed_in_segment_ticks,
        consumed_final.elapsed_in_segment_ticks
    );
    assert_eq!(retained_final.phase, consumed_final.phase);
    assert_eq!(
        retained_final.last_advanced_at_ticks,
        consumed_final.last_advanced_at_ticks
    );
    assert_eq!(
        retained.snapshot().services[0].identity,
        *retained_service.identity()
    );
}

#[test]
fn adapter_transit_domain_and_frontier_resume_arrival() {
    let mut domain = adapter(8, 1, 2, 0, vec![]);
    let (_, t) = start(&mut domain, "p3", 2, graph_route(false), 0);
    let ds = drain(&mut domain);
    let arrival_dispatch = ds
        .iter()
        .find(|d| {
            d.observations.iter().any(|o| {
                matches!(
                    o,
                    FoundationObservation::Transit {
                        arrival: Some(_),
                        ..
                    }
                )
            })
        })
        .unwrap();
    assert_eq!(arrival_dispatch.native.error, None);
    assert!(arrival_dispatch.native.records.is_empty());
    assert!(arrival_dispatch
        .native
        .callback_batches
        .iter()
        .any(|batch| matches!(
            batch,
            kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.is_empty()
        )));
    assert_eq!(
        domain.transit_arrival(&t).unwrap().unwrap().arrival_event(),
        arrival_dispatch.native.event
    );
    assert_eq!(
        domain
            .transit_snapshot(&t)
            .unwrap()
            .arrival_witness
            .unwrap()
            .cause,
        kairo_ecs_abm::TransitArrivalCauseCheckpointV2::Domain
    );

    let mut frontier = adapter(8, 1, 2, 2, vec![]);
    let (_, t2) = start(&mut frontier, "p4", 2, graph_route(false), 0);
    let start_dispatch = frontier.step().unwrap().unwrap();
    assert_eq!(start_dispatch.native.at, SimTime::from_ticks(0));
    let first_progress = frontier.step().unwrap().unwrap();
    assert_eq!(first_progress.native.at, SimTime::from_ticks(2));
    let old_domain = frontier.peek_next().unwrap().id;
    frontier
        .schedule_transit_control(&t2, FlowDomainControl::Pause, 4)
        .unwrap();
    frontier
        .schedule_transit_control(&t2, FlowDomainControl::Resume, 4)
        .unwrap();
    let mut frontier_dispatches = Vec::new();
    let mut arrival_dispatch = None;
    while let Some(d) = frontier.step().unwrap() {
        let arrived = d.observations.iter().any(|o| {
            matches!(
                o,
                FoundationObservation::Transit {
                    arrival: Some(_),
                    ..
                }
            )
        });
        frontier_dispatches.push(d.clone());
        if arrived {
            arrival_dispatch = Some(d);
            break;
        }
    }
    let arrival_dispatch = arrival_dispatch.unwrap();
    assert_eq!(arrival_dispatch.native.error, None);
    assert!(arrival_dispatch.native.records.is_empty());
    assert!(arrival_dispatch
        .native
        .callback_batches
        .iter()
        .any(|batch| matches!(
            batch,
            kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.is_empty()
        )));
    let arrival = frontier.transit_arrival(&t2).unwrap().unwrap();
    assert_eq!(arrival.arrived_at(), SimTime::from_ticks(4));
    assert_eq!(arrival_dispatch.native.event, arrival.arrival_event());
    assert_eq!(
        frontier
            .transit_snapshot(&t2)
            .unwrap()
            .arrival_witness
            .unwrap()
            .cause,
        kairo_ecs_abm::TransitArrivalCauseCheckpointV2::Resume
    );
    let before_stale_image = frontier.transit_snapshot(&t2).unwrap();
    let before_stale_snapshot = frontier.snapshot();
    let stale = frontier.step().unwrap().unwrap();
    assert_eq!(stale.native.event, old_domain);
    assert!(stale.native.error.is_none());
    assert!(stale.native.records.is_empty());
    assert!(stale.native.callback_batches.iter().any(|b| matches!(b,
        kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.is_empty())));
    let after_stale_image = frontier.transit_snapshot(&t2).unwrap();
    assert_eq!(after_stale_image, before_stale_image);
    let after_stale_snapshot = frontier.snapshot();
    assert_stale_projection_unchanged(&before_stale_snapshot, &after_stale_snapshot);
    let carrier = frontier_dispatches
        .iter()
        .find_map(|d| {
            d.observations.iter().find_map(|o| match o {
                FoundationObservation::Transit { carrier, .. } => Some(*carrier),
                _ => None,
            })
        })
        .unwrap();
    assert!(stale.observations.iter().any(|o| matches!(o, FoundationObservation::Transit { event, carrier: observed, arrival: None, .. } if *event == old_domain && *observed == carrier)));
    assert!(frontier_dispatches
        .iter()
        .any(|d| d.native.at == SimTime::from_ticks(4)));
}

#[test]
fn adapter_transit_admits_once_and_suppresses() {
    let mut a = adapter(
        8,
        1,
        2,
        0,
        vec![ResourceSpec {
            id: "room".into(),
            capacity: 1,
        }],
    );
    let mut source = service("p5", 1);
    source.origin = SampleOrigin::Sampled {
        root_seed: 91,
        stream_slot: 17,
        draws_before: 41,
        draws_after: 43,
    };
    let s = a.create_service(source.clone()).unwrap();
    let route = graph_route(false);
    let t = a
        .begin_transit(
            &s,
            TransitInput {
                route_id: "route-p5".into(),
                graph: Arc::clone(&route.0),
                route: route.1.clone(),
                start_tick: 0,
            },
        )
        .unwrap();
    let before_reject = a.snapshot();
    assert!(matches!(
        a.admit_service(
            &s,
            Admission {
                resource_id: "room".into(),
                key: QueueKey::from_request(0, "q1".into()).unwrap(),
                urgency: 1,
                can_preempt: false,
                preemptible: false
            }
        ),
        Err(AdapterError::WrongNativeState)
    ));
    assert_eq!(a.snapshot(), before_reject);
    let arrived = until_arrival(&mut a);
    let witness_event = arrived
        .iter()
        .flat_map(|d| d.observations.iter())
        .find_map(|o| match o {
            FoundationObservation::Transit {
                arrival: Some(h), ..
            } => Some(h.arrival_event()),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        a.transit_snapshot(&t)
            .unwrap()
            .arrival_witness
            .unwrap()
            .event,
        (witness_event.index, witness_event.generation)
    );
    let before_duplicate_transit = a.snapshot();
    let before_duplicate_transit_context = a.transit_snapshot(&t).unwrap();
    let before_duplicate_transit_preview = a.peek_next();
    assert!(matches!(
        a.begin_transit(
            &s,
            TransitInput {
                route_id: "route-p5-duplicate".into(),
                graph: Arc::clone(&route.0),
                route: route.1.clone(),
                start_tick: 0,
            }
        ),
        Err(AdapterError::WrongHandleRole)
    ));
    assert_eq!(a.snapshot(), before_duplicate_transit);
    assert_eq!(
        a.transit_snapshot(&t).unwrap(),
        before_duplicate_transit_context
    );
    assert_eq!(a.peek_next(), before_duplicate_transit_preview);
    let admission = Admission {
        resource_id: "room".into(),
        key: QueueKey::from_request(4, "q1".into()).unwrap(),
        urgency: 1,
        can_preempt: false,
        preemptible: false,
    };
    let request = a.admit_service(&s, admission.clone()).unwrap();
    let before_duplicate_admission = a.snapshot();
    let before_duplicate_admission_preview = a.peek_next();
    assert!(matches!(
        a.admit_service(&s, admission),
        Err(AdapterError::WrongHandleRole)
    ));
    assert_eq!(a.snapshot(), before_duplicate_admission);
    assert_eq!(a.peek_next(), before_duplicate_admission_preview);
    assert!(a.transit_arrival(&t).unwrap().is_none());
    a.flush_dirty().unwrap();
    let ds = drain(&mut a);
    let completed = ds
        .iter()
        .find(|d| {
            d.native.records.iter().any(|r| {
                r.request == request.request_id()
                    && r.transition == kairo_ecs_des::LifecycleTransition::Completed
            })
        })
        .unwrap();
    assert!(completed.observations.iter().any(|o| matches!(o, FoundationObservation::Service(sv)
        if sv.request == request.request_id() && sv.work == s.work_id() && sv.identity == source.identity && sv.origin == source.origin)));
    assert!(!ds.iter().any(|d| d.observations.iter().any(|o| matches!(
        o,
        FoundationObservation::Transit {
            arrival: Some(_),
            ..
        }
    ))));
}

#[test]
fn adapter_transit_identity_and_owner_controls() {
    let mut a = adapter(8, 1, 2, 1, vec![]);
    let (_, t) = start(&mut a, "p6", 1, graph_route(false), 0);
    let before = a.transit_snapshot(&t).unwrap();
    let foreign = adapter(8, 1, 2, 1, vec![]);
    assert!(matches!(
        foreign.transit_snapshot(&t),
        Err(AdapterError::ForeignHandle)
    ));
    let snapshot_before = a.snapshot();
    let preview_before = a.peek_next();
    assert!(matches!(
        a.schedule_transit_control(&t, FlowDomainControl::Pause, 9),
        Err(AdapterError::InvalidInput(_))
    ));
    assert_eq!(a.transit_snapshot(&t).unwrap(), before);
    assert_eq!(a.snapshot(), snapshot_before);
    assert_eq!(a.peek_next(), preview_before);

    let mut bound = adapter(
        8,
        1,
        2,
        0,
        vec![ResourceSpec {
            id: "room".into(),
            capacity: 1,
        }],
    );
    let bound_service = bound.create_service(service("bound", 1)).unwrap();
    bound
        .admit_service(
            &bound_service,
            Admission {
                resource_id: "room".into(),
                key: QueueKey::from_request(0, "bound".into()).unwrap(),
                urgency: 1,
                can_preempt: false,
                preemptible: false,
            },
        )
        .unwrap();
    let bound_before = bound.snapshot();
    let route = graph_route(false);
    assert!(matches!(
        bound.begin_transit(
            &bound_service,
            TransitInput {
                route_id: "bound".into(),
                graph: route.0,
                route: route.1,
                start_tick: 0
            }
        ),
        Err(AdapterError::WrongHandleRole)
    ));
    assert_eq!(bound.snapshot(), bound_before);
    bound.flush_dirty().unwrap();
    let bound_completion = drain(&mut bound);
    assert!(bound_completion
        .iter()
        .any(|dispatch| dispatch
            .native
            .records
            .iter()
            .any(
                |record| record.snapshot.work == Some(bound_service.work_id())
                    && record.transition == kairo_ecs_des::LifecycleTransition::Completed
            )));
    let terminal_before = bound.snapshot();
    let terminal_preview = bound.peek_next();
    let terminal_route = graph_route(false);
    assert!(matches!(
        bound.begin_transit(
            &bound_service,
            TransitInput {
                route_id: "bound-terminal".into(),
                graph: terminal_route.0,
                route: terminal_route.1,
                start_tick: 0,
            }
        ),
        Err(AdapterError::WrongHandleRole)
    ));
    assert_eq!(bound.snapshot(), terminal_before);
    assert_eq!(bound.peek_next(), terminal_preview);

    let mut collision = adapter(8, 2, 4, 0, vec![]);
    let first_service = collision.create_service(service("collision-a", 1)).unwrap();
    let graph = graph_route(false);
    collision
        .begin_transit(
            &first_service,
            TransitInput {
                route_id: "same-route".into(),
                graph: graph.0,
                route: graph.1,
                start_tick: 0,
            },
        )
        .unwrap();
    let second_service = collision.create_service(service("collision-b", 1)).unwrap();
    let before = collision.snapshot();
    let changed = graph_route(true);
    assert!(matches!(
        collision.begin_transit(
            &second_service,
            TransitInput {
                route_id: "same-route".into(),
                graph: changed.0,
                route: changed.1,
                start_tick: 0
            }
        ),
        Err(AdapterError::InvalidInput("route id graph collision"))
    ));
    assert_eq!(collision.snapshot(), before);
}

#[test]
fn adapter_transit_horizon_and_zero_route() {
    let mut at_h = adapter(
        4,
        1,
        2,
        0,
        vec![ResourceSpec {
            id: "room".into(),
            capacity: 1,
        }],
    );
    let (s, t) = start(&mut at_h, "p7", 2, graph_route(false), 0);
    let at_h_dispatches = until_arrival(&mut at_h);
    assert!(at_h_dispatches
        .iter()
        .any(|d| d.native.at == SimTime::from_ticks(4)));
    assert!(at_h.transit_arrival(&t).unwrap().is_some());
    assert_eq!(at_h.now(), SimTime::from_ticks(4));
    let at_h_req = at_h
        .admit_service(
            &s,
            Admission {
                resource_id: "room".into(),
                key: QueueKey::from_request(4, "at-h".into()).unwrap(),
                urgency: 1,
                can_preempt: false,
                preemptible: false,
            },
        )
        .unwrap();
    at_h.flush_dirty().unwrap();
    let ds = drain(&mut at_h);
    let positive = ds
        .iter()
        .find(|d| {
            d.native.records.iter().any(|r| {
                r.request == at_h_req.request_id()
                    && r.transition == kairo_ecs_des::LifecycleTransition::Granted
            })
        })
        .unwrap();
    assert_eq!(positive.native.at, SimTime::from_ticks(4));
    assert!(positive.observations.iter().any(|o| matches!(o, FoundationObservation::Service(sv)
        if sv.request == at_h_req.request_id() && sv.useful_elapsed_ticks == 0 && sv.remaining_ticks == 2 && sv.state == WorkState::Active)));
    assert!(at_h
        .snapshot()
        .requests
        .iter()
        .any(|r| r.request == at_h_req.request_id()
            && r.state == kairo_ecs_des::RequestState::Active));
    assert_eq!(at_h.peek_next().unwrap().at, SimTime::from_ticks(6));

    let mut h3 = adapter(3, 1, 2, 0, vec![]);
    let (_, future) = start(&mut h3, "future", 1, graph_route(false), 0);
    let h3_events = drain(&mut h3);
    assert_eq!(h3.now(), SimTime::from_ticks(3));
    assert!(h3.transit_arrival(&future).unwrap().is_none());
    assert_eq!(
        h3.transit_snapshot(&future).unwrap().phase,
        kairo_ecs_abm::TransitPhase::Moving
    );
    let future_preview = h3.peek_next().unwrap();
    assert_eq!(future_preview.at, SimTime::from_ticks(4));
    assert!(!h3_events
        .iter()
        .any(|d| d.native.event == future_preview.id));

    let mut zero = adapter(2, 1, 0, 0, vec![]);
    let (_, z) = start(&mut zero, "p8", 1, graph_route(true), 0);
    let zds = until_arrival(&mut zero);
    assert!(zds.iter().any(|d| d.observations.iter().any(|o| matches!(
        o,
        FoundationObservation::Transit {
            arrival: Some(_),
            ..
        }
    ))));
    let zero_arrival = zero.transit_arrival(&z).unwrap().unwrap();
    assert_eq!(zds.len(), 1);
    assert_eq!(zds[0].native.event, zero_arrival.arrival_event());
    assert!(zds[0]
        .native
        .callback_batches
        .iter()
        .any(|b| matches!(b, kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.is_empty())));
    let zero_service = zero
        .snapshot()
        .services
        .into_iter()
        .find(|s| s.identity.patient_id == "p8")
        .unwrap();
    assert_eq!(zero_service.request, None);
    assert_eq!(zero_service.progress.state, WorkState::Pending);
    let horizon_remainder = drain(&mut zero);
    assert_eq!(horizon_remainder.len(), 1);
    assert_eq!(zero.now(), SimTime::from_ticks(2));
    assert!(horizon_remainder
        .iter()
        .all(|dispatch| dispatch.native.error.is_none()));
    assert!(!horizon_remainder
        .iter()
        .any(
            |dispatch| dispatch.observations.iter().any(|observation| matches!(
                observation,
                FoundationObservation::Transit {
                    arrival: Some(_),
                    ..
                }
            ))
        ));
    assert_eq!(
        zero.snapshot()
            .services
            .into_iter()
            .find(|row| row.identity.patient_id == "p8")
            .unwrap(),
        zero_service
    );
}

#[test]
fn adapter_transit_class_limits_and_callback_budget() {
    let mut a = adapter(8, 2, 4, 0, vec![]);
    let (s, t) = start(&mut a, "p9", 1, graph_route(false), 0);
    let snapshot = a.transit_snapshot(&t).unwrap();
    let route = graph_route(false);
    assert!(matches!(
        a.begin_transit(
            &s,
            TransitInput {
                route_id: "duplicate".into(),
                graph: route.0,
                route: route.1,
                start_tick: 0
            }
        ),
        Err(AdapterError::WrongHandleRole)
    ));
    assert_eq!(a.transit_snapshot(&t).unwrap(), snapshot);

    let mut capped = adapter(8, 1, 2, 0, vec![]);
    let (_, first) = start(&mut capped, "p10", 1, graph_route(false), 0);
    let second_service = capped.create_service(service("p11", 1)).unwrap();
    let before = capped.snapshot();
    let route = graph_route(false);
    assert!(matches!(
        capped.begin_transit(
            &second_service,
            TransitInput {
                route_id: "over".into(),
                graph: route.0,
                route: route.1,
                start_tick: 0
            }
        ),
        Err(AdapterError::BudgetExceeded("transits"))
    ));
    assert_eq!(capped.snapshot(), before);
    assert!(capped.transit_arrival(&first).unwrap().is_none());

    let mut segment_limited = adapter(8, 1, 1, 0, vec![]);
    let route_service = segment_limited.create_service(service("p12", 1)).unwrap();
    let before = segment_limited.snapshot();
    let route = graph_route(false);
    assert!(matches!(
        segment_limited.begin_transit(
            &route_service,
            TransitInput {
                route_id: "too-many-segments".into(),
                graph: route.0,
                route: route.1,
                start_tick: 0
            }
        ),
        Err(AdapterError::BudgetExceeded("route segments"))
    ));
    assert_eq!(segment_limited.snapshot(), before);

    let mut control_limited = adapter(8, 1, 2, 0, vec![]);
    let (_, controlled) = start(&mut control_limited, "p13", 1, graph_route(false), 0);
    let before = control_limited.snapshot();
    assert!(matches!(
        control_limited.schedule_transit_control(&controlled, FlowDomainControl::Pause, 1),
        Err(AdapterError::BudgetExceeded("transit controls"))
    ));
    assert_eq!(control_limited.snapshot(), before);

    let limits = TransitContextCheckpointLimitsV1::new(1, 65_536, 1_024, 131_072);
    let plan = AdapterPlan::validate(vec![], vec![], 8, bounds()).unwrap();
    let mut native_limited = EdFlowAdapter::new_with_transit(
        plan,
        TransitBounds {
            transits: 1,
            total_route_segments: 2,
            controls: 0,
            native_snapshot_limits: limits,
        },
    )
    .unwrap();
    let native_service = native_limited.create_service(service("p14", 1)).unwrap();
    let before = native_limited.snapshot();
    let route = graph_route(false);
    assert!(matches!(
        native_limited.begin_transit(
            &native_service,
            TransitInput {
                route_id: "native-limit".into(),
                graph: route.0,
                route: route.1,
                start_tick: 0
            }
        ),
        Err(AdapterError::InvalidInput("transit snapshot limits"))
    ));
    assert_eq!(native_limited.snapshot(), before);

    let route = graph_route(false);
    let graph_bytes = route.0.canonical_bytes_ref().len();
    let exact_total = 2 * 64 + graph_bytes + "walk".len() + 512;
    let exact = TransitContextCheckpointLimitsV1::new(2, graph_bytes, "walk".len(), exact_total);
    let mut exact_adapter = adapter_with_native_limits(exact);
    let exact_service = exact_adapter
        .create_service(service("exact-limits", 1))
        .unwrap();
    exact_adapter
        .begin_transit(
            &exact_service,
            TransitInput {
                route_id: "exact".into(),
                graph: route.0,
                route: route.1,
                start_tick: 0,
            },
        )
        .unwrap();
    for limits in [
        TransitContextCheckpointLimitsV1::new(1, graph_bytes, "walk".len(), exact_total),
        TransitContextCheckpointLimitsV1::new(2, graph_bytes - 1, "walk".len(), exact_total),
        TransitContextCheckpointLimitsV1::new(2, graph_bytes, "walk".len() - 1, exact_total),
        TransitContextCheckpointLimitsV1::new(2, graph_bytes, "walk".len(), exact_total - 1),
    ] {
        let mut limited = adapter_with_native_limits(limits);
        let candidate = limited.create_service(service("under-limit", 1)).unwrap();
        let before = limited.snapshot();
        let r = graph_route(false);
        assert!(matches!(
            limited.begin_transit(
                &candidate,
                TransitInput {
                    route_id: "under".into(),
                    graph: r.0,
                    route: r.1,
                    start_tick: 0
                }
            ),
            Err(AdapterError::InvalidInput("transit snapshot limits"))
        ));
        assert_eq!(limited.snapshot(), before);
    }

    let plan = AdapterPlan::validate(vec![], vec![], 8, bounds()).unwrap();
    assert!(matches!(
        EdFlowAdapter::new_with_transit(
            plan,
            TransitBounds {
                transits: u64::MAX,
                total_route_segments: 0,
                controls: 0,
                native_snapshot_limits: TransitContextCheckpointLimitsV1::new(1, 1, 1, 1)
            }
        ),
        Err(AdapterError::BudgetExceeded("transit entity bounds"))
    ));
    let plan = AdapterPlan::validate(vec![], vec![], 8, bounds()).unwrap();
    assert!(matches!(
        EdFlowAdapter::new_with_transit(
            plan,
            TransitBounds {
                transits: (u32::MAX as u64 / 2) + 1,
                total_route_segments: 0,
                controls: 0,
                native_snapshot_limits: TransitContextCheckpointLimitsV1::new(1, 1, 1, 1),
            }
        ),
        Err(AdapterError::BudgetExceeded("native entity representation"))
    ));
    let plan = AdapterPlan::validate(vec![], vec![], 8, bounds()).unwrap();
    assert!(matches!(
        EdFlowAdapter::new_with_transit(
            plan,
            TransitBounds {
                transits: u64::MAX,
                total_route_segments: 1,
                controls: 0,
                native_snapshot_limits: TransitContextCheckpointLimitsV1::new(1, 1, 1, 1),
            }
        ),
        Err(AdapterError::BudgetExceeded("transit operation bounds"))
    ));

    let mut concurrent = adapter(8, 2, 4, 0, vec![]);
    let (s1, _) = start(&mut concurrent, "p15", 1, graph_route(false), 0);
    let (s2, _) = start(&mut concurrent, "p16", 1, graph_route(false), 0);
    let mut rows = Vec::new();
    while let Some(d) = concurrent.step().unwrap() {
        let transit: Vec<_> = d
            .observations
            .iter()
            .filter_map(|o| match o {
                FoundationObservation::Transit {
                    identity,
                    carrier,
                    event,
                    ..
                } => Some((identity.patient_id.clone(), *carrier, *event)),
                _ => None,
            })
            .collect();
        assert!(
            transit.len() <= 1,
            "dispatch attributed to at most its owning carrier"
        );
        rows.extend(transit);
    }
    assert!(rows.iter().any(|(patient, _, _)| patient == "p15"));
    assert!(rows.iter().any(|(patient, _, _)| patient == "p16"));
    assert_ne!(s1.work_id(), s2.work_id());
}
