use careops_core::allocation::{
    AllocationError, AllocationState, ClaimRequest, EngineRequestId, EngineResourceId,
    GrantDisposition, GrantReceipt, LeaseId, ResourceId, StagedAllocator,
};

fn setup() -> (StagedAllocator, ResourceId, ResourceId) {
    (
        StagedAllocator::default(),
        ResourceId::new(10),
        ResourceId::new(20),
    )
}

fn lease(value: u64) -> LeaseId {
    LeaseId::new(EngineRequestId::new(value, 0), 0)
}

fn grant(
    allocator: &mut StagedAllocator,
    request: ClaimRequest,
    engine_request: u64,
    at: u64,
) -> GrantDisposition {
    let request_id = EngineRequestId::new(engine_request, 0);
    let engine_resource = EngineResourceId::new(request.resource().value(), 0);
    if allocator
        .bind_engine_request(request, request_id, engine_resource)
        .is_ok()
    {
        let _ = allocator.bind_engine_lease_revision(request, request_id, 0);
    }
    allocator
        .grant(
            GrantReceipt {
                claim_request: request,
                careops_resource: request.resource(),
                engine_resource,
                lease: LeaseId::new(request_id, 0),
            },
            at,
        )
        .unwrap()
}

#[test]
fn claims_are_issued_in_stable_resource_order_and_fifo_operation_order() {
    let (mut allocator, a, b) = setup();
    let first = allocator.submit([b, a], None, 5, true).unwrap();
    let second = allocator.submit([a], None, 3, true).unwrap();

    let issued = allocator.next_claim_requests().unwrap();
    assert_eq!(issued.len(), 2);
    assert_eq!(issued[0].operation_id(), first);
    assert_eq!(issued[0].resource(), a);
    assert_eq!(issued[1].operation_id(), second);
    assert_eq!(issued[1].resource(), a);

    let again = allocator.next_claim_requests().unwrap();
    assert_eq!(issued, again, "re-dispatch must be idempotent");
}

#[test]
fn partial_failure_retains_claim_until_release_acknowledgement() {
    let (mut allocator, a, b) = setup();
    let operation = allocator.submit([a, b], None, 5, true).unwrap();
    let first = allocator.next_claim_requests().unwrap()[0];
    grant(&mut allocator, first, 100, 1);
    let second = allocator.next_claim_requests().unwrap()[0];
    allocator.reject(second, 2).unwrap();

    assert_eq!(
        allocator.state(operation),
        Some(AllocationState::RollingBack)
    );
    assert_eq!(allocator.releases_due(operation), vec![lease(100)]);
    allocator.release_failed(operation, lease(100)).unwrap();
    assert_eq!(allocator.releases_due(operation), vec![lease(100)]);
    allocator.ack_release(operation, lease(100)).unwrap();
    assert_eq!(allocator.state(operation), Some(AllocationState::Failed));
    assert_eq!(allocator.claim_count(), 0);
}

#[test]
fn deadline_at_grant_tie_times_out_and_late_lease_is_explicitly_released() {
    let (mut allocator, a, _) = setup();
    let operation = allocator.submit([a], Some(5), 5, true).unwrap();
    let token = allocator.next_claim_requests().unwrap()[0];
    assert_eq!(
        grant(&mut allocator, token, 200, 5),
        GrantDisposition::ReleaseLateGrant
    );
    assert_eq!(allocator.state(operation), Some(AllocationState::TimedOut));
    assert_eq!(allocator.orphan_releases_due(), vec![lease(200)]);
    allocator.ack_orphan_release(lease(200)).unwrap();
    assert_eq!(allocator.claim_count(), 0);
}

#[test]
fn cancellation_is_idempotent_and_a_late_grant_becomes_an_orphan_release() {
    let (mut allocator, a, _) = setup();
    let operation = allocator.submit([a], None, 2, true).unwrap();
    let token = allocator.next_claim_requests().unwrap()[0];
    allocator.cancel(operation, 1).unwrap();
    allocator.cancel(operation, 1).unwrap();
    assert_eq!(allocator.state(operation), Some(AllocationState::Cancelled));
    assert_eq!(
        grant(&mut allocator, token, 300, 2),
        GrantDisposition::ReleaseLateGrant
    );
    assert_eq!(allocator.orphan_releases_due(), vec![lease(300)]);
}

#[test]
fn cancellation_with_partial_claims_waits_for_known_and_late_release_acknowledgements() {
    let (mut allocator, a, b) = setup();
    let operation = allocator.submit([a, b], None, 2, true).unwrap();
    let first = allocator.next_claim_requests().unwrap()[0];
    grant(&mut allocator, first, 350, 1);
    let pending = allocator.next_claim_requests().unwrap()[0];
    allocator.cancel(operation, 2).unwrap();
    allocator.cancel(operation, 2).unwrap();
    assert_eq!(
        allocator.state(operation),
        Some(AllocationState::RollingBack)
    );
    assert_eq!(allocator.releases_due(operation), vec![lease(350)]);

    assert_eq!(
        grant(&mut allocator, pending, 351, 3),
        GrantDisposition::ReleaseLateGrant
    );
    allocator.ack_release(operation, lease(350)).unwrap();
    assert_eq!(allocator.state(operation), Some(AllocationState::Cancelled));
    assert_eq!(allocator.orphan_releases_due(), vec![lease(351)]);
    allocator.ack_orphan_release(lease(351)).unwrap();
    assert_eq!(allocator.claim_count(), 0);
}

#[test]
fn expiry_is_exclusive_and_repeat_expiry_does_not_change_state() {
    let (mut allocator, a, _) = setup();
    let operation = allocator.submit([a], Some(8), 1, true).unwrap();
    let token = allocator.next_claim_requests().unwrap()[0];
    assert!(allocator.expire(7).unwrap().is_empty());
    assert_eq!(allocator.expire(8).unwrap(), vec![operation]);
    assert!(allocator.expire(9).unwrap().is_empty());
    assert_eq!(allocator.state(operation), Some(AllocationState::TimedOut));
    assert_eq!(
        grant(&mut allocator, token, 400, 9),
        GrantDisposition::ReleaseLateGrant
    );
}

#[test]
fn stale_grant_and_stale_completion_cannot_satisfy_a_new_revision() {
    let (mut allocator, a, _) = setup();
    let operation = allocator.submit([a], None, 4, true).unwrap();
    let old_token = allocator.next_claim_requests().unwrap()[0];
    allocator.cancel(operation, 1).unwrap();
    let new_operation = allocator.submit([a], None, 4, true).unwrap();
    let new_token = allocator.next_claim_requests().unwrap()[0];
    assert_eq!(new_token.operation_id(), new_operation);

    assert_eq!(
        grant(&mut allocator, old_token, 500, 2),
        GrantDisposition::ReleaseLateGrant
    );
    grant(&mut allocator, new_token, 501, 2);
    let revision = allocator.start(new_operation).unwrap();
    assert_eq!(
        allocator.complete(new_operation, revision + 1),
        Err(AllocationError::StaleWorkRevision)
    );
    assert_eq!(
        allocator.state(new_operation),
        Some(AllocationState::Active)
    );
}

#[test]
fn mismatched_engine_request_cannot_be_recorded_as_the_claimed_resource_lease() {
    let (mut allocator, resource, _) = setup();
    let operation = allocator.submit([resource], None, 1, true).unwrap();
    let token = allocator.next_claim_requests().unwrap()[0];
    let expected_request = EngineRequestId::new(900, 0);
    let engine_resource = EngineResourceId::new(10, 0);
    allocator
        .bind_engine_request(token, expected_request, engine_resource)
        .unwrap();
    allocator
        .bind_engine_lease_revision(token, expected_request, 0)
        .unwrap();

    let mismatched = GrantReceipt {
        claim_request: token,
        careops_resource: resource,
        engine_resource,
        lease: lease(901),
    };
    assert_eq!(
        allocator.grant(mismatched, 1).unwrap(),
        GrantDisposition::ReleaseLateGrant
    );
    assert_eq!(allocator.state(operation), Some(AllocationState::Failed));
    assert_eq!(allocator.claims(operation), Vec::new());
    assert_eq!(allocator.orphan_releases_due(), vec![lease(901)]);
}

#[test]
fn stale_lease_revision_for_the_same_engine_request_is_rejected() {
    let (mut allocator, resource, _) = setup();
    let operation = allocator.submit([resource], None, 1, true).unwrap();
    let token = allocator.next_claim_requests().unwrap()[0];
    let engine_request = EngineRequestId::new(910, 4);
    let engine_resource = EngineResourceId::new(10, 2);
    allocator
        .bind_engine_request(token, engine_request, engine_resource)
        .unwrap();
    allocator
        .bind_engine_lease_revision(token, engine_request, 0)
        .unwrap();

    let stale = GrantReceipt {
        claim_request: token,
        careops_resource: resource,
        engine_resource,
        lease: LeaseId::new(engine_request, 1),
    };
    assert_eq!(
        allocator.grant(stale, 1).unwrap(),
        GrantDisposition::ReleaseLateGrant
    );
    assert_eq!(allocator.state(operation), Some(AllocationState::Failed));
    assert_eq!(allocator.claims(operation), Vec::new());
    assert_eq!(
        allocator.orphan_releases_due(),
        vec![LeaseId::new(engine_request, 1)]
    );
}

#[test]
fn non_interruptible_active_work_rejects_suspension_without_mutation() {
    let (mut allocator, a, _) = setup();
    let operation = allocator.submit([a], None, 8, false).unwrap();
    let token = allocator.next_claim_requests().unwrap()[0];
    grant(&mut allocator, token, 600, 1);
    let revision = allocator.start(operation).unwrap();
    let before = allocator.snapshot(operation).unwrap();
    assert_eq!(
        allocator.suspend_at_segment_boundary(operation, revision, 3),
        Err(AllocationError::NotInterruptible)
    );
    assert_eq!(allocator.snapshot(operation).unwrap(), before);
    assert_eq!(allocator.releases_due(operation), Vec::<LeaseId>::new());
}

#[test]
fn segment_boundary_suspend_retains_claims_and_progress_and_stale_completion_is_ignored() {
    let (mut allocator, a, _) = setup();
    let operation = allocator.submit([a], None, 8, true).unwrap();
    let token = allocator.next_claim_requests().unwrap()[0];
    grant(&mut allocator, token, 700, 1);
    let active_revision = allocator.start(operation).unwrap();
    let suspended_revision = allocator
        .suspend_at_segment_boundary(operation, active_revision, 3)
        .unwrap();

    assert_ne!(active_revision, suspended_revision);
    assert_eq!(allocator.state(operation), Some(AllocationState::Suspended));
    assert_eq!(allocator.progress(operation), Some(3));
    assert_eq!(allocator.claims(operation), vec![(a, lease(700))]);
    assert_eq!(
        allocator.complete(operation, active_revision),
        Err(AllocationError::StaleWorkRevision)
    );

    let resumed_revision = allocator.resume(operation).unwrap();
    assert_ne!(suspended_revision, resumed_revision);
    assert_eq!(allocator.progress(operation), Some(3));
}

#[test]
fn completed_work_releases_each_claim_only_after_acknowledgement() {
    let (mut allocator, a, b) = setup();
    let operation = allocator.submit([a, b], None, 2, true).unwrap();
    let first = allocator.next_claim_requests().unwrap()[0];
    grant(&mut allocator, first, 800, 1);
    let second = allocator.next_claim_requests().unwrap()[0];
    grant(&mut allocator, second, 801, 2);
    let revision = allocator.start(operation).unwrap();
    allocator.complete(operation, revision).unwrap();
    assert_eq!(
        allocator.complete(operation, revision),
        Err(AllocationError::InvalidState)
    );

    assert_eq!(allocator.state(operation), Some(AllocationState::Releasing));
    assert_eq!(allocator.claim_count(), 2);
    assert_eq!(
        allocator.releases_due(operation),
        vec![lease(801), lease(800)]
    );
    allocator.ack_release(operation, lease(800)).unwrap();
    assert_eq!(allocator.claim_count(), 1);
    assert_eq!(
        allocator.ack_release(operation, lease(800)),
        Err(AllocationError::UnknownLease)
    );
    assert_eq!(allocator.claim_count(), 1);
    allocator.ack_release(operation, lease(801)).unwrap();
    assert_eq!(allocator.state(operation), Some(AllocationState::Completed));
    assert_eq!(allocator.progress(operation), Some(2));
    assert_eq!(allocator.claim_count(), 0);
}

#[test]
fn duplicate_resources_and_empty_requests_are_rejected_without_mutation() {
    let (mut allocator, a, _) = setup();
    assert_eq!(
        allocator.submit([], None, 1, true),
        Err(AllocationError::EmptyResourceSet)
    );
    assert_eq!(
        allocator.submit([a, a], None, 1, true),
        Err(AllocationError::DuplicateResource)
    );
    assert_eq!(allocator.operation_count(), 0);
}
