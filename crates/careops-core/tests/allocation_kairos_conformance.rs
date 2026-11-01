use careops_core::allocation::{
    EngineRequestId, EngineResourceId, GrantDisposition, GrantReceipt, LeaseId as CareOpsLeaseId,
    ResourceId as CareOpsResourceId, StagedAllocator,
};
use kairo_ecs_des::{FlowRuntime, RequestId, RequestState, ResourceId};
use kairo_ecs_types::SimTime;

fn engine_request_id(request: RequestId) -> EngineRequestId {
    let id = request.entity_id();
    EngineRequestId::new(id.index, id.generation)
}

fn engine_resource_id(resource: ResourceId) -> EngineResourceId {
    let id = resource.entity_id();
    EngineResourceId::new(id.index, id.generation)
}

fn record_observed_grant(
    allocator: &mut StagedAllocator,
    local_request: careops_core::allocation::ClaimRequest,
    engine: &FlowRuntime,
    request: RequestId,
    resource: ResourceId,
) {
    let observed = engine.request(request).unwrap();
    assert_eq!(observed.state, RequestState::Active);
    let lease = observed.lease.expect("active Kairos request has a lease");
    let request_identity = engine_request_id(request);
    let resource_identity = engine_resource_id(resource);
    assert_eq!(lease.request_id(), request);
    allocator
        .bind_engine_lease_revision(local_request, request_identity, lease.revision())
        .unwrap();
    let receipt = GrantReceipt {
        claim_request: local_request,
        careops_resource: local_request.resource(),
        engine_resource: resource_identity,
        lease: CareOpsLeaseId::new(request_identity, lease.revision()),
    };
    assert_eq!(
        allocator.grant(receipt, u64::try_from(engine.now().ticks()).unwrap()),
        Ok(GrantDisposition::Accepted)
    );
}

#[test]
fn capacity_one_contention_keeps_one_active_lease_and_grants_queued_work_after_release() {
    let mut engine = FlowRuntime::new();
    let first_owner = engine.spawn_actor().unwrap();
    let second_owner = engine.spawn_actor().unwrap();
    let engine_resource = engine.create_resource(1).unwrap();
    let first_engine_request = engine
        .acquire(engine_resource)
        .owner(first_owner)
        .at(SimTime::from_ticks(1))
        .submit()
        .unwrap();
    let second_engine_request = engine
        .acquire(engine_resource)
        .owner(second_owner)
        .at(SimTime::from_ticks(1))
        .submit()
        .unwrap();

    let mut careops = StagedAllocator::default();
    let careops_resource = CareOpsResourceId::new(77);
    let first_operation = careops.submit([careops_resource], None, 1, false).unwrap();
    let second_operation = careops.submit([careops_resource], None, 1, false).unwrap();
    let issued = careops.next_claim_requests().unwrap();
    let first_claim = issued[0];
    let second_claim = issued[1];

    // Bind local claim tokens to the actual Kairos requests before dispatch.
    let request_identity = engine_request_id(first_engine_request);
    let resource_identity = engine_resource_id(engine_resource);
    careops
        .bind_engine_request(first_claim, request_identity, resource_identity)
        .unwrap();
    careops
        .bind_engine_request(
            second_claim,
            engine_request_id(second_engine_request),
            resource_identity,
        )
        .unwrap();

    engine.run_for(8).unwrap();
    let snapshot = engine.resource(engine_resource).unwrap();
    assert_eq!(snapshot.total, 1);
    assert_eq!(snapshot.available, 0);
    assert_eq!(snapshot.active.len(), 1);
    assert_eq!(snapshot.queued, vec![second_engine_request]);
    assert_eq!(
        engine.request(first_engine_request).unwrap().state,
        RequestState::Active
    );
    assert_eq!(
        engine.request(second_engine_request).unwrap().state,
        RequestState::Queued
    );

    let first_observed = engine.request(first_engine_request).unwrap();
    let first_engine_lease = first_observed.lease.unwrap();
    careops
        .bind_engine_lease_revision(first_claim, request_identity, first_engine_lease.revision())
        .unwrap();
    let first_receipt = GrantReceipt {
        claim_request: first_claim,
        careops_resource,
        engine_resource: resource_identity,
        lease: CareOpsLeaseId::new(request_identity, first_engine_lease.revision()),
    };
    assert_eq!(
        careops.grant(first_receipt, 1),
        Ok(GrantDisposition::Accepted)
    );
    assert_eq!(
        careops.state(first_operation).unwrap(),
        careops_core::allocation::AllocationState::Ready
    );
    assert_eq!(
        careops.state(second_operation).unwrap(),
        careops_core::allocation::AllocationState::Acquiring
    );

    let first_work_revision = careops.start(first_operation).unwrap();
    careops
        .complete(first_operation, first_work_revision)
        .unwrap();
    assert_eq!(
        careops.releases_due(first_operation),
        vec![CareOpsLeaseId::new(
            request_identity,
            first_engine_lease.revision()
        )]
    );

    engine.release(first_engine_lease, engine.now()).unwrap();
    engine.run_for(8).unwrap();
    assert_eq!(
        engine.request(first_engine_request).unwrap().state,
        RequestState::Released
    );
    careops
        .ack_release(
            first_operation,
            CareOpsLeaseId::new(request_identity, first_engine_lease.revision()),
        )
        .unwrap();
    assert_eq!(careops.claim_count(), 0);
    assert_eq!(
        careops.state(first_operation).unwrap(),
        careops_core::allocation::AllocationState::Completed
    );
    let snapshot_after_release = engine.resource(engine_resource).unwrap();
    assert_eq!(snapshot_after_release.active.len(), 1);
    assert_eq!(snapshot_after_release.queued, Vec::new());
    assert_eq!(
        engine.request(second_engine_request).unwrap().state,
        RequestState::Active
    );
    record_observed_grant(
        &mut careops,
        second_claim,
        &engine,
        second_engine_request,
        engine_resource,
    );
    assert_eq!(
        careops.state(second_operation).unwrap(),
        careops_core::allocation::AllocationState::Ready
    );
}
