//! Finite synthetic composition, with serial cases within each replication.
//! This coordinator batches identity effects once per tick, audits responsibility
//! after the whole tick, and retains logical transfer holds after physical vacancy.
//! It is not a shared ED/birthing scheduler or a clinical/site model.

use crate::{Result, Row, Scenario};
use careops_birthing::{
    delivery::*, spaces::*, team::*, theatre::*, transfers::*, BirthingEpisode, EventEffect,
    PatientState, ServiceState,
};
use careops_core::capacity::{Calendar, CapabilitySet, UtcInterval};
use careops_core::pathway::{
    CompetingRiskModel, HazardRate, ObservedPolicyState, PathwayRngStreams,
    VersionedUrgencyAssessment,
};
use careops_core::staffing::{BookingId, DutyKind, EscortChainId, StaffId};
use careops_core::{birth::*, identity as id, transfer::*};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

fn calendar(start: u64, end: u64) -> Calendar {
    Calendar::new(vec![UtcInterval::new(start as i64, end as i64).unwrap()]).unwrap()
}
fn ratio(role: &str) -> RatioProfile {
    RatioProfile {
        revision: "B7-static-role-v1".into(),
        cohort_unit: "synthetic_person".into(),
        patients_per_staff: BTreeMap::from([(role.into(), 1)]),
    }
}
fn emit(
    rows: &mut Vec<Row>,
    tick: u64,
    person: u64,
    episode: u64,
    kind: &str,
    data: serde_json::Value,
) -> Result<()> {
    if rows.len() >= super::MAX_ROWS {
        return Err("finite capture limit exceeded".into());
    }
    rows.push(Row {
        tick,
        person,
        episode,
        kind: kind.into(),
        data,
    });
    Ok(())
}
fn audit(ids: &id::IdentityLedger, team: &TeamAllocator, spaces: &SpacePool) -> Result<()> {
    let c = ids.counts();
    if c.service_exits > c.service_entries
        || c.service_entries - c.service_exits != c.people_created - c.deaths
    {
        return Err("identity/episode conservation failed".into());
    }
    team.audit_responsibility(spaces, spaces.current_tick())?;
    Ok(())
}
fn advance(
    theatre: &mut TheatreService,
    spaces: &mut SpacePool,
    team: &TeamAllocator,
    tick: u64,
    rows: &mut Vec<Row>,
) -> Result<Vec<Callback>> {
    let callbacks = theatre.advance_to(tick, spaces, team)?;
    for c in &callbacks {
        emit(
            rows,
            c.tick,
            0,
            c.episode,
            "theatre_callback",
            json!({"case":c.case_id,"callback":format!("{:?}",c.kind)}),
        )?;
    }
    Ok(callbacks)
}
fn envelope(
    op: u64,
    part: u64,
    revision: u64,
    tick: u64,
    command: TransferCommand,
) -> TransferEnvelope {
    TransferEnvelope::new(
        TransferOperationId::new(op),
        MessageId::new(op * 100 + part),
        revision,
        tick,
        command,
    )
}
fn transfer(
    boundary: &mut TransferBoundary,
    message: TransferEnvelope,
    ids: &id::IdentityLedger,
    team: &TeamAllocator,
    rows: &mut Vec<Row>,
) -> Result<()> {
    let before = boundary.core().events().len();
    boundary.apply(message, ids, team)?;
    for e in &boundary.core().events()[before..] {
        let s = boundary
            .core()
            .snapshot(e.operation_id)
            .ok_or("missing transfer snapshot")?;
        let kind = if e.effect == TransferEffect::CompensationCompleted {
            "compensation_completed".into()
        } else {
            format!("transfer_{:?}", e.effect)
        };
        emit(
            rows,
            boundary.core().now(),
            s.person_id.value(),
            s.episode_id.value(),
            &kind,
            json!({"operation":e.operation_id.value(),"message_tick":e.tick,"owner":s.owner_service.value(),"location":format!("{:?}",s.location)}),
        )?;
    }
    Ok(())
}
fn policy(s: &Scenario) -> QueuePolicy {
    QueuePolicy {
        revision: s.revision.clone(),
        jurisdiction_id: "synthetic".into(),
        scheme_id: "B7".into(),
        scheme_version: "1".into(),
        source_authority: s.provenance.clone(),
        citation: "fixture:B7".into(),
        rules: BTreeMap::from([(
            1,
            QueueRule {
                priority_rank: 0,
                targets: Targets {
                    transfer: Some(0),
                    incision: Some(0),
                    birth: Some(s.target_birth_seconds),
                },
            },
        )]),
    }
}
fn observed(s: &Scenario, tick: u64) -> ObservedPolicyState {
    ObservedPolicyState {
        observed_at_tick: tick,
        elapsed_wait_ticks: 0,
        urgency: VersionedUrgencyAssessment {
            jurisdiction_id: "synthetic".into(),
            scheme_id: "B7".into(),
            scheme_version: "1".into(),
            category: 1,
            assessed_at_tick: tick,
            effective_date: "2026-11-03".into(),
            source_authority: s.provenance.clone(),
            citation: "fixture:B7".into(),
            source_sha256: None,
        },
    }
}

pub fn simulate(
    s: &Scenario,
    seed: u64,
    namespace: u64,
    extra_routing_draws: usize,
) -> Result<Vec<Row>> {
    s.validate()?;
    if namespace == 0 || extra_routing_draws > 32 {
        return Err("invalid namespace/stream-control bound".into());
    }
    let horizon = u64::from(s.days) * 86400;
    let mut spaces = SpacePool::new(vec![
        SpaceProfile {
            id: 1,
            revision: s.revision.clone(),
            kind: SpaceKind::Ldr,
            capabilities: CapabilitySet::new(["ldr"])?,
            open: calendar(0, horizon),
            staffed: Some(calendar(0, horizon)),
            equipped: Some(true),
        },
        SpaceProfile {
            id: 2,
            revision: s.revision.clone(),
            kind: SpaceKind::Theatre,
            capabilities: CapabilitySet::new(["theatre"])?,
            open: Calendar::new(
                (0..s.days)
                    .map(|day| {
                        let base = u64::from(day) * 86400 + 300;
                        UtcInterval::new(
                            (base + s.theatre_open_delay_seconds) as i64,
                            (base + 200) as i64,
                        )
                        .unwrap()
                    })
                    .collect(),
            )?,
            staffed: Some(calendar(0, horizon)),
            equipped: Some(true),
        },
    ])?;
    let members = [(1, "midwife"), (2, "operator"), (3, "escort")]
        .into_iter()
        .map(|(staff, role)| MemberProfile {
            staff: StaffId::new(staff),
            revision: s.revision.clone(),
            roles: CapabilitySet::new([role]).unwrap(),
            roster: calendar(0, horizon),
            unavailable: Calendar::default(),
            overtime_per_shift_seconds: 0,
            call_in_latency_seconds: 0,
            call_windows: None,
        })
        .collect();
    let mut team = TeamAllocator::new(members)?;
    let mut theatre = TheatreService::new(TheatreProfile {
        revision: s.revision.clone(),
        preparation_ticks: s.preparation_seconds,
        birth_after_incision_ticks: s.incision_to_birth_seconds,
        completion_after_birth_ticks: s.completion_after_birth_seconds,
        cleaning_ticks: s.cleaning_seconds,
        required_capabilities: vec!["theatre".into()],
        staffing: ratio("operator"),
    })?;
    let mut ids = id::IdentityLedger::new(namespace);
    let mut deliveries = DeliveryLedger::default();
    let mut previous: Vec<TransferBoundary> = Vec::new();
    let mut rows = Vec::new();
    for case in 0..u64::from(s.days) * 2 {
        let base = (case / 2) * 86400 + (case % 2) * 300;
        let key = 100 + case * 100;
        let operative = case % 2 == 1;
        if previous
            .iter()
            .flat_map(|b| b.core().claims())
            .any(|c| c.resource_id == ResourceId::new(1) || c.resource_id == ResourceId::new(2))
        {
            return Err("prior logical origin hold forbids room reuse".into());
        }
        advance(&mut theatre, &mut spaces, &team, base, &mut rows)?;
        let mother = id::person_id_for_spawn(namespace, id::SpawnEventId::new(key + 1))?;
        let ldr_episode = id::episode_id_for_entry(namespace, id::EpisodeEventId::new(key + 10))?;
        ids.apply_tick(
            base,
            vec![
                id::IdentityCommand::Spawn {
                    event_id: id::SpawnEventId::new(key + 1),
                    origin: id::PersonOrigin::ExternalEntry,
                },
                id::IdentityCommand::EnterService {
                    event_id: id::EpisodeEventId::new(key + 10),
                    person_id: mother,
                    service_id: ServiceId::new(1),
                },
            ],
        )?;
        team.admit(
            &mut spaces,
            ldr_episode.value(),
            CapabilitySet::new(["ldr"])?,
            TeamRequest {
                booking: BookingId::new(key + 1),
                patients: 1,
                profile: ratio("midwife"),
                interval: UtcInterval::new(base as i64, (base + 200) as i64)?,
                kind: DutyKind::Service,
            },
        )?;
        spaces.start(ldr_episode.value(), base as i64)?;
        audit(&ids, &team, &spaces)?;
        let mut streams = PathwayRngStreams::new(seed.wrapping_add(key));
        let service = s.service_min_seconds
            + (streams.service_stream.next_uniform() * (s.service_jitter_seconds + 1) as f64)
                as u64;
        for _ in 0..extra_routing_draws {
            streams.routing_stream.next_uniform();
        }
        let travel = s.travel_min_seconds
            + (streams.routing_stream.next_uniform() * (s.travel_jitter_seconds + 1) as f64) as u64;
        let mut episode = BirthingEpisode::new(
            mother.value(),
            PatientState::ActiveLabour,
            ServiceState::InSuite,
            seed.wrapping_add(key),
        );
        episode.advance_through(base)?;
        let sampled = episode.sample_progression_tick(
            key,
            "synthetic competing progression",
            0,
            &{
                let mut model = CompetingRiskModel::new();
                model.add_hazard(HazardRate::new("progress", s.progression_rate));
                model
            },
            &HashMap::new(),
            &HashMap::from([("progress".into(), PatientState::LaterLabourStage)]),
        )?;
        episode.advance_through(base + 1)?;
        emit(
            &mut rows,
            base + 1,
            mother.value(),
            ldr_episode.value(),
            "progression",
            json!({"sampled":sampled,"draw_counts":episode.draw_counts(),"state":episode.observation().patient_state.as_str()}),
        )?;
        emit(
            &mut rows,
            base + 1,
            mother.value(),
            ldr_episode.value(),
            "service_draw",
            json!({"seconds":service,"stream":"service","draws":streams.service_stream.draw_count}),
        )?;
        let (source_episode, source_service, birth_tick, completion, origin_room) = if operative {
            let decision = base + 2;
            let source = id::episode_id_for_entry(namespace, id::EpisodeEventId::new(key + 20))?;
            theatre.submit(TheatreRequest {
                id: key,
                episode: source.value(),
                booking: BookingId::new(key + 2),
                decision_tick: decision,
                cohort_size: 1,
                observed: observed(s, decision),
                policy: policy(s),
            })?;
            theatre.accept(key, decision)?;
            theatre.transfer_departed(key, base + 3)?;
            spaces.depart(ldr_episode.value(), (base + 3) as i64)?;
            spaces.finish_cleaning(ldr_episode.value(), (base + 3 + s.cleaning_seconds) as i64)?;
            let arrival = base + 4 + s.cleaning_seconds;
            ids.apply_tick(
                arrival,
                vec![
                    id::IdentityCommand::ExitService {
                        event_id: id::LifecycleEventId::new(key + 60),
                        episode_id: ldr_episode,
                    },
                    id::IdentityCommand::EnterService {
                        event_id: id::EpisodeEventId::new(key + 20),
                        person_id: mother,
                        service_id: ServiceId::new(5),
                    },
                ],
            )?;
            theatre.transfer_arrived(key, arrival)?;
            let actual_start = if theatre
                .start_next(arrival, &mut spaces, &mut team)?
                .is_none()
            {
                let cause = &theatre
                    .case(key)
                    .ok_or("missing theatre case")?
                    .resource_failures
                    .last()
                    .ok_or("no resource failure reason")?
                    .reason;
                emit(
                    &mut rows,
                    arrival,
                    mother.value(),
                    source.value(),
                    "theatre_resource_unavailable",
                    json!({"cause":cause}),
                )?;
                let ready = arrival.max(base + s.theatre_open_delay_seconds);
                advance(&mut theatre, &mut spaces, &team, ready, &mut rows)?;
                if theatre.start_next(ready, &mut spaces, &mut team)? != Some(key) {
                    return Err("theatre remained unavailable at declared opening".into());
                }
                ready
            } else {
                arrival
            };
            let birth = actual_start + s.preparation_seconds + s.incision_to_birth_seconds;
            let mut saw_birth = false;
            while let Some(tick) = theatre.next_event_tick().filter(|t| *t <= birth) {
                for callback in advance(&mut theatre, &mut spaces, &team, tick, &mut rows)? {
                    if callback.case_id == key && callback.kind == CallbackKind::Birth {
                        saw_birth = true;
                    }
                }
                audit(&ids, &team, &spaces)?;
            }
            if !saw_birth {
                return Err("operative birth requires actual B5 Birth callback".into());
            }
            let metric = theatre
                .case(key)
                .ok_or("missing theatre case")?
                .metrics
                .birth
                .as_ref()
                .ok_or("missing birth metric")?;
            if metric.breached == Some(true) {
                emit(
                    &mut rows,
                    birth,
                    mother.value(),
                    source.value(),
                    "target_breach",
                    json!({"elapsed":metric.elapsed,"target":metric.target,"policy_revision":metric.policy_revision}),
                )?;
            }
            (
                source,
                ServiceId::new(5),
                birth,
                birth + s.completion_after_birth_seconds,
                2,
            )
        } else {
            (ldr_episode, ServiceId::new(1), base + service, 0, 1)
        };
        episode.schedule(
            key + 1,
            "actual delivery",
            birth_tick,
            0,
            EventEffect::progression(PatientState::DeliveryComplete),
        )?;
        episode.advance_through(birth_tick)?;
        let infants: Vec<_> = (0..s.infants_per_birth)
            .map(|n| BirthInfantInput {
                infant_ordinal: n,
                spawn_event_id: id::SpawnEventId::new(key + 30 + u64::from(n)),
                relationship_event_id: id::RelationshipEventId::new(key + 50 + u64::from(n)),
                destination_service: ServiceId::new(3),
            })
            .collect();
        let mut people = vec![mother];
        let mut episodes = vec![source_episode];
        let mut birth_commands = Vec::new();
        for infant in &infants {
            let person = id::person_id_for_spawn(namespace, infant.spawn_event_id)?;
            let entry = id::EpisodeEventId::new(key + 40 + u64::from(infant.infant_ordinal));
            people.push(person);
            episodes.push(id::episode_id_for_entry(namespace, entry)?);
            birth_commands.extend([
                id::IdentityCommand::Spawn {
                    event_id: infant.spawn_event_id,
                    origin: id::PersonOrigin::Birth {
                        birth_event_id: id::BirthEventId::new(key + 2),
                        infant_ordinal: infant.infant_ordinal,
                    },
                },
                id::IdentityCommand::AddRelationship {
                    event_id: infant.relationship_event_id,
                    kind: id::RelationshipKind::BirthParentOf,
                    from: mother,
                    to: person,
                },
                id::IdentityCommand::EnterService {
                    event_id: entry,
                    person_id: person,
                    service_id: source_service,
                },
            ]);
        }
        ids.apply_tick(birth_tick, birth_commands)?;
        let message = BirthMessageId::new(key + 3);
        deliveries.register(
            &mut ids,
            birth_tick,
            DeliveryRecord {
                route: if operative {
                    DeliveryRoute::Operative
                } else {
                    DeliveryRoute::Vaginal
                },
                birth: BirthMessageInput {
                    message_id: message,
                    birth_event_id: id::BirthEventId::new(key + 2),
                    mother_person_id: mother,
                    maternal_destination_service: ServiceId::new(2),
                    infants,
                },
                observations: vec![NeonatalObservation {
                    infant_ordinal: 0,
                    observed_tick: birth_tick,
                    available_tick: birth_tick + 1,
                    variable: "synthetic_score".into(),
                    value: 1,
                    unit: "ordinal".into(),
                    provenance: s.provenance.clone(),
                }],
            },
        )?;
        emit(
            &mut rows,
            birth_tick,
            mother.value(),
            source_episode.value(),
            "birth",
            json!({"route":if operative {"operative"}else{"vaginal"},"infants":&people[1..],"source_service":source_service.value(),"delivery_complete":episode.observation().patient_state==PatientState::DeliveryComplete}),
        )?;
        audit(&ids, &team, &spaces)?;
        let reserve_at = (birth_tick + 2).max(base + s.destination_open_delay_seconds);
        let departure = if operative {
            completion
        } else {
            reserve_at + 1
        };
        let last = people.len() as u64;
        team.allocate(TeamRequest {
            booking: BookingId::new(key + 3),
            patients: 1,
            profile: ratio("escort"),
            interval: UtcInterval::new(departure as i64, (departure + 100) as i64)?,
            kind: DutyKind::Escort {
                chain: EscortChainId::new(key),
            },
        })?;
        let origins: Vec<_> = (0..people.len())
            .map(|i| {
                ResourceId::new(if i == 0 {
                    origin_room
                } else {
                    1000 + key + i as u64
                })
            })
            .collect();
        let destinations: Vec<_> = (0..people.len())
            .map(|i| ResourceId::new(10000 + key + i as u64))
            .collect();
        let fallback = ResourceId::new(20000 + key);
        let mut resources = BTreeMap::new();
        for (i, (origin, dest)) in origins.iter().zip(&destinations).enumerate() {
            let unit = if i == 0 { "bed" } else { "cot" };
            resources.insert(*origin, unit.into());
            resources.insert(*dest, unit.into());
        }
        resources.insert(fallback, "retrieval_place".into());
        let window = Some(vec![AvailabilityWindow {
            start: base + s.destination_open_delay_seconds,
            end: base + 200,
        }]);
        let profile = RoutingProfile {
            revision: s.revision.clone(),
            resources,
            destinations: BTreeMap::from([
                (
                    ServiceId::new(2),
                    DestinationProfile {
                        revision: s.revision.clone(),
                        resources: vec![destinations[0]],
                        availability: window.clone(),
                    },
                ),
                (
                    ServiceId::new(3),
                    DestinationProfile {
                        revision: s.revision.clone(),
                        resources: destinations[1..].to_vec(),
                        availability: window.clone(),
                    },
                ),
                (
                    ServiceId::new(4),
                    DestinationProfile {
                        revision: s.revision.clone(),
                        resources: vec![fallback],
                        availability: window,
                    },
                ),
            ]),
        };
        let plans = people
            .iter()
            .enumerate()
            .map(|(i, person)| RecipientPlan {
                operation: TransferOperationId::new(i as u64 + 1),
                person: *person,
                source_episode: episodes[i],
                origin_resources: vec![origins[i]],
                request_expires_at: None,
                fallback_services: if i + 1 == people.len() {
                    vec![ServiceId::new(4)]
                } else {
                    Vec::new()
                },
                escort: (i + 1 == people.len()).then_some(EscortBinding {
                    staff: 3,
                    booking: key + 3,
                    chain: key,
                }),
            })
            .collect();
        let mut boundary =
            TransferBoundary::bind_delivery(&deliveries, message, &ids, profile, plans)?;
        for op in 1..=last {
            boundary.request(
                TransferOperationId::new(op),
                MessageId::new(op * 100),
                birth_tick,
                &ids,
                &team,
            )?;
        }
        for op in 1..=last {
            transfer(
                &mut boundary,
                envelope(
                    op,
                    1,
                    1,
                    birth_tick + 1,
                    TransferCommand::Offer { expires_at: None },
                ),
                &ids,
                &team,
                &mut rows,
            )?;
        }
        advance(&mut theatre, &mut spaces, &team, birth_tick + 2, &mut rows)?;
        if birth_tick + 2 < reserve_at {
            let before = boundary.core().checkpoint()?;
            if boundary
                .apply(
                    envelope(
                        1,
                        2,
                        2,
                        birth_tick + 2,
                        TransferCommand::Reserve {
                            resources: vec![destinations[0]],
                            expires_at: None,
                        },
                    ),
                    &ids,
                    &team,
                )
                .is_ok()
                || boundary.core().checkpoint()? != before
            {
                return Err("closed destination changed claims".into());
            }
            emit(
                &mut rows,
                birth_tick + 2,
                mother.value(),
                source_episode.value(),
                "destination_unavailable",
                json!({"ready_at":reserve_at}),
            )?;
        }
        advance(&mut theatre, &mut spaces, &team, reserve_at, &mut rows)?;
        for op in 1..=last {
            transfer(
                &mut boundary,
                envelope(
                    op,
                    2,
                    2,
                    reserve_at,
                    TransferCommand::Reserve {
                        resources: vec![destinations[op as usize - 1]],
                        expires_at: None,
                    },
                ),
                &ids,
                &team,
                &mut rows,
            )?;
        }
        for op in 1..=last {
            transfer(
                &mut boundary,
                envelope(op, 3, 3, departure, TransferCommand::BeginDeparture),
                &ids,
                &team,
                &mut rows,
            )?;
        }
        // Operative profile: actual family departure is exactly B5 completion.
        // Vaginal co-location profile: infants depart first, mother last; the
        // room remains physically occupied until the last modeled occupant leaves.
        let depart_end = if operative {
            departure
        } else {
            departure + last - 1
        };
        for (rank, op) in (2..=last).chain(std::iter::once(1)).enumerate() {
            let tick = if operative {
                departure
            } else {
                departure + rank as u64
            };
            advance(&mut theatre, &mut spaces, &team, tick, &mut rows)?;
            if !operative && op == 1 {
                spaces.depart(source_episode.value(), tick as i64)?;
            }
            transfer(
                &mut boundary,
                envelope(op, 4, 4, tick, TransferCommand::Depart),
                &ids,
                &team,
                &mut rows,
            )?;
            if op == 1 {
                if boundary.resource_unclaimed(ResourceId::new(origin_room))? {
                    return Err("origin released before acknowledgement".into());
                }
                emit(
                    &mut rows,
                    tick,
                    mother.value(),
                    source_episode.value(),
                    "origin_reuse_blocked",
                    json!({"physically_vacant":!spaces.occupies(source_episode.value()),"logical_origin_held":true}),
                )?;
            }
            audit(&ids, &team, &spaces)?;
        }
        let arrival = depart_end + travel;
        if !operative {
            spaces.finish_cleaning(
                source_episode.value(),
                (depart_end + s.cleaning_seconds) as i64,
            )?;
        }
        advance(&mut theatre, &mut spaces, &team, arrival, &mut rows)?;
        for op in 1..=last {
            transfer(
                &mut boundary,
                envelope(op, 5, 5, arrival, TransferCommand::Arrive),
                &ids,
                &team,
                &mut rows,
            )?;
        }
        deliveries.transition(
            message,
            people[last as usize - 1],
            arrival,
            RecoveryState::Escalated,
        )?;
        transfer(
            &mut boundary,
            envelope(
                last,
                7,
                6,
                arrival,
                TransferCommand::DestinationRejected {
                    compensation: CompensationRequest {
                        operation_id: TransferOperationId::new(100),
                        destination_service: ServiceId::new(4),
                        request_expires_at: None,
                    },
                },
            ),
            &ids,
            &team,
            &mut rows,
        )?;
        // One global tick batch contains all acknowledgements and identity effects.
        let mut actions = Vec::new();
        for op in 1..last {
            actions.push((
                arrival + s.acknowledgement_delay_seconds,
                op,
                6,
                6,
                TransferCommand::CompleteHandover,
            ));
        }
        actions.extend([
            (
                arrival + 1,
                100,
                1,
                1,
                TransferCommand::Offer { expires_at: None },
            ),
            (
                arrival + 2,
                100,
                2,
                2,
                TransferCommand::Reserve {
                    resources: vec![fallback],
                    expires_at: None,
                },
            ),
            (arrival + 3, 100, 3, 3, TransferCommand::BeginDeparture),
            (arrival + 4, 100, 4, 4, TransferCommand::Depart),
            (arrival + 5, 100, 5, 5, TransferCommand::Arrive),
            (
                arrival + 5 + s.acknowledgement_delay_seconds,
                100,
                6,
                6,
                TransferCommand::CompleteHandover,
            ),
        ]);
        actions.sort_by_key(|a| (a.0, a.1, a.2));
        let mut cursor = 0;
        while cursor < actions.len() {
            let tick = actions[cursor].0;
            advance(&mut theatre, &mut spaces, &team, tick, &mut rows)?;
            let mut identity_commands = Vec::new();
            let mut retries = Vec::new();
            while cursor < actions.len() && actions[cursor].0 == tick {
                let (_, op, part, revision, command) = actions[cursor].clone();
                let handover = matches!(command, TransferCommand::CompleteHandover);
                let envelope = envelope(op, part, revision, tick, command);
                transfer(&mut boundary, envelope.clone(), &ids, &team, &mut rows)?;
                if handover {
                    let i = if op == 100 {
                        last as usize - 1
                    } else {
                        op as usize - 1
                    };
                    let destination = if op == 100 {
                        4
                    } else if op == 1 {
                        2
                    } else {
                        3
                    };
                    deliveries.transition(message, people[i], tick, RecoveryState::Discharged)?;
                    identity_commands.extend([
                        id::IdentityCommand::ExitService {
                            event_id: id::LifecycleEventId::new(key + 70 + i as u64),
                            episode_id: episodes[i],
                        },
                        id::IdentityCommand::EnterService {
                            event_id: id::EpisodeEventId::new(key + 80 + i as u64),
                            person_id: people[i],
                            service_id: ServiceId::new(destination),
                        },
                    ]);
                    retries.push(envelope);
                }
                cursor += 1;
            }
            if !identity_commands.is_empty() {
                ids.apply_tick(tick, identity_commands)?;
            }
            for message in retries {
                let before = boundary.core().events().len();
                if boundary.apply(message, &ids, &team)? != ApplyOutcome::Duplicate
                    || boundary.core().events().len() != before
                {
                    return Err("retry duplicated effects".into());
                }
                emit(
                    &mut rows,
                    tick,
                    0,
                    0,
                    "duplicate_ack_ignored",
                    json!({"effects_added":0}),
                )?;
            }
            audit(&ids, &team, &spaces)?;
        }
        let end = actions.last().unwrap().0;
        advance(
            &mut theatre,
            &mut spaces,
            &team,
            end.max(completion + s.cleaning_seconds),
            &mut rows,
        )?;
        let held = boundary
            .core()
            .claims()
            .iter()
            .filter(|c| {
                c.role != ClaimRole::ServiceOccupancy || c.disposition != ClaimDisposition::Occupied
            })
            .count();
        if held != 0 || boundary.core().claims().len() != people.len() {
            return Err("final transfer claim conservation failed".into());
        }
        for (i, person) in people.iter().enumerate() {
            let expected = if i == 0 {
                2
            } else if i + 1 == people.len() {
                4
            } else {
                3
            };
            let snapshot = boundary
                .core()
                .person(PersonId::new(person.value()))
                .ok_or("missing transfer person")?;
            if snapshot.owner_service != ServiceId::new(expected) || snapshot.episodes.len() != 1 {
                return Err("split destination ownership failed".into());
            }
        }
        let counts = ids.counts();
        emit(
            &mut rows,
            end,
            0,
            0,
            "conservation",
            json!({"people":counts.people_created,"births":counts.births,"open_episodes":counts.service_entries-counts.service_exits,
            "held_origin_claims":held,"destination_claims":boundary.core().claims().len(),"origin_occupied_seconds":spaces.occupied_seconds(if operative {ldr_episode.value()} else {source_episode.value()})?}),
        )?;
        audit(&ids, &team, &spaces)?;
        previous.push(boundary);
    }
    Ok(rows)
}
