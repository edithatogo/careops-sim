use careops_ed::flow_adapter::{
    AdapterError, AdapterPlan, Admission, EdFlowAdapter, FoundationBounds, FrozenService,
    GrantBoundary, QueueKey, ResourceSpec, SampleOrigin, ServiceIdentity,
};
use kairo_ecs_des::{LifecycleTransition, RequestState, WorkState};
use kairo_ecs_types::EventId;

fn bounds(owners: u64, requests: u64, mutations: u64) -> FoundationBounds {
    FoundationBounds {
        patient_owners: owners,
        services: requests,
        requests,
        preempt_capable_requests: 0,
        releases_or_cancels: mutations,
        parent_markers: 0,
    }
}

fn resource(id: &str, capacity: u32) -> ResourceSpec {
    ResourceSpec {
        id: id.into(),
        capacity,
    }
}

fn make(resources: Vec<ResourceSpec>, request_count: u64, mutations: u64) -> EdFlowAdapter {
    EdFlowAdapter::new(
        AdapterPlan::validate(
            resources,
            Vec::<GrantBoundary>::new(),
            4,
            bounds(request_count, request_count, mutations),
        )
        .unwrap(),
    )
    .unwrap()
}

fn admission(resource_id: &str, patient_id: &str) -> Admission {
    Admission {
        resource_id: resource_id.into(),
        key: QueueKey::from_request(0, patient_id.into()).unwrap(),
        urgency: 0,
        can_preempt: false,
        preemptible: false,
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

fn dispatch_through(
    adapter: &mut EdFlowAdapter,
    event: EventId,
) -> Vec<careops_ed::flow_adapter::AdapterDispatch> {
    let mut dispatches = Vec::new();
    loop {
        let dispatch = adapter.step().unwrap().unwrap();
        let done = dispatch.native.event == event;
        dispatches.push(dispatch);
        if done {
            return dispatches;
        }
    }
}

fn service_progress(
    adapter: &EdFlowAdapter,
    identity: &ServiceIdentity,
) -> kairo_ecs_des::WorkProgress {
    adapter
        .snapshot()
        .services
        .into_iter()
        .find(|service| &service.identity == identity)
        .unwrap()
        .progress
}

#[test]
fn selected_flush_only_selected_dirty_resources() {
    let mut adapter = make(vec![resource("space", 1), resource("care", 1)], 2, 0);
    let space = adapter
        .admit_manual("space-patient", admission("space", "space-patient"))
        .unwrap();
    let care = adapter
        .admit_manual("care-patient", admission("care", "care-patient"))
        .unwrap();
    let scheduled = adapter.flush_dirty_selected(&["space"]).unwrap();
    assert_eq!(scheduled.len(), 1);
    let dispatches = dispatch_through(&mut adapter, scheduled[0]);
    let grant = dispatches.last().unwrap();
    assert!(grant
        .native
        .records
        .iter()
        .any(|record| record.request == space.request_id()
            && record.transition == LifecycleTransition::Granted));
    assert!(grant
        .native
        .records
        .iter()
        .all(|record| record.request != care.request_id()
            || record.transition != LifecycleTransition::Granted));
    assert_eq!(
        adapter
            .snapshot()
            .requests
            .iter()
            .find(|request| request.request == care.request_id())
            .unwrap()
            .state,
        RequestState::Queued
    );
    let care_flush = adapter.flush_dirty_selected(&["care"]).unwrap();
    assert_eq!(care_flush.len(), 1);
    assert!(dispatch_through(&mut adapter, care_flush[0])
        .iter()
        .any(|dispatch| dispatch
            .native
            .records
            .iter()
            .any(|record| record.request == care.request_id()
                && record.transition == LifecycleTransition::Granted)));
}

#[test]
fn selected_flush_pending_and_wrapper_parity() {
    let resources = vec![resource("room", 2)];
    let mut selected = make(resources.clone(), 2, 0);
    let mut wrapper = make(resources, 2, 0);
    let selected_request = selected
        .admit_manual("patient", admission("room", "patient"))
        .unwrap();
    let wrapper_request = wrapper
        .admit_manual("patient", admission("room", "patient"))
        .unwrap();
    let selected_ids = selected.flush_dirty_selected(&["room"]).unwrap();
    assert_eq!(selected_ids.len(), 1);
    assert!(selected.flush_dirty_selected(&["room"]).unwrap().is_empty());
    let wrapper_ids = wrapper.flush_dirty().unwrap();
    assert_eq!(selected_ids, wrapper_ids);
    let selected_dispatches = dispatch_through(&mut selected, selected_ids[0]);
    let wrapper_dispatches = dispatch_through(&mut wrapper, wrapper_ids[0]);
    assert_eq!(selected_dispatches, wrapper_dispatches);
    assert!(selected_dispatches.iter().any(|dispatch| dispatch
        .native
        .records
        .iter()
        .any(|record| record.request == selected_request.request_id()
            && record.transition == LifecycleTransition::Granted)));
    assert!(wrapper_dispatches.iter().any(|dispatch| dispatch
        .native
        .records
        .iter()
        .any(|record| record.request == wrapper_request.request_id()
            && record.transition == LifecycleTransition::Granted)));
    assert_eq!(selected.snapshot(), wrapper.snapshot());
    let next = selected
        .admit_manual("next-patient", admission("room", "next-patient"))
        .unwrap();
    let next_flush = selected.flush_dirty_selected(&["room"]).unwrap();
    assert_eq!(next_flush.len(), 1);
    assert!(dispatch_through(&mut selected, next_flush[0])
        .iter()
        .any(|dispatch| dispatch
            .native
            .records
            .iter()
            .any(|record| record.request == next.request_id()
                && record.transition == LifecycleTransition::Granted)));
}

#[test]
fn selected_flush_space_offer_care_barrier() {
    let mut with_offer = make(vec![resource("space", 1), resource("care", 1)], 2, 1);
    let space = with_offer
        .admit_manual("space-patient", admission("space", "space-patient"))
        .unwrap();
    let care_service = with_offer
        .create_service(service("care-patient", "care-episode", 2))
        .unwrap();
    let care_identity = care_service.identity().clone();
    let care = with_offer
        .admit_service(&care_service, admission("care", "care-patient"))
        .unwrap();
    assert_eq!(
        service_progress(&with_offer, &care_identity).state,
        WorkState::Pending
    );

    let space_flush = with_offer.flush_dirty_selected(&["space"]).unwrap();
    let space_dispatches = dispatch_through(&mut with_offer, space_flush[0]);
    assert!(space_dispatches.iter().any(|dispatch| {
        dispatch.native.records.iter().any(|record| {
            record.request == space.request_id() && record.transition == LifecycleTransition::Granted
        }) && dispatch.observations.iter().any(|observation| matches!(
            observation,
            careops_ed::flow_adapter::FoundationObservation::Manual(manual)
                if manual.request == space.request_id() && manual.transition == LifecycleTransition::Granted
        ))
    }));
    assert_eq!(
        with_offer
            .snapshot()
            .requests
            .iter()
            .find(|request| request.request == space.request_id())
            .unwrap()
            .state,
        RequestState::Active
    );
    assert_eq!(
        service_progress(&with_offer, &care_identity).state,
        WorkState::Pending
    );

    // This represents the offer decision after observing space grant, through the adapter cancellation seam.
    with_offer.cancel_waiting(&care).unwrap();
    assert!(matches!(
        with_offer.flush_dirty_selected(&["care"]),
        Err(AdapterError::UnclosedCancellationBatch)
    ));
    let mut all_dispatches = space_dispatches.clone();
    loop {
        let dispatch = with_offer.step().unwrap().unwrap();
        let cancelled = dispatch.native.records.iter().any(|record| {
            record.request == care.request_id()
                && record.transition == LifecycleTransition::Cancelled
        });
        assert!(dispatch
            .native
            .records
            .iter()
            .all(|record| record.request != care.request_id()
                || !matches!(
                    record.transition,
                    LifecycleTransition::Granted
                        | LifecycleTransition::Completed
                        | LifecycleTransition::Resumed
                        | LifecycleTransition::Restarted
                )));
        all_dispatches.push(dispatch);
        if cancelled {
            break;
        }
    }
    assert_eq!(
        service_progress(&with_offer, &care_identity).state,
        WorkState::Cancelled
    );
    let cancelled_progress = service_progress(&with_offer, &care_identity);
    assert_eq!(cancelled_progress.useful_elapsed.ticks(), 0);
    assert_eq!(cancelled_progress.cumulative_busy.ticks(), 0);
    assert_eq!(cancelled_progress.remaining.ticks(), 2);
    assert_eq!(cancelled_progress.segment_started_at, None);
    assert_eq!(cancelled_progress.completion_at, None);
    let care_flush = with_offer.flush_dirty_selected(&["care"]).unwrap();
    assert_eq!(care_flush.len(), 1);
    for dispatch in dispatch_through(&mut with_offer, care_flush[0]) {
        assert!(dispatch
            .native
            .records
            .iter()
            .all(|record| record.request != care.request_id()
                || !matches!(
                    record.transition,
                    LifecycleTransition::Granted
                        | LifecycleTransition::Completed
                        | LifecycleTransition::Resumed
                        | LifecycleTransition::Restarted
                )));
        all_dispatches.push(dispatch);
    }
    while let Some(dispatch) = with_offer.step().unwrap() {
        assert!(dispatch
            .native
            .records
            .iter()
            .all(|record| record.request != care.request_id()
                || !matches!(
                    record.transition,
                    LifecycleTransition::Granted
                        | LifecycleTransition::Completed
                        | LifecycleTransition::Resumed
                        | LifecycleTransition::Restarted
                )));
        all_dispatches.push(dispatch);
    }
    assert!(!all_dispatches.is_empty());
    assert!(all_dispatches
        .iter()
        .all(|dispatch| dispatch.native.records.iter().all(|record| {
            record.request != care.request_id()
                || !matches!(
                    record.transition,
                    LifecycleTransition::Granted
                        | LifecycleTransition::Completed
                        | LifecycleTransition::Resumed
                        | LifecycleTransition::Restarted
                )
        })));
    assert_eq!(
        service_progress(&with_offer, &care_identity).state,
        WorkState::Cancelled
    );
    let final_cancelled_progress = service_progress(&with_offer, &care_identity);
    assert_eq!(final_cancelled_progress.useful_elapsed.ticks(), 0);
    assert_eq!(final_cancelled_progress.cumulative_busy.ticks(), 0);
    assert_eq!(final_cancelled_progress.remaining.ticks(), 2);
    assert_eq!(final_cancelled_progress.segment_started_at, None);
    assert_eq!(final_cancelled_progress.completion_at, None);

    let mut no_offer = make(vec![resource("space", 1), resource("care", 1)], 2, 0);
    let no_offer_space = no_offer
        .admit_manual("space-patient", admission("space", "space-patient"))
        .unwrap();
    let no_offer_care_service = no_offer
        .create_service(service("care-patient", "care-episode", 2))
        .unwrap();
    let no_offer_care = no_offer
        .admit_service(&no_offer_care_service, admission("care", "care-patient"))
        .unwrap();
    let no_offer_care_identity = no_offer_care_service.identity().clone();
    assert_eq!(
        service_progress(&no_offer, &no_offer_care_identity).state,
        WorkState::Pending
    );
    let space_flush = no_offer.flush_dirty_selected(&["space"]).unwrap();
    let no_offer_space_dispatches = dispatch_through(&mut no_offer, space_flush[0]);
    assert!(no_offer_space_dispatches.iter().any(|dispatch| {
        dispatch.native.records.iter().any(|record| {
            record.request == no_offer_space.request_id()
                && record.transition == LifecycleTransition::Granted
        }) && dispatch.observations.iter().any(|observation| matches!(
            observation,
            careops_ed::flow_adapter::FoundationObservation::Manual(manual)
                if manual.request == no_offer_space.request_id() && manual.transition == LifecycleTransition::Granted
        ))
    }));
    assert_eq!(
        no_offer
            .snapshot()
            .requests
            .iter()
            .find(|request| request.request == no_offer_space.request_id())
            .unwrap()
            .state,
        RequestState::Active
    );
    assert_eq!(
        service_progress(&no_offer, &no_offer_care_identity).state,
        WorkState::Pending
    );
    let care_flush = no_offer.flush_dirty_selected(&["care"]).unwrap();
    let care_dispatches = dispatch_through(&mut no_offer, care_flush[0]);
    assert!(care_dispatches.iter().any(|dispatch| dispatch
        .native
        .records
        .iter()
        .any(|record| record.request == no_offer_care.request_id()
            && record.transition == LifecycleTransition::Granted)));
    assert_eq!(
        service_progress(&no_offer, &no_offer_care_identity).state,
        WorkState::Active
    );
    let horizon_dispatches = std::iter::from_fn(|| no_offer.step().unwrap()).collect::<Vec<_>>();
    assert!(horizon_dispatches.iter().any(|dispatch| dispatch
        .native
        .records
        .iter()
        .any(|record| record.request == no_offer_care.request_id()
            && record.transition == LifecycleTransition::Completed)));
    let progressed = service_progress(&no_offer, &no_offer_care_identity);
    assert_eq!(progressed.state, WorkState::Completed);
    assert!(progressed.useful_elapsed.ticks() > 0);
    assert!(progressed.cumulative_busy.ticks() > 0);
}

#[test]
fn selected_flush_cancel_batch_and_budget() {
    let mut adapter = make(
        vec![
            resource("cancelled", 1),
            resource("work", 1),
            resource("clean", 1),
        ],
        2,
        1,
    );
    let cancelled = adapter
        .admit_manual(
            "cancelled-patient",
            admission("cancelled", "cancelled-patient"),
        )
        .unwrap();
    let work = adapter
        .admit_manual("work-patient", admission("work", "work-patient"))
        .unwrap();
    let expected_queued = [cancelled.request_id(), work.request_id()]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let mut queued = std::collections::BTreeSet::new();
    let mut submit_dispatches = Vec::new();
    while queued != expected_queued {
        let dispatch = adapter.step().unwrap().unwrap();
        for record in &dispatch.native.records {
            if expected_queued.contains(&record.request)
                && record.transition == LifecycleTransition::Queued
            {
                queued.insert(record.request);
            }
            assert!(
                !expected_queued.contains(&record.request)
                    || record.transition != LifecycleTransition::Granted
            );
        }
        submit_dispatches.push(dispatch);
    }
    assert_eq!(queued, expected_queued);
    for request in &expected_queued {
        assert_eq!(
            adapter
                .snapshot()
                .requests
                .iter()
                .find(|snapshot| snapshot.request == *request)
                .unwrap()
                .state,
            RequestState::Queued
        );
    }
    assert!(submit_dispatches.iter().all(|dispatch| dispatch
        .native
        .records
        .iter()
        .all(|record| !expected_queued.contains(&record.request)
            || record.transition != LifecycleTransition::Granted)));
    assert_eq!(adapter.snapshot().counters.flushes, 0);
    adapter.cancel_waiting(&cancelled).unwrap();
    let before_rejection = adapter.snapshot();
    assert!(matches!(
        adapter.flush_dirty_selected(&["work"]),
        Err(AdapterError::UnclosedCancellationBatch)
    ));
    assert_eq!(adapter.snapshot(), before_rejection);
    let mut cancellation_consumed = false;
    while !cancellation_consumed {
        let dispatch = adapter.step().unwrap().unwrap();
        cancellation_consumed = dispatch.native.records.iter().any(|record| {
            record.request == cancelled.request_id()
                && record.transition == LifecycleTransition::Cancelled
        });
    }
    let before_noops = adapter.snapshot();
    assert!(adapter.flush_dirty_selected(&[]).unwrap().is_empty());
    assert!(adapter.flush_dirty_selected(&["clean"]).unwrap().is_empty());
    assert_eq!(adapter.snapshot(), before_noops);
    let snapshot_before = adapter.snapshot();
    let scheduled = adapter.flush_dirty_selected(&["work"]).unwrap();
    assert_eq!(scheduled.len(), 1);
    let after = adapter.snapshot();
    assert_eq!(
        after.native_budget.scheduler.scheduled_events,
        snapshot_before.native_budget.scheduler.scheduled_events + 1
    );
    assert_eq!(after.counters.flushes, snapshot_before.counters.flushes + 1);
    assert_eq!(
        after.counters.scheduled_adapter_events,
        snapshot_before.counters.scheduled_adapter_events + 1
    );
    assert!(dispatch_through(&mut adapter, scheduled[0])
        .iter()
        .any(|dispatch| dispatch
            .native
            .records
            .iter()
            .any(|record| record.transition == LifecycleTransition::Granted)));
}
