use careops_core::transfer::{
    ApplyOutcome, ClaimDisposition, ClaimRole, CompensationRequest, EpisodeId, MessageId, PersonId,
    ResourceId, ServiceId, TransferCommand, TransferEffect, TransferEnvelope, TransferLocation,
    TransferManager, TransferOperationId, TransferRequest, TransferState,
    TRANSFER_ENVELOPE_VERSION,
};

fn op(value: u64) -> TransferOperationId {
    TransferOperationId::new(value)
}
fn msg(value: u64) -> MessageId {
    MessageId::new(value)
}
fn resource(value: u64) -> ResourceId {
    ResourceId::new(value)
}
fn service(value: u64) -> ServiceId {
    ServiceId::new(value)
}
fn envelope(
    operation_id: TransferOperationId,
    message_id: u64,
    revision: u64,
    tick: u64,
    command: TransferCommand,
) -> TransferEnvelope {
    TransferEnvelope::new(operation_id, msg(message_id), revision, tick, command)
}
fn request(
    operation_id: TransferOperationId,
    person_id: u64,
    episode_id: u64,
    origin: u64,
    destination: u64,
    request_expires_at: Option<u64>,
    origin_resources: Vec<ResourceId>,
) -> TransferEnvelope {
    envelope(
        operation_id,
        operation_id.value() * 100,
        0,
        0,
        TransferCommand::Request {
            request: TransferRequest {
                person_id: PersonId::new(person_id),
                episode_id: EpisodeId::new(episode_id),
                origin_service: service(origin),
                destination_service: service(destination),
                request_expires_at,
                origin_resources,
            },
        },
    )
}
fn normal_prefix(manager: &mut TransferManager, operation_id: TransferOperationId) {
    manager
        .apply(request(
            operation_id,
            1,
            101,
            1,
            2,
            None,
            vec![resource(10)],
        ))
        .unwrap();
    manager
        .apply(envelope(
            operation_id,
            operation_id.value() * 100 + 1,
            1,
            1,
            TransferCommand::Offer {
                expires_at: Some(8),
            },
        ))
        .unwrap();
    manager
        .apply(envelope(
            operation_id,
            operation_id.value() * 100 + 2,
            2,
            2,
            TransferCommand::Reserve {
                resources: vec![resource(20)],
                expires_at: Some(10),
            },
        ))
        .unwrap();
}
fn complete_normal(manager: &mut TransferManager, operation_id: TransferOperationId) {
    manager
        .apply(envelope(
            operation_id,
            operation_id.value() * 100 + 3,
            3,
            3,
            TransferCommand::BeginDeparture,
        ))
        .unwrap();
    manager
        .apply(envelope(
            operation_id,
            operation_id.value() * 100 + 4,
            4,
            4,
            TransferCommand::Depart,
        ))
        .unwrap();
    manager
        .apply(envelope(
            operation_id,
            operation_id.value() * 100 + 5,
            5,
            5,
            TransferCommand::Arrive,
        ))
        .unwrap();
    manager
        .apply(envelope(
            operation_id,
            operation_id.value() * 100 + 6,
            6,
            6,
            TransferCommand::CompleteHandover,
        ))
        .unwrap();
}

#[test]
fn normal_path_conserves_person_owner_location_and_resource_claims() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    normal_prefix(&mut manager, transfer);
    let reserved = manager.snapshot(transfer).unwrap();
    assert_eq!(reserved.state, TransferState::Reserved);
    assert_eq!(reserved.owner_service, service(1));
    assert_eq!(reserved.location, TransferLocation::Service(service(1)));
    assert_eq!(manager.claims().len(), 2);
    assert_eq!(manager.claims()[1].disposition, ClaimDisposition::Reserved);

    complete_normal(&mut manager, transfer);
    let done = manager.snapshot(transfer).unwrap();
    assert_eq!(done.state, TransferState::HandoverComplete);
    assert_eq!(done.owner_service, service(2));
    assert_eq!(done.location, TransferLocation::Service(service(2)));
    assert_eq!(manager.person(PersonId::new(1)).unwrap().episodes.len(), 1);
    let claims = manager.claims();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].resource_id, resource(20));
    assert_eq!(claims[0].operation_id, None);
    assert_eq!(claims[0].owner_service, service(2));
    assert_eq!(claims[0].role, ClaimRole::ServiceOccupancy);
    assert_eq!(claims[0].disposition, ClaimDisposition::Occupied);
}

#[test]
fn invalid_transition_and_unknown_schema_leave_all_state_unchanged() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    manager
        .apply(request(transfer, 1, 101, 1, 2, None, vec![resource(10)]))
        .unwrap();
    let before_snapshot = manager.snapshot(transfer).unwrap();
    let before_claims = manager.claims();
    let before_event_count = manager.events().len();
    assert!(manager
        .apply(envelope(transfer, 102, 1, 0, TransferCommand::Arrive,))
        .is_err());
    assert_eq!(manager.snapshot(transfer).unwrap(), before_snapshot);
    assert_eq!(manager.claims(), before_claims);
    assert_eq!(manager.events().len(), before_event_count);

    let before = manager.checkpoint().unwrap();
    let mut unsupported = envelope(
        transfer,
        103,
        1,
        1,
        TransferCommand::Offer { expires_at: None },
    );
    unsupported.schema_version = TRANSFER_ENVELOPE_VERSION + 1;
    assert!(manager.apply(unsupported).is_err());
    assert_eq!(manager.checkpoint().unwrap(), before);
}

#[test]
fn invalid_future_tick_transition_does_not_commit_clock_or_unrelated_expiry() {
    let mut manager = TransferManager::default();
    manager
        .apply(request(op(1), 1, 101, 1, 2, Some(8), Vec::new()))
        .unwrap();
    manager
        .apply(request(op(2), 2, 201, 1, 2, None, Vec::new()))
        .unwrap();
    let before = manager.checkpoint().unwrap();

    assert_eq!(
        manager.apply(envelope(op(2), 201, 1, 8, TransferCommand::Arrive)),
        Err(careops_core::transfer::TransferError::InvalidTransition)
    );

    assert_eq!(manager.checkpoint().unwrap(), before);
    assert_eq!(manager.now(), 0);
    assert_eq!(
        manager.snapshot(op(1)).unwrap().state,
        TransferState::Requested
    );
}

#[test]
fn failed_expiry_revision_overflow_does_not_advance_clock() {
    let mut manager = TransferManager::default();
    manager
        .apply(request(op(1), 1, 101, 1, 2, Some(8), vec![resource(10)]))
        .unwrap();
    let mut checkpoint: serde_json::Value =
        serde_json::from_slice(&manager.checkpoint().unwrap()).unwrap();
    checkpoint["operations"]["1"]["revision"] = serde_json::json!(u64::MAX);
    let mut restored = TransferManager::restore(&serde_json::to_vec(&checkpoint).unwrap()).unwrap();
    let before = restored.checkpoint().unwrap();

    assert_eq!(
        restored.advance_to(8),
        Err(careops_core::transfer::TransferError::RevisionOverflow)
    );

    assert_eq!(restored.checkpoint().unwrap(), before);
    assert_eq!(restored.now(), 0);
}

#[test]
fn duplicate_ack_is_idempotent_and_conflicting_reuse_rejects_atomically() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    let request_event = request(transfer, 1, 101, 1, 2, None, vec![resource(10)]);
    manager.apply(request_event.clone()).unwrap();
    assert_eq!(
        manager.apply(request_event).unwrap(),
        ApplyOutcome::Duplicate
    );
    let offer = envelope(
        transfer,
        101,
        1,
        1,
        TransferCommand::Offer {
            expires_at: Some(9),
        },
    );
    manager.apply(offer.clone()).unwrap();
    assert_eq!(
        manager.apply(offer.clone()).unwrap(),
        ApplyOutcome::Duplicate
    );
    let before_snapshot = manager.snapshot(transfer).unwrap();
    let before_claims = manager.claims();
    let before_event_count = manager.events().len();
    let conflict = envelope(
        transfer,
        101,
        1,
        1,
        TransferCommand::Offer {
            expires_at: Some(10),
        },
    );
    assert!(manager.apply(conflict).is_err());
    assert_eq!(manager.snapshot(transfer).unwrap(), before_snapshot);
    assert_eq!(manager.claims(), before_claims);
    assert_eq!(manager.events().len(), before_event_count);
}

#[test]
fn cancellation_after_reservation_releases_destination_once_and_retains_origin() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    normal_prefix(&mut manager, transfer);
    let cancel = envelope(transfer, 103, 3, 3, TransferCommand::Cancel);
    manager.apply(cancel.clone()).unwrap();
    assert_eq!(manager.apply(cancel).unwrap(), ApplyOutcome::Duplicate);
    assert_eq!(
        manager.snapshot(transfer).unwrap().state,
        TransferState::Cancelled
    );
    let claims = manager.claims();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].resource_id, resource(10));
    assert_eq!(claims[0].operation_id, None);
    assert_eq!(claims[0].owner_service, service(1));
    assert_eq!(
        manager.person(PersonId::new(1)).unwrap().active_operation,
        None
    );
}

#[test]
fn rejection_from_requested_offered_and_reserved_releases_only_destination_claims() {
    for stage in 0..=2 {
        let mut manager = TransferManager::default();
        let transfer = op(stage + 1);
        manager
            .apply(request(
                transfer,
                stage + 1,
                stage + 101,
                1,
                2,
                None,
                vec![resource(10)],
            ))
            .unwrap();
        if stage >= 1 {
            manager
                .apply(envelope(
                    transfer,
                    transfer.value() * 100 + 1,
                    1,
                    1,
                    TransferCommand::Offer { expires_at: None },
                ))
                .unwrap();
        }
        if stage >= 2 {
            manager
                .apply(envelope(
                    transfer,
                    transfer.value() * 100 + 2,
                    2,
                    2,
                    TransferCommand::Reserve {
                        resources: vec![resource(20)],
                        expires_at: None,
                    },
                ))
                .unwrap();
        }
        let revision = stage + 1;
        let rejected = envelope(
            transfer,
            transfer.value() * 100 + 9,
            revision,
            revision,
            TransferCommand::Reject,
        );
        manager.apply(rejected.clone()).unwrap();
        assert_eq!(manager.apply(rejected).unwrap(), ApplyOutcome::Duplicate);
        assert_eq!(
            manager.snapshot(transfer).unwrap().state,
            TransferState::Rejected
        );
        let claims = manager.claims();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].resource_id, resource(10));
        assert_eq!(claims[0].owner_service, service(1));
        assert_eq!(claims[0].operation_id, None);
        assert_eq!(
            manager
                .person(PersonId::new(stage + 1))
                .unwrap()
                .active_operation,
            None
        );
    }
}

#[test]
fn cancellation_from_departing_releases_destination_and_retains_origin() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    normal_prefix(&mut manager, transfer);
    manager
        .apply(envelope(
            transfer,
            103,
            3,
            3,
            TransferCommand::BeginDeparture,
        ))
        .unwrap();
    let cancel = envelope(transfer, 104, 4, 4, TransferCommand::Cancel);
    manager.apply(cancel.clone()).unwrap();
    assert_eq!(manager.apply(cancel).unwrap(), ApplyOutcome::Duplicate);
    assert_eq!(
        manager.snapshot(transfer).unwrap().state,
        TransferState::Cancelled
    );
    let claims = manager.claims();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].resource_id, resource(10));
    assert_eq!(claims[0].owner_service, service(1));
    assert_eq!(claims[0].operation_id, None);
}

#[test]
fn request_offer_and_reservation_expiry_have_distinct_claim_effects() {
    let mut request_expiry = TransferManager::default();
    let transfer = op(1);
    request_expiry
        .apply(request(transfer, 1, 101, 1, 2, Some(2), vec![resource(10)]))
        .unwrap();
    request_expiry.advance_to(2).unwrap();
    assert_eq!(
        request_expiry.snapshot(transfer).unwrap().state,
        TransferState::Expired
    );
    assert_eq!(request_expiry.claims().len(), 1);
    assert_eq!(request_expiry.claims()[0].resource_id, resource(10));
    assert_eq!(request_expiry.claims()[0].operation_id, None);

    let mut offer_expiry = TransferManager::default();
    offer_expiry
        .apply(request(transfer, 2, 102, 1, 2, None, vec![resource(11)]))
        .unwrap();
    offer_expiry
        .apply(envelope(
            transfer,
            101,
            1,
            1,
            TransferCommand::Offer {
                expires_at: Some(2),
            },
        ))
        .unwrap();
    offer_expiry.advance_to(2).unwrap();
    assert_eq!(
        offer_expiry.snapshot(transfer).unwrap().state,
        TransferState::Expired
    );
    assert_eq!(offer_expiry.claims().len(), 1);
    assert_eq!(offer_expiry.claims()[0].resource_id, resource(11));

    // A reservation deadline at tick 4 releases only the reserved destination.
    let mut reservation_expiry = TransferManager::default();
    reservation_expiry
        .apply(request(transfer, 3, 103, 1, 2, None, vec![resource(12)]))
        .unwrap();
    reservation_expiry
        .apply(envelope(
            transfer,
            101,
            1,
            1,
            TransferCommand::Offer { expires_at: None },
        ))
        .unwrap();
    reservation_expiry
        .apply(envelope(
            transfer,
            102,
            2,
            2,
            TransferCommand::Reserve {
                resources: vec![resource(22)],
                expires_at: Some(4),
            },
        ))
        .unwrap();
    reservation_expiry.advance_to(4).unwrap();
    assert_eq!(
        reservation_expiry.snapshot(transfer).unwrap().state,
        TransferState::Expired
    );
    let claims = reservation_expiry.claims();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].resource_id, resource(12));
}

#[test]
fn in_transit_expiry_at_arrival_converts_hold_to_occupancy_without_release() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    manager
        .apply(request(transfer, 1, 101, 1, 2, None, vec![resource(10)]))
        .unwrap();
    manager
        .apply(envelope(
            transfer,
            101,
            1,
            1,
            TransferCommand::Offer { expires_at: None },
        ))
        .unwrap();
    manager
        .apply(envelope(
            transfer,
            102,
            2,
            2,
            TransferCommand::Reserve {
                resources: vec![resource(20)],
                expires_at: Some(5),
            },
        ))
        .unwrap();
    manager
        .apply(envelope(
            transfer,
            103,
            3,
            3,
            TransferCommand::BeginDeparture,
        ))
        .unwrap();
    manager
        .apply(envelope(transfer, 104, 4, 4, TransferCommand::Depart))
        .unwrap();
    manager.advance_to(5).unwrap();
    assert_eq!(
        manager.snapshot(transfer).unwrap().state,
        TransferState::InTransit
    );
    assert_eq!(manager.claims().len(), 2);
    manager
        .apply(envelope(transfer, 105, 5, 5, TransferCommand::Arrive))
        .unwrap();
    assert_eq!(
        manager.snapshot(transfer).unwrap().state,
        TransferState::Arrived
    );
    let destination = manager
        .claims()
        .into_iter()
        .find(|claim| claim.resource_id == resource(20))
        .unwrap();
    assert_eq!(destination.disposition, ClaimDisposition::Occupied);
    assert_eq!(destination.operation_id, Some(transfer));
}

#[test]
fn reordered_ack_waits_for_revision_gap_then_replays_causally() {
    let transfer = op(1);
    let mut reordered = TransferManager::default();
    reordered
        .apply(request(transfer, 1, 101, 1, 2, None, vec![resource(10)]))
        .unwrap();
    reordered
        .apply(envelope(
            transfer,
            101,
            1,
            1,
            TransferCommand::Offer { expires_at: None },
        ))
        .unwrap();
    let begin = envelope(transfer, 103, 3, 4, TransferCommand::BeginDeparture);
    assert_eq!(reordered.apply(begin).unwrap(), ApplyOutcome::Buffered);
    assert_eq!(
        reordered.snapshot(transfer).unwrap().state,
        TransferState::Offered
    );
    reordered
        .apply(envelope(
            transfer,
            102,
            2,
            2,
            TransferCommand::Reserve {
                resources: vec![resource(20)],
                expires_at: None,
            },
        ))
        .unwrap();
    assert_eq!(
        reordered.snapshot(transfer).unwrap().state,
        TransferState::Departing
    );
    assert_eq!(reordered.snapshot(transfer).unwrap().revision, 4);
    assert_eq!(
        reordered.events().last().unwrap().effect,
        TransferEffect::DeparturePrepared
    );
}

#[test]
fn conflicting_future_messages_cannot_replace_a_buffered_revision() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    manager
        .apply(request(transfer, 1, 101, 1, 2, None, vec![resource(10)]))
        .unwrap();
    let first = envelope(transfer, 103, 3, 3, TransferCommand::BeginDeparture);
    let conflict = envelope(transfer, 104, 3, 3, TransferCommand::Cancel);
    assert_eq!(manager.apply(first).unwrap(), ApplyOutcome::Buffered);
    assert_eq!(
        manager.apply(conflict),
        Err(careops_core::transfer::TransferError::MessageConflict)
    );
    let checkpoint = manager.checkpoint().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    assert_eq!(
        value["operations"]["1"]["pending_messages"]["3"]["message_id"],
        103
    );
}

#[test]
fn terminal_operations_reject_future_revision_messages() {
    let mut manager = TransferManager::default();
    manager
        .apply(request(op(1), 1, 101, 1, 2, Some(1), vec![resource(10)]))
        .unwrap();
    let outcome = manager.apply(envelope(op(1), 105, 5, 2, TransferCommand::BeginDeparture));
    assert_eq!(
        outcome,
        Err(careops_core::transfer::TransferError::InvalidTransition)
    );
    let json: serde_json::Value = serde_json::from_slice(&manager.checkpoint().unwrap()).unwrap();
    assert!(json["operations"]["1"]["pending_messages"]
        .as_object()
        .unwrap()
        .is_empty());
}

#[test]
fn stale_delayed_ack_cannot_rewind_state() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    normal_prefix(&mut manager, transfer);
    let before = manager.snapshot(transfer).unwrap();
    assert_eq!(
        manager
            .apply(envelope(
                transfer,
                199,
                1,
                1,
                TransferCommand::Offer { expires_at: None },
            ))
            .unwrap(),
        ApplyOutcome::StaleIgnored
    );
    assert_eq!(manager.snapshot(transfer).unwrap(), before);
}

#[test]
fn checkpoint_restart_preserves_deduplication_buffer_and_continuation() {
    let transfer = op(1);
    let mut before_restart = TransferManager::default();
    before_restart
        .apply(request(transfer, 1, 101, 1, 2, None, vec![resource(10)]))
        .unwrap();
    let offer = envelope(
        transfer,
        101,
        1,
        1,
        TransferCommand::Offer { expires_at: None },
    );
    before_restart.apply(offer.clone()).unwrap();
    before_restart
        .apply(envelope(
            transfer,
            103,
            3,
            3,
            TransferCommand::BeginDeparture,
        ))
        .unwrap();
    let checkpoint = before_restart.checkpoint().unwrap();
    let mut restored = TransferManager::restore(&checkpoint).unwrap();
    assert_eq!(restored.apply(offer).unwrap(), ApplyOutcome::Duplicate);
    restored
        .apply(envelope(
            transfer,
            102,
            2,
            2,
            TransferCommand::Reserve {
                resources: vec![resource(20)],
                expires_at: None,
            },
        ))
        .unwrap();
    assert_eq!(
        restored.snapshot(transfer).unwrap().state,
        TransferState::Departing
    );
    restored
        .apply(envelope(transfer, 104, 4, 4, TransferCommand::Depart))
        .unwrap();
    let after_resume = restored.checkpoint().unwrap();

    let mut uninterrupted = TransferManager::default();
    uninterrupted
        .apply(request(transfer, 1, 101, 1, 2, None, vec![resource(10)]))
        .unwrap();
    uninterrupted
        .apply(envelope(
            transfer,
            101,
            1,
            1,
            TransferCommand::Offer { expires_at: None },
        ))
        .unwrap();
    uninterrupted
        .apply(envelope(
            transfer,
            102,
            2,
            2,
            TransferCommand::Reserve {
                resources: vec![resource(20)],
                expires_at: None,
            },
        ))
        .unwrap();
    uninterrupted
        .apply(envelope(
            transfer,
            103,
            3,
            3,
            TransferCommand::BeginDeparture,
        ))
        .unwrap();
    uninterrupted
        .apply(envelope(transfer, 104, 4, 4, TransferCommand::Depart))
        .unwrap();
    assert_eq!(after_resume, uninterrupted.checkpoint().unwrap());
}

#[test]
fn malformed_or_unsupported_checkpoint_fails_closed() {
    let mut manager = TransferManager::default();
    manager
        .apply(request(op(1), 1, 101, 1, 2, None, vec![resource(10)]))
        .unwrap();
    let checkpoint = manager.checkpoint().unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    json["checkpoint_version"] = serde_json::json!(2);
    let unsupported = serde_json::to_vec(&json).unwrap();
    assert!(TransferManager::restore(&unsupported).is_err());
    assert!(TransferManager::restore(b"not a checkpoint").is_err());
}

#[test]
fn checkpoint_rejects_inconsistent_owner_and_missing_claims() {
    let mut manager = TransferManager::default();
    normal_prefix(&mut manager, op(1));
    let checkpoint = manager.checkpoint().unwrap();
    let mut wrong_owner: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    wrong_owner["people"]["1"]["owner_service"] = serde_json::json!(9);
    assert!(TransferManager::restore(&serde_json::to_vec(&wrong_owner).unwrap()).is_err());

    let mut wrong_location: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    wrong_location["people"]["1"]["location"]["service"] = serde_json::json!(9);
    assert!(TransferManager::restore(&serde_json::to_vec(&wrong_location).unwrap()).is_err());

    let mut missing_claim: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    missing_claim["claims"]
        .as_object_mut()
        .unwrap()
        .remove("10");
    assert!(TransferManager::restore(&serde_json::to_vec(&missing_claim).unwrap()).is_err());
    let mut detached_operation: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    detached_operation["people"]["1"]["active_operation"] = serde_json::Value::Null;
    assert!(TransferManager::restore(&serde_json::to_vec(&detached_operation).unwrap()).is_err());
}

#[test]
fn checkpoint_rejects_claims_still_attached_to_terminal_operations() {
    let mut manager = TransferManager::default();
    manager
        .apply(request(op(1), 1, 101, 1, 2, None, vec![resource(10)]))
        .unwrap();
    manager
        .apply(envelope(op(1), 101, 1, 1, TransferCommand::Cancel))
        .unwrap();
    let checkpoint = manager.checkpoint().unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    json["claims"]["10"]["operation_id"] = serde_json::json!(1);
    json["claims"]["10"]["role"] = serde_json::json!("origin");
    assert!(TransferManager::restore(&serde_json::to_vec(&json).unwrap()).is_err());

    let mut missing_person: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    missing_person["people"]
        .as_object_mut()
        .unwrap()
        .remove("1");
    assert!(TransferManager::restore(&serde_json::to_vec(&missing_person).unwrap()).is_err());

    let mut invalid_unowned_claim: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    invalid_unowned_claim["claims"]["10"]["role"] = serde_json::json!("origin");
    assert!(
        TransferManager::restore(&serde_json::to_vec(&invalid_unowned_claim).unwrap()).is_err()
    );
}

#[test]
fn checkpoint_rejects_consistent_but_wrong_owner_after_completed_transfer() {
    let mut manager = TransferManager::default();
    normal_prefix(&mut manager, op(1));
    complete_normal(&mut manager, op(1));
    let checkpoint = manager.checkpoint().unwrap();
    let mut wrong_owner_and_location: serde_json::Value =
        serde_json::from_slice(&checkpoint).unwrap();
    wrong_owner_and_location["people"]["1"]["owner_service"] = serde_json::json!(9);
    wrong_owner_and_location["people"]["1"]["location"]["service"] = serde_json::json!(9);
    assert!(
        TransferManager::restore(&serde_json::to_vec(&wrong_owner_and_location).unwrap()).is_err()
    );
}

#[test]
fn in_transit_rejection_waits_for_arrival_before_compensation() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    normal_prefix(&mut manager, transfer);
    manager
        .apply(envelope(
            transfer,
            103,
            3,
            3,
            TransferCommand::BeginDeparture,
        ))
        .unwrap();
    manager
        .apply(envelope(transfer, 104, 4, 4, TransferCommand::Depart))
        .unwrap();
    let before = manager.snapshot(transfer).unwrap();
    assert_eq!(
        manager.apply(envelope(
            transfer,
            105,
            5,
            5,
            TransferCommand::DestinationRejected {
                compensation: CompensationRequest {
                    operation_id: op(2),
                    destination_service: service(3),
                    request_expires_at: Some(20),
                },
            },
        )),
        Err(careops_core::transfer::TransferError::InvalidTransition)
    );
    assert_eq!(manager.snapshot(transfer).unwrap(), before);
    assert_eq!(
        manager.person(PersonId::new(1)).unwrap().location,
        TransferLocation::Transport {
            from: service(1),
            to: service(2)
        }
    );
}

#[test]
fn post_departure_rejection_creates_compensation_without_duplicating_person() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    normal_prefix(&mut manager, transfer);
    manager
        .apply(envelope(
            transfer,
            103,
            3,
            3,
            TransferCommand::BeginDeparture,
        ))
        .unwrap();
    manager
        .apply(envelope(transfer, 104, 4, 4, TransferCommand::Depart))
        .unwrap();
    manager
        .apply(envelope(transfer, 105, 5, 5, TransferCommand::Arrive))
        .unwrap();
    manager
        .apply(envelope(
            transfer,
            106,
            6,
            6,
            TransferCommand::DestinationRejected {
                compensation: CompensationRequest {
                    operation_id: op(2),
                    destination_service: service(3),
                    request_expires_at: Some(20),
                },
            },
        ))
        .unwrap();
    let parent = manager.snapshot(transfer).unwrap();
    assert_eq!(parent.state, TransferState::Arrived);
    assert_eq!(parent.owner_service, service(1));
    assert_eq!(parent.active_compensation, Some(op(2)));
    assert_eq!(manager.person(PersonId::new(1)).unwrap().episodes.len(), 1);

    manager
        .apply(envelope(
            op(2),
            201,
            1,
            7,
            TransferCommand::Offer { expires_at: None },
        ))
        .unwrap();
    manager
        .apply(envelope(
            op(2),
            202,
            2,
            8,
            TransferCommand::Reserve {
                resources: vec![resource(30)],
                expires_at: Some(30),
            },
        ))
        .unwrap();
    manager
        .apply(envelope(op(2), 203, 3, 9, TransferCommand::BeginDeparture))
        .unwrap();
    manager
        .apply(envelope(op(2), 204, 4, 10, TransferCommand::Depart))
        .unwrap();
    assert_eq!(
        manager.snapshot(op(2)).unwrap().state,
        TransferState::InTransit
    );
    assert_eq!(
        manager.snapshot(op(2)).unwrap().origin_resources,
        vec![resource(20)]
    );
    manager
        .apply(envelope(op(2), 205, 5, 11, TransferCommand::Arrive))
        .unwrap();
    let before_nested_compensation = manager.checkpoint().unwrap();
    assert_eq!(
        manager.apply(envelope(
            op(2),
            207,
            6,
            12,
            TransferCommand::DestinationRejected {
                compensation: CompensationRequest {
                    operation_id: op(3),
                    destination_service: service(4),
                    request_expires_at: Some(20),
                },
            },
        )),
        Err(careops_core::transfer::TransferError::InvalidTransition)
    );
    assert_eq!(manager.checkpoint().unwrap(), before_nested_compensation);
    manager
        .apply(envelope(
            op(2),
            206,
            6,
            12,
            TransferCommand::CompleteHandover,
        ))
        .unwrap();
    assert_eq!(
        manager.snapshot(transfer).unwrap().state,
        TransferState::HandoverComplete
    );
    assert_eq!(
        manager.snapshot(transfer).unwrap().actual_handover_service,
        Some(service(3))
    );
    let person = manager.person(PersonId::new(1)).unwrap();
    assert_eq!(person.episodes.len(), 1);
    assert_eq!(person.owner_service, service(3));
    assert_eq!(person.location, TransferLocation::Service(service(3)));
    assert_eq!(person.active_operation, None);
    let claims = manager.claims();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].resource_id, resource(30));
    assert_eq!(claims[0].owner_service, service(3));
}

#[test]
fn expired_compensation_restores_parent_as_active_without_releasing_origin() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    normal_prefix(&mut manager, transfer);
    manager
        .apply(envelope(
            transfer,
            103,
            3,
            3,
            TransferCommand::BeginDeparture,
        ))
        .unwrap();
    manager
        .apply(envelope(transfer, 104, 4, 4, TransferCommand::Depart))
        .unwrap();
    manager
        .apply(envelope(transfer, 105, 5, 5, TransferCommand::Arrive))
        .unwrap();
    manager
        .apply(envelope(
            transfer,
            106,
            6,
            6,
            TransferCommand::DestinationRejected {
                compensation: CompensationRequest {
                    operation_id: op(2),
                    destination_service: service(3),
                    request_expires_at: Some(8),
                },
            },
        ))
        .unwrap();
    manager.advance_to(8).unwrap();
    assert_eq!(
        manager.snapshot(op(2)).unwrap().state,
        TransferState::Expired
    );
    assert_eq!(
        manager.person(PersonId::new(1)).unwrap().active_operation,
        Some(transfer)
    );
    assert_eq!(
        manager.snapshot(transfer).unwrap().state,
        TransferState::Arrived
    );
    assert_eq!(manager.claims().len(), 2);
}

#[test]
fn sequential_episode_can_adopt_destination_service_occupancy() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    normal_prefix(&mut manager, transfer);
    complete_normal(&mut manager, transfer);
    let mut second_request = request(op(2), 1, 102, 2, 3, None, vec![resource(20)]);
    second_request.tick = manager.now();
    manager.apply(second_request).unwrap();
    let person = manager.person(PersonId::new(1)).unwrap();
    assert_eq!(person.episodes.len(), 2);
    assert_eq!(person.active_operation, Some(op(2)));
    let origin_claim = manager
        .claims()
        .into_iter()
        .find(|claim| claim.resource_id == resource(20))
        .unwrap();
    assert_eq!(origin_claim.operation_id, Some(op(2)));
    assert_eq!(origin_claim.role, ClaimRole::Origin);
}

#[test]
fn release_and_reservation_failures_do_not_partially_mutate_claims() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    manager
        .apply(request(transfer, 1, 101, 1, 2, None, vec![resource(10)]))
        .unwrap();
    manager
        .apply(envelope(
            transfer,
            101,
            1,
            1,
            TransferCommand::Offer { expires_at: None },
        ))
        .unwrap();
    let before_snapshot = manager.snapshot(transfer).unwrap();
    let before_claims = manager.claims();
    let before_event_count = manager.events().len();
    assert!(manager
        .apply(envelope(
            transfer,
            102,
            2,
            2,
            TransferCommand::Reserve {
                resources: vec![resource(10), resource(20)],
                expires_at: None,
            },
        ))
        .is_err());
    assert_eq!(manager.snapshot(transfer).unwrap(), before_snapshot);
    assert_eq!(manager.claims(), before_claims);
    assert_eq!(manager.events().len(), before_event_count);
}

#[test]
fn horizon_censoring_keeps_in_transit_owner_and_claims_open() {
    let mut manager = TransferManager::default();
    let transfer = op(1);
    normal_prefix(&mut manager, transfer);
    manager
        .apply(envelope(
            transfer,
            103,
            3,
            3,
            TransferCommand::BeginDeparture,
        ))
        .unwrap();
    manager
        .apply(envelope(transfer, 104, 4, 4, TransferCommand::Depart))
        .unwrap();
    manager.advance_to(100).unwrap();
    let snapshot = manager.snapshot(transfer).unwrap();
    assert_eq!(snapshot.state, TransferState::InTransit);
    assert_eq!(snapshot.owner_service, service(1));
    assert_eq!(
        snapshot.location,
        TransferLocation::Transport {
            from: service(1),
            to: service(2)
        }
    );
    assert_eq!(manager.claims().len(), 2);
    assert_eq!(
        manager.person(PersonId::new(1)).unwrap().active_operation,
        Some(transfer)
    );
}

#[test]
fn resource_identity_and_request_validation_fail_closed() {
    let mut manager = TransferManager::default();
    assert!(manager
        .apply(request(
            op(1),
            1,
            101,
            1,
            2,
            None,
            vec![resource(10), resource(10)],
        ))
        .is_err());
    assert!(manager.snapshot(op(1)).is_none());
    assert!(manager
        .apply(request(op(1), 1, 101, 1, 1, None, vec![]))
        .is_err());
}
