use careops_core::identity::{BirthEventId, PersonOrigin, RelationshipKind, SpawnEventId};
use careops_core::transfer::{
    EpisodeId, MessageId, PersonId, ServiceId, TransferCommand, TransferEnvelope, TransferManager,
    TransferOperationId, TransferRequest,
};
use careops_ed::identity::{EdAdmissionInput, EdIdentityAdapter};
use careops_ed::pathway::{
    run_pathway, PathwayConfig, PathwayPatient, PathwayState, StageKind, StageWork,
};
use careops_ed::{
    AggregateCapacity, CapacityLocationProfile, Eligibility, KnowledgeStatus, Location, Provenance,
    ResourceBucket, StaffCount, StaffRole, TaskClass, Zone,
};

fn pathway_config(mother_key: String, infant_key: String) -> PathwayConfig {
    PathwayConfig {
        version: 1,
        horizon_ticks: 3,
        provenance: Provenance {
            class: "synthetic".into(),
            notes: "identity adapter test".into(),
        },
        capacity_location: CapacityLocationProfile {
            schema_version: 3,
            status: "proposed".into(),
            capacity: AggregateCapacity {
                unit: "treatment_space".into(),
                bucket_completeness: "complete".into(),
                physical_status: KnowledgeStatus::Known,
                physical_count: Some(2),
                availability_status: KnowledgeStatus::Known,
                open_count: Some(2),
                staffed_count: Some(2),
                availability_reason: None,
                equipment: vec![],
            },
            zones: vec![Zone {
                zone_id: "zone.care".into(),
                name: "Synthetic care".into(),
            }],
            locations: vec![Location {
                location_id: "loc.care".into(),
                name: "Synthetic care".into(),
                zone_id: "zone.care".into(),
            }],
            resource_buckets: ["triage", "assessment", "treatment"]
                .into_iter()
                .map(|stage| ResourceBucket {
                    resource_id: format!("res.{stage}"),
                    resource_class: format!("{stage}_space"),
                    location_id: "loc.care".into(),
                    zone_id: "zone.care".into(),
                    pool_group_id: None,
                    physical_status: KnowledgeStatus::Known,
                    physical_count: Some(2),
                    availability_status: KnowledgeStatus::Known,
                    open_count: Some(2),
                    staffed_count: Some(2),
                    availability_reason: None,
                    capabilities: vec![],
                })
                .collect(),
            task_classes: ["triage", "assessment", "treatment"]
                .into_iter()
                .map(|stage| TaskClass {
                    task_class_id: format!("task.{stage}"),
                    name: format!("Synthetic {stage}"),
                    required_resource_classes: vec![format!("{stage}_space")],
                })
                .collect(),
            staff_roles: vec![StaffRole {
                staff_role_id: "staff.care".into(),
                name: "Synthetic staff".into(),
                counts: vec![StaffCount {
                    basis: "present".into(),
                    status: KnowledgeStatus::Known,
                    count: Some(2),
                    reference: "synthetic.fixture".into(),
                    reason: None,
                }],
            }],
            eligibility: ["triage", "assessment", "treatment"]
                .into_iter()
                .map(|stage| Eligibility {
                    staff_role_id: "staff.care".into(),
                    task_class_id: format!("task.{stage}"),
                    zone_id: "zone.care".into(),
                    location_id: Some("loc.care".into()),
                })
                .collect(),
            routes: vec![],
        },
        patients: vec![
            PathwayPatient {
                patient_id: mother_key,
                arrival_tick: 0,
                mode: "synthetic_maternal".into(),
                acuity: 3,
                stages: vec![
                    StageWork {
                        kind: StageKind::Triage,
                        resource_id: "res.triage".into(),
                        task_class_id: "task.triage".into(),
                        duration_ticks: 1,
                    },
                    StageWork {
                        kind: StageKind::Assessment,
                        resource_id: "res.assessment".into(),
                        task_class_id: "task.assessment".into(),
                        duration_ticks: 1,
                    },
                    StageWork {
                        kind: StageKind::Treatment,
                        resource_id: "res.treatment".into(),
                        task_class_id: "task.treatment".into(),
                        duration_ticks: 1,
                    },
                ],
            },
            PathwayPatient {
                patient_id: infant_key,
                arrival_tick: 0,
                mode: "synthetic_infant".into(),
                acuity: 3,
                stages: vec![
                    StageWork {
                        kind: StageKind::Triage,
                        resource_id: "res.triage".into(),
                        task_class_id: "task.triage".into(),
                        duration_ticks: 1,
                    },
                    StageWork {
                        kind: StageKind::Assessment,
                        resource_id: "res.assessment".into(),
                        task_class_id: "task.assessment".into(),
                        duration_ticks: 1,
                    },
                    StageWork {
                        kind: StageKind::Treatment,
                        resource_id: "res.treatment".into(),
                        task_class_id: "task.treatment".into(),
                        duration_ticks: 5,
                    },
                ],
            },
        ],
    }
}

#[test]
fn ed_keys_are_stable_across_readmission_and_match_derivation_vectors() {
    let mut adapter = EdIdentityAdapter::new(42);
    let first = adapter
        .admit(
            0,
            EdAdmissionInput {
                source_person_key: "synthetic-patient-α".into(),
                admission_key: "admission-1".into(),
                service_id: ServiceId::new(1),
            },
        )
        .unwrap();
    assert_eq!(first.person_id.value(), 8_979_869_846_555_893_168);
    assert_eq!(first.episode_id.value(), 8_754_272_787_240_046_413);
    let readmission = adapter
        .admit(
            1,
            EdAdmissionInput {
                source_person_key: "synthetic-patient-α".into(),
                admission_key: "admission-2".into(),
                service_id: ServiceId::new(2),
            },
        )
        .unwrap();
    assert_eq!(readmission.person_id, first.person_id);
    assert_ne!(readmission.episode_id, first.episode_id);
    assert_eq!(
        EdIdentityAdapter::pathway_subject_key(first.episode_id),
        format!("episode.{}", first.episode_id.value())
    );
    let replay = adapter
        .admit(
            1,
            EdAdmissionInput {
                source_person_key: "synthetic-patient-α".into(),
                admission_key: "admission-2".into(),
                service_id: ServiceId::new(2),
            },
        )
        .unwrap();
    assert_eq!(replay, readmission);
    assert_eq!(
        EdIdentityAdapter::restore(&adapter.checkpoint().unwrap()).unwrap(),
        adapter
    );
}

#[test]
fn maternal_and_infant_have_independent_pathway_recovery_and_transfer_lifecycles() {
    let mut adapter = EdIdentityAdapter::new(42);
    let admissions = adapter
        .admit_batch(
            0,
            &[
                EdAdmissionInput {
                    source_person_key: "mother-1".into(),
                    admission_key: "mother-episode-1".into(),
                    service_id: ServiceId::new(10),
                },
                EdAdmissionInput {
                    source_person_key: "infant-1".into(),
                    admission_key: "infant-episode-1".into(),
                    service_id: ServiceId::new(10),
                },
            ],
        )
        .unwrap();
    let mother = admissions[0];
    let infant = admissions[1];
    assert_ne!(mother.person_id, infant.person_id);
    assert_ne!(mother.episode_id, infant.episode_id);

    // Birth relationship is typed; deleting it changes no person or episode record.
    adapter
        .add_birth_parent(1, 1, mother.person_id, infant.person_id)
        .unwrap();
    assert_eq!(
        adapter.ledger().relationships()[0].kind,
        RelationshipKind::BirthParentOf
    );
    adapter
        .remove_birth_parent(2, 2, mother.person_id, infant.person_id)
        .unwrap();
    assert!(adapter.ledger().person(mother.person_id).is_some());
    assert!(adapter.ledger().person(infant.person_id).is_some());
    assert!(adapter.episode(mother.episode_id).is_some());
    assert!(adapter.episode(infant.episode_id).is_some());

    let results = run_pathway(&pathway_config(
        EdIdentityAdapter::pathway_subject_key(mother.episode_id),
        EdIdentityAdapter::pathway_subject_key(infant.episode_id),
    ))
    .unwrap();
    let mother_result = results
        .iter()
        .find(|result| {
            result.patient_id == EdIdentityAdapter::pathway_subject_key(mother.episode_id)
        })
        .unwrap();
    let infant_result = results
        .iter()
        .find(|result| {
            result.patient_id == EdIdentityAdapter::pathway_subject_key(infant.episode_id)
        })
        .unwrap();
    assert_eq!(mother_result.state, PathwayState::TreatmentComplete);
    assert_eq!(mother_result.episodes[2].completion_tick, Some(3));
    assert_eq!(infant_result.episodes[2].completion_tick, None);
    assert!(matches!(
        infant_result.state,
        PathwayState::Active(StageKind::Treatment)
    ));

    // Each identity and episode can own an independent transfer operation.
    let mut transfers = TransferManager::default();
    for (operation, admission) in [(100, mother), (200, infant)] {
        transfers
            .apply(TransferEnvelope::new(
                TransferOperationId::new(operation),
                MessageId::new(operation + 1),
                0,
                0,
                TransferCommand::Request {
                    request: TransferRequest {
                        person_id: admission.person_id,
                        episode_id: admission.episode_id,
                        origin_service: ServiceId::new(10),
                        destination_service: ServiceId::new(11),
                        request_expires_at: None,
                        origin_resources: vec![],
                    },
                },
            ))
            .unwrap();
    }
    assert_eq!(
        transfers.person(mother.person_id).unwrap().active_operation,
        Some(TransferOperationId::new(100))
    );
    assert_eq!(
        transfers.person(infant.person_id).unwrap().active_operation,
        Some(TransferOperationId::new(200))
    );
    assert_ne!(
        PersonId::new(mother.person_id.value()),
        PersonId::new(infant.person_id.value())
    );
    assert_ne!(
        EpisodeId::new(mother.episode_id.value()),
        EpisodeId::new(infant.episode_id.value())
    );
}

#[test]
fn simultaneous_birth_children_are_distinct_people_and_same_parent_links_are_one_to_many() {
    let mut adapter = EdIdentityAdapter::new(7);
    let parent = adapter
        .admit(
            0,
            EdAdmissionInput {
                source_person_key: "parent".into(),
                admission_key: "parent-admission".into(),
                service_id: ServiceId::new(1),
            },
        )
        .unwrap();
    let infant_ids = adapter.ledger();
    assert!(infant_ids.person(parent.person_id).is_some());
    let mut ledger = careops_core::identity::IdentityLedger::new(7);
    let parent_id = PersonId::new(derive_person(7, 1));
    let one = PersonId::new(derive_person(7, 101));
    let two = PersonId::new(derive_person(7, 102));
    assert_ne!(one, two);
    ledger
        .apply_tick(
            0,
            vec![
                careops_core::identity::IdentityCommand::Spawn {
                    event_id: SpawnEventId::new(1),
                    origin: PersonOrigin::ExternalEntry,
                },
                careops_core::identity::IdentityCommand::Spawn {
                    event_id: SpawnEventId::new(101),
                    origin: PersonOrigin::Birth {
                        birth_event_id: BirthEventId::new(9),
                        infant_ordinal: 0,
                    },
                },
                careops_core::identity::IdentityCommand::Spawn {
                    event_id: SpawnEventId::new(102),
                    origin: PersonOrigin::Birth {
                        birth_event_id: BirthEventId::new(9),
                        infant_ordinal: 1,
                    },
                },
                careops_core::identity::IdentityCommand::AddRelationship {
                    event_id: careops_core::identity::RelationshipEventId::new(1),
                    kind: RelationshipKind::BirthParentOf,
                    from: parent_id,
                    to: one,
                },
                careops_core::identity::IdentityCommand::AddRelationship {
                    event_id: careops_core::identity::RelationshipEventId::new(2),
                    kind: RelationshipKind::BirthParentOf,
                    from: parent_id,
                    to: two,
                },
            ],
        )
        .unwrap();
    assert_eq!(ledger.relationships().len(), 2);
    assert!(ledger.person(parent_id).is_some());
    assert_eq!(ledger.counts().births, 2);
}

fn derive_person(run: u64, event: u64) -> u64 {
    careops_core::identity::person_id_for_spawn(run, SpawnEventId::new(event))
        .unwrap()
        .value()
}

#[test]
fn ed_rejects_empty_source_key_without_mutating_identity_state() {
    let mut adapter = EdIdentityAdapter::new(42);
    let before = adapter.checkpoint().unwrap();
    assert!(adapter
        .admit(
            0,
            EdAdmissionInput {
                source_person_key: String::new(),
                admission_key: "admission-1".into(),
                service_id: ServiceId::new(1)
            }
        )
        .is_err());
    assert_eq!(adapter.checkpoint().unwrap(), before);
}

#[test]
fn ed_checkpoint_rejects_remapped_source_and_admission_identity_keys() {
    let mut adapter = EdIdentityAdapter::new(77);
    let admissions = adapter
        .admit_batch(
            0,
            &[
                EdAdmissionInput {
                    source_person_key: "person-a".into(),
                    admission_key: "admission-a".into(),
                    service_id: ServiceId::new(4),
                },
                EdAdmissionInput {
                    source_person_key: "person-a".into(),
                    admission_key: "admission-b".into(),
                    service_id: ServiceId::new(4),
                },
                EdAdmissionInput {
                    source_person_key: "person-b".into(),
                    admission_key: "admission-c".into(),
                    service_id: ServiceId::new(4),
                },
            ],
        )
        .unwrap();
    assert_ne!(admissions[0].episode_id, admissions[1].episode_id);
    let original: serde_json::Value =
        serde_json::from_slice(&adapter.checkpoint().unwrap()).unwrap();

    let mut person_remap = original.clone();
    let person_a = person_remap["people"]["person-a"].clone();
    let person_b = person_remap["people"]["person-b"].clone();
    person_remap["people"]["person-a"] = person_b;
    person_remap["people"]["person-b"] = person_a;
    assert!(EdIdentityAdapter::restore(&serde_json::to_vec(&person_remap).unwrap()).is_err());

    let mut episode_remap = original;
    let episode_b = episode_remap["admissions"]["admission-b"]["episode_id"].clone();
    episode_remap["admissions"]["admission-a"]["episode_id"] = episode_b;
    assert!(EdIdentityAdapter::restore(&serde_json::to_vec(&episode_remap).unwrap()).is_err());
}

#[test]
fn ed_checkpoint_rejects_omitted_adapter_mappings_for_core_records() {
    let mut adapter = EdIdentityAdapter::new(78);
    adapter
        .admit(
            0,
            EdAdmissionInput {
                source_person_key: "person-a".into(),
                admission_key: "admission-a".into(),
                service_id: ServiceId::new(4),
            },
        )
        .unwrap();
    let mut tampered: serde_json::Value =
        serde_json::from_slice(&adapter.checkpoint().unwrap()).unwrap();
    tampered["people"] = serde_json::json!({});
    tampered["spawn_keys"] = serde_json::json!({});
    tampered["admissions"] = serde_json::json!({});
    tampered["episode_keys"] = serde_json::json!({});
    assert!(EdIdentityAdapter::restore(&serde_json::to_vec(&tampered).unwrap()).is_err());
}
