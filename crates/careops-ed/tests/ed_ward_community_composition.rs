use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use careops_core::experiment::{
    resume_generic_batch, run_generic_batch, CancellationToken, ExperimentError, IndexedReplication,
};
use careops_core::pathway::streams::{
    PathwayRngStreams, STREAM_DOMAIN_PROGRESSION, STREAM_DOMAIN_ROUTING, STREAM_DOMAIN_SERVICE,
};
use careops_core::transfer::{
    ClaimDisposition, ClaimRole, EpisodeId, MessageId, PersonId, ResourceId, ServiceId,
    TransferCommand, TransferEffect, TransferEnvelope, TransferLocation, TransferManager,
    TransferOperationId, TransferRequest, TransferState,
};
use careops_core::ward_community::{simulate, WardCommunityScenario, WardCommunityState};
use careops_ed::identity::{EdAdmissionInput, EdIdentityAdapter};
use careops_ed::{parse_scenario_json, run_scenario, RunSummary};

const ED: ServiceId = ServiceId::new(1);
const WARD: ServiceId = ServiceId::new(2);
const COMMUNITY: ServiceId = ServiceId::new(3);
const ED_BED: ResourceId = ResourceId::new(10);
const WARD_BED: ResourceId = ResourceId::new(20);
const COMMUNITY_PLACE: ResourceId = ResourceId::new(30);

fn operation(value: u64) -> TransferOperationId {
    TransferOperationId::new(value)
}

fn run_nominal_ed() -> RunSummary {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = crate_dir
        .parent()
        .expect("workspace crates directory")
        .parent()
        .expect("workspace root");
    let input = fs::read(root.join("crates/careops-ed/examples/nominal_mvp.json"))
        .expect("read synthetic nominal ED scenario");
    let scenario = parse_scenario_json(std::str::from_utf8(&input).unwrap()).unwrap();
    run_scenario(&scenario).expect("run nominal synthetic ED scenario")
}

fn person_and_episode(patient_id: &str) -> (PersonId, EpisodeId) {
    person_and_episode_with_seed(patient_id, 0xA6_02)
}

fn person_and_episode_with_seed(patient_id: &str, seed: u64) -> (PersonId, EpisodeId) {
    let mut identities = EdIdentityAdapter::new(seed);
    let admission = identities
        .admit(
            0,
            EdAdmissionInput {
                source_person_key: patient_id.to_owned(),
                admission_key: format!("encounter-{patient_id}"),
                service_id: ED,
            },
        )
        .unwrap();
    (admission.person_id, admission.episode_id)
}

fn send(
    manager: &mut TransferManager,
    operation_id: TransferOperationId,
    revision: u64,
    tick: u64,
    command: TransferCommand,
) {
    manager.advance_to(tick).unwrap();
    manager
        .apply(TransferEnvelope::new(
            operation_id,
            MessageId::new(operation_id.value() * 100 + revision + 1),
            revision,
            tick,
            command,
        ))
        .unwrap();
}

fn request(
    manager: &mut TransferManager,
    operation_id: TransferOperationId,
    tick: u64,
    transfer_request: TransferRequest,
) {
    send(
        manager,
        operation_id,
        0,
        tick,
        TransferCommand::Request {
            request: transfer_request,
        },
    );
}

fn admit_from_ed_to_ward(
    manager: &mut TransferManager,
    operation_id: TransferOperationId,
    person_id: PersonId,
    episode_id: EpisodeId,
    ready_tick: u64,
) -> u64 {
    request(
        manager,
        operation_id,
        ready_tick,
        TransferRequest {
            person_id,
            episode_id,
            origin_service: ED,
            destination_service: WARD,
            request_expires_at: None,
            origin_resources: vec![ED_BED],
        },
    );
    send(
        manager,
        operation_id,
        1,
        ready_tick + 1,
        TransferCommand::Offer {
            expires_at: Some(ready_tick + 20),
        },
    );
    send(
        manager,
        operation_id,
        2,
        ready_tick + 2,
        TransferCommand::Reserve {
            resources: vec![WARD_BED],
            expires_at: Some(ready_tick + 30),
        },
    );
    send(
        manager,
        operation_id,
        3,
        ready_tick + 3,
        TransferCommand::BeginDeparture,
    );
    send(
        manager,
        operation_id,
        4,
        ready_tick + 4,
        TransferCommand::Depart,
    );
    send(
        manager,
        operation_id,
        5,
        ready_tick + 5,
        TransferCommand::Arrive,
    );
    let handover_tick = ready_tick + 6;
    send(
        manager,
        operation_id,
        6,
        handover_tick,
        TransferCommand::CompleteHandover,
    );
    handover_tick
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct CompositionReplication {
    replication_index: usize,
    seed: u64,
    person_id: u64,
    episode_id: u64,
    ed_completed: usize,
    ward_admission_tick: u64,
    ward_departure_tick: u64,
    community_arrival_tick: u64,
    community_handover_tick: u64,
    final_owner_service: u64,
    final_claim_count: usize,
    final_claim_resource: u64,
}

impl IndexedReplication for CompositionReplication {
    fn replication_index(&self) -> usize {
        self.replication_index
    }

    fn seed(&self) -> u64 {
        self.seed
    }
}

fn compose_nominal_replication(replication_index: usize, seed: u64) -> CompositionReplication {
    let ed = run_nominal_ed();
    let patient = ed
        .patients
        .iter()
        .find(|patient| patient.patient_id == "patient-001")
        .unwrap();
    let (person_id, episode_id) =
        person_and_episode_with_seed(&format!("{}-{replication_index}", seed), seed);
    let mut manager = TransferManager::default();
    let ed_to_ward = operation(100 + replication_index as u64 * 2);
    let ward_admission_tick = admit_from_ed_to_ward(
        &mut manager,
        ed_to_ward,
        person_id,
        episode_id,
        patient.completion_tick.unwrap(),
    );
    assert_eq!(manager.person(person_id).unwrap().owner_service, WARD);
    assert_eq!(
        manager.snapshot(ed_to_ward).unwrap().state,
        TransferState::HandoverComplete
    );

    let ward_ready_tick = ward_admission_tick + 5;
    let community_place_tick = ward_ready_tick + 3;
    let transport_pickup_tick = ward_ready_tick + 6;
    let transport_duration = 2;
    let handover_delay = 2;
    let ward_to_community = simulate(WardCommunityScenario {
        ward_patient_ready_tick: ward_ready_tick,
        ed_patient_ready_tick: ed.patients[1].completion_tick.unwrap(),
        community_place_available_tick: Some(community_place_tick),
        transport_pickup_available_tick: Some(transport_pickup_tick),
        transport_duration_ticks: transport_duration,
        handover_acceptance_delay_ticks: Some(handover_delay),
        horizon_tick: transport_pickup_tick + transport_duration + handover_delay,
    })
    .unwrap();
    let departure_tick = ward_to_community.actual_ward_departure_tick.unwrap();
    let arrival_tick = ward_to_community.community_arrival_tick.unwrap();
    let handover_tick = ward_to_community.handover_complete_tick.unwrap();

    let operation_id = operation(ed_to_ward.value() + 1);
    request(
        &mut manager,
        operation_id,
        ward_ready_tick,
        TransferRequest {
            person_id,
            episode_id,
            origin_service: WARD,
            destination_service: COMMUNITY,
            request_expires_at: None,
            origin_resources: vec![WARD_BED],
        },
    );
    send(
        &mut manager,
        operation_id,
        1,
        ward_ready_tick + 1,
        TransferCommand::Offer {
            expires_at: Some(departure_tick + 5),
        },
    );
    send(
        &mut manager,
        operation_id,
        2,
        ward_ready_tick + 2,
        TransferCommand::Reserve {
            resources: vec![COMMUNITY_PLACE],
            expires_at: Some(departure_tick + 5),
        },
    );
    send(
        &mut manager,
        operation_id,
        3,
        departure_tick - 1,
        TransferCommand::BeginDeparture,
    );
    send(
        &mut manager,
        operation_id,
        4,
        departure_tick,
        TransferCommand::Depart,
    );
    send(
        &mut manager,
        operation_id,
        5,
        arrival_tick,
        TransferCommand::Arrive,
    );
    send(
        &mut manager,
        operation_id,
        6,
        handover_tick,
        TransferCommand::CompleteHandover,
    );

    let owner = manager.person(person_id).unwrap();
    let claims = manager.claims();
    let transfer_snapshot = manager.snapshot(operation_id).unwrap();
    assert_eq!(owner.owner_service, COMMUNITY);
    assert_eq!(owner.active_operation, None);
    assert_eq!(transfer_snapshot.state, TransferState::HandoverComplete);
    assert_eq!(transfer_snapshot.person_id, person_id);
    assert_eq!(transfer_snapshot.episode_id, episode_id);
    assert_eq!(
        claims.len(),
        1,
        "one destination claim remains after handover"
    );
    assert_eq!(claims[0].resource_id, COMMUNITY_PLACE);
    assert_eq!(claims[0].disposition, ClaimDisposition::Occupied);

    CompositionReplication {
        replication_index,
        seed,
        person_id: person_id.value(),
        episode_id: episode_id.value(),
        ed_completed: ed.completed,
        ward_admission_tick,
        ward_departure_tick: departure_tick,
        community_arrival_tick: arrival_tick,
        community_handover_tick: handover_tick,
        final_owner_service: owner.owner_service.value(),
        final_claim_count: claims.len(),
        final_claim_resource: claims[0].resource_id.value(),
    }
}

fn test_output_dir(label: &str) -> tempfile::TempDir {
    let mut builder = tempfile::Builder::new();
    builder.prefix(label);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(std::fs::Permissions::from_mode(0o700));
    }
    builder.tempdir().unwrap()
}

#[test]
fn nominal_ed_person_transfers_to_ward_then_community_after_delayed_discharge() {
    let ed = run_nominal_ed();
    assert_eq!(ed.completed, 5);
    let patient = ed
        .patients
        .iter()
        .find(|patient| patient.patient_id == "patient-001")
        .unwrap();
    let ed_ready_tick = patient.completion_tick.unwrap();
    let (person_id, episode_id) = person_and_episode(&patient.patient_id);
    let mut manager = TransferManager::default();

    let ward_admission_tick = admit_from_ed_to_ward(
        &mut manager,
        operation(1),
        person_id,
        episode_id,
        ed_ready_tick,
    );
    assert_eq!(
        manager.snapshot(operation(1)).unwrap().state,
        TransferState::HandoverComplete
    );
    assert_eq!(manager.person(person_id).unwrap().owner_service, WARD);
    assert!(manager
        .claims()
        .iter()
        .any(|claim| claim.resource_id == WARD_BED
            && claim.disposition == ClaimDisposition::Occupied));

    let ward_ready_tick = ward_admission_tick + 5;
    let ed_waiter_ready_tick = ed.patients[1].completion_tick.unwrap();
    let community_place_tick = ward_ready_tick + 3;
    let transport_pickup_tick = ward_ready_tick + 6;
    let transport_duration = 2;
    let handover_delay = 2;
    let community = simulate(WardCommunityScenario {
        ward_patient_ready_tick: ward_ready_tick,
        ed_patient_ready_tick: ed_waiter_ready_tick,
        community_place_available_tick: Some(community_place_tick),
        transport_pickup_available_tick: Some(transport_pickup_tick),
        transport_duration_ticks: transport_duration,
        handover_acceptance_delay_ticks: Some(handover_delay),
        horizon_tick: transport_pickup_tick + transport_duration + handover_delay,
    })
    .unwrap();
    let departure_tick = transport_pickup_tick;
    let arrival_tick = departure_tick + transport_duration;
    let handover_complete_tick = arrival_tick + handover_delay;
    assert_eq!(community.state, WardCommunityState::HandoverComplete);
    assert_eq!(community.actual_ward_departure_tick, Some(departure_tick));
    assert_eq!(community.community_arrival_tick, Some(arrival_tick));
    assert_eq!(
        community.handover_complete_tick,
        Some(handover_complete_tick)
    );
    assert_eq!(community.ed_admission_tick, Some(handover_complete_tick));
    assert_eq!(
        community.ed_boarding_ticks,
        handover_complete_tick - ed_waiter_ready_tick
    );

    request(
        &mut manager,
        operation(2),
        ward_ready_tick,
        TransferRequest {
            person_id,
            episode_id,
            origin_service: WARD,
            destination_service: COMMUNITY,
            request_expires_at: None,
            origin_resources: vec![WARD_BED],
        },
    );
    send(
        &mut manager,
        operation(2),
        1,
        ward_ready_tick + 1,
        TransferCommand::Offer {
            expires_at: Some(departure_tick + 5),
        },
    );
    send(
        &mut manager,
        operation(2),
        2,
        ward_ready_tick + 2,
        TransferCommand::Reserve {
            resources: vec![COMMUNITY_PLACE],
            expires_at: Some(departure_tick + 5),
        },
    );
    send(
        &mut manager,
        operation(2),
        3,
        departure_tick - 1,
        TransferCommand::BeginDeparture,
    );
    send(
        &mut manager,
        operation(2),
        4,
        departure_tick,
        TransferCommand::Depart,
    );
    assert_eq!(
        manager.snapshot(operation(2)).unwrap().state,
        TransferState::InTransit
    );
    assert!(manager.claims().iter().any(|claim| {
        claim.resource_id == WARD_BED
            && claim.role == ClaimRole::Origin
            && claim.disposition == ClaimDisposition::Held
    }));
    assert!(manager.claims().iter().any(|claim| {
        claim.resource_id == COMMUNITY_PLACE && claim.disposition == ClaimDisposition::InFlightHold
    }));
    send(
        &mut manager,
        operation(2),
        5,
        arrival_tick,
        TransferCommand::Arrive,
    );
    send(
        &mut manager,
        operation(2),
        6,
        handover_complete_tick,
        TransferCommand::CompleteHandover,
    );

    let snapshot = manager.snapshot(operation(2)).unwrap();
    assert_eq!(snapshot.state, TransferState::HandoverComplete);
    assert_eq!(snapshot.person_id, person_id);
    assert_eq!(snapshot.episode_id, episode_id);
    assert_eq!(snapshot.actual_handover_service, Some(COMMUNITY));
    assert_eq!(manager.person(person_id).unwrap().owner_service, COMMUNITY);
    assert_eq!(manager.person(person_id).unwrap().active_operation, None);
    let claims = manager.claims();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].resource_id, COMMUNITY_PLACE);
    assert_eq!(claims[0].owner_service, COMMUNITY);
    assert_eq!(claims[0].disposition, ClaimDisposition::Occupied);
}

#[test]
fn rejected_and_expired_ward_offers_keep_the_ed_person_and_origin_bed() {
    let ed = run_nominal_ed();
    let patient = &ed.patients[0];
    let ready_tick = patient.completion_tick.unwrap();
    let (person_id, episode_id) = person_and_episode(&patient.patient_id);

    let mut rejected = TransferManager::default();
    request(
        &mut rejected,
        operation(3),
        ready_tick,
        TransferRequest {
            person_id,
            episode_id,
            origin_service: ED,
            destination_service: WARD,
            request_expires_at: None,
            origin_resources: vec![ED_BED],
        },
    );
    send(
        &mut rejected,
        operation(3),
        1,
        ready_tick + 1,
        TransferCommand::Offer { expires_at: None },
    );
    send(
        &mut rejected,
        operation(3),
        2,
        ready_tick + 2,
        TransferCommand::Reject,
    );
    assert_eq!(
        rejected.snapshot(operation(3)).unwrap().state,
        TransferState::Rejected
    );
    assert_eq!(rejected.person(person_id).unwrap().owner_service, ED);
    assert_eq!(rejected.claims().len(), 1);
    assert_eq!(rejected.claims()[0].resource_id, ED_BED);

    let mut expired = TransferManager::default();
    request(
        &mut expired,
        operation(4),
        ready_tick,
        TransferRequest {
            person_id,
            episode_id,
            origin_service: ED,
            destination_service: WARD,
            request_expires_at: None,
            origin_resources: vec![ED_BED],
        },
    );
    send(
        &mut expired,
        operation(4),
        1,
        ready_tick + 1,
        TransferCommand::Offer {
            expires_at: Some(ready_tick + 4),
        },
    );
    expired.advance_to(ready_tick + 4).unwrap();
    assert_eq!(
        expired.snapshot(operation(4)).unwrap().state,
        TransferState::Expired
    );
    assert_eq!(expired.person(person_id).unwrap().owner_service, ED);
    assert_eq!(expired.claims().len(), 1);
    assert_eq!(expired.claims()[0].resource_id, ED_BED);
    assert_eq!(
        expired.events().last().unwrap().effect,
        TransferEffect::Expired
    );
}

#[test]
fn cancellation_after_ward_reservation_releases_destination_but_keeps_ed_claim() {
    let ed = run_nominal_ed();
    let patient = &ed.patients[0];
    let ready_tick = patient.completion_tick.unwrap();
    let (person_id, episode_id) = person_and_episode(&patient.patient_id);
    let mut manager = TransferManager::default();
    request(
        &mut manager,
        operation(5),
        ready_tick,
        TransferRequest {
            person_id,
            episode_id,
            origin_service: ED,
            destination_service: WARD,
            request_expires_at: None,
            origin_resources: vec![ED_BED],
        },
    );
    send(
        &mut manager,
        operation(5),
        1,
        ready_tick + 1,
        TransferCommand::Offer { expires_at: None },
    );
    send(
        &mut manager,
        operation(5),
        2,
        ready_tick + 2,
        TransferCommand::Reserve {
            resources: vec![WARD_BED],
            expires_at: Some(ready_tick + 10),
        },
    );
    send(
        &mut manager,
        operation(5),
        3,
        ready_tick + 3,
        TransferCommand::Cancel,
    );

    assert_eq!(
        manager.snapshot(operation(5)).unwrap().state,
        TransferState::Cancelled
    );
    assert_eq!(manager.person(person_id).unwrap().owner_service, ED);
    assert_eq!(manager.person(person_id).unwrap().active_operation, None);
    assert_eq!(manager.claims().len(), 1);
    assert_eq!(manager.claims()[0].resource_id, ED_BED);
    assert_eq!(manager.claims()[0].operation_id, None);
}

#[test]
fn transport_past_horizon_is_censored_and_keeps_the_ward_origin_claim() {
    let ed = run_nominal_ed();
    let patient = &ed.patients[0];
    let (person_id, episode_id) = person_and_episode(&patient.patient_id);
    let mut manager = TransferManager::default();
    let ward_admission_tick = admit_from_ed_to_ward(
        &mut manager,
        operation(6),
        person_id,
        episode_id,
        patient.completion_tick.unwrap(),
    );
    let ward_ready_tick = ward_admission_tick + 5;
    let departure_tick = ward_ready_tick + 2;
    let horizon_tick = ward_ready_tick + 4;
    let community = simulate(WardCommunityScenario {
        ward_patient_ready_tick: ward_ready_tick,
        ed_patient_ready_tick: ed.patients[1].completion_tick.unwrap(),
        community_place_available_tick: Some(ward_ready_tick),
        transport_pickup_available_tick: Some(departure_tick),
        transport_duration_ticks: 5,
        handover_acceptance_delay_ticks: Some(0),
        horizon_tick,
    })
    .unwrap();
    assert_eq!(community.state, WardCommunityState::InTransit);
    assert_eq!(community.actual_ward_departure_tick, Some(departure_tick));
    assert_eq!(community.community_arrival_tick, None);
    assert_eq!(community.handover_complete_tick, None);
    assert!(community.origin_claim_held_at_horizon);

    request(
        &mut manager,
        operation(7),
        ward_ready_tick,
        TransferRequest {
            person_id,
            episode_id,
            origin_service: WARD,
            destination_service: COMMUNITY,
            request_expires_at: None,
            origin_resources: vec![WARD_BED],
        },
    );
    send(
        &mut manager,
        operation(7),
        1,
        ward_ready_tick + 1,
        TransferCommand::Offer { expires_at: None },
    );
    send(
        &mut manager,
        operation(7),
        2,
        ward_ready_tick + 2,
        TransferCommand::Reserve {
            resources: vec![COMMUNITY_PLACE],
            expires_at: Some(horizon_tick + 10),
        },
    );
    send(
        &mut manager,
        operation(7),
        3,
        departure_tick,
        TransferCommand::BeginDeparture,
    );
    send(
        &mut manager,
        operation(7),
        4,
        departure_tick,
        TransferCommand::Depart,
    );
    manager.advance_to(horizon_tick).unwrap();

    assert_eq!(
        manager.snapshot(operation(7)).unwrap().state,
        TransferState::InTransit
    );
    assert_eq!(
        manager.person(person_id).unwrap().location,
        TransferLocation::Transport {
            from: WARD,
            to: COMMUNITY
        }
    );
    let claims = manager.claims();
    assert_eq!(claims.len(), 2);
    assert!(claims.iter().any(|claim| {
        claim.resource_id == WARD_BED
            && claim.role == ClaimRole::Origin
            && claim.disposition == ClaimDisposition::Held
    }));
    assert!(claims.iter().any(|claim| {
        claim.resource_id == COMMUNITY_PLACE && claim.disposition == ClaimDisposition::InFlightHold
    }));
}

#[test]
fn integrated_composition_is_worker_invariant_at_one_two_and_four_workers() {
    let run = |workers| {
        run_generic_batch::<CompositionReplication, (), _>(
            "a63-worker-invariance",
            0xA603_2026,
            8,
            workers,
            None,
            (),
            None,
            None,
            None,
            |index, seed, _| Ok(compose_nominal_replication(index, seed)),
        )
        .unwrap()
    };

    let one_worker = run(1);
    let two_workers = run(2);
    let four_workers = run(4);
    assert_eq!(one_worker, two_workers);
    assert_eq!(one_worker, four_workers);
    assert_eq!(one_worker.len(), 8);
    assert!(one_worker
        .iter()
        .enumerate()
        .all(|(index, result)| result.replication_index == index));
    assert!(one_worker.iter().all(|result| {
        result.ed_completed == 5
            && result.final_owner_service == COMMUNITY.value()
            && result.final_claim_count == 1
            && result.final_claim_resource == COMMUNITY_PLACE.value()
            && result.ward_admission_tick < result.ward_departure_tick
            && result.ward_departure_tick < result.community_arrival_tick
            && result.community_arrival_tick < result.community_handover_tick
    }));
    assert_eq!(
        serde_json::to_vec(&one_worker).unwrap(),
        serde_json::to_vec(&two_workers).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&one_worker).unwrap(),
        serde_json::to_vec(&four_workers).unwrap()
    );
}

#[test]
fn interrupted_integrated_batch_resumes_to_the_uninterrupted_result() {
    let base_seed = 0xA603_2027;
    let replications = 6;
    let runner = |index, seed, _metadata: &()| {
        Ok::<_, ExperimentError>(compose_nominal_replication(index, seed))
    };
    let baseline = run_generic_batch::<CompositionReplication, (), _>(
        "a63-recovery-baseline",
        base_seed,
        replications,
        2,
        None,
        (),
        None,
        None,
        None,
        runner,
    )
    .unwrap();

    let output_owner = test_output_dir("recovery");
    let output_dir = output_owner.path();
    let cancel_token = CancellationToken::new();
    let completed_calls = Arc::new(AtomicUsize::new(0));
    let calls = Arc::clone(&completed_calls);
    let cancel = cancel_token.clone();
    let interrupted_runner = move |index, seed, _metadata: &()| {
        if calls.fetch_add(1, Ordering::SeqCst) >= 2 {
            cancel.cancel();
        }
        Ok::<_, ExperimentError>(compose_nominal_replication(index, seed))
    };

    let interrupted = run_generic_batch::<CompositionReplication, (), _>(
        "a63-recovery-interrupted",
        base_seed,
        replications,
        1,
        Some(1),
        (),
        Some(&output_dir),
        Some(&cancel_token),
        None,
        interrupted_runner,
    );
    assert!(interrupted.is_err());
    assert_eq!(
        interrupted.unwrap_err().kind,
        careops_core::experiment::ExperimentErrorKind::Interrupted
    );
    let checkpoint_path = output_dir.join("checkpoint.json");
    assert!(checkpoint_path.exists());
    assert!(output_dir.join("batch_run.partial").exists());
    assert!(!output_dir.join("batch_run.json").exists());

    let (checkpoint, resumed) = resume_generic_batch::<CompositionReplication, (), _>(
        &checkpoint_path,
        2,
        &output_dir,
        None,
        None,
        runner,
    )
    .unwrap();
    assert_eq!(checkpoint.total_replications, replications);
    assert_eq!(resumed, baseline);
    assert_eq!(
        resumed
            .iter()
            .map(|result| result.replication_index)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        replications,
        "recovery must contain every replication identity once"
    );
}

#[test]
fn stable_pathway_namespaces_isolate_progression_from_service_and_routing_activity() {
    assert_eq!(STREAM_DOMAIN_PROGRESSION, 0x5052_4F47_5245_5353);
    assert_eq!(STREAM_DOMAIN_SERVICE, 0x5345_5256_4943_4553);
    assert_eq!(STREAM_DOMAIN_ROUTING, 0x524F_5554_494E_4753);
    assert_ne!(STREAM_DOMAIN_PROGRESSION, STREAM_DOMAIN_SERVICE);
    assert_ne!(STREAM_DOMAIN_PROGRESSION, STREAM_DOMAIN_ROUTING);
    assert_ne!(STREAM_DOMAIN_SERVICE, STREAM_DOMAIN_ROUTING);

    let mut baseline = PathwayRngStreams::new(0xA603_2028);
    let mut downstream_activity = PathwayRngStreams::new(0xA603_2028);
    for _ in 0..100 {
        downstream_activity.service_stream.next_uniform();
        downstream_activity.routing_stream.next_uniform();
    }
    let expected = (0..10)
        .map(|_| baseline.progression_stream.next_exponential(0.05))
        .collect::<Vec<_>>();
    let observed = (0..10)
        .map(|_| {
            downstream_activity
                .progression_stream
                .next_exponential(0.05)
        })
        .collect::<Vec<_>>();
    assert_eq!(expected, observed);
}
