//! Stable synthetic person/episode identity and relationship lifecycle ledger.
//!
//! Identity IDs are derived from logical event keys using the A5.2 packet's
//! byte encoding. Commands are applied as one canonical batch per sealed virtual
//! tick; this ledger is not an ECS store, clinical policy engine, or scheduler.

use crate::transfer::ServiceId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const IDENTITY_CHECKPOINT_VERSION: u16 = 1;

macro_rules! identity_id {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
        )]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }
            pub const fn value(self) -> u64 {
                self.0
            }
            pub(crate) const fn is_valid(self) -> bool {
                self.0 != 0
            }
        }
    };
}

identity_id!(PersonId);
identity_id!(EpisodeId);
identity_id!(SpawnEventId);
identity_id!(EpisodeEventId);
identity_id!(LifecycleEventId);
identity_id!(RelationshipEventId);
identity_id!(BirthEventId);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "origin", rename_all = "snake_case", deny_unknown_fields)]
pub enum PersonOrigin {
    ExternalEntry,
    Birth {
        birth_event_id: BirthEventId,
        infant_ordinal: u16,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    BirthParentOf,
    AssociatedWith,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
pub enum IdentityCommand {
    Spawn {
        event_id: SpawnEventId,
        origin: PersonOrigin,
    },
    EnterService {
        event_id: EpisodeEventId,
        person_id: PersonId,
        service_id: ServiceId,
    },
    ExitService {
        event_id: LifecycleEventId,
        episode_id: EpisodeId,
    },
    Death {
        event_id: LifecycleEventId,
        person_id: PersonId,
    },
    AddRelationship {
        event_id: RelationshipEventId,
        kind: RelationshipKind,
        from: PersonId,
        to: PersonId,
    },
    RemoveRelationship {
        event_id: RelationshipEventId,
        kind: RelationshipKind,
        from: PersonId,
        to: PersonId,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityOutcome {
    PersonCreated(PersonId),
    EpisodeEntered(EpisodeId),
    EpisodeExited(EpisodeId),
    PersonDied(PersonId),
    RelationshipAdded,
    RelationshipRemoved,
    Duplicate,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonStatus {
    Living,
    Deceased,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeStatus {
    Open,
    Exited,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersonSnapshot {
    pub person_id: PersonId,
    pub origin: PersonOrigin,
    pub status: PersonStatus,
    pub episodes: BTreeSet<EpisodeId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeSnapshot {
    pub episode_id: EpisodeId,
    pub person_id: PersonId,
    pub service_id: ServiceId,
    pub entered_at: u64,
    pub exited_at: Option<u64>,
    pub status: EpisodeStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Relationship {
    pub kind: RelationshipKind,
    pub from: PersonId,
    pub to: PersonId,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityCounts {
    pub people_created: u64,
    pub births: u64,
    pub service_entries: u64,
    pub service_exits: u64,
    pub deaths: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityError {
    InvalidIdentifier,
    InvalidSourceKey,
    EmptyBatch,
    TimeAlreadySealed,
    LateEvent,
    EventConflict,
    IdCollision,
    UnknownPerson,
    UnknownEpisode,
    PersonDeceased,
    OpenEpisodesAtDeath,
    EpisodeAlreadyExited,
    InvalidRelationship,
    RelationshipExists,
    UnknownRelationship,
    BirthOrdinalExists,
    InvalidCheckpoint,
}

impl std::fmt::Display for IdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidIdentifier => "identity identifiers must be non-zero",
            Self::InvalidSourceKey => "synthetic identity source key is empty or too long",
            Self::EmptyBatch => "identity tick batch must contain at least one command",
            Self::TimeAlreadySealed => "identity tick has already been sealed",
            Self::LateEvent => "new identity event arrived after its logical tick was sealed",
            Self::EventConflict => "identity event ID was reused with different content",
            Self::IdCollision => "deterministic identity derivation collided",
            Self::UnknownPerson => "identity command references an unknown person",
            Self::UnknownEpisode => "identity command references an unknown episode",
            Self::PersonDeceased => "deceased person cannot enter a new service episode",
            Self::OpenEpisodesAtDeath => {
                "person death requires every service episode to be explicitly exited"
            }
            Self::EpisodeAlreadyExited => "service episode is already exited",
            Self::InvalidRelationship => "invalid relationship or birth-parent cycle",
            Self::RelationshipExists => "relationship already exists",
            Self::UnknownRelationship => "relationship does not exist",
            Self::BirthOrdinalExists => "birth event and infant ordinal already identify a person",
            Self::InvalidCheckpoint => "invalid or unsupported identity checkpoint",
        };
        f.write_str(message)
    }
}
impl std::error::Error for IdentityError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PersonRecord {
    origin: PersonOrigin,
    status: PersonStatus,
    episodes: BTreeSet<EpisodeId>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct EpisodeRecord {
    person_id: PersonId,
    service_id: ServiceId,
    entered_at: u64,
    exited_at: Option<u64>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct EventRecord {
    command: IdentityCommand,
    tick: u64,
    outcome: IdentityOutcome,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityLedger {
    checkpoint_version: u16,
    run_namespace: u64,
    sealed_through: Option<u64>,
    people: BTreeMap<PersonId, PersonRecord>,
    episodes: BTreeMap<EpisodeId, EpisodeRecord>,
    relationships: BTreeSet<Relationship>,
    birth_ordinals: BTreeSet<(BirthEventId, u16)>,
    events: BTreeMap<String, EventRecord>,
}

impl IdentityCommand {
    fn event_key(&self) -> String {
        match self {
            Self::Spawn { event_id, .. } => format!("spawn:{}", event_id.value()),
            Self::EnterService { event_id, .. } => format!("episode:{}", event_id.value()),
            Self::ExitService { event_id, .. } | Self::Death { event_id, .. } => {
                format!("lifecycle:{}", event_id.value())
            }
            Self::AddRelationship { event_id, .. } | Self::RemoveRelationship { event_id, .. } => {
                format!("relationship:{}", event_id.value())
            }
        }
    }
    fn order_key(&self) -> (u8, u64) {
        let (rank, event_id) = match self {
            Self::Spawn { event_id, .. } => (0, event_id.value()),
            Self::EnterService { event_id, .. } => (1, event_id.value()),
            Self::AddRelationship { event_id, .. } => (2, event_id.value()),
            Self::RemoveRelationship { event_id, .. } => (3, event_id.value()),
            Self::ExitService { event_id, .. } => (4, event_id.value()),
            Self::Death { event_id, .. } => (5, event_id.value()),
        };
        (rank, event_id)
    }
    fn id_valid(&self) -> bool {
        match self {
            Self::Spawn { event_id, origin } => {
                event_id.is_valid()
                    && match origin {
                        PersonOrigin::ExternalEntry => true,
                        PersonOrigin::Birth { birth_event_id, .. } => birth_event_id.is_valid(),
                    }
            }
            Self::EnterService {
                event_id,
                person_id,
                service_id,
            } => event_id.is_valid() && person_id.is_valid() && service_id.value() != 0,
            Self::ExitService {
                event_id,
                episode_id,
            } => event_id.is_valid() && episode_id.is_valid(),
            Self::Death {
                event_id,
                person_id,
            } => event_id.is_valid() && person_id.is_valid(),
            Self::AddRelationship {
                event_id, from, to, ..
            }
            | Self::RemoveRelationship {
                event_id, from, to, ..
            } => event_id.is_valid() && from.is_valid() && to.is_valid(),
        }
    }
}

impl IdentityLedger {
    pub fn new(run_namespace: u64) -> Self {
        Self {
            checkpoint_version: IDENTITY_CHECKPOINT_VERSION,
            run_namespace,
            sealed_through: None,
            people: BTreeMap::new(),
            episodes: BTreeMap::new(),
            relationships: BTreeSet::new(),
            birth_ordinals: BTreeSet::new(),
            events: BTreeMap::new(),
        }
    }

    pub const fn run_namespace(&self) -> u64 {
        self.run_namespace
    }
    pub fn people_count(&self) -> usize {
        self.people.len()
    }
    pub fn episodes_count(&self) -> usize {
        self.episodes.len()
    }
    pub const fn sealed_through(&self) -> Option<u64> {
        self.sealed_through
    }

    /// Apply one complete logical tick atomically. Commands are sorted by
    /// lifecycle rank and typed event key before effects. Retrying an already
    /// applied exact event is duplicate-safe even after its tick is sealed.
    pub fn apply_tick(
        &mut self,
        tick: u64,
        commands: Vec<IdentityCommand>,
    ) -> Result<Vec<IdentityOutcome>, IdentityError> {
        if commands.is_empty() {
            return Err(IdentityError::EmptyBatch);
        }
        let mut candidate = self.clone();
        let mut ordered = commands;
        ordered.sort_by_key(IdentityCommand::order_key);
        let mut outcomes = Vec::with_capacity(ordered.len());
        for command in ordered {
            if !command.id_valid() {
                return Err(IdentityError::InvalidIdentifier);
            }
            let key = command.event_key();
            if let Some(previous) = candidate.events.get(&key) {
                if previous.command == command {
                    outcomes.push(IdentityOutcome::Duplicate);
                    continue;
                }
                return Err(IdentityError::EventConflict);
            }
            if candidate
                .sealed_through
                .is_some_and(|sealed| tick <= sealed)
            {
                return Err(IdentityError::LateEvent);
            }
            let outcome = candidate.apply_new(tick, &command)?;
            candidate.events.insert(
                key,
                EventRecord {
                    command,
                    tick,
                    outcome,
                },
            );
            outcomes.push(outcome);
        }
        if candidate
            .sealed_through
            .is_some_and(|sealed| tick <= sealed)
        {
            // A batch made entirely of known exact retries is harmless.
        } else {
            candidate.sealed_through = Some(tick);
        }
        *self = candidate;
        Ok(outcomes)
    }

    fn apply_new(
        &mut self,
        tick: u64,
        command: &IdentityCommand,
    ) -> Result<IdentityOutcome, IdentityError> {
        match *command {
            IdentityCommand::Spawn { event_id, origin } => {
                if let PersonOrigin::Birth {
                    birth_event_id,
                    infant_ordinal,
                } = origin
                {
                    if !self.birth_ordinals.insert((birth_event_id, infant_ordinal)) {
                        return Err(IdentityError::BirthOrdinalExists);
                    }
                }
                let person_id =
                    PersonId::new(derive_id(b"person", self.run_namespace, event_id.value())?);
                if self.people.contains_key(&person_id) {
                    return Err(IdentityError::IdCollision);
                }
                self.people.insert(
                    person_id,
                    PersonRecord {
                        origin,
                        status: PersonStatus::Living,
                        episodes: BTreeSet::new(),
                    },
                );
                Ok(IdentityOutcome::PersonCreated(person_id))
            }
            IdentityCommand::EnterService {
                event_id,
                person_id,
                service_id,
            } => {
                let person = self
                    .people
                    .get(&person_id)
                    .ok_or(IdentityError::UnknownPerson)?;
                if person.status == PersonStatus::Deceased {
                    return Err(IdentityError::PersonDeceased);
                }
                let episode_id =
                    EpisodeId::new(derive_id(b"episode", self.run_namespace, event_id.value())?);
                if self.episodes.contains_key(&episode_id) {
                    return Err(IdentityError::IdCollision);
                }
                self.episodes.insert(
                    episode_id,
                    EpisodeRecord {
                        person_id,
                        service_id,
                        entered_at: tick,
                        exited_at: None,
                    },
                );
                self.people
                    .get_mut(&person_id)
                    .expect("person checked above")
                    .episodes
                    .insert(episode_id);
                Ok(IdentityOutcome::EpisodeEntered(episode_id))
            }
            IdentityCommand::ExitService { episode_id, .. } => {
                let episode = self
                    .episodes
                    .get_mut(&episode_id)
                    .ok_or(IdentityError::UnknownEpisode)?;
                if episode.exited_at.is_some() {
                    return Err(IdentityError::EpisodeAlreadyExited);
                }
                episode.exited_at = Some(tick);
                Ok(IdentityOutcome::EpisodeExited(episode_id))
            }
            IdentityCommand::Death { person_id, .. } => {
                let person = self
                    .people
                    .get_mut(&person_id)
                    .ok_or(IdentityError::UnknownPerson)?;
                if person.status == PersonStatus::Deceased {
                    return Err(IdentityError::EventConflict);
                }
                if person.episodes.iter().any(|episode| {
                    self.episodes
                        .get(episode)
                        .is_some_and(|record| record.exited_at.is_none())
                }) {
                    return Err(IdentityError::OpenEpisodesAtDeath);
                }
                person.status = PersonStatus::Deceased;
                Ok(IdentityOutcome::PersonDied(person_id))
            }
            IdentityCommand::AddRelationship { kind, from, to, .. } => {
                if !self.people.contains_key(&from) || !self.people.contains_key(&to) || from == to
                {
                    return Err(IdentityError::InvalidRelationship);
                }
                let edge = Relationship { kind, from, to };
                if self.relationships.contains(&edge) {
                    return Err(IdentityError::RelationshipExists);
                }
                if kind == RelationshipKind::BirthParentOf && self.path_exists(to, from) {
                    return Err(IdentityError::InvalidRelationship);
                }
                self.relationships.insert(edge);
                Ok(IdentityOutcome::RelationshipAdded)
            }
            IdentityCommand::RemoveRelationship { kind, from, to, .. } => {
                let edge = Relationship { kind, from, to };
                if !self.relationships.remove(&edge) {
                    return Err(IdentityError::UnknownRelationship);
                }
                Ok(IdentityOutcome::RelationshipRemoved)
            }
        }
    }

    fn path_exists(&self, start: PersonId, goal: PersonId) -> bool {
        path_exists_in(&self.relationships, start, goal)
    }

    pub fn spawn_record(&self, event_id: SpawnEventId) -> Option<(PersonId, PersonOrigin)> {
        let key = format!("spawn:{}", event_id.value());
        self.events
            .get(&key)
            .and_then(|record| match &record.command {
                IdentityCommand::Spawn { origin, .. } => Some((
                    PersonId::new(derive_id(b"person", self.run_namespace, event_id.value()).ok()?),
                    *origin,
                )),
                _ => None,
            })
    }

    pub fn episode_entry(&self, event_id: EpisodeEventId) -> Option<EpisodeSnapshot> {
        let key = format!("episode:{}", event_id.value());
        self.events
            .get(&key)
            .and_then(|record| match &record.command {
                IdentityCommand::EnterService { .. } => match record.outcome {
                    IdentityOutcome::EpisodeEntered(id) => self.episode(id),
                    _ => None,
                },
                _ => None,
            })
    }

    /// Return the relationship created by this add event even when a later
    /// explicit remove event has ended the active link.
    pub fn relationship_add_record(&self, event_id: RelationshipEventId) -> Option<Relationship> {
        let key = format!("relationship:{}", event_id.value());
        self.events
            .get(&key)
            .and_then(|record| match &record.command {
                IdentityCommand::AddRelationship { kind, from, to, .. } => Some(Relationship {
                    kind: *kind,
                    from: *from,
                    to: *to,
                }),
                _ => None,
            })
    }

    pub fn person(&self, person_id: PersonId) -> Option<PersonSnapshot> {
        self.people.get(&person_id).map(|person| PersonSnapshot {
            person_id,
            origin: person.origin,
            status: person.status,
            episodes: person.episodes.clone(),
        })
    }
    pub fn episode(&self, episode_id: EpisodeId) -> Option<EpisodeSnapshot> {
        self.episodes
            .get(&episode_id)
            .map(|episode| EpisodeSnapshot {
                episode_id,
                person_id: episode.person_id,
                service_id: episode.service_id,
                entered_at: episode.entered_at,
                exited_at: episode.exited_at,
                status: if episode.exited_at.is_some() {
                    EpisodeStatus::Exited
                } else {
                    EpisodeStatus::Open
                },
            })
    }
    pub fn relationships(&self) -> Vec<Relationship> {
        self.relationships.iter().cloned().collect()
    }
    pub fn counts(&self) -> IdentityCounts {
        self.events
            .values()
            .fold(IdentityCounts::default(), |mut counts, event| {
                match &event.command {
                    IdentityCommand::Spawn { origin, .. } => {
                        counts.people_created += 1;
                        if matches!(origin, PersonOrigin::Birth { .. }) {
                            counts.births += 1;
                        }
                    }
                    IdentityCommand::EnterService { .. } => counts.service_entries += 1,
                    IdentityCommand::ExitService { .. } => counts.service_exits += 1,
                    IdentityCommand::Death { .. } => counts.deaths += 1,
                    IdentityCommand::AddRelationship { .. }
                    | IdentityCommand::RemoveRelationship { .. } => {}
                }
                counts
            })
    }
    pub fn checkpoint(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
    pub fn validate_checkpoint(&self) -> Result<(), IdentityError> {
        self.validate()
    }
    pub fn restore(bytes: &[u8]) -> Result<Self, IdentityError> {
        let ledger: Self =
            serde_json::from_slice(bytes).map_err(|_| IdentityError::InvalidCheckpoint)?;
        ledger
            .validate()
            .map_err(|_| IdentityError::InvalidCheckpoint)?;
        Ok(ledger)
    }

    fn validate(&self) -> Result<(), IdentityError> {
        if self.checkpoint_version != IDENTITY_CHECKPOINT_VERSION {
            return Err(IdentityError::InvalidCheckpoint);
        }
        for (person_id, person) in &self.people {
            if !person_id.is_valid() {
                return Err(IdentityError::InvalidCheckpoint);
            }
            for episode_id in &person.episodes {
                let episode = self
                    .episodes
                    .get(episode_id)
                    .ok_or(IdentityError::InvalidCheckpoint)?;
                if episode.person_id != *person_id {
                    return Err(IdentityError::InvalidCheckpoint);
                }
            }
            if person.status == PersonStatus::Deceased
                && person.episodes.iter().any(|id| {
                    self.episodes
                        .get(id)
                        .is_some_and(|episode| episode.exited_at.is_none())
                })
            {
                return Err(IdentityError::InvalidCheckpoint);
            }
        }
        for (episode_id, episode) in &self.episodes {
            if !episode_id.is_valid()
                || !self
                    .people
                    .get(&episode.person_id)
                    .is_some_and(|p| p.episodes.contains(episode_id))
                || episode
                    .exited_at
                    .is_some_and(|end| end < episode.entered_at)
            {
                return Err(IdentityError::InvalidCheckpoint);
            }
        }
        for edge in &self.relationships {
            if !self.people.contains_key(&edge.from)
                || !self.people.contains_key(&edge.to)
                || edge.from == edge.to
            {
                return Err(IdentityError::InvalidCheckpoint);
            }
        }
        for edge in self
            .relationships
            .iter()
            .filter(|edge| edge.kind == RelationshipKind::BirthParentOf)
        {
            if path_exists_in(&self.relationships, edge.to, edge.from) {
                return Err(IdentityError::InvalidCheckpoint);
            }
        }
        let mut expected_birth_ordinals = BTreeSet::new();
        let mut expected_people = BTreeSet::new();
        let mut expected_episodes = BTreeSet::new();
        let mut ordered_relationship_events: Vec<_> = self
            .events
            .values()
            .filter(|event| {
                matches!(
                    event.command,
                    IdentityCommand::AddRelationship { .. }
                        | IdentityCommand::RemoveRelationship { .. }
                )
            })
            .collect();
        ordered_relationship_events.sort_by_key(|event| (event.tick, event.command.order_key()));
        let mut reconstructed_relationships = BTreeSet::new();
        for event in ordered_relationship_events {
            match event.command {
                IdentityCommand::AddRelationship { kind, from, to, .. } => {
                    if !self.people.contains_key(&from)
                        || !self.people.contains_key(&to)
                        || from == to
                    {
                        return Err(IdentityError::InvalidCheckpoint);
                    }
                    let edge = Relationship { kind, from, to };
                    if !reconstructed_relationships.insert(edge) {
                        return Err(IdentityError::InvalidCheckpoint);
                    }
                    if kind == RelationshipKind::BirthParentOf
                        && path_exists_in(&reconstructed_relationships, to, from)
                    {
                        return Err(IdentityError::InvalidCheckpoint);
                    }
                }
                IdentityCommand::RemoveRelationship { kind, from, to, .. } => {
                    if !reconstructed_relationships.remove(&Relationship { kind, from, to }) {
                        return Err(IdentityError::InvalidCheckpoint);
                    }
                }
                _ => unreachable!(),
            }
        }
        if reconstructed_relationships != self.relationships {
            return Err(IdentityError::InvalidCheckpoint);
        }
        for (key, event) in &self.events {
            if event.command.event_key() != *key
                || self.sealed_through.is_none_or(|tick| event.tick > tick)
                || !event.command.id_valid()
            {
                return Err(IdentityError::InvalidCheckpoint);
            }
            match &event.command {
                IdentityCommand::Spawn { event_id, origin } => {
                    let expected = PersonId::new(
                        derive_id(b"person", self.run_namespace, event_id.value())
                            .map_err(|_| IdentityError::InvalidCheckpoint)?,
                    );
                    if self
                        .people
                        .get(&expected)
                        .is_none_or(|record| record.origin != *origin)
                        || event.outcome != IdentityOutcome::PersonCreated(expected)
                    {
                        return Err(IdentityError::InvalidCheckpoint);
                    }
                    if !expected_people.insert(expected) {
                        return Err(IdentityError::InvalidCheckpoint);
                    }
                    if let PersonOrigin::Birth {
                        birth_event_id,
                        infant_ordinal,
                    } = origin
                    {
                        if !expected_birth_ordinals.insert((*birth_event_id, *infant_ordinal)) {
                            return Err(IdentityError::InvalidCheckpoint);
                        }
                    }
                }
                IdentityCommand::EnterService {
                    event_id,
                    person_id,
                    service_id,
                } => {
                    let expected = EpisodeId::new(
                        derive_id(b"episode", self.run_namespace, event_id.value())
                            .map_err(|_| IdentityError::InvalidCheckpoint)?,
                    );
                    if self.episodes.get(&expected).is_none_or(|record| {
                        record.person_id != *person_id
                            || record.service_id != *service_id
                            || record.entered_at != event.tick
                    }) || event.outcome != IdentityOutcome::EpisodeEntered(expected)
                        || !expected_episodes.insert(expected)
                    {
                        return Err(IdentityError::InvalidCheckpoint);
                    }
                }
                IdentityCommand::ExitService { episode_id, .. } => {
                    if self
                        .episodes
                        .get(episode_id)
                        .is_none_or(|record| record.exited_at != Some(event.tick))
                        || event.outcome != IdentityOutcome::EpisodeExited(*episode_id)
                    {
                        return Err(IdentityError::InvalidCheckpoint);
                    }
                }
                IdentityCommand::Death { person_id, .. } => {
                    if self
                        .people
                        .get(person_id)
                        .is_none_or(|record| record.status != PersonStatus::Deceased)
                        || event.outcome != IdentityOutcome::PersonDied(*person_id)
                    {
                        return Err(IdentityError::InvalidCheckpoint);
                    }
                }
                IdentityCommand::AddRelationship { from, to, .. }
                | IdentityCommand::RemoveRelationship { from, to, .. } => {
                    if !self.people.contains_key(from) || !self.people.contains_key(to) {
                        return Err(IdentityError::InvalidCheckpoint);
                    }
                    match &event.command {
                        IdentityCommand::AddRelationship { .. }
                            if event.outcome != IdentityOutcome::RelationshipAdded =>
                        {
                            return Err(IdentityError::InvalidCheckpoint)
                        }
                        IdentityCommand::RemoveRelationship { .. }
                            if event.outcome != IdentityOutcome::RelationshipRemoved =>
                        {
                            return Err(IdentityError::InvalidCheckpoint)
                        }
                        _ => {}
                    }
                }
            }
        }
        if expected_birth_ordinals != self.birth_ordinals
            || expected_people != self.people.keys().copied().collect()
            || expected_episodes != self.episodes.keys().copied().collect()
        {
            return Err(IdentityError::InvalidCheckpoint);
        }

        // Re-execute every accepted command in canonical time/order. Structural
        // cross-checks above are intentionally supplemented by this replay so a
        // checkpoint cannot claim unlogged deaths/exits or repeat transitions.
        let mut ordered_events: Vec<_> = self.events.iter().collect();
        ordered_events.sort_by_key(|(_, event)| (event.tick, event.command.order_key()));
        let mut replay = IdentityLedger::new(self.run_namespace);
        for (key, event) in ordered_events {
            let outcome = replay
                .apply_new(event.tick, &event.command)
                .map_err(|_| IdentityError::InvalidCheckpoint)?;
            if outcome != event.outcome {
                return Err(IdentityError::InvalidCheckpoint);
            }
            replay.events.insert(key.clone(), event.clone());
            replay.sealed_through = Some(
                replay
                    .sealed_through
                    .map_or(event.tick, |sealed| sealed.max(event.tick)),
            );
        }
        // The watermark may exceed the latest recorded event: apply_tick
        // preserves a later-tick exact retry as a seal without adding a second
        // event record. Require it to cover replayed history, but preserve the
        // separately stored later watermark.
        let watermark_invalid = match (replay.sealed_through, self.sealed_through) {
            (None, None) => false,
            (None, Some(_)) => true,
            (Some(_), None) => true,
            (Some(replayed), Some(stored)) => stored < replayed,
        };
        if replay.people != self.people
            || replay.episodes != self.episodes
            || replay.relationships != self.relationships
            || replay.birth_ordinals != self.birth_ordinals
            || watermark_invalid
        {
            return Err(IdentityError::InvalidCheckpoint);
        }
        Ok(())
    }
}

fn path_exists_in(relationships: &BTreeSet<Relationship>, start: PersonId, goal: PersonId) -> bool {
    let mut pending = vec![start];
    let mut seen = BTreeSet::new();
    while let Some(node) = pending.pop() {
        if node == goal {
            return true;
        }
        if !seen.insert(node) {
            continue;
        }
        pending.extend(
            relationships
                .iter()
                .filter(|edge| edge.kind == RelationshipKind::BirthParentOf && edge.from == node)
                .map(|edge| edge.to),
        );
    }
    false
}

/// Derive an ED source-person spawn event key using the byte-exact A5.2 packet encoding.
pub fn ed_person_spawn_event_id(
    run_namespace: u64,
    source_person_key: &str,
) -> Result<SpawnEventId, IdentityError> {
    source_event_key(b"person", run_namespace, source_person_key).map(SpawnEventId::new)
}

/// Derive an ED admission event key; a new stable admission key creates a new EpisodeId.
pub fn ed_admission_event_id(
    run_namespace: u64,
    admission_key: &str,
) -> Result<EpisodeEventId, IdentityError> {
    source_event_key(b"admission", run_namespace, admission_key).map(EpisodeEventId::new)
}

pub fn person_id_for_spawn(
    run_namespace: u64,
    event_id: SpawnEventId,
) -> Result<PersonId, IdentityError> {
    derive_id(b"person", run_namespace, event_id.value()).map(PersonId::new)
}

pub fn episode_id_for_entry(
    run_namespace: u64,
    event_id: EpisodeEventId,
) -> Result<EpisodeId, IdentityError> {
    derive_id(b"episode", run_namespace, event_id.value()).map(EpisodeId::new)
}

fn source_event_key(kind: &[u8], run_namespace: u64, key: &str) -> Result<u64, IdentityError> {
    if key.is_empty() || key.len() > 4096 || key.len() > u32::MAX as usize {
        return Err(IdentityError::InvalidSourceKey);
    }
    let mut hasher = Sha256::new();
    hasher.update(b"careops.ed.key.v1\0");
    hasher.update(kind);
    hasher.update([0]);
    hasher.update(run_namespace.to_be_bytes());
    hasher.update((key.len() as u32).to_be_bytes());
    hasher.update(key.as_bytes());
    let digest = hasher.finalize();
    let value = u64::from_be_bytes(
        digest[..8]
            .try_into()
            .expect("SHA-256 has at least eight bytes"),
    );
    if value == 0 {
        Err(IdentityError::IdCollision)
    } else {
        Ok(value)
    }
}

/// A5.2 packet-defined SHA-256 derivation. The first digest u64 is big-endian.
fn derive_id(kind: &[u8], run_namespace: u64, event_key: u64) -> Result<u64, IdentityError> {
    let mut hasher = Sha256::new();
    hasher.update(b"careops.identity.id.v1\0");
    hasher.update(kind);
    hasher.update([0]);
    hasher.update(run_namespace.to_be_bytes());
    hasher.update(event_key.to_be_bytes());
    let digest = hasher.finalize();
    let value = u64::from_be_bytes(
        digest[..8]
            .try_into()
            .expect("SHA-256 has at least eight bytes"),
    );
    if value == 0 {
        Err(IdentityError::IdCollision)
    } else {
        Ok(value)
    }
}
