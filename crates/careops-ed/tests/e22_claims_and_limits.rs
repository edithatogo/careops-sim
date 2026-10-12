use careops_ed::flow_adapter::{
    AdapterError, AdapterPlan, Admission, EdFlowAdapter, FoundationBounds, FrozenService,
    GrantBoundary, QueueKey, ResourceSpec, SampleOrigin, ServiceIdentity,
};
use kairo_ecs_des::{LifecycleTransition, RequestState};
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

/// E2.2.claims oracle:
/// Single-unit/resource limitation is explicit: each work item or admission binds to exactly
/// one resource pool. Attempting to acquire space and staff in an unpartitioned atomic call
/// is prohibited; each must be explicitly staged.
#[test]
fn claims_single_resource_per_work_item_invariant() {
    let mut adapter = make(
        vec![resource("bed.space", 1), resource("nurse.triage", 1)],
        2,
        0,
    );

    // Bed and nurse are distinct single-unit resources.
    let bed_adm = admission("bed.space", "patient-1");
    let nurse_adm = admission("nurse.triage", "patient-1");

    assert_eq!(bed_adm.resource_id, "bed.space");
    assert_eq!(nurse_adm.resource_id, "nurse.triage");

    let bed_handle = adapter.admit_manual("patient-1", bed_adm).unwrap();
    let nurse_handle = adapter.admit_manual("patient-1", nurse_adm).unwrap();

    // Requests are partitioned into distinct native queues.
    assert_ne!(bed_handle.request_id(), nurse_handle.request_id());

    // Flush bed first: bed grants, nurse remains queued.
    let bed_flush = adapter.flush_dirty_selected(&["bed.space"]).unwrap();
    let bed_dispatches = dispatch_through(&mut adapter, bed_flush[0]);
    assert!(bed_dispatches.iter().any(|d| {
        d.native.records.iter().any(|r| {
            r.request == bed_handle.request_id() && r.transition == LifecycleTransition::Granted
        })
    }));

    // Verify nurse is still queued (not granted implicitly).
    let nurse_state = adapter
        .snapshot()
        .requests
        .into_iter()
        .find(|r| r.request == nurse_handle.request_id())
        .unwrap()
        .state;
    assert_eq!(nurse_state, RequestState::Queued);
}

/// E2.2.claims oracle:
/// Staged staff and space ownership lifecycle: space is held across multiple clinical stages,
/// and clinical staff tasks complete and release independently without dropping space occupancy.
#[test]
fn staged_space_and_staff_sequential_coordination() {
    let mut adapter = make(
        vec![
            resource("cubicle.space", 1),
            resource("doctor.consult", 1),
            resource("nurse.discharge", 1),
        ],
        3,
        2,
    );

    // Stage 1: Space is acquired.
    let space_handle = adapter
        .admit_manual("patient-seq", admission("cubicle.space", "patient-seq"))
        .unwrap();
    let space_flush = adapter.flush_dirty_selected(&["cubicle.space"]).unwrap();
    dispatch_through(&mut adapter, space_flush[0]);

    // Stage 2: Doctor consult work item is created and admitted while space is held.
    let doc_srv = adapter
        .create_service(service("patient-seq", "ep-consult", 5))
        .unwrap();
    let doc_handle = adapter
        .admit_service(&doc_srv, admission("doctor.consult", "patient-seq"))
        .unwrap();
    let doc_flush = adapter.flush_dirty_selected(&["doctor.consult"]).unwrap();
    dispatch_through(&mut adapter, doc_flush[0]);

    // Space remains active while doctor work progresses.
    let space_status = adapter
        .snapshot()
        .requests
        .into_iter()
        .find(|r| r.request == space_handle.request_id())
        .unwrap()
        .state;
    assert_eq!(space_status, RequestState::Active);

    // Release doctor upon consultation finish. Space is NOT released.
    adapter.release_active(&doc_handle).unwrap();
    assert_eq!(
        adapter
            .snapshot()
            .requests
            .into_iter()
            .find(|r| r.request == space_handle.request_id())
            .unwrap()
            .state,
        RequestState::Active
    );
}

/// E2.2.limits oracle:
/// Multi-role procedures (sedation, resuscitation, assisted transfer) requiring concurrent multi-resource
/// attention without dedicated composite representation fail closed.
#[test]
fn limits_multi_role_concurrent_procedure_rejection() {
    let mut adapter = make(vec![resource("resus.bay", 1)], 1, 0);

    // Attempting to admit against an unsupported non-composite identifier for multi-person resus
    // fails closed: the adapter enforces known validated resource identity.
    let invalid_composite_adm = admission("resus.team.unsupported.multi_role", "patient-resus");
    let result = adapter.admit_manual("patient-resus", invalid_composite_adm);

    assert!(matches!(result, Err(AdapterError::UnknownResource)));
}

/// E2.2.limits oracle:
/// Dedicated composite approximation (e.g. reviewed single-token `team.sedation_pair` or `team.resus_composite`)
/// is the approved approximation for multi-role procedures under ST007/ST008.
#[test]
fn limits_dedicated_composite_approximation_allowed() {
    let mut adapter = make(
        vec![
            resource("space.resus_bay", 1),
            resource("team.resus_composite", 1),
        ],
        2,
        0,
    );

    let space_adm = admission("space.resus_bay", "patient-resus");
    let team_adm = admission("team.resus_composite", "patient-resus");

    let space_handle = adapter.admit_manual("patient-resus", space_adm).unwrap();
    let team_handle = adapter.admit_manual("patient-resus", team_adm).unwrap();

    let flush = adapter
        .flush_dirty_selected(&["space.resus_bay", "team.resus_composite"])
        .unwrap();
    for event in flush {
        dispatch_through(&mut adapter, event);
    }

    let snapshot = adapter.snapshot();
    let space_granted = snapshot
        .requests
        .iter()
        .any(|r| r.request == space_handle.request_id() && r.state == RequestState::Active);
    let team_granted = snapshot
        .requests
        .iter()
        .any(|r| r.request == team_handle.request_id() && r.state == RequestState::Active);

    assert!(space_granted);
    assert!(team_granted);
}
