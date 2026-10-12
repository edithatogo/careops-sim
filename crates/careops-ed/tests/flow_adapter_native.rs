use careops_ed::flow_adapter::{
    AdapterError, AdapterPlan, Admission, EdFlowAdapter, FoundationBounds, FoundationObservation,
    FrozenService, GrantBoundary, ParentPhase, QueueKey, ResourceSpec, SampleOrigin,
    ServiceIdentity,
};
use kairo_ecs_des::{
    FlowAllocationMode, FlowQueueOrderKey, FlowQueueOrdering, FlowRuntime, LifecycleTransition,
    RequestState, WorkState,
};
use kairo_ecs_types::{SimDuration, SimTime};

fn bounds(
    owners: u64,
    services: u64,
    requests: u64,
    preempt: u64,
    mutations: u64,
    markers: u64,
) -> FoundationBounds {
    FoundationBounds {
        patient_owners: owners,
        services,
        requests,
        preempt_capable_requests: preempt,
        releases_or_cancels: mutations,
        parent_markers: markers,
    }
}
fn make(
    resources: Vec<ResourceSpec>,
    boundaries: Vec<GrantBoundary>,
    horizon: u64,
    b: FoundationBounds,
) -> EdFlowAdapter {
    EdFlowAdapter::new(AdapterPlan::validate(resources, boundaries, horizon, b).unwrap()).unwrap()
}
fn resource(id: &str, capacity: u32) -> ResourceSpec {
    ResourceSpec {
        id: id.into(),
        capacity,
    }
}
fn service(patient: &str, episode: &str, ticks: u64) -> FrozenService {
    FrozenService {
        identity: ServiceIdentity {
            patient_id: patient.into(),
            episode_id: episode.into(),
            stage_index: 0,
            segment_index: 0,
        },
        original_ticks: ticks,
        useful_before_ticks: 0,
        remaining_ticks: ticks,
        origin: SampleOrigin::Fixed,
    }
}
fn admission(
    resource_id: &str,
    key: QueueKey,
    urgency: i32,
    can_preempt: bool,
    preemptible: bool,
) -> Admission {
    Admission {
        resource_id: resource_id.into(),
        key,
        urgency,
        can_preempt,
        preemptible,
    }
}
fn drain_through_horizon(
    adapter: &mut EdFlowAdapter,
) -> Vec<careops_ed::flow_adapter::AdapterDispatch> {
    let mut out = Vec::new();
    while let Some(event) = adapter.step().unwrap() {
        out.push(event);
    }
    out
}

#[test]
fn foundation_native_queue_keys_and_urgency() {
    let mut adapter = make(
        vec![resource("room", 1)],
        vec![],
        5,
        bounds(2, 0, 2, 0, 0, 0),
    );
    let low = adapter
        .admit_manual(
            "low",
            admission(
                "room",
                QueueKey::from_request(0, "z".into()).unwrap(),
                i32::MIN,
                false,
                false,
            ),
        )
        .unwrap();
    let high = adapter
        .admit_manual(
            "high",
            admission(
                "room",
                QueueKey::from_request(0, "a".into()).unwrap(),
                i32::MAX,
                false,
                false,
            ),
        )
        .unwrap();
    adapter.flush_dirty().unwrap();
    let _ = adapter.step().unwrap().unwrap();
    let _ = adapter.step().unwrap().unwrap();
    let grant = adapter.step().unwrap().unwrap();
    let requests = adapter.snapshot().requests;
    let high_id = high.request_id();
    let low_id = low.request_id();
    assert!(grant
        .native
        .records
        .iter()
        .any(|r| { r.request == high_id && r.transition == LifecycleTransition::Granted }));
    assert!(grant.observations.iter().any(|observation| matches!(
        observation,
        FoundationObservation::Manual(m) if m.request == high_id && m.patient_id == "high"
    )));
    assert_eq!(
        requests.iter().find(|r| r.request == low_id).unwrap().state,
        RequestState::Queued
    );
    let mut key_order = make(
        vec![resource("room", 1)],
        vec![],
        2,
        bounds(2, 0, 2, 0, 0, 0),
    );
    let later = key_order
        .admit_manual(
            "later",
            admission(
                "room",
                QueueKey::from_request(0, "z".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let earlier = key_order
        .admit_manual(
            "earlier",
            admission(
                "room",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let later_id = later.request_id();
    let earlier_id = earlier.request_id();
    key_order.flush_dirty().unwrap();
    let _ = key_order.step().unwrap().unwrap();
    let _ = key_order.step().unwrap().unwrap();
    let keyed_grant = key_order.step().unwrap().unwrap();
    assert!(keyed_grant
        .native
        .records
        .iter()
        .any(|r| r.request == earlier_id && r.transition == LifecycleTransition::Granted));
    assert!(keyed_grant
        .native
        .records
        .iter()
        .all(|r| r.request != later_id || r.transition != LifecycleTransition::Granted));
    let mut ready_order = make(
        vec![resource("room", 1)],
        vec![],
        2,
        bounds(2, 0, 2, 0, 0, 1),
    );
    ready_order
        .schedule_marker("tick-one", 1, ParentPhase::Ready)
        .unwrap();
    let _ = ready_order.step().unwrap().unwrap();
    let older = ready_order
        .admit_manual(
            "older",
            admission(
                "room",
                QueueKey::from_request(0, "z".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let newer = ready_order
        .admit_manual(
            "newer",
            admission(
                "room",
                QueueKey::from_request(1, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let _ = ready_order.step().unwrap().unwrap();
    let _ = ready_order.step().unwrap().unwrap();
    ready_order.flush_dirty().unwrap();
    let ready_grant = ready_order.step().unwrap().unwrap();
    assert!(ready_grant
        .native
        .records
        .iter()
        .any(|r| r.request == older.request_id() && r.transition == LifecycleTransition::Granted));
    assert!(!ready_grant
        .native
        .records
        .iter()
        .any(|r| r.request == newer.request_id() && r.transition == LifecycleTransition::Granted));

    let mut e1_order = make(
        vec![resource("room", 1)],
        vec![],
        2,
        bounds(2, 0, 2, 0, 0, 0),
    );
    let later_ordinal = e1_order
        .admit_manual(
            "later",
            admission(
                "room",
                QueueKey::from_e1_ordinal(0, 2, 0, 0),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let earlier_ordinal = e1_order
        .admit_manual(
            "earlier",
            admission(
                "room",
                QueueKey::from_e1_ordinal(0, 1, 9, 9),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let _ = e1_order.step().unwrap().unwrap();
    let _ = e1_order.step().unwrap().unwrap();
    e1_order.flush_dirty().unwrap();
    let e1_grant = e1_order.step().unwrap().unwrap();
    assert!(e1_grant
        .native
        .records
        .iter()
        .any(|r| r.request == earlier_ordinal.request_id()
            && r.transition == LifecycleTransition::Granted));
    assert!(!e1_grant
        .native
        .records
        .iter()
        .any(|r| r.request == later_ordinal.request_id()
            && r.transition == LifecycleTransition::Granted));
    let ordinal_key = QueueKey::from_e1_ordinal(7, 1, 2, 3);
    assert_eq!(
        &ordinal_key.as_bytes()[8..],
        b"0000000000000001:0000000000000002:0000000000000003"
    );
}

#[test]
fn foundation_native_positive_service_and_lineage() {
    let mut adapter = make(
        vec![resource("staff", 1)],
        vec![],
        5,
        bounds(1, 1, 1, 0, 0, 0),
    );
    let s = service("p1", "p1-episode-0", 2);
    let expected_identity = s.identity.clone();
    let handle = adapter.create_service(s.clone()).unwrap();
    let request = adapter
        .admit_service(
            &handle,
            admission(
                "staff",
                QueueKey::from_request(0, "svc".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let request_id = request.request_id();
    let work_id = handle.work_id();
    assert_eq!(handle.identity(), &expected_identity);
    let created = adapter.snapshot().services[0].progress.clone();
    assert_eq!(created.useful_elapsed.ticks(), 0);
    assert_eq!(created.remaining.ticks(), 2);
    let submit = adapter.step().unwrap().unwrap();
    adapter.flush_dirty().unwrap();
    let grant = adapter.step().unwrap().unwrap();
    assert!(grant
        .native
        .records
        .iter()
        .any(|r| r.transition == LifecycleTransition::Granted && r.request == request_id));
    let mut events = vec![submit, grant];
    events.extend(drain_through_horizon(&mut adapter));
    let completed = events
        .iter()
        .flat_map(|d| d.observations.iter())
        .find_map(|o| match o {
            FoundationObservation::Service(s) if s.transition == LifecycleTransition::Completed => {
                Some(s)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(completed.identity, expected_identity);
    assert_eq!(completed.request, request_id);
    assert_eq!(completed.work, work_id);
    assert_eq!(completed.origin, s.origin);
    assert_eq!(completed.total_useful_ticks, 2);
    assert_eq!(completed.state, WorkState::Completed);
    for transition in [
        LifecycleTransition::Queued,
        LifecycleTransition::Granted,
        LifecycleTransition::Completed,
    ] {
        assert!(events.iter().flat_map(|d| d.observations.iter()).any(|o| matches!(o, FoundationObservation::Service(s) if s.request == request_id && s.work == work_id && s.transition == transition)));
    }
}

#[test]
fn foundation_native_sampled_service_lineage_pass_through() {
    let mut adapter = make(
        vec![resource("sampled-staff", 1)],
        vec![],
        3,
        bounds(1, 1, 1, 0, 0, 0),
    );
    let origin = SampleOrigin::Sampled {
        root_seed: 0x5eed_cafe_1234_9876,
        stream_slot: 17,
        draws_before: 41,
        draws_after: 43,
    };
    let mut sampled = service("p-sampled", "p-sampled-episode", 4);
    sampled.useful_before_ticks = 2;
    sampled.remaining_ticks = 2;
    sampled.origin = origin.clone();
    let identity = sampled.identity.clone();
    let handle = adapter.create_service(sampled.clone()).unwrap();
    let request = adapter
        .admit_service(
            &handle,
            admission(
                "sampled-staff",
                QueueKey::from_request(0, "sampled".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let request_id = request.request_id();
    let work_id = handle.work_id();
    assert_eq!(handle.identity(), &identity);
    let before = adapter.snapshot();
    assert_eq!(before.services[0].progress.useful_elapsed.ticks(), 0);
    assert_eq!(before.services[0].progress.remaining.ticks(), 2);

    let submit = adapter.step().unwrap().unwrap();
    adapter.flush_dirty().unwrap();
    let grant = adapter.step().unwrap().unwrap();
    let mut dispatches = vec![submit, grant];
    dispatches.extend(drain_through_horizon(&mut adapter));

    for (transition, elapsed, remaining, state) in [
        (LifecycleTransition::Queued, 0, 2, WorkState::Pending),
        (LifecycleTransition::Granted, 0, 2, WorkState::Active),
        (LifecycleTransition::Completed, 2, 0, WorkState::Completed),
    ] {
        let matches: Vec<_> = dispatches
            .iter()
            .flat_map(|dispatch| dispatch.observations.iter())
            .filter_map(|observation| match observation {
                FoundationObservation::Service(observation)
                    if observation.request == request_id
                        && observation.work == work_id
                        && observation.transition == transition =>
                {
                    Some(observation)
                }
                _ => None,
            })
            .collect();
        assert_eq!(matches.len(), 1, "expected one {transition:?} observation");
        let observation = matches[0];
        assert_eq!(observation.identity, identity);
        assert_eq!(observation.origin, origin);
        assert_eq!(observation.original_ticks, 4);
        assert_eq!(observation.useful_before_ticks, 2);
        assert_eq!(observation.useful_elapsed_ticks, elapsed);
        assert_eq!(observation.total_useful_ticks, 2 + elapsed);
        assert_eq!(observation.remaining_ticks, remaining);
        assert_eq!(observation.state, state);
    }
    for transition in [
        LifecycleTransition::Queued,
        LifecycleTransition::Granted,
        LifecycleTransition::Completed,
    ] {
        assert!(dispatches.iter().any(|dispatch| dispatch
            .native
            .records
            .iter()
            .any(|record| record.request == request_id && record.transition == transition)));
    }
    assert_eq!(sampled.origin, origin);
}

#[test]
fn foundation_native_manual_claims_and_release() {
    let mut adapter = make(
        vec![resource("room", 2)],
        vec![],
        5,
        bounds(3, 1, 3, 0, 1, 0),
    );
    let manual = adapter
        .admit_manual(
            "p1",
            admission(
                "room",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let work = adapter.create_service(service("p2", "e2", 1)).unwrap();
    let timed = adapter
        .admit_service(
            &work,
            admission(
                "room",
                QueueKey::from_request(0, "b".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let waiting = adapter
        .admit_manual(
            "p3",
            admission(
                "room",
                QueueKey::from_request(0, "c".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let manual_id = manual.request_id();
    let timed_id = timed.request_id();
    let waiting_id = waiting.request_id();
    for _ in 0..3 {
        let _ = adapter.step().unwrap().unwrap();
    }
    adapter.flush_dirty().unwrap();
    let initial_grant = adapter.step().unwrap().unwrap();
    assert!(initial_grant
        .native
        .records
        .iter()
        .any(|r| r.request == manual_id && r.transition == LifecycleTransition::Granted));
    assert!(initial_grant
        .native
        .records
        .iter()
        .any(|r| r.request == timed_id && r.transition == LifecycleTransition::Granted));
    assert_eq!(
        adapter
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == waiting_id)
            .unwrap()
            .state,
        RequestState::Queued
    );
    assert!(adapter
        .snapshot()
        .requests
        .iter()
        .find(|r| r.request == waiting_id)
        .unwrap()
        .lease
        .is_none());

    let mut completed = false;
    while let Some(d) = adapter.step().unwrap() {
        completed |= d
            .native
            .records
            .iter()
            .any(|r| r.request == timed_id && r.transition == LifecycleTransition::Completed);
        if completed {
            break;
        }
    }
    assert!(completed);
    let state = adapter.snapshot();
    assert_eq!(
        state
            .requests
            .iter()
            .find(|r| r.request == manual_id)
            .unwrap()
            .state,
        RequestState::Active
    );
    assert_eq!(
        state
            .requests
            .iter()
            .find(|r| r.request == waiting_id)
            .unwrap()
            .state,
        RequestState::Queued
    );

    adapter.flush_dirty().unwrap();
    let flush = adapter.step().unwrap().unwrap();
    assert!(flush
        .native
        .records
        .iter()
        .any(|r| r.request == waiting_id && r.transition == LifecycleTransition::Granted));
    let before_release = adapter.snapshot();
    adapter.release_active(&manual).unwrap();
    let release = adapter.step().unwrap().unwrap();
    assert!(release
        .native
        .records
        .iter()
        .any(|r| r.request == manual_id && r.transition == LifecycleTransition::Released));
    let after_release = adapter.snapshot();
    assert!(matches!(
        adapter.release_active(&manual),
        Err(AdapterError::WrongNativeState)
    ));
    assert_eq!(adapter.snapshot(), after_release);
    assert_ne!(
        before_release
            .requests
            .iter()
            .find(|r| r.request == manual_id)
            .unwrap()
            .state,
        after_release
            .requests
            .iter()
            .find(|r| r.request == manual_id)
            .unwrap()
            .state
    );

    let mut simultaneous = make(
        vec![resource("r1", 1), resource("r2", 1)],
        vec![],
        5,
        bounds(2, 2, 2, 0, 2, 0),
    );
    let work_a = simultaneous.create_service(service("a", "a-e", 2)).unwrap();
    let work_b = simultaneous.create_service(service("b", "b-e", 2)).unwrap();
    let req_a = simultaneous
        .admit_service(
            &work_a,
            admission(
                "r1",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let req_b = simultaneous
        .admit_service(
            &work_b,
            admission(
                "r2",
                QueueKey::from_request(0, "b".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let _ = simultaneous.step().unwrap().unwrap();
    let _ = simultaneous.step().unwrap().unwrap();
    simultaneous.flush_dirty().unwrap();
    let _ = simultaneous.step().unwrap().unwrap();
    let _ = simultaneous.step().unwrap().unwrap();
    let first_completion = simultaneous.step().unwrap().unwrap();
    assert!(first_completion
        .native
        .records
        .iter()
        .any(|r| r.transition == LifecycleTransition::Completed));
    let remaining_handle =
        if first_completion.native.records.iter().any(|r| {
            r.request == req_a.request_id() && r.transition == LifecycleTransition::Completed
        }) {
            &req_b
        } else {
            &req_a
        };
    let before_zero_remainder_release = simultaneous.snapshot();
    assert_eq!(
        before_zero_remainder_release
            .requests
            .iter()
            .find(|r| r.request == remaining_handle.request_id())
            .unwrap()
            .state,
        RequestState::Active
    );
    assert!(matches!(
        simultaneous.release_active(remaining_handle),
        Err(AdapterError::WrongNativeState)
    ));
    assert_eq!(simultaneous.snapshot(), before_zero_remainder_release);
    let second_completion = simultaneous.step().unwrap().unwrap();
    assert!(second_completion
        .native
        .records
        .iter()
        .any(|r| r.request == remaining_handle.request_id()
            && r.transition == LifecycleTransition::Completed));
}

#[test]
fn foundation_native_suspend_and_finite_bound() {
    let mut adapter = make(
        vec![resource("chair", 2)],
        vec![],
        20,
        bounds(4, 4, 4, 2, 0, 2),
    );
    let a_service = service("pA", "eA", 15);
    let b_service = service("pB", "eB", 12);
    let a = adapter.create_service(a_service.clone()).unwrap();
    let b = adapter.create_service(b_service.clone()).unwrap();
    let a_request = adapter
        .admit_service(
            &a,
            admission(
                "chair",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let b_request = adapter
        .admit_service(
            &b,
            admission(
                "chair",
                QueueKey::from_request(0, "b".into()).unwrap(),
                1,
                false,
                true,
            ),
        )
        .unwrap();
    let marker_one = adapter
        .schedule_marker("decision-one", 1, ParentPhase::Decision)
        .unwrap();
    let marker_two = adapter
        .schedule_marker("decision-two", 2, ParentPhase::Decision)
        .unwrap();
    let mut dispatches = Vec::new();
    dispatches.push(adapter.step().unwrap().unwrap());
    dispatches.push(adapter.step().unwrap().unwrap());
    adapter.flush_dirty().unwrap();
    dispatches.push(adapter.step().unwrap().unwrap());
    assert_eq!(
        adapter
            .snapshot()
            .requests
            .iter()
            .filter(|r| r.state == RequestState::Active)
            .count(),
        2
    );
    while let Some(d) = adapter.step().unwrap() {
        let found = d.native.event == marker_one;
        dispatches.push(d);
        if found {
            break;
        }
    }

    let c_service = service("pC", "eC", 6);
    let c = adapter.create_service(c_service.clone()).unwrap();
    let c_request = adapter
        .admit_service(
            &c,
            admission(
                "chair",
                QueueKey::from_request(1, "c".into()).unwrap(),
                2,
                true,
                true,
            ),
        )
        .unwrap();
    dispatches.push(adapter.step().unwrap().unwrap());
    adapter.flush_dirty().unwrap();
    dispatches.push(adapter.step().unwrap().unwrap());
    assert_eq!(
        adapter
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == a_request.request_id())
            .unwrap()
            .state,
        RequestState::Active
    );
    assert_eq!(
        adapter
            .snapshot()
            .services
            .iter()
            .find(|s| s.identity == b_service.identity)
            .unwrap()
            .progress
            .remaining
            .ticks(),
        11
    );

    while let Some(d) = adapter.step().unwrap() {
        let found = d.native.event == marker_two;
        dispatches.push(d);
        if found {
            break;
        }
    }
    let d_service = service("pD", "eD", 2);
    let d = adapter.create_service(d_service.clone()).unwrap();
    let d_request = adapter
        .admit_service(
            &d,
            admission(
                "chair",
                QueueKey::from_request(2, "d".into()).unwrap(),
                3,
                true,
                false,
            ),
        )
        .unwrap();
    dispatches.push(adapter.step().unwrap().unwrap());
    adapter.flush_dirty().unwrap();
    dispatches.push(adapter.step().unwrap().unwrap());
    assert_eq!(
        adapter
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == a_request.request_id())
            .unwrap()
            .state,
        RequestState::Active
    );
    assert_eq!(
        adapter
            .snapshot()
            .services
            .iter()
            .find(|s| s.identity == c_service.identity)
            .unwrap()
            .progress
            .remaining
            .ticks(),
        5
    );

    while let Some(d) = adapter.step().unwrap() {
        let completed = d
            .native
            .records
            .iter()
            .any(|r| r.transition == LifecycleTransition::Completed);
        dispatches.push(d);
        if completed && !adapter.flush_dirty().unwrap().is_empty() {
            dispatches.push(adapter.step().unwrap().unwrap());
        }
    }
    let rows: Vec<_> = dispatches
        .iter()
        .flat_map(|d| d.native.records.iter())
        .collect();
    let preemptions = rows
        .iter()
        .filter(|r| r.transition == LifecycleTransition::Preempted)
        .count();
    let resumptions = rows
        .iter()
        .filter(|r| r.transition == LifecycleTransition::Resumed)
        .count();
    let grants = rows
        .iter()
        .filter(|r| r.transition == LifecycleTransition::Granted)
        .count();
    let starts = grants + resumptions;
    assert_eq!(preemptions, 2);
    assert_eq!(resumptions, 2);
    assert_eq!(grants, 4);
    assert_eq!(starts, 6);
    let has_transition_at = |request, transition, tick| {
        dispatches.iter().any(|dispatch| {
            dispatch.native.at == SimTime::from_ticks(tick)
                && dispatch
                    .native
                    .records
                    .iter()
                    .any(|record| record.request == request && record.transition == transition)
        })
    };
    for (request, transition, tick) in [
        (b_request.request_id(), LifecycleTransition::Preempted, 1),
        (c_request.request_id(), LifecycleTransition::Granted, 1),
        (c_request.request_id(), LifecycleTransition::Preempted, 2),
        (d_request.request_id(), LifecycleTransition::Granted, 2),
        (d_request.request_id(), LifecycleTransition::Completed, 4),
        (c_request.request_id(), LifecycleTransition::Resumed, 4),
        (c_request.request_id(), LifecycleTransition::Completed, 9),
        (b_request.request_id(), LifecycleTransition::Resumed, 9),
        (a_request.request_id(), LifecycleTransition::Completed, 15),
        (b_request.request_id(), LifecycleTransition::Completed, 20),
    ] {
        assert!(
            has_transition_at(request, transition, tick),
            "missing {transition:?} for request {request:?} at {tick}"
        );
    }
    assert_eq!(rows.len(), 16);
    assert!(preemptions as u64 <= adapter.snapshot().counters.preempt_capable_requests);
    assert!(starts as u64 <= 4 + 2);
    assert!(rows.len() <= 3 * 4 + 2 * 2);
    assert_eq!(
        adapter.snapshot().native_budget.limit.get(),
        3 * 4 + 2 * 2 + 3
    );
    assert!(adapter.snapshot().native_budget.scheduler.scheduled_events <= 24);
    for (handle, request, expected) in [
        (&a, &a_request, &a_service),
        (&b, &b_request, &b_service),
        (&c, &c_request, &c_service),
        (&d, &d_request, &d_service),
    ] {
        let linked: Vec<_> = dispatches
            .iter()
            .flat_map(|x| x.observations.iter())
            .filter_map(|o| match o {
                FoundationObservation::Service(s) if s.request == request.request_id() => Some(s),
                _ => None,
            })
            .collect();
        assert!(!linked.is_empty());
        assert!(linked.iter().all(|s| s.work == handle.work_id()
            && s.identity == expected.identity
            && s.origin == expected.origin));
    }

    let mut equal = make(vec![resource("r", 1)], vec![], 4, bounds(2, 2, 2, 1, 0, 0));
    let holder = equal
        .create_service(service("holder", "holder-e", 4))
        .unwrap();
    let holder_req = equal
        .admit_service(
            &holder,
            admission(
                "r",
                QueueKey::from_request(0, "a".into()).unwrap(),
                4,
                false,
                true,
            ),
        )
        .unwrap();
    let _ = equal.step().unwrap().unwrap();
    equal.flush_dirty().unwrap();
    let _ = equal.step().unwrap().unwrap();
    let equal_waiter = equal
        .create_service(service("waiter", "waiter-e", 1))
        .unwrap();
    let waiter_req = equal
        .admit_service(
            &equal_waiter,
            admission(
                "r",
                QueueKey::from_request(0, "b".into()).unwrap(),
                4,
                true,
                false,
            ),
        )
        .unwrap();
    let _ = equal.step().unwrap().unwrap();
    equal.flush_dirty().unwrap();
    let pass = equal.step().unwrap().unwrap();
    assert!(!pass
        .native
        .records
        .iter()
        .any(|r| r.request == holder_req.request_id()
            && r.transition == LifecycleTransition::Preempted));
    assert_eq!(
        equal
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == holder_req.request_id())
            .unwrap()
            .state,
        RequestState::Active
    );
    assert_eq!(
        equal
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == waiter_req.request_id())
            .unwrap()
            .state,
        RequestState::Queued
    );
}

#[test]
fn foundation_native_calendar_and_reflush() {
    let mut adapter = make(
        vec![resource("bed", 1)],
        vec![
            GrantBoundary {
                resource_id: "bed".into(),
                at_tick: 0,
                enabled: false,
            },
            GrantBoundary {
                resource_id: "bed".into(),
                at_tick: 3,
                enabled: true,
            },
        ],
        5,
        bounds(3, 0, 3, 0, 2, 0),
    );
    let cancel_one = adapter
        .admit_manual(
            "p1",
            admission(
                "bed",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let cancel_two = adapter
        .admit_manual(
            "p2",
            admission(
                "bed",
                QueueKey::from_request(0, "b".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let waiting = adapter
        .admit_manual(
            "p3",
            admission(
                "bed",
                QueueKey::from_request(0, "c".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let one_id = cancel_one.request_id();
    let two_id = cancel_two.request_id();
    let waiting_id = waiting.request_id();
    assert_eq!(
        adapter.step().unwrap().unwrap().native.at,
        SimTime::from_ticks(0)
    ); // Disable before Submit.
    for _ in 0..3 {
        let _ = adapter.step().unwrap().unwrap();
    }
    let scheduled = adapter.snapshot().native_budget.scheduler.scheduled_events;
    assert_eq!(adapter.flush_dirty().unwrap().len(), 1);
    assert!(adapter.flush_dirty().unwrap().is_empty());
    assert_eq!(
        adapter.snapshot().native_budget.scheduler.scheduled_events,
        scheduled + 1
    );
    let before_rejected_cancel = adapter.snapshot();
    assert!(matches!(
        adapter.cancel_waiting(&cancel_one),
        Err(AdapterError::UnclosedGrantPhase)
    ));
    assert_eq!(adapter.snapshot(), before_rejected_cancel);
    let blocked = adapter.step().unwrap().unwrap();
    assert!(blocked
        .native
        .records
        .iter()
        .all(|r| r.transition != LifecycleTransition::Granted));
    assert_eq!(
        adapter
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == waiting_id)
            .unwrap()
            .state,
        RequestState::Queued
    );

    adapter.cancel_waiting(&cancel_one).unwrap();
    adapter.cancel_waiting(&cancel_two).unwrap();
    assert!(matches!(
        adapter.flush_dirty(),
        Err(AdapterError::UnclosedCancellationBatch)
    ));
    let first_cancel = adapter.step().unwrap().unwrap();
    assert!(first_cancel
        .native
        .records
        .iter()
        .any(|r| r.transition == LifecycleTransition::Cancelled));
    assert!(matches!(
        adapter.flush_dirty(),
        Err(AdapterError::UnclosedCancellationBatch)
    ));
    let second_cancel = adapter.step().unwrap().unwrap();
    assert!(second_cancel
        .native
        .records
        .iter()
        .any(|r| r.transition == LifecycleTransition::Cancelled));
    let cancel_flushes = adapter.flush_dirty().unwrap();
    assert_eq!(cancel_flushes.len(), 1);
    let _ = adapter.step().unwrap().unwrap();
    assert_eq!(
        adapter
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == one_id)
            .unwrap()
            .state,
        RequestState::Cancelled
    );
    assert_eq!(
        adapter
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == two_id)
            .unwrap()
            .state,
        RequestState::Cancelled
    );

    let mut enabled = false;
    while !enabled {
        let dispatch = adapter.step().unwrap().unwrap();
        enabled = dispatch.native.at == SimTime::from_ticks(3);
    }
    let reflush = adapter.flush_dirty().unwrap();
    assert_eq!(reflush.len(), 1);
    let grant = adapter.step().unwrap().unwrap();
    assert!(grant
        .native
        .records
        .iter()
        .any(|r| r.request == waiting_id && r.transition == LifecycleTransition::Granted));
    assert_eq!(
        adapter
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == waiting_id)
            .unwrap()
            .state,
        RequestState::Active
    );
    assert_eq!(adapter.snapshot().now, SimTime::from_ticks(3));
}

#[test]
fn foundation_native_horizon_and_wide_duration() {
    let mut equality = make(
        vec![resource("staff", 1)],
        vec![],
        5,
        bounds(1, 1, 1, 0, 0, 0),
    );
    let exact_service = service("exact", "exact-e", 5);
    let exact = equality.create_service(exact_service).unwrap();
    let exact_req = equality
        .admit_service(
            &exact,
            admission(
                "staff",
                QueueKey::from_request(0, "exact".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let _ = equality.step().unwrap().unwrap();
    equality.flush_dirty().unwrap();
    let grant = equality.step().unwrap().unwrap();
    assert!(grant.native.records.iter().any(
        |r| r.request == exact_req.request_id() && r.transition == LifecycleTransition::Granted
    ));
    let mut completion_at_h = false;
    let mut horizon_at_h = false;
    while !horizon_at_h {
        let d = equality.step().unwrap().unwrap();
        if d.native.at == SimTime::from_ticks(5)
            && d.native.records.iter().any(|r| {
                r.request == exact_req.request_id()
                    && r.transition == LifecycleTransition::Completed
            })
        {
            completion_at_h = true;
            assert!(!equality.flush_dirty().unwrap().is_empty());
            let f = equality.step().unwrap().unwrap();
            assert_eq!(f.native.at, SimTime::from_ticks(5));
        }
        horizon_at_h = d.observations.iter().any(|o| matches!(o, FoundationObservation::Marker { operation_id, .. } if operation_id == "__ed_horizon__"));
    }
    assert!(completion_at_h);
    assert_eq!(equality.now(), SimTime::from_ticks(5));

    let mut at_h = make(
        vec![resource("room", 1)],
        vec![],
        3,
        bounds(2, 2, 2, 0, 0, 1),
    );
    let first = at_h.create_service(service("p1", "p1-e", 3)).unwrap();
    let second = at_h.create_service(service("p2", "p2-e", 2)).unwrap();
    let first_req = at_h
        .admit_service(
            &first,
            admission(
                "room",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let second_req = at_h
        .admit_service(
            &second,
            admission(
                "room",
                QueueKey::from_request(0, "b".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let _ = at_h.step().unwrap().unwrap();
    let _ = at_h.step().unwrap().unwrap();
    at_h.flush_dirty().unwrap();
    let _ = at_h.step().unwrap().unwrap();
    let completed = at_h.step().unwrap().unwrap();
    assert_eq!(completed.native.at, SimTime::from_ticks(3));
    assert!(completed
        .native
        .records
        .iter()
        .any(|r| r.request == first_req.request_id()
            && r.transition == LifecycleTransition::Completed));
    at_h.flush_dirty().unwrap();
    let positive_grant = at_h.step().unwrap().unwrap();
    assert_eq!(positive_grant.native.at, SimTime::from_ticks(3));
    assert!(positive_grant
        .native
        .records
        .iter()
        .any(|r| r.request == second_req.request_id()
            && r.transition == LifecycleTransition::Granted));
    assert!(positive_grant.observations.iter().any(|o| matches!(o, FoundationObservation::Service(s) if s.request == second_req.request_id() && s.work == second.work_id() && s.useful_elapsed_ticks == 0 && s.remaining_ticks == 2 && s.state == WorkState::Active)));
    assert_eq!(
        at_h.snapshot()
            .services
            .iter()
            .find(|s| s.identity.patient_id == "p2")
            .unwrap()
            .progress
            .remaining
            .ticks(),
        2
    );
    let horizon = at_h.step().unwrap().unwrap();
    assert!(horizon.observations.iter().any(|o| matches!(o, FoundationObservation::Marker { operation_id, .. } if operation_id == "__ed_horizon__")));
    let pending = at_h.peek_next().unwrap();
    assert_eq!(pending.at, SimTime::from_ticks(5));
    assert!(at_h.step().unwrap().is_none());
    assert_eq!(at_h.now(), SimTime::from_ticks(3));

    let mut wide = make(
        vec![resource("long", 1)],
        vec![],
        3,
        bounds(1, 1, 1, 0, 0, 0),
    );
    let long_work = wide
        .create_service(service("long", "long-e", u64::MAX))
        .unwrap();
    let long_request = wide
        .admit_service(
            &long_work,
            admission(
                "long",
                QueueKey::from_request(0, "max".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let _ = wide.step().unwrap().unwrap();
    wide.flush_dirty().unwrap();
    let _ = wide.step().unwrap().unwrap();
    let _ = drain_through_horizon(&mut wide);
    assert_eq!(wide.now(), SimTime::from_ticks(3));
    assert_eq!(
        wide.peek_next().unwrap().at,
        SimTime::from_ticks(u64::MAX as u128)
    );
    assert!(wide.step().unwrap().is_none());
    assert_eq!(
        wide.snapshot()
            .requests
            .iter()
            .find(|r| r.request == long_request.request_id())
            .unwrap()
            .state,
        RequestState::Active
    );
    assert_eq!(
        wide.snapshot().services[0].progress.remaining.ticks(),
        u64::MAX as u128 - 3
    );
}

#[test]
fn foundation_native_causal_microsteps() {
    let mut adapter = make(vec![resource("r", 1)], vec![], 4, bounds(2, 2, 2, 0, 1, 1));
    let first = adapter.create_service(service("p1", "e1", 3)).unwrap();
    let first_request = adapter
        .admit_service(
            &first,
            admission(
                "r",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let _ = adapter.step().unwrap().unwrap();
    adapter.flush_dirty().unwrap();
    let _ = adapter.step().unwrap().unwrap();
    let marker = adapter
        .schedule_marker("decision", 0, ParentPhase::Decision)
        .unwrap();
    let marker_priority = adapter.peek_next().unwrap().priority;
    let decision = adapter.step().unwrap().unwrap();
    assert_eq!(decision.native.event, marker);
    assert_eq!(marker_priority, ParentPhase::Decision.priority());
    adapter.release_active(&first_request).unwrap();
    let release_priority = adapter.peek_next().unwrap().priority;
    let release = adapter.step().unwrap().unwrap();
    assert_eq!(release_priority, 0);
    assert!(release
        .native
        .records
        .iter()
        .any(|r| r.request == first_request.request_id()
            && r.transition == LifecycleTransition::Released));

    let second = adapter.create_service(service("p2", "e2", 1)).unwrap();
    let second_request = adapter
        .admit_service(
            &second,
            admission(
                "r",
                QueueKey::from_request(0, "b".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let submit_priority = adapter.peek_next().unwrap().priority;
    let submit = adapter.step().unwrap().unwrap();
    assert_eq!(submit_priority, 50);
    assert!(submit
        .native
        .records
        .iter()
        .any(|r| r.request == second_request.request_id()
            && r.transition == LifecycleTransition::Queued));
    adapter.flush_dirty().unwrap();
    let flush_priority = adapter.peek_next().unwrap().priority;
    let flush = adapter.step().unwrap().unwrap();
    assert_eq!(flush_priority, 60);
    assert!(flush
        .native
        .records
        .iter()
        .any(|r| r.request == second_request.request_id()
            && r.transition == LifecycleTransition::Granted));
    assert_eq!(
        [
            marker_priority,
            release_priority,
            submit_priority,
            flush_priority
        ],
        [30, 0, 50, 60]
    );

    let mut cancellations = make(vec![resource("c", 1)], vec![], 3, bounds(3, 0, 3, 0, 2, 0));
    let first = cancellations
        .admit_manual(
            "a",
            admission(
                "c",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let second = cancellations
        .admit_manual(
            "b",
            admission(
                "c",
                QueueKey::from_request(0, "b".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let third = cancellations
        .admit_manual(
            "c",
            admission(
                "c",
                QueueKey::from_request(0, "c".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let _ = cancellations.step().unwrap().unwrap();
    let _ = cancellations.step().unwrap().unwrap();
    let _ = cancellations.step().unwrap().unwrap();
    cancellations.flush_dirty().unwrap();
    let grant = cancellations.step().unwrap().unwrap();
    assert!(grant
        .native
        .records
        .iter()
        .any(|r| r.request == first.request_id() && r.transition == LifecycleTransition::Granted));
    assert_eq!(
        cancellations
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == second.request_id())
            .unwrap()
            .state,
        RequestState::Queued
    );
    assert!(cancellations
        .snapshot()
        .requests
        .iter()
        .find(|r| r.request == third.request_id())
        .unwrap()
        .lease
        .is_none());
    cancellations.cancel_waiting(&second).unwrap();
    cancellations.cancel_waiting(&third).unwrap();
    assert!(matches!(
        cancellations.flush_dirty(),
        Err(AdapterError::UnclosedCancellationBatch)
    ));
    let cancel_a = cancellations.step().unwrap().unwrap();
    assert!(cancel_a
        .native
        .records
        .iter()
        .any(|r| r.transition == LifecycleTransition::Cancelled));
    assert!(matches!(
        cancellations.flush_dirty(),
        Err(AdapterError::UnclosedCancellationBatch)
    ));
    let cancel_b = cancellations.step().unwrap().unwrap();
    assert!(cancel_b
        .native
        .records
        .iter()
        .any(|r| r.transition == LifecycleTransition::Cancelled));
    assert!(!cancellations.flush_dirty().unwrap().is_empty());
    let _ = cancellations.step().unwrap().unwrap();
    assert_eq!(
        cancellations
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == second.request_id())
            .unwrap()
            .state,
        RequestState::Cancelled
    );
    assert_eq!(
        cancellations
            .snapshot()
            .requests
            .iter()
            .find(|r| r.request == third.request_id())
            .unwrap()
            .state,
        RequestState::Cancelled
    );

    let mut zero = FlowRuntime::new();
    let native_resource = zero
        .create_resource_with_queue_ordering(
            1,
            FlowAllocationMode::Deferred,
            FlowQueueOrdering::StableKey,
        )
        .unwrap();
    let owner = zero.spawn_actor().unwrap();
    let zero_work = zero
        .create_work(owner, SimDuration::ZERO, "native-zero-control", ())
        .unwrap();
    let native_request = zero
        .acquire(native_resource)
        .owner(owner)
        .timed_work(zero_work)
        .at(SimTime::from_ticks(0))
        .scheduler_priority(50)
        .queue_order_key(FlowQueueOrderKey::new(b"zero").unwrap())
        .submit()
        .unwrap();
    let submission = zero.step().unwrap().unwrap();
    assert!(submission
        .records
        .iter()
        .any(|r| r.request == native_request && r.transition == LifecycleTransition::Queued));
    zero.schedule_allocation_flush(native_resource, SimTime::from_ticks(0), 60)
        .unwrap();
    let synchronous = zero.step().unwrap().unwrap();
    assert_eq!(
        synchronous
            .records
            .iter()
            .map(|r| r.transition)
            .collect::<Vec<_>>(),
        vec![LifecycleTransition::Granted, LifecycleTransition::Completed]
    );
    let stale = zero.step().unwrap().unwrap();
    assert!(stale.records.is_empty());
    assert!(stale.error.is_none());
    assert!(zero.step().unwrap().is_none());
}

#[test]
fn foundation_native_closed_phase_rejection_is_atomic() {
    let mut adapter = make(vec![], vec![], 3, bounds(0, 0, 0, 0, 0, 2));
    let decision_event = adapter
        .schedule_marker("closed-phase", 1, ParentPhase::Decision)
        .unwrap();
    let decision = adapter.step().unwrap().unwrap();
    assert_eq!(decision.native.event, decision_event);
    assert_eq!(decision.native.at, SimTime::from_ticks(1));
    assert!(decision.observations.iter().any(|observation| matches!(
        observation,
        FoundationObservation::Marker { operation_id, event, at }
            if operation_id == "closed-phase" && *event == decision_event && *at == SimTime::from_ticks(1)
    )));

    let before_rejection = adapter.snapshot();
    assert!(matches!(
        adapter.schedule_marker("reusable", 1, ParentPhase::CalendarExpiry),
        Err(AdapterError::ClosedPhase)
    ));
    assert_eq!(adapter.snapshot(), before_rejection);

    let accepted_event = adapter
        .schedule_marker("reusable", 1, ParentPhase::Ready)
        .unwrap();
    let after_acceptance = adapter.snapshot();
    assert_eq!(
        after_acceptance.counters.parent_markers,
        before_rejection.counters.parent_markers + 1
    );
    assert_eq!(
        after_acceptance.counters.scheduled_adapter_events,
        before_rejection.counters.scheduled_adapter_events + 1
    );
    let next = adapter.peek_next().unwrap();
    assert_eq!(next.id, accepted_event);
    assert_eq!(next.at, SimTime::from_ticks(1));
    assert_eq!(next.priority, ParentPhase::Ready.priority());
    let ready = adapter.step().unwrap().unwrap();
    assert_eq!(ready.native.event, accepted_event);
    assert!(ready.observations.iter().any(|observation| matches!(
        observation,
        FoundationObservation::Marker { operation_id, event, at }
            if operation_id == "reusable" && *event == accepted_event && *at == SimTime::from_ticks(1)
    )));
}

#[test]
fn foundation_native_bounds_and_handle_atomicity() {
    // Each admission class accepts its exact bound and rejects the first excess without mutation.
    let mut owners = make(
        vec![resource("owner", 2)],
        vec![],
        4,
        bounds(1, 1, 3, 0, 0, 0),
    );
    let manual = owners
        .admit_manual(
            "p1",
            admission(
                "owner",
                QueueKey::from_request(0, "m".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let same_owner_service = owners
        .create_service(service("p1", &format!("episode-{}", "x".repeat(70)), 1))
        .unwrap();
    assert_eq!(owners.snapshot().counters.owners, 1); // actor reused across manual and timed work.
    let before_owner_over = owners.snapshot();
    assert!(matches!(
        owners.admit_manual(
            "p2",
            admission(
                "owner",
                QueueKey::from_request(0, "new".into()).unwrap(),
                0,
                false,
                false
            )
        ),
        Err(AdapterError::BudgetExceeded("patient owners"))
    ));
    assert_eq!(owners.snapshot(), before_owner_over);
    let service_req = owners
        .admit_service(
            &same_owner_service,
            admission(
                "owner",
                QueueKey::from_request(0, "s".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let before_duplicate_service = owners.snapshot();
    assert!(matches!(
        owners.admit_service(
            &same_owner_service,
            admission(
                "owner",
                QueueKey::from_request(0, "s2".into()).unwrap(),
                0,
                false,
                false
            )
        ),
        Err(AdapterError::WrongHandleRole)
    ));
    assert_eq!(owners.snapshot(), before_duplicate_service);
    let owner_handles = (manual, service_req);
    assert_eq!(owner_handles.0.patient_id(), "p1");

    let mut services = make(vec![], vec![], 4, bounds(2, 1, 2, 0, 0, 0));
    let _ = services.create_service(service("p1", "e1", 1)).unwrap();
    let before_service_over = services.snapshot();
    assert!(matches!(
        services.create_service(service("p2", "e2", 1)),
        Err(AdapterError::BudgetExceeded("services"))
    ));
    assert_eq!(services.snapshot(), before_service_over);

    let mut requests = make(
        vec![resource("request", 2)],
        vec![],
        4,
        bounds(2, 0, 1, 0, 0, 0),
    );
    let first = requests
        .admit_manual(
            "p1",
            admission(
                "request",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let before_request_over = requests.snapshot();
    assert!(matches!(
        requests.admit_manual(
            "p2",
            admission(
                "request",
                QueueKey::from_request(0, "b".into()).unwrap(),
                0,
                false,
                false
            )
        ),
        Err(AdapterError::BudgetExceeded("requests"))
    ));
    assert_eq!(requests.snapshot(), before_request_over);
    assert_eq!(first.request_id(), requests.snapshot().requests[0].request);

    let mut preempts = make(vec![resource("k", 2)], vec![], 4, bounds(2, 2, 2, 1, 0, 0));
    let p1 = preempts.create_service(service("p1", "e1", 1)).unwrap();
    let p2 = preempts.create_service(service("p2", "e2", 1)).unwrap();
    let first_preempt = preempts
        .admit_service(
            &p1,
            admission(
                "k",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                true,
                false,
            ),
        )
        .unwrap();
    let before_preempt_over = preempts.snapshot();
    assert!(matches!(
        preempts.admit_service(
            &p2,
            admission(
                "k",
                QueueKey::from_request(0, "b".into()).unwrap(),
                0,
                true,
                false
            )
        ),
        Err(AdapterError::BudgetExceeded("preempt-capable requests"))
    ));
    assert_eq!(preempts.snapshot(), before_preempt_over);
    assert_eq!(
        first_preempt.request_id(),
        preempts.snapshot().requests[0].request
    );

    let mut mutations = make(
        vec![resource("mutation", 2)],
        vec![],
        4,
        bounds(2, 0, 2, 0, 1, 0),
    );
    let m1 = mutations
        .admit_manual(
            "p1",
            admission(
                "mutation",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let m2 = mutations
        .admit_manual(
            "p2",
            admission(
                "mutation",
                QueueKey::from_request(0, "b".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let _ = mutations.step().unwrap().unwrap();
    let _ = mutations.step().unwrap().unwrap();
    mutations.flush_dirty().unwrap();
    let _ = mutations.step().unwrap().unwrap();
    mutations.release_active(&m1).unwrap();
    let before_mutation_over = mutations.snapshot();
    assert!(matches!(
        mutations.release_active(&m2),
        Err(AdapterError::BudgetExceeded("releases or cancels"))
    ));
    assert_eq!(mutations.snapshot(), before_mutation_over);
    assert!(matches!(
        mutations.cancel_waiting(&m2),
        Err(AdapterError::WrongNativeState)
    ));
    assert_eq!(mutations.snapshot(), before_mutation_over);

    let mut markers = make(vec![], vec![], 4, bounds(0, 0, 0, 0, 0, 1));
    let empty = markers
        .schedule_marker("empty", 1, ParentPhase::Ready)
        .unwrap();
    let before_marker_over = markers.snapshot();
    assert!(matches!(
        markers.schedule_marker("over", 1, ParentPhase::Ready),
        Err(AdapterError::BudgetExceeded("parent markers"))
    ));
    assert_eq!(markers.snapshot(), before_marker_over);
    let marker_dispatch = loop {
        let d = markers.step().unwrap().unwrap();
        if d.native.event == empty {
            break d;
        }
    };
    assert!(marker_dispatch.native.records.is_empty());
    assert!(marker_dispatch.observations.iter().any(|o| matches!(o, FoundationObservation::Marker { operation_id, .. } if operation_id == "empty")));

    // Checked representational/aggregate arithmetic rejects overflow in each bound dimension.
    for b in [
        bounds(u64::MAX, 0, 0, 0, 0, 0),
        bounds(0, u64::MAX, u64::MAX, 0, 0, 0),
        bounds(0, 0, u64::MAX, 0, 0, 0),
        bounds(0, u64::MAX, u64::MAX, u64::MAX, 0, 0),
        bounds(0, 0, 0, 0, u64::MAX, 0),
        bounds(0, 0, 0, 0, 0, u64::MAX),
    ] {
        assert!(matches!(
            AdapterPlan::validate(vec![], vec![], 1, b),
            Err(AdapterError::BudgetExceeded(_))
        ));
    }

    let duplicate_resources = AdapterPlan::validate(
        vec![resource("r", 1), resource("r", 1)],
        vec![],
        3,
        bounds(0, 0, 0, 0, 0, 0),
    );
    assert!(matches!(
        duplicate_resources,
        Err(AdapterError::DuplicateIdentity)
    ));
    let conflicting_boundaries = AdapterPlan::validate(
        vec![resource("r", 1)],
        vec![
            GrantBoundary {
                resource_id: "r".into(),
                at_tick: 2,
                enabled: false,
            },
            GrantBoundary {
                resource_id: "r".into(),
                at_tick: 2,
                enabled: true,
            },
        ],
        3,
        bounds(0, 0, 0, 0, 0, 0),
    );
    assert!(matches!(
        conflicting_boundaries,
        Err(AdapterError::InvalidInput(_))
    ));

    // Foreign-runtime rejection is atomic on the receiver; the original adapter is not the receiver.
    let mut local = make(
        vec![resource("local", 1)],
        vec![],
        5,
        bounds(1, 1, 1, 0, 0, 1),
    );
    let local_service = local
        .create_service(service("p1", "episode-id-longer-than-patient-limit", 1))
        .unwrap();
    let foreign = make(
        vec![resource("local", 1)],
        vec![],
        5,
        bounds(1, 1, 1, 0, 0, 0),
    );
    let mut foreign = foreign;
    let foreign_before = foreign.snapshot();
    assert!(matches!(
        foreign.admit_service(
            &local_service,
            admission(
                "local",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false
            )
        ),
        Err(AdapterError::ForeignHandle)
    ));
    assert_eq!(foreign.snapshot(), foreign_before);
    let local_request = local
        .admit_service(
            &local_service,
            admission(
                "local",
                QueueKey::from_request(0, "a".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    assert_eq!(
        local_request.request_id(),
        local.snapshot().requests[0].request
    );
    assert!(local.schedule_marker("m", 0, ParentPhase::Ready).is_ok());
    let before_duplicate_marker = local.snapshot();
    assert!(matches!(
        local.schedule_marker("m", 0, ParentPhase::Ready),
        Err(AdapterError::DuplicateOperation)
    ));
    assert_eq!(local.snapshot(), before_duplicate_marker);
}
