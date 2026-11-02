use careops_birthing::delivery::*;
use careops_core::birth::*;
use careops_core::identity::*;
use careops_core::transfer::ServiceId;

fn fixture(route: DeliveryRoute) -> (IdentityLedger, DeliveryRecord) {
    let mut ids = IdentityLedger::new(61);
    ids.apply_tick(
        0,
        vec![IdentityCommand::Spawn {
            event_id: SpawnEventId::new(1),
            origin: PersonOrigin::ExternalEntry,
        }],
    )
    .unwrap();
    let mother = person_id_for_spawn(61, SpawnEventId::new(1)).unwrap();
    let infants = (0..2)
        .map(|ordinal| BirthInfantInput {
            infant_ordinal: ordinal,
            spawn_event_id: SpawnEventId::new(2 + u64::from(ordinal)),
            relationship_event_id: RelationshipEventId::new(10 + u64::from(ordinal)),
            destination_service: ServiceId::new(20 + u64::from(ordinal)),
        })
        .collect();
    (
        ids,
        DeliveryRecord {
            route,
            birth: BirthMessageInput {
                message_id: BirthMessageId::new(4),
                birth_event_id: BirthEventId::new(5),
                mother_person_id: mother,
                maternal_destination_service: ServiceId::new(19),
                infants,
            },
            observations: (0..2)
                .map(|infant_ordinal| NeonatalObservation {
                    infant_ordinal,
                    observed_tick: 1,
                    available_tick: 2,
                    variable: "synthetic_measure".into(),
                    value: 7,
                    unit: "fixture_unit".into(),
                    provenance: "synthetic:b2:v1".into(),
                })
                .collect(),
        },
    )
}

#[test]
fn both_routes_register_correlated_twins_and_reject_changed_duplicate_atomically() {
    for route in [DeliveryRoute::Vaginal, DeliveryRoute::Operative] {
        let (mut ids, record) = fixture(route);
        let mut deliveries = DeliveryLedger::default();
        let result = deliveries.register(&mut ids, 1, record.clone()).unwrap();
        assert_eq!(result.recipients.len(), 3);
        assert_eq!(ids.counts().births, 2);
        assert_eq!(ids.relationships().len(), 2);
        let before = ids.checkpoint().unwrap();
        let mut shuffled = record.clone();
        shuffled.birth.infants.reverse();
        shuffled.observations.reverse();
        assert_eq!(deliveries.register(&mut ids, 1, shuffled).unwrap(), result);
        let mut conflict = record.clone();
        conflict.route = if route == DeliveryRoute::Vaginal {
            DeliveryRoute::Operative
        } else {
            DeliveryRoute::Vaginal
        };
        assert!(deliveries.register(&mut ids, 2, conflict).is_err());
        assert_eq!(ids.checkpoint().unwrap(), before);
        assert_eq!(deliveries.record(record.birth.message_id).unwrap(), &record);
        assert!(!deliveries.models_fetal_identity());
    }
}

#[test]
fn malformed_provenance_or_ordinal_does_not_spawn_partial_infants() {
    for mode in 0..4 {
        let (mut ids, mut record) = fixture(DeliveryRoute::Vaginal);
        match mode {
            0 => record.observations[0].provenance.clear(),
            1 => record.observations[0].infant_ordinal = 99,
            2 => record.observations[0].available_tick = 0,
            _ => record.observations.push(record.observations[0].clone()),
        }
        let before = ids.checkpoint().unwrap();
        let mut deliveries = DeliveryLedger::default();
        assert!(deliveries.register(&mut ids, 1, record).is_err());
        assert_eq!(ids.checkpoint().unwrap(), before);
    }
}

#[test]
fn changed_observation_retry_and_core_rejection_preserve_all_delivery_state() {
    let (mut ids, record) = fixture(DeliveryRoute::Vaginal);
    let mut deliveries = DeliveryLedger::default();
    deliveries.register(&mut ids, 1, record.clone()).unwrap();
    let before_ids = ids.checkpoint().unwrap();
    let before_record = deliveries.record(record.birth.message_id).unwrap().clone();
    let before_state = deliveries.state(record.birth.message_id, record.birth.mother_person_id);
    let mut changed = record.clone();
    changed.observations[0].value += 1;
    assert!(deliveries.register(&mut ids, 2, changed).is_err());
    let mut rejected = record.clone();
    rejected.birth.message_id = BirthMessageId::new(99);
    rejected.birth.birth_event_id = BirthEventId::new(100);
    // Reusing the original spawn keys must fail in the core, after local validation.
    assert!(deliveries.register(&mut ids, 2, rejected).is_err());
    assert_eq!(ids.checkpoint().unwrap(), before_ids);
    assert_eq!(
        deliveries.record(record.birth.message_id),
        Some(&before_record)
    );
    assert!(deliveries.record(BirthMessageId::new(99)).is_none());
    assert_eq!(
        deliveries.state(record.birth.message_id, record.birth.mother_person_id),
        before_state
    );
}

#[test]
fn maternal_discharge_preserves_infant_recovery_escalation_and_episode_counts() {
    let (mut ids, record) = fixture(DeliveryRoute::Operative);
    let mut deliveries = DeliveryLedger::default();
    let result = deliveries.register(&mut ids, 1, record).unwrap();
    let people: Vec<_> = result
        .recipients
        .iter()
        .map(|recipient| match recipient {
            BirthRecipient::Mother { person_id, .. } | BirthRecipient::Infant { person_id, .. } => {
                *person_id
            }
        })
        .collect();
    ids.apply_tick(
        2,
        people
            .iter()
            .enumerate()
            .map(|(n, person)| IdentityCommand::EnterService {
                event_id: EpisodeEventId::new(30 + n as u64),
                person_id: *person,
                service_id: ServiceId::new(9),
            })
            .collect(),
    )
    .unwrap();
    deliveries
        .transition(
            BirthMessageId::new(4),
            people[0],
            3,
            RecoveryState::Discharged,
        )
        .unwrap();
    deliveries
        .transition(
            BirthMessageId::new(4),
            people[1],
            3,
            RecoveryState::Escalated,
        )
        .unwrap();
    assert_eq!(
        deliveries.state(BirthMessageId::new(4), people[2]),
        Some((1, RecoveryState::Recovering))
    );
    assert_eq!(
        deliveries.state(BirthMessageId::new(4), people[1]),
        Some((3, RecoveryState::Escalated))
    );
    assert!(deliveries
        .transition(
            BirthMessageId::new(4),
            people[0],
            4,
            RecoveryState::Recovering
        )
        .is_err());
    assert!(deliveries
        .transition(
            BirthMessageId::new(4),
            people[1],
            2,
            RecoveryState::Discharged
        )
        .is_err());
    ids.apply_tick(
        3,
        vec![IdentityCommand::ExitService {
            event_id: LifecycleEventId::new(40),
            episode_id: episode_id_for_entry(61, EpisodeEventId::new(30)).unwrap(),
        }],
    )
    .unwrap();
    assert_eq!(ids.counts().people_created, 3);
    assert_eq!(ids.counts().service_entries - ids.counts().service_exits, 2);
    assert_eq!(ids.person(people[1]).unwrap().status, PersonStatus::Living);
    assert_eq!(ids.relationships().len(), 2);
}

#[test]
fn later_delivery_has_new_recovery_without_reopening_previous_delivery() {
    let (mut ids, record) = fixture(DeliveryRoute::Vaginal);
    let mother = record.birth.mother_person_id;
    let mut deliveries = DeliveryLedger::default();
    deliveries.register(&mut ids, 1, record.clone()).unwrap();
    deliveries
        .transition(BirthMessageId::new(4), mother, 2, RecoveryState::Discharged)
        .unwrap();
    let mut later = record;
    later.birth.message_id = BirthMessageId::new(104);
    later.birth.birth_event_id = BirthEventId::new(105);
    for infant in &mut later.birth.infants {
        infant.spawn_event_id = SpawnEventId::new(102 + u64::from(infant.infant_ordinal));
        infant.relationship_event_id =
            RelationshipEventId::new(110 + u64::from(infant.infant_ordinal));
    }
    for observation in &mut later.observations {
        observation.observed_tick = 10;
        observation.available_tick = 11;
    }
    deliveries.register(&mut ids, 10, later).unwrap();
    assert_eq!(
        deliveries.state(BirthMessageId::new(104), mother),
        Some((10, RecoveryState::Recovering))
    );
    assert_eq!(
        deliveries.state(BirthMessageId::new(4), mother),
        Some((2, RecoveryState::Discharged))
    );
    deliveries
        .transition(
            BirthMessageId::new(104),
            mother,
            11,
            RecoveryState::Escalated,
        )
        .unwrap();
    assert!(deliveries
        .transition(
            BirthMessageId::new(4),
            mother,
            12,
            RecoveryState::Recovering
        )
        .is_err());
    assert_eq!(ids.counts().births, 4);
}

#[test]
fn pre_birth_observation_rejects_without_creating_a_delivery() {
    let (mut ids, record) = fixture(DeliveryRoute::Vaginal);
    let before = ids.checkpoint().unwrap();
    let mut deliveries = DeliveryLedger::default();
    assert!(deliveries.register(&mut ids, 10, record.clone()).is_err());
    assert_eq!(ids.checkpoint().unwrap(), before);
    assert!(deliveries.record(record.birth.message_id).is_none());
    assert!(deliveries
        .state(record.birth.message_id, record.birth.mother_person_id)
        .is_none());
}
