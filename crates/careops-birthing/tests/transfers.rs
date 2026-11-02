use careops_birthing::{delivery::*, team::*, transfers::*};
use careops_core::capacity::{Calendar, CapabilitySet, UtcInterval};
use careops_core::staffing::{BookingId, DutyKind, EscortChainId, StaffId};
use careops_core::{birth::*, identity, transfer::*};
use std::collections::BTreeMap;

fn fixture() -> (
    identity::IdentityLedger,
    DeliveryLedger,
    RoutingProfile,
    Vec<RecipientPlan>,
    TeamAllocator,
) {
    let mut ids = identity::IdentityLedger::new(61);
    let mother = identity::person_id_for_spawn(61, identity::SpawnEventId::new(1)).unwrap();
    ids.apply_tick(
        0,
        vec![
            identity::IdentityCommand::Spawn {
                event_id: identity::SpawnEventId::new(1),
                origin: identity::PersonOrigin::ExternalEntry,
            },
            identity::IdentityCommand::EnterService {
                event_id: identity::EpisodeEventId::new(10),
                person_id: mother,
                service_id: ServiceId::new(1),
            },
        ],
    )
    .unwrap();
    let infants: Vec<_> = (0..2)
        .map(|n| BirthInfantInput {
            infant_ordinal: n,
            spawn_event_id: identity::SpawnEventId::new(2 + u64::from(n)),
            relationship_event_id: identity::RelationshipEventId::new(101 + u64::from(n)),
            destination_service: ServiceId::new(3),
        })
        .collect();
    // One canonical identity tick includes all births, links and newborn entries.
    // Delivery registration then consumes its exact birth-command retries.
    let mut commands = Vec::new();
    let mut people = vec![mother];
    for infant in &infants {
        let person = identity::person_id_for_spawn(61, infant.spawn_event_id).unwrap();
        people.push(person);
        commands.push(identity::IdentityCommand::Spawn {
            event_id: infant.spawn_event_id,
            origin: identity::PersonOrigin::Birth {
                birth_event_id: identity::BirthEventId::new(5),
                infant_ordinal: infant.infant_ordinal,
            },
        });
        commands.push(identity::IdentityCommand::AddRelationship {
            event_id: infant.relationship_event_id,
            kind: identity::RelationshipKind::BirthParentOf,
            from: mother,
            to: person,
        });
        commands.push(identity::IdentityCommand::EnterService {
            event_id: identity::EpisodeEventId::new(11 + u64::from(infant.infant_ordinal)),
            person_id: person,
            service_id: ServiceId::new(1),
        });
    }
    ids.apply_tick(1, commands).unwrap();
    let mut delivery = DeliveryLedger::default();
    delivery
        .register(
            &mut ids,
            1,
            DeliveryRecord {
                route: DeliveryRoute::Operative,
                birth: BirthMessageInput {
                    message_id: BirthMessageId::new(4),
                    birth_event_id: identity::BirthEventId::new(5),
                    mother_person_id: mother,
                    maternal_destination_service: ServiceId::new(2),
                    infants,
                },
                observations: Vec::new(),
            },
        )
        .unwrap();
    let profile = RoutingProfile {
        revision: "synthetic:v1".into(),
        resources: BTreeMap::from([
            (ResourceId::new(10), "bed".into()),
            (ResourceId::new(11), "cot".into()),
            (ResourceId::new(12), "cot".into()),
            (ResourceId::new(20), "bed".into()),
            (ResourceId::new(30), "cot".into()),
            (ResourceId::new(31), "cot".into()),
            (ResourceId::new(40), "seat".into()),
        ]),
        destinations: BTreeMap::from([
            (ServiceId::new(2), destination("ward:v1", vec![20])),
            (ServiceId::new(3), destination("neonatal:v1", vec![30, 31])),
            (ServiceId::new(4), destination("retrieval:v1", vec![40])),
        ]),
    };
    let plans = people
        .iter()
        .enumerate()
        .map(|(n, person)| RecipientPlan {
            operation: TransferOperationId::new(1 + n as u64),
            person: *person,
            source_episode: identity::episode_id_for_entry(
                61,
                identity::EpisodeEventId::new(10 + n as u64),
            )
            .unwrap(),
            origin_resources: vec![ResourceId::new(10 + n as u64)],
            request_expires_at: None,
            fallback_services: vec![ServiceId::new(4)],
            escort: if n == 2 {
                Some(EscortBinding {
                    staff: 7,
                    booking: 70,
                    chain: 700,
                })
            } else {
                None
            },
        })
        .collect();
    let mut team = TeamAllocator::new(vec![MemberProfile {
        staff: StaffId::new(7),
        revision: "escort:v1".into(),
        roles: CapabilitySet::new(["escort"]).unwrap(),
        roster: Calendar::new(vec![UtcInterval::new(0, 100).unwrap()]).unwrap(),
        unavailable: Calendar::default(),
        overtime_per_shift_seconds: 0,
        call_in_latency_seconds: 0,
        call_windows: None,
    }])
    .unwrap();
    team.allocate(TeamRequest {
        booking: BookingId::new(70),
        patients: 1,
        profile: RatioProfile {
            revision: "escort:v1".into(),
            cohort_unit: "synthetic_person".into(),
            patients_per_staff: BTreeMap::from([("escort".into(), 1)]),
        },
        interval: UtcInterval::new(0, 20).unwrap(),
        kind: DutyKind::Escort {
            chain: EscortChainId::new(700),
        },
    })
    .unwrap();
    (ids, delivery, profile, plans, team)
}
fn destination(revision: &str, resources: Vec<u64>) -> DestinationProfile {
    DestinationProfile {
        revision: revision.into(),
        resources: resources.into_iter().map(ResourceId::new).collect(),
        availability: Some(vec![AvailabilityWindow { start: 0, end: 100 }]),
    }
}
fn env(op: u64, part: u64, revision: u64, tick: u64, command: TransferCommand) -> TransferEnvelope {
    TransferEnvelope::new(
        TransferOperationId::new(op),
        MessageId::new(op * 100 + part),
        revision,
        tick,
        command,
    )
}
fn bind(
    ids: &identity::IdentityLedger,
    delivery: &DeliveryLedger,
    profile: RoutingProfile,
    plans: Vec<RecipientPlan>,
) -> TransferBoundary {
    TransferBoundary::bind_delivery(delivery, BirthMessageId::new(4), ids, profile, plans).unwrap()
}
fn prefix(
    boundary: &mut TransferBoundary,
    op: u64,
    resource: u64,
    ids: &identity::IdentityLedger,
    team: &TeamAllocator,
) {
    boundary
        .request(
            TransferOperationId::new(op),
            MessageId::new(op * 100),
            1,
            ids,
            team,
        )
        .unwrap();
    boundary
        .apply(
            env(op, 1, 1, 2, TransferCommand::Offer { expires_at: None }),
            ids,
            team,
        )
        .unwrap();
    boundary
        .apply(
            env(
                op,
                2,
                2,
                3,
                TransferCommand::Reserve {
                    resources: vec![ResourceId::new(resource)],
                    expires_at: None,
                },
            ),
            ids,
            team,
        )
        .unwrap();
}

#[test]
fn same_tick_birth_entries_and_split_destinations_conserve_people_episodes_and_claim_units() {
    let (mut ids, delivery, profile, plans, team) = fixture();
    let mother_episode = plans[0].source_episode;
    let mut boundary = bind(&ids, &delivery, profile, plans.clone());
    assert_eq!(ids.counts().people_created, 3);
    assert_eq!(ids.counts().births, 2);
    assert_eq!(ids.counts().service_entries, 3);
    for op in 1..=3 {
        boundary
            .request(
                TransferOperationId::new(op),
                MessageId::new(op * 100),
                1,
                &ids,
                &team,
            )
            .unwrap();
    }
    for op in 1..=3 {
        boundary
            .apply(
                env(op, 1, 1, 2, TransferCommand::Offer { expires_at: None }),
                &ids,
                &team,
            )
            .unwrap();
    }
    for (op, resource) in [(1, 20), (2, 30), (3, 31)] {
        boundary
            .apply(
                env(
                    op,
                    2,
                    2,
                    3,
                    TransferCommand::Reserve {
                        resources: vec![ResourceId::new(resource)],
                        expires_at: None,
                    },
                ),
                &ids,
                &team,
            )
            .unwrap();
    }
    for (part, revision, tick, command) in [
        (3, 3, 4, TransferCommand::BeginDeparture),
        (4, 4, 5, TransferCommand::Depart),
        (5, 5, 6, TransferCommand::Arrive),
        (6, 6, 7, TransferCommand::CompleteHandover),
    ] {
        for op in 1..=3 {
            boundary
                .apply(env(op, part, revision, tick, command.clone()), &ids, &team)
                .unwrap();
        }
    }
    assert_eq!(boundary.core().claims().len(), 3);
    assert_eq!(boundary.resource_unit(ResourceId::new(20)), Some("bed"));
    assert_eq!(boundary.resource_unit(ResourceId::new(30)), Some("cot"));
    for plan in &plans {
        let person = boundary
            .core()
            .person(PersonId::new(plan.person.value()))
            .unwrap();
        assert_eq!(person.episodes.len(), 1);
        assert_eq!(
            person.owner_service,
            ServiceId::new(if plan.operation == TransferOperationId::new(1) {
                2
            } else {
                3
            })
        );
    }
    ids.apply_tick(
        8,
        vec![identity::IdentityCommand::ExitService {
            event_id: identity::LifecycleEventId::new(500),
            episode_id: mother_episode,
        }],
    )
    .unwrap();
    boundary
        .apply(
            env(1, 6, 6, 7, TransferCommand::CompleteHandover),
            &ids,
            &team,
        )
        .unwrap();
    assert_eq!(ids.counts().service_exits, 1);
    assert!(plans[1..]
        .iter()
        .all(|p| ids.episode(p.source_episode).unwrap().status == identity::EpisodeStatus::Open));
    let checkpoint = boundary.core().checkpoint().unwrap();
    let restored = TransferManager::restore(&checkpoint).unwrap();
    assert_eq!(restored.checkpoint().unwrap(), checkpoint); // core ledger scope only
}

#[test]
fn unavailable_or_undeclared_destination_capacity_never_grants_and_cancel_retains_origin() {
    for unknown in [false, true] {
        let (ids, delivery, mut profile, plans, team) = fixture();
        profile
            .destinations
            .get_mut(&ServiceId::new(3))
            .unwrap()
            .availability = if unknown {
            None
        } else {
            Some(vec![AvailabilityWindow { start: 10, end: 20 }])
        };
        let mut boundary = bind(&ids, &delivery, profile, plans);
        boundary
            .request(
                TransferOperationId::new(2),
                MessageId::new(200),
                1,
                &ids,
                &team,
            )
            .unwrap();
        boundary
            .apply(
                env(2, 1, 1, 2, TransferCommand::Offer { expires_at: None }),
                &ids,
                &team,
            )
            .unwrap();
        let before = boundary.core().checkpoint().unwrap();
        assert!(boundary
            .apply(
                env(
                    2,
                    2,
                    2,
                    3,
                    TransferCommand::Reserve {
                        resources: vec![ResourceId::new(30)],
                        expires_at: None
                    }
                ),
                &ids,
                &team
            )
            .is_err());
        assert_eq!(boundary.core().checkpoint().unwrap(), before);
        boundary
            .apply(env(2, 3, 2, 4, TransferCommand::Cancel), &ids, &team)
            .unwrap();
        assert_eq!(
            boundary
                .core()
                .snapshot(TransferOperationId::new(2))
                .unwrap()
                .state,
            TransferState::Cancelled
        );
        assert_eq!(
            boundary.core().claims()[0].disposition,
            ClaimDisposition::Occupied
        );
    }
}

#[test]
fn expiry_retains_origin_and_new_attempt_preserves_the_same_person_and_episode() {
    let (ids, delivery, profile, mut plans, team) = fixture();
    let mut retry = plans[1].clone();
    retry.operation = TransferOperationId::new(9);
    plans.push(retry);
    let mut boundary = bind(&ids, &delivery, profile, plans);
    boundary
        .request(
            TransferOperationId::new(2),
            MessageId::new(200),
            1,
            &ids,
            &team,
        )
        .unwrap();
    boundary
        .apply(
            env(
                2,
                1,
                1,
                2,
                TransferCommand::Offer {
                    expires_at: Some(4),
                },
            ),
            &ids,
            &team,
        )
        .unwrap();
    boundary
        .apply(
            env(
                2,
                2,
                2,
                3,
                TransferCommand::Reserve {
                    resources: vec![ResourceId::new(30)],
                    expires_at: Some(5),
                },
            ),
            &ids,
            &team,
        )
        .unwrap();
    boundary.advance_to(5, &ids, &team).unwrap();
    assert_eq!(
        boundary
            .core()
            .snapshot(TransferOperationId::new(2))
            .unwrap()
            .state,
        TransferState::Expired
    );
    assert!(boundary.resource_unclaimed(ResourceId::new(30)).unwrap());
    assert!(!boundary.resource_unclaimed(ResourceId::new(11)).unwrap());
    boundary
        .request(
            TransferOperationId::new(9),
            MessageId::new(900),
            6,
            &ids,
            &team,
        )
        .unwrap();
    for (part, revision, tick, command) in [
        (1, 1, 7, TransferCommand::Offer { expires_at: None }),
        (
            2,
            2,
            8,
            TransferCommand::Reserve {
                resources: vec![ResourceId::new(30)],
                expires_at: None,
            },
        ),
        (3, 3, 9, TransferCommand::BeginDeparture),
        (4, 4, 10, TransferCommand::Depart),
        (5, 5, 11, TransferCommand::Arrive),
        (6, 6, 12, TransferCommand::CompleteHandover),
    ] {
        boundary
            .apply(env(9, part, revision, tick, command), &ids, &team)
            .unwrap();
    }
    let snapshot = boundary
        .core()
        .snapshot(TransferOperationId::new(9))
        .unwrap();
    assert_eq!(snapshot.state, TransferState::HandoverComplete);
    assert_eq!(
        boundary
            .core()
            .person(snapshot.person_id)
            .unwrap()
            .episodes
            .len(),
        1
    );
}

#[test]
fn buffered_acknowledgements_fallback_and_escort_coverage_are_checked_on_actual_effects() {
    let (ids, delivery, profile, plans, team) = fixture();
    let mut boundary = bind(&ids, &delivery, profile, plans);
    prefix(&mut boundary, 3, 31, &ids, &team);
    let empty_team = TeamAllocator::default();
    let before = boundary.core().checkpoint().unwrap();
    // Future Depart buffers; when BeginDeparture releases it, escort must exist.
    assert_eq!(
        boundary
            .apply(env(3, 4, 4, 5, TransferCommand::Depart), &ids, &empty_team)
            .unwrap(),
        ApplyOutcome::Buffered
    );
    let buffered = boundary.core().checkpoint().unwrap();
    assert_ne!(buffered, before);
    assert!(boundary
        .apply(
            env(3, 3, 3, 4, TransferCommand::BeginDeparture),
            &ids,
            &empty_team
        )
        .is_err());
    assert_eq!(boundary.core().checkpoint().unwrap(), buffered);
    boundary
        .apply(
            env(3, 3, 3, 4, TransferCommand::BeginDeparture),
            &ids,
            &team,
        )
        .unwrap();
    assert_eq!(
        boundary
            .core()
            .snapshot(TransferOperationId::new(3))
            .unwrap()
            .state,
        TransferState::InTransit
    );
    // The qualified core supports rejection after physical arrival, before
    // acknowledgement transfers ownership. In-transit diversion is not its v1 contract.
    boundary
        .apply(env(3, 5, 5, 6, TransferCommand::Arrive), &ids, &team)
        .unwrap();
    let bad = env(
        3,
        7,
        6,
        6,
        TransferCommand::DestinationRejected {
            compensation: CompensationRequest {
                operation_id: TransferOperationId::new(4),
                destination_service: ServiceId::new(99),
                request_expires_at: None,
            },
        },
    );
    assert!(boundary.apply(bad, &ids, &team).is_err());
    boundary
        .apply(
            env(
                3,
                7,
                6,
                6,
                TransferCommand::DestinationRejected {
                    compensation: CompensationRequest {
                        operation_id: TransferOperationId::new(4),
                        destination_service: ServiceId::new(4),
                        request_expires_at: None,
                    },
                },
            ),
            &ids,
            &team,
        )
        .unwrap();
    for (part, revision, tick, command) in [
        (1, 1, 7, TransferCommand::Offer { expires_at: None }),
        (
            2,
            2,
            8,
            TransferCommand::Reserve {
                resources: vec![ResourceId::new(40)],
                expires_at: None,
            },
        ),
        (3, 3, 9, TransferCommand::BeginDeparture),
        (4, 4, 10, TransferCommand::Depart),
        (5, 5, 11, TransferCommand::Arrive),
        (6, 6, 12, TransferCommand::CompleteHandover),
    ] {
        boundary
            .apply(env(4, part, revision, tick, command), &ids, &team)
            .unwrap();
    }
    let child = boundary
        .core()
        .snapshot(TransferOperationId::new(4))
        .unwrap();
    assert_eq!(child.owner_service, ServiceId::new(4));
    assert_eq!(child.state, TransferState::HandoverComplete);
    assert!(boundary.resource_unclaimed(ResourceId::new(12)).unwrap());
    let count = boundary.core().events().len();
    boundary
        .apply(
            env(4, 6, 6, 12, TransferCommand::CompleteHandover),
            &ids,
            &team,
        )
        .unwrap();
    assert_eq!(boundary.core().events().len(), count);
}

#[test]
fn malformed_identity_profiles_and_compensation_aliases_are_rejected_atomically() {
    let (ids, delivery, profile, plans, team) = fixture();
    let mut wrong = plans.clone();
    wrong[1].source_episode = plans[0].source_episode;
    assert!(TransferBoundary::bind_delivery(
        &delivery,
        BirthMessageId::new(4),
        &ids,
        profile.clone(),
        wrong
    )
    .is_err());
    let mut malformed = profile.clone();
    malformed
        .resources
        .insert(ResourceId::new(10), String::new());
    assert!(TransferBoundary::bind_delivery(
        &delivery,
        BirthMessageId::new(4),
        &ids,
        malformed,
        plans.clone()
    )
    .is_err());
    let mut boundary = bind(&ids, &delivery, profile, plans);
    prefix(&mut boundary, 3, 31, &ids, &team);
    boundary
        .apply(
            env(3, 3, 3, 4, TransferCommand::BeginDeparture),
            &ids,
            &team,
        )
        .unwrap();
    boundary
        .apply(env(3, 4, 4, 5, TransferCommand::Depart), &ids, &team)
        .unwrap();
    let before = boundary.core().checkpoint().unwrap();
    assert!(boundary
        .apply(
            env(
                3,
                7,
                5,
                6,
                TransferCommand::DestinationRejected {
                    compensation: CompensationRequest {
                        operation_id: TransferOperationId::new(1),
                        destination_service: ServiceId::new(4),
                        request_expires_at: None
                    }
                }
            ),
            &ids,
            &team
        )
        .is_err());
    assert_eq!(boundary.core().checkpoint().unwrap(), before);
}
