use careops_core::identity::{
    ed_admission_event_id, ed_person_spawn_event_id, BirthEventId, EpisodeEventId, EpisodeId,
    EpisodeStatus, IdentityCommand, IdentityError, IdentityLedger, IdentityOutcome,
    LifecycleEventId, PersonId, PersonOrigin, PersonStatus, RelationshipEventId, RelationshipKind,
    SpawnEventId,
};
use careops_core::transfer::ServiceId;
use sha2::{Digest, Sha256};

fn service(id: u64) -> ServiceId {
    ServiceId::new(id)
}
fn spawn(id: u64, origin: PersonOrigin) -> IdentityCommand {
    IdentityCommand::Spawn {
        event_id: SpawnEventId::new(id),
        origin,
    }
}
fn enter(id: u64, person_id: PersonId, service_id: u64) -> IdentityCommand {
    IdentityCommand::EnterService {
        event_id: EpisodeEventId::new(id),
        person_id,
        service_id: service(service_id),
    }
}
fn person(run: u64, event: u64) -> PersonId {
    PersonId::new(id(b"person", run, event))
}
fn episode(run: u64, event: u64) -> EpisodeId {
    EpisodeId::new(id(b"episode", run, event))
}
fn id(kind: &[u8], run: u64, event: u64) -> u64 {
    let mut hash = Sha256::new();
    hash.update(b"careops.identity.id.v1\0");
    hash.update(kind);
    hash.update([0]);
    hash.update(run.to_be_bytes());
    hash.update(event.to_be_bytes());
    u64::from_be_bytes(hash.finalize()[..8].try_into().unwrap())
}

#[test]
fn fixed_sha256_identity_derivation_vectors_are_stable() {
    assert_eq!(person(1, 1).value(), 5_152_514_178_808_668_989);
    assert_eq!(episode(1, 1).value(), 4_708_837_883_263_603_961);
    let spawn_key = ed_person_spawn_event_id(42, "synthetic-patient-α")
        .unwrap()
        .value();
    let admission_key = ed_admission_event_id(42, "admission-1").unwrap().value();
    assert_eq!(spawn_key, 1_541_036_490_742_137_449);
    assert_eq!(admission_key, 51_449_270_824_498_736);
    assert_eq!(person(42, spawn_key).value(), 8_979_869_846_555_893_168);
    assert_eq!(
        episode(42, admission_key).value(),
        8_754_272_787_240_046_413
    );
}

#[test]
fn canonical_tick_batches_are_order_invariant_and_replay_safe() {
    let mother = person(17, 1);
    let infant = person(17, 2);
    let commands = vec![
        IdentityCommand::AddRelationship {
            event_id: RelationshipEventId::new(9),
            kind: RelationshipKind::BirthParentOf,
            from: mother,
            to: infant,
        },
        spawn(
            2,
            PersonOrigin::Birth {
                birth_event_id: BirthEventId::new(7),
                infant_ordinal: 0,
            },
        ),
        spawn(1, PersonOrigin::ExternalEntry),
    ];
    let mut first = IdentityLedger::new(17);
    let mut second = IdentityLedger::new(17);
    first.apply_tick(0, commands.clone()).unwrap();
    let mut reverse = commands;
    reverse.reverse();
    second.apply_tick(0, reverse).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.person(mother).unwrap().status, PersonStatus::Living);
    assert_eq!(first.relationships().len(), 1);
    assert_eq!(
        first
            .apply_tick(0, vec![spawn(1, PersonOrigin::ExternalEntry)])
            .unwrap(),
        [IdentityOutcome::Duplicate]
    );
    let checkpoint = first.checkpoint().unwrap();
    let mut changed = first.clone();
    assert_eq!(
        changed.apply_tick(
            0,
            vec![spawn(
                1,
                PersonOrigin::Birth {
                    birth_event_id: BirthEventId::new(7),
                    infant_ordinal: 1
                }
            )]
        ),
        Err(IdentityError::EventConflict)
    );
    assert_eq!(changed.checkpoint().unwrap(), checkpoint);
}

#[test]
fn episodes_are_one_to_many_readmission_and_death_is_separate_from_exit() {
    let mut ledger = IdentityLedger::new(11);
    let person_id = person(11, 1);
    ledger
        .apply_tick(
            0,
            vec![
                spawn(1, PersonOrigin::ExternalEntry),
                enter(10, person_id, 2),
                enter(11, person_id, 3),
            ],
        )
        .unwrap();
    let first = episode(11, 10);
    let concurrent = episode(11, 11);
    assert_ne!(first, concurrent);
    assert_eq!(ledger.person(person_id).unwrap().episodes.len(), 2);
    let before_death = ledger.checkpoint().unwrap();
    assert_eq!(
        ledger.apply_tick(
            1,
            vec![IdentityCommand::Death {
                event_id: LifecycleEventId::new(20),
                person_id
            }]
        ),
        Err(IdentityError::OpenEpisodesAtDeath)
    );
    assert_eq!(ledger.checkpoint().unwrap(), before_death);
    assert_eq!(
        ledger.apply_tick(
            1,
            vec![
                IdentityCommand::ExitService {
                    event_id: LifecycleEventId::new(21),
                    episode_id: first
                },
                IdentityCommand::Death {
                    event_id: LifecycleEventId::new(20),
                    person_id
                }
            ]
        ),
        Err(IdentityError::OpenEpisodesAtDeath)
    );
    // The rejected tick is atomic; the valid same-tick command set exits both episodes before death.
    assert_eq!(ledger.checkpoint().unwrap(), before_death);
    assert_eq!(ledger.sealed_through(), Some(0));
    ledger
        .apply_tick(
            1,
            vec![
                IdentityCommand::ExitService {
                    event_id: LifecycleEventId::new(21),
                    episode_id: first,
                },
                IdentityCommand::ExitService {
                    event_id: LifecycleEventId::new(22),
                    episode_id: concurrent,
                },
                IdentityCommand::Death {
                    event_id: LifecycleEventId::new(23),
                    person_id,
                },
            ],
        )
        .unwrap();
    assert_eq!(
        ledger.person(person_id).unwrap().status,
        PersonStatus::Deceased
    );
    assert_eq!(ledger.episode(first).unwrap().status, EpisodeStatus::Exited);
    assert_eq!(ledger.counts().service_entries, 2);
    assert_eq!(ledger.counts().service_exits, 2);
    assert_eq!(ledger.counts().deaths, 1);
    assert_eq!(
        ledger.apply_tick(2, vec![enter(12, person_id, 4)]),
        Err(IdentityError::PersonDeceased)
    );
}

#[test]
fn typed_relationship_cycles_are_scoped_and_removal_does_not_destroy_people() {
    let mut ledger = IdentityLedger::new(5);
    let p1 = person(5, 1);
    let p2 = person(5, 2);
    let p3 = person(5, 3);
    ledger
        .apply_tick(
            0,
            vec![
                spawn(1, PersonOrigin::ExternalEntry),
                spawn(2, PersonOrigin::ExternalEntry),
                spawn(3, PersonOrigin::ExternalEntry),
            ],
        )
        .unwrap();
    ledger
        .apply_tick(
            1,
            vec![
                IdentityCommand::AddRelationship {
                    event_id: RelationshipEventId::new(1),
                    kind: RelationshipKind::BirthParentOf,
                    from: p1,
                    to: p2,
                },
                IdentityCommand::AddRelationship {
                    event_id: RelationshipEventId::new(2),
                    kind: RelationshipKind::BirthParentOf,
                    from: p2,
                    to: p3,
                },
                IdentityCommand::AddRelationship {
                    event_id: RelationshipEventId::new(3),
                    kind: RelationshipKind::AssociatedWith,
                    from: p1,
                    to: p2,
                },
                IdentityCommand::AddRelationship {
                    event_id: RelationshipEventId::new(4),
                    kind: RelationshipKind::AssociatedWith,
                    from: p2,
                    to: p1,
                },
            ],
        )
        .unwrap();
    let before = ledger.checkpoint().unwrap();
    assert_eq!(
        ledger.apply_tick(
            2,
            vec![IdentityCommand::AddRelationship {
                event_id: RelationshipEventId::new(5),
                kind: RelationshipKind::BirthParentOf,
                from: p3,
                to: p1
            }]
        ),
        Err(IdentityError::InvalidRelationship)
    );
    assert_eq!(ledger.checkpoint().unwrap(), before);
    ledger
        .apply_tick(
            2,
            vec![IdentityCommand::RemoveRelationship {
                event_id: RelationshipEventId::new(6),
                kind: RelationshipKind::BirthParentOf,
                from: p1,
                to: p2,
            }],
        )
        .unwrap();
    assert!(ledger.person(p1).is_some());
    assert!(ledger.person(p2).is_some());
    assert!(ledger.person(p3).is_some());
    assert_eq!(ledger.relationships().len(), 3);
}

#[test]
fn late_events_and_birth_ordinal_collisions_reject_atomically() {
    let mut ledger = IdentityLedger::new(9);
    ledger
        .apply_tick(4, vec![spawn(1, PersonOrigin::ExternalEntry)])
        .unwrap();
    let before = ledger.checkpoint().unwrap();
    assert_eq!(
        ledger.apply_tick(4, vec![spawn(2, PersonOrigin::ExternalEntry)]),
        Err(IdentityError::LateEvent)
    );
    assert_eq!(ledger.checkpoint().unwrap(), before);
    ledger
        .apply_tick(
            5,
            vec![spawn(
                2,
                PersonOrigin::Birth {
                    birth_event_id: BirthEventId::new(3),
                    infant_ordinal: 0,
                },
            )],
        )
        .unwrap();
    let before = ledger.checkpoint().unwrap();
    assert_eq!(
        ledger.apply_tick(
            6,
            vec![spawn(
                3,
                PersonOrigin::Birth {
                    birth_event_id: BirthEventId::new(3),
                    infant_ordinal: 0
                }
            )]
        ),
        Err(IdentityError::BirthOrdinalExists)
    );
    assert_eq!(ledger.checkpoint().unwrap(), before);
}

#[test]
fn checkpoint_roundtrip_and_suffix_match_uninterrupted_ledger() {
    let mut original = IdentityLedger::new(29);
    let mother = person(29, 1);
    let infant = person(29, 2);
    let episode_id = episode(29, 10);
    original
        .apply_tick(
            0,
            vec![
                spawn(1, PersonOrigin::ExternalEntry),
                spawn(
                    2,
                    PersonOrigin::Birth {
                        birth_event_id: BirthEventId::new(3),
                        infant_ordinal: 0,
                    },
                ),
                enter(10, mother, 4),
                enter(11, infant, 4),
            ],
        )
        .unwrap();
    original
        .apply_tick(
            1,
            vec![IdentityCommand::AddRelationship {
                event_id: RelationshipEventId::new(7),
                kind: RelationshipKind::BirthParentOf,
                from: mother,
                to: infant,
            }],
        )
        .unwrap();
    assert!(original.episode(episode_id).is_some());
    let bytes = original.checkpoint().unwrap();
    let mut restored = IdentityLedger::restore(&bytes).unwrap();
    assert_eq!(restored, original);
    let suffix = vec![IdentityCommand::ExitService {
        event_id: LifecycleEventId::new(12),
        episode_id,
    }];
    original.apply_tick(2, suffix.clone()).unwrap();
    restored.apply_tick(2, suffix).unwrap();
    assert_eq!(restored, original);
}

#[test]
fn checkpoint_rejects_relationship_state_not_reconstructible_from_events() {
    let mut ledger = IdentityLedger::new(31);
    let first = person(31, 1);
    let second = person(31, 2);
    let third = person(31, 3);
    ledger
        .apply_tick(
            0,
            vec![
                spawn(1, PersonOrigin::ExternalEntry),
                spawn(2, PersonOrigin::ExternalEntry),
                spawn(3, PersonOrigin::ExternalEntry),
            ],
        )
        .unwrap();
    ledger
        .apply_tick(
            1,
            vec![IdentityCommand::AddRelationship {
                event_id: RelationshipEventId::new(1),
                kind: RelationshipKind::AssociatedWith,
                from: first,
                to: second,
            }],
        )
        .unwrap();
    let active: serde_json::Value = serde_json::from_slice(&ledger.checkpoint().unwrap()).unwrap();
    let mut omitted_added_edge = active.clone();
    omitted_added_edge["relationships"] = serde_json::json!([]);
    assert!(IdentityLedger::restore(&serde_json::to_vec(&omitted_added_edge).unwrap()).is_err());
    let mut unlogged_edge = active.clone();
    unlogged_edge["relationships"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "kind": "associated_with", "from": first.value(), "to": third.value()
        }));
    assert!(IdentityLedger::restore(&serde_json::to_vec(&unlogged_edge).unwrap()).is_err());

    ledger
        .apply_tick(
            2,
            vec![IdentityCommand::RemoveRelationship {
                event_id: RelationshipEventId::new(2),
                kind: RelationshipKind::AssociatedWith,
                from: first,
                to: second,
            }],
        )
        .unwrap();
    let mut retained_removed_edge: serde_json::Value =
        serde_json::from_slice(&ledger.checkpoint().unwrap()).unwrap();
    retained_removed_edge["relationships"] = serde_json::json!([{
        "kind": "associated_with", "from": first.value(), "to": second.value()
    }]);
    assert!(IdentityLedger::restore(&serde_json::to_vec(&retained_removed_edge).unwrap()).is_err());
}

#[test]
fn checkpoint_replays_lifecycle_and_rejects_unlogged_or_duplicate_transitions() {
    let mut ledger = IdentityLedger::new(41);
    let person_id = person(41, 1);
    let episode_id = episode(41, 10);
    ledger
        .apply_tick(
            0,
            vec![
                spawn(1, PersonOrigin::ExternalEntry),
                enter(10, person_id, 5),
            ],
        )
        .unwrap();
    let open_state: serde_json::Value =
        serde_json::from_slice(&ledger.checkpoint().unwrap()).unwrap();

    let mut unlogged_death = open_state.clone();
    let person_key = person_id.value().to_string();
    let episode_key = episode_id.value().to_string();
    unlogged_death["people"][person_key.as_str()]["status"] = serde_json::json!("deceased");
    assert!(IdentityLedger::restore(&serde_json::to_vec(&unlogged_death).unwrap()).is_err());

    let mut unlogged_exit = open_state;
    unlogged_exit["episodes"][episode_key.as_str()]["exited_at"] = serde_json::json!(1);
    assert!(IdentityLedger::restore(&serde_json::to_vec(&unlogged_exit).unwrap()).is_err());

    ledger
        .apply_tick(
            1,
            vec![IdentityCommand::ExitService {
                event_id: LifecycleEventId::new(20),
                episode_id,
            }],
        )
        .unwrap();
    ledger
        .apply_tick(
            2,
            vec![IdentityCommand::Death {
                event_id: LifecycleEventId::new(21),
                person_id,
            }],
        )
        .unwrap();
    let valid: serde_json::Value = serde_json::from_slice(&ledger.checkpoint().unwrap()).unwrap();

    let mut duplicate_exit = valid.clone();
    let mut exit = duplicate_exit["events"]["lifecycle:20"].clone();
    exit["command"]["event_id"] = serde_json::json!(22);
    duplicate_exit["events"]["lifecycle:22"] = exit;
    assert!(IdentityLedger::restore(&serde_json::to_vec(&duplicate_exit).unwrap()).is_err());

    let mut duplicate_death = valid;
    let mut death = duplicate_death["events"]["lifecycle:21"].clone();
    death["command"]["event_id"] = serde_json::json!(23);
    duplicate_death["events"]["lifecycle:23"] = death;
    assert!(IdentityLedger::restore(&serde_json::to_vec(&duplicate_death).unwrap()).is_err());
}

#[test]
fn checkpoint_preserves_watermark_advanced_by_later_exact_retry() {
    let mut ledger = IdentityLedger::new(43);
    let person_id = person(43, 1);
    let command = spawn(1, PersonOrigin::ExternalEntry);
    ledger.apply_tick(0, vec![command.clone()]).unwrap();
    assert_eq!(
        ledger.apply_tick(5, vec![command]).unwrap(),
        [IdentityOutcome::Duplicate]
    );
    assert_eq!(ledger.sealed_through(), Some(5));

    let mut restored = IdentityLedger::restore(&ledger.checkpoint().unwrap()).unwrap();
    assert_eq!(restored, ledger);
    assert_eq!(restored.sealed_through(), Some(5));
    assert_eq!(
        restored.apply_tick(5, vec![enter(10, person_id, 1)]),
        Err(IdentityError::LateEvent)
    );
}
