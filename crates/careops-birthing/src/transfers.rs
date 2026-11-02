//! Birth-recipient routing over the qualified single-scheduler transfer ledger.
//! Resource declarations are model inputs, not proof of physical availability.
//! Physical vacancy, transfer claims and escort duties remain separate ledgers.
//! The coordinator owns actual identity service-entry/exit effects and gates
//! physical reuse while an origin claim is still held for acknowledgement.

use crate::{delivery::DeliveryLedger, team::TeamAllocator};
use careops_core::staffing::{BookingId, DutyKind, EscortChainId, StaffId};
use careops_core::{birth::BirthMessageId, identity, transfer::*};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvailabilityWindow {
    pub start: u64,
    pub end: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DestinationProfile {
    pub revision: String,
    pub resources: Vec<ResourceId>,
    pub availability: Option<Vec<AvailabilityWindow>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutingProfile {
    pub revision: String,
    pub resources: BTreeMap<ResourceId, String>,
    pub destinations: BTreeMap<ServiceId, DestinationProfile>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EscortBinding {
    pub staff: u64,
    pub booking: u64,
    pub chain: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipientPlan {
    pub operation: TransferOperationId,
    pub person: identity::PersonId,
    pub source_episode: identity::EpisodeId,
    pub origin_resources: Vec<ResourceId>,
    pub request_expires_at: Option<u64>,
    pub fallback_services: Vec<ServiceId>,
    pub escort: Option<EscortBinding>,
}
#[derive(Clone, Debug)]
struct BoundRoute {
    plan: RecipientPlan,
    request: TransferRequest,
}
#[derive(Clone, Debug)]
pub struct TransferBoundary {
    profile: RoutingProfile,
    roots: BTreeMap<TransferOperationId, BoundRoute>,
    manager: TransferManager,
}

impl TransferBoundary {
    /// Bind immutable birth provenance and source-episode references. Historical
    /// closed episodes may be referenced; new effects require live source care.
    /// More than one planned attempt is allowed after a prior terminal result.
    pub fn bind_delivery(
        deliveries: &DeliveryLedger,
        message: BirthMessageId,
        identities: &identity::IdentityLedger,
        profile: RoutingProfile,
        plans: Vec<RecipientPlan>,
    ) -> Result<Self, String> {
        validate_profile(&profile)?;
        if plans.is_empty() || plans.len() > 1024 {
            return Err("unsupported route-plan count".into());
        }
        let record = deliveries
            .record(message)
            .ok_or("unknown delivery message")?;
        let birth = &record.birth;
        if identities.person(birth.mother_person_id).is_none() {
            return Err("unpaired maternal identity".into());
        }
        let mut expected =
            BTreeMap::from([(birth.mother_person_id, birth.maternal_destination_service)]);
        for infant in &birth.infants {
            let person =
                identity::person_id_for_spawn(identities.run_namespace(), infant.spawn_event_id)
                    .map_err(|e| e.to_string())?;
            let origin = identity::PersonOrigin::Birth {
                birth_event_id: birth.birth_event_id,
                infant_ordinal: infant.infant_ordinal,
            };
            let relation = identity::Relationship {
                kind: identity::RelationshipKind::BirthParentOf,
                from: birth.mother_person_id,
                to: person,
            };
            if identities.spawn_record(infant.spawn_event_id) != Some((person, origin))
                || identities.relationship_add_record(infant.relationship_event_id)
                    != Some(relation)
            {
                return Err("unpaired infant birth provenance".into());
            }
            if expected
                .insert(person, infant.destination_service)
                .is_some()
            {
                return Err("duplicate birth recipient".into());
            }
        }
        let mut seen = BTreeSet::new();
        let mut roots = BTreeMap::new();
        for mut plan in plans {
            let destination = *expected
                .get(&plan.person)
                .ok_or("route is not a delivery recipient")?;
            let episode = identities
                .episode(plan.source_episode)
                .ok_or("unknown source episode")?;
            if episode.person_id != plan.person
                || episode.service_id == destination
                || plan.operation.value() == 0
                || !profile.destinations.contains_key(&destination)
                || roots.contains_key(&plan.operation)
                || !unique_declared(&plan.origin_resources, &profile.resources)
                || plan
                    .fallback_services
                    .iter()
                    .any(|s| !profile.destinations.contains_key(s))
                || plan
                    .fallback_services
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>()
                    .len()
                    != plan.fallback_services.len()
                || plan
                    .escort
                    .is_some_and(|e| e.staff == 0 || e.booking == 0 || e.chain == 0)
            {
                return Err("invalid identity, resource or fallback route binding".into());
            }
            plan.origin_resources.sort();
            plan.fallback_services.sort();
            seen.insert(plan.person);
            let request = TransferRequest {
                person_id: PersonId::new(plan.person.value()),
                episode_id: EpisodeId::new(plan.source_episode.value()),
                origin_service: episode.service_id,
                destination_service: destination,
                request_expires_at: plan.request_expires_at,
                origin_resources: plan.origin_resources.clone(),
            };
            roots.insert(plan.operation, BoundRoute { plan, request });
        }
        if seen != expected.keys().copied().collect() {
            return Err("every birth recipient needs a route plan".into());
        }
        Ok(Self {
            profile,
            roots,
            manager: TransferManager::default(),
        })
    }
    pub fn core(&self) -> &TransferManager {
        &self.manager
    }
    pub fn profile(&self) -> &RoutingProfile {
        &self.profile
    }
    pub fn resource_unit(&self, resource: ResourceId) -> Option<&str> {
        self.profile.resources.get(&resource).map(String::as_str)
    }
    /// This is only logical transfer-claim availability. Combine it with actual
    /// physical/effective capacity; it is never an allocation grant by itself.
    pub fn resource_unclaimed(&self, resource: ResourceId) -> Result<bool, String> {
        if !self.profile.resources.contains_key(&resource) {
            return Err("undeclared resource".into());
        }
        Ok(!self
            .manager
            .claims()
            .iter()
            .any(|claim| claim.resource_id == resource))
    }
    pub fn request(
        &mut self,
        operation: TransferOperationId,
        message: MessageId,
        tick: u64,
        identities: &identity::IdentityLedger,
        team: &TeamAllocator,
    ) -> Result<ApplyOutcome, String> {
        let request = self
            .roots
            .get(&operation)
            .ok_or("unknown root route")?
            .request
            .clone();
        self.apply(
            TransferEnvelope::new(
                operation,
                message,
                0,
                tick,
                TransferCommand::Request { request },
            ),
            identities,
            team,
        )
    }
    pub fn apply(
        &mut self,
        envelope: TransferEnvelope,
        identities: &identity::IdentityLedger,
        team: &TeamAllocator,
    ) -> Result<ApplyOutcome, String> {
        let root = self.root_for(&self.manager, envelope.operation_id)?;
        match &envelope.command {
            TransferCommand::Request { request } => {
                if !self.roots.contains_key(&envelope.operation_id) || *request != root.request {
                    return Err("request does not match declared source route".into());
                }
            }
            TransferCommand::Reserve { resources, .. } => {
                let destination = self
                    .manager
                    .snapshot(envelope.operation_id)
                    .map_or(root.request.destination_service, |s| s.destination_service);
                self.validate_reservation(destination, resources, envelope.tick)?;
            }
            TransferCommand::DestinationRejected { compensation }
                if !root
                    .plan
                    .fallback_services
                    .contains(&compensation.destination_service)
                    || self.roots.contains_key(&compensation.operation_id) =>
            {
                return Err("fallback destination or operation is outside declared scope".into());
            }
            _ => {}
        }
        let before = self.manager.events().len();
        let mut candidate = self.manager.clone();
        let outcome = candidate.apply(envelope).map_err(|e| e.to_string())?;
        self.validate_effects(&candidate, before, identities, team)?;
        self.manager = candidate;
        Ok(outcome)
    }
    pub fn advance_to(
        &mut self,
        tick: u64,
        identities: &identity::IdentityLedger,
        team: &TeamAllocator,
    ) -> Result<(), String> {
        let before = self.manager.events().len();
        let mut candidate = self.manager.clone();
        candidate.advance_to(tick).map_err(|e| e.to_string())?;
        self.validate_effects(&candidate, before, identities, team)?;
        self.manager = candidate;
        Ok(())
    }
    fn root_for<'a>(
        &'a self,
        manager: &TransferManager,
        mut operation: TransferOperationId,
    ) -> Result<&'a BoundRoute, String> {
        for _ in 0..64 {
            if let Some(root) = self.roots.get(&operation) {
                return Ok(root);
            }
            operation = manager
                .snapshot(operation)
                .and_then(|s| s.compensation_parent)
                .ok_or("unbound transfer operation")?;
        }
        Err("compensation depth exceeds bounded route contract".into())
    }
    fn validate_reservation(
        &self,
        service: ServiceId,
        resources: &[ResourceId],
        tick: u64,
    ) -> Result<(), String> {
        let profile = self
            .profile
            .destinations
            .get(&service)
            .ok_or("unknown destination profile")?;
        let windows = profile
            .availability
            .as_ref()
            .ok_or("destination availability is unknown")?;
        if !windows.iter().any(|w| w.start <= tick && tick < w.end) {
            return Err("destination is closed".into());
        }
        if resources.is_empty() || resources.iter().any(|r| !profile.resources.contains(r)) {
            return Err("destination resources are undeclared".into());
        }
        Ok(())
    }
    fn validate_effects(
        &self,
        manager: &TransferManager,
        before: usize,
        identities: &identity::IdentityLedger,
        team: &TeamAllocator,
    ) -> Result<(), String> {
        if manager.events().len() > 100_000 {
            return Err("bounded transfer event limit exceeded".into());
        }
        for event in &manager.events()[before..] {
            let root = self.root_for(manager, event.operation_id)?;
            let person = identities
                .person(root.plan.person)
                .ok_or("unpaired transfer person")?;
            let episode = identities
                .episode(root.plan.source_episode)
                .ok_or("unpaired transfer episode")?;
            if person.status != identity::PersonStatus::Living
                || episode.person_id != root.plan.person
                || episode.service_id != root.request.origin_service
                || episode.status != identity::EpisodeStatus::Open
                || episode.entered_at > event.tick
            {
                return Err("new transfer effect lacks live source-episode ownership".into());
            }
            if event.effect == TransferEffect::Reserved {
                let snapshot = manager
                    .snapshot(event.operation_id)
                    .ok_or("missing transfer snapshot")?;
                // Input resources are validated before buffering. Availability is
                // also checked at actual application time when a buffer drains.
                let destination = self
                    .profile
                    .destinations
                    .get(&snapshot.destination_service)
                    .ok_or("unprofiled destination")?;
                if !destination.availability.as_ref().is_some_and(|windows| {
                    windows
                        .iter()
                        .any(|w| w.start <= event.tick && event.tick < w.end)
                }) {
                    return Err("destination unavailable at applied reservation time".into());
                }
            }
            if matches!(
                event.effect,
                TransferEffect::Departed | TransferEffect::HandoverCompleted
            ) {
                if let Some(escort) = root.plan.escort {
                    let from = manager
                        .events()
                        .iter()
                        .filter(|e| e.effect == TransferEffect::Departed)
                        .filter(|e| {
                            self.root_for(manager, e.operation_id)
                                .is_ok_and(|r| r.plan.operation == root.plan.operation)
                        })
                        .map(|e| e.tick)
                        .min()
                        .ok_or("escort has no departure anchor")?;
                    validate_escort(
                        team,
                        escort,
                        from,
                        event.tick,
                        event.effect == TransferEffect::Departed,
                    )?;
                }
            }
        }
        Ok(())
    }
}

fn validate_profile(profile: &RoutingProfile) -> Result<(), String> {
    if profile.revision.trim().is_empty()
        || profile.resources.len() > 4096
        || profile.destinations.len() > 1024
        || profile
            .resources
            .iter()
            .any(|(id, unit)| id.value() == 0 || unit.trim().is_empty())
    {
        return Err("invalid resource/profile declaration".into());
    }
    for (service, destination) in &profile.destinations {
        if service.value() == 0
            || destination.revision.trim().is_empty()
            || !unique_declared(&destination.resources, &profile.resources)
        {
            return Err("invalid destination declaration".into());
        }
        if let Some(windows) = &destination.availability {
            let mut ordered = windows.clone();
            ordered.sort_by_key(|w| w.start);
            if ordered.iter().any(|w| w.start >= w.end)
                || ordered.windows(2).any(|w| w[0].end > w[1].start)
            {
                return Err("invalid destination availability windows".into());
            }
        }
    }
    Ok(())
}
fn unique_declared(resources: &[ResourceId], declared: &BTreeMap<ResourceId, String>) -> bool {
    resources.iter().all(|id| declared.contains_key(id))
        && resources.iter().copied().collect::<BTreeSet<_>>().len() == resources.len()
}
fn validate_escort(
    team: &TeamAllocator,
    binding: EscortBinding,
    from: u64,
    until: u64,
    departing: bool,
) -> Result<(), String> {
    let from = i64::try_from(from).map_err(|_| "escort time overflow")?;
    let until = i64::try_from(until).map_err(|_| "escort time overflow")?;
    let mut windows: Vec<_> = team
        .registry()
        .assignments()
        .into_iter()
        .filter(|a| {
            a.staff == StaffId::new(binding.staff)
                && a.booking == BookingId::new(binding.booking)
                && a.kind
                    == DutyKind::Escort {
                        chain: EscortChainId::new(binding.chain),
                    }
        })
        .map(|a| a.interval)
        .collect();
    windows.sort();
    if departing && !windows.iter().any(|w| w.contains(until)) {
        return Err("departure has no committed escort".into());
    }
    if from == until {
        return if windows.iter().any(|w| w.contains(from)) {
            Ok(())
        } else {
            Err("zero-time transfer still lacks declared escort".into())
        };
    }
    let mut cursor = from;
    for window in windows {
        if window.end() <= cursor {
            continue;
        }
        if window.start() > cursor {
            break;
        }
        cursor = window.end();
        if cursor >= until {
            return Ok(());
        }
    }
    Err("escort coverage does not span the actual transfer".into())
}
