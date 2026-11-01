use careops_core::birth::{
    BirthInfantInput, BirthMessageError, BirthMessageId, BirthMessageInput, BirthMessageLedger,
    BirthRecipient,
};
use careops_core::identity::{
    BirthEventId, IdentityCommand, IdentityLedger, PersonId, PersonOrigin, RelationshipEventId,
    RelationshipKind, SpawnEventId,
};
use careops_core::transfer::ServiceId;

fn mother_ledger() -> (IdentityLedger, PersonId) {
    let mut identities = IdentityLedger::new(93);
    let person = careops_core::identity::person_id_for_spawn(93, SpawnEventId::new(1)).unwrap();
    identities
        .apply_tick(
            0,
            vec![IdentityCommand::Spawn {
                event_id: SpawnEventId::new(1),
                origin: PersonOrigin::ExternalEntry,
            }],
        )
        .unwrap();
    (identities, person)
}

fn input(mother: PersonId) -> BirthMessageInput {
    BirthMessageInput {
        message_id: BirthMessageId::new(50),
        birth_event_id: BirthEventId::new(70),
        mother_person_id: mother,
        maternal_destination_service: ServiceId::new(10),
        infants: vec![
            BirthInfantInput {
                infant_ordinal: 1,
                spawn_event_id: SpawnEventId::new(3),
                relationship_event_id: RelationshipEventId::new(102),
                destination_service: ServiceId::new(30),
            },
            BirthInfantInput {
                infant_ordinal: 0,
                spawn_event_id: SpawnEventId::new(2),
                relationship_event_id: RelationshipEventId::new(101),
                destination_service: ServiceId::new(20),
            },
        ],
    }
}

#[test]
fn multiple_birth_is_atomic_and_returns_independent_destinations() {
    let (mut identities, mother) = mother_ledger();
    let mut births = BirthMessageLedger::new();
    let result = births.register(&mut identities, 1, input(mother)).unwrap();
    assert_eq!(result.recipients.len(), 3);
    assert_eq!(
        result.recipients[0],
        BirthRecipient::Mother {
            person_id: mother,
            destination_service: ServiceId::new(10),
        }
    );
    let infant_zero = match result.recipients[1] {
        BirthRecipient::Infant {
            person_id,
            infant_ordinal: 0,
            destination_service,
        } => {
            assert_eq!(destination_service, ServiceId::new(20));
            person_id
        }
        _ => panic!("recipients are returned in canonical infant ordinal order"),
    };
    let infant_one = match result.recipients[2] {
        BirthRecipient::Infant {
            person_id,
            infant_ordinal: 1,
            destination_service,
        } => {
            assert_eq!(destination_service, ServiceId::new(30));
            person_id
        }
        _ => panic!("recipients are returned in canonical infant ordinal order"),
    };
    assert_ne!(infant_zero, infant_one);
    assert_eq!(
        identities.person(infant_zero).unwrap().origin,
        PersonOrigin::Birth {
            birth_event_id: BirthEventId::new(70),
            infant_ordinal: 0,
        }
    );
    assert_eq!(
        identities.person(infant_one).unwrap().origin,
        PersonOrigin::Birth {
            birth_event_id: BirthEventId::new(70),
            infant_ordinal: 1,
        }
    );
    assert_eq!(identities.relationships().len(), 2);
    assert!(identities
        .relationships()
        .iter()
        .all(|edge| { edge.kind == RelationshipKind::BirthParentOf && edge.from == mother }));
}

#[test]
fn exact_message_retry_is_order_insensitive_and_payload_conflicts_are_atomic() {
    let (mut identities, mother) = mother_ledger();
    let mut births = BirthMessageLedger::new();
    let initial = input(mother);
    let result = births
        .register(&mut identities, 1, initial.clone())
        .unwrap();
    let before = identities.checkpoint().unwrap();
    let mut reordered = initial.clone();
    reordered.infants.reverse();
    assert_eq!(
        births.register(&mut identities, 1, reordered).unwrap(),
        result
    );
    assert_eq!(identities.checkpoint().unwrap(), before);

    let mut changed = initial;
    changed.infants[0].destination_service = ServiceId::new(99);
    assert_eq!(
        births.register(&mut identities, 2, changed),
        Err(BirthMessageError::MessagePayloadConflict)
    );
    assert_eq!(identities.checkpoint().unwrap(), before);
    assert_eq!(births.message_count(), 1);
}

#[test]
fn replay_against_unpaired_identity_ledger_rejects_without_mutation() {
    let (mut identities, mother) = mother_ledger();
    let mut births = BirthMessageLedger::new();
    let initial = input(mother);
    births
        .register(&mut identities, 1, initial.clone())
        .unwrap();
    let mut unrelated_identities = IdentityLedger::new(94);
    let before_births = births.checkpoint().unwrap();
    let before_unrelated = unrelated_identities.checkpoint().unwrap();
    assert!(matches!(
        births.register(&mut unrelated_identities, 1, initial),
        Err(BirthMessageError::InvalidCheckpoint)
    ));
    assert_eq!(births.checkpoint().unwrap(), before_births);
    assert_eq!(unrelated_identities.checkpoint().unwrap(), before_unrelated);
}

#[test]
fn empty_or_duplicate_ordinals_reject_without_partial_birth_state() {
    let (mut identities, mother) = mother_ledger();
    let mut births = BirthMessageLedger::new();
    let before_identity = identities.checkpoint().unwrap();
    let mut empty = input(mother);
    empty.infants.clear();
    assert_eq!(
        births.register(&mut identities, 1, empty),
        Err(BirthMessageError::EmptyInfantSet)
    );
    let mut duplicate_ordinal = input(mother);
    duplicate_ordinal.infants[1].infant_ordinal = duplicate_ordinal.infants[0].infant_ordinal;
    assert_eq!(
        births.register(&mut identities, 1, duplicate_ordinal),
        Err(BirthMessageError::DuplicateInfantOrdinal)
    );
    assert_eq!(identities.checkpoint().unwrap(), before_identity);
    assert_eq!(births.message_count(), 0);
}

#[test]
fn paired_restore_accepts_later_link_removal_and_rejects_tampered_mapping() {
    let (mut identities, mother) = mother_ledger();
    let mut births = BirthMessageLedger::new();
    let result = births.register(&mut identities, 1, input(mother)).unwrap();
    let birth_checkpoint = births.checkpoint().unwrap();
    let infant = match result.recipients[1] {
        BirthRecipient::Infant { person_id, .. } => person_id,
        _ => unreachable!(),
    };
    identities
        .apply_tick(
            2,
            vec![IdentityCommand::RemoveRelationship {
                event_id: RelationshipEventId::new(201),
                kind: RelationshipKind::BirthParentOf,
                from: mother,
                to: infant,
            }],
        )
        .unwrap();
    assert_eq!(identities.relationships().len(), 1);
    assert_eq!(
        BirthMessageLedger::restore(&birth_checkpoint, &identities).unwrap(),
        births
    );

    let mut tampered: serde_json::Value = serde_json::from_slice(&birth_checkpoint).unwrap();
    tampered["messages"]["50"]["result"]["recipients"][1]["infant"]["person_id"] =
        serde_json::json!(mother.value());
    assert!(
        BirthMessageLedger::restore(&serde_json::to_vec(&tampered).unwrap(), &identities).is_err()
    );
}
