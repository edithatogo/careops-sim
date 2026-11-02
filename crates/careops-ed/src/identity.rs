//! Synthetic ED identity adapter for the reusable CareOps identity ledger.
//!
//! Source keys are opaque synthetic UTF-8 identifiers. Their exact bytes are
//! hashed; this adapter does not normalize, infer clinical identity, or accept
//! real patient identifiers as a deployment claim.

use careops_core::identity::{
    ed_admission_event_id, ed_person_spawn_event_id, episode_id_for_entry, person_id_for_spawn,
    EpisodeEventId, EpisodeId, EpisodeSnapshot, IdentityCommand, IdentityError, IdentityLedger,
    PersonId, PersonOrigin, RelationshipKind, SpawnEventId,
};
use careops_core::transfer::ServiceId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EdAdmissionInput {
    pub source_person_key: String,
    pub admission_key: String,
    pub service_id: ServiceId,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EdAdmissionIdentity {
    pub person_id: PersonId,
    pub episode_id: EpisodeId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct EdAdmissionRecord {
    source_person_key: String,
    person_id: PersonId,
    episode_id: EpisodeId,
    service_id: ServiceId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EdIdentityError {
    InvalidSourceKey,
    SourceKeyCollision,
    AdmissionKeyConflict,
    IdCollision,
    Ledger(IdentityError),
}
impl std::fmt::Display for EdIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSourceKey => f.write_str("synthetic ED identity key is empty or too long"),
            Self::SourceKeyCollision => {
                f.write_str("different ED source keys derived the same event ID")
            }
            Self::AdmissionKeyConflict => {
                f.write_str("admission key was reused with different person or service")
            }
            Self::IdCollision => {
                f.write_str("ED identity key derivation produced a zero or colliding ID")
            }
            Self::Ledger(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for EdIdentityError {}
impl From<IdentityError> for EdIdentityError {
    fn from(value: IdentityError) -> Self {
        Self::Ledger(value)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EdIdentityAdapter {
    run_namespace: u64,
    ledger: IdentityLedger,
    people: BTreeMap<String, PersonId>,
    spawn_keys: BTreeMap<SpawnEventId, String>,
    admissions: BTreeMap<String, EdAdmissionRecord>,
    episode_keys: BTreeMap<EpisodeEventId, String>,
}

impl EdIdentityAdapter {
    pub fn new(run_namespace: u64) -> Self {
        Self {
            run_namespace,
            ledger: IdentityLedger::new(run_namespace),
            people: BTreeMap::new(),
            spawn_keys: BTreeMap::new(),
            admissions: BTreeMap::new(),
            episode_keys: BTreeMap::new(),
        }
    }

    /// Register one or more service admissions at a single logical tick. A
    /// repeated source person key keeps its PersonId; a new admission key gets
    /// a new EpisodeId. The core ledger sorts events canonically and commits the
    /// complete batch atomically.
    pub fn admit_batch(
        &mut self,
        tick: u64,
        inputs: &[EdAdmissionInput],
    ) -> Result<Vec<EdAdmissionIdentity>, EdIdentityError> {
        let mut candidate = self.clone();
        let mut commands = Vec::new();
        let mut pending_spawn_keys = BTreeMap::<SpawnEventId, String>::new();
        let mut result = Vec::with_capacity(inputs.len());
        let mut pending = Vec::with_capacity(inputs.len());

        for input in inputs {
            let spawn_event =
                ed_person_spawn_event_id(candidate.run_namespace, &input.source_person_key)?;
            let person_id = if let Some(existing) = candidate.people.get(&input.source_person_key) {
                *existing
            } else {
                if candidate
                    .spawn_keys
                    .get(&spawn_event)
                    .is_some_and(|key| key != &input.source_person_key)
                    || pending_spawn_keys
                        .get(&spawn_event)
                        .is_some_and(|key| key != &input.source_person_key)
                {
                    return Err(EdIdentityError::SourceKeyCollision);
                }
                if !candidate.spawn_keys.contains_key(&spawn_event)
                    && !pending_spawn_keys.contains_key(&spawn_event)
                {
                    commands.push(IdentityCommand::Spawn {
                        event_id: spawn_event,
                        origin: PersonOrigin::ExternalEntry,
                    });
                    pending_spawn_keys.insert(spawn_event, input.source_person_key.clone());
                }
                person_id_for_spawn(candidate.run_namespace, spawn_event)?
            };

            let episode_event =
                ed_admission_event_id(candidate.run_namespace, &input.admission_key)?;
            if let Some(prior) = candidate.admissions.get(&input.admission_key) {
                if prior.person_id != person_id
                    || prior.service_id != input.service_id
                    || prior.source_person_key != input.source_person_key
                {
                    return Err(EdIdentityError::AdmissionKeyConflict);
                }
                result.push(EdAdmissionIdentity {
                    person_id: prior.person_id,
                    episode_id: prior.episode_id,
                });
                pending.push(None);
                continue;
            }
            if candidate
                .episode_keys
                .get(&episode_event)
                .is_some_and(|key| key != &input.admission_key)
                || pending.iter().flatten().any(
                    |(event, key, _, _): &(EpisodeEventId, String, PersonId, ServiceId)| {
                        *event == episode_event && key != &input.admission_key
                    },
                )
            {
                return Err(EdIdentityError::SourceKeyCollision);
            }
            if candidate
                .episode_keys
                .get(&episode_event)
                .is_some_and(|key| key == &input.admission_key)
            {
                return Err(EdIdentityError::AdmissionKeyConflict);
            }
            commands.push(IdentityCommand::EnterService {
                event_id: episode_event,
                person_id,
                service_id: input.service_id,
            });
            let episode_id = episode_id_for_entry(candidate.run_namespace, episode_event)?;
            result.push(EdAdmissionIdentity {
                person_id,
                episode_id,
            });
            pending.push(Some((
                episode_event,
                input.admission_key.clone(),
                person_id,
                input.service_id,
            )));
        }

        if !commands.is_empty() {
            candidate.ledger.apply_tick(tick, commands)?;
        }
        for input in inputs {
            if let Some(identity) = candidate.people.get(&input.source_person_key) {
                // Existing source identities remain stable across readmission.
                let _ = identity;
            } else {
                let event =
                    ed_person_spawn_event_id(candidate.run_namespace, &input.source_person_key)?;
                let person = person_id_for_spawn(candidate.run_namespace, event)?;
                candidate
                    .people
                    .insert(input.source_person_key.clone(), person);
                candidate
                    .spawn_keys
                    .insert(event, input.source_person_key.clone());
            }
        }
        for (index, maybe) in pending.into_iter().enumerate() {
            if let Some((event, key, person, service)) = maybe {
                let episode = result[index].episode_id;
                candidate.admissions.insert(
                    key.clone(),
                    EdAdmissionRecord {
                        source_person_key: inputs[index].source_person_key.clone(),
                        person_id: person,
                        episode_id: episode,
                        service_id: service,
                    },
                );
                candidate.episode_keys.insert(event, key);
            }
        }
        *self = candidate;
        Ok(result)
    }

    pub fn admit(
        &mut self,
        tick: u64,
        input: EdAdmissionInput,
    ) -> Result<EdAdmissionIdentity, EdIdentityError> {
        self.admit_batch(tick, &[input])
            .map(|mut values| values.remove(0))
    }

    pub fn add_birth_parent(
        &mut self,
        tick: u64,
        event_id: u64,
        parent: PersonId,
        infant: PersonId,
    ) -> Result<(), EdIdentityError> {
        self.ledger.apply_tick(
            tick,
            vec![IdentityCommand::AddRelationship {
                event_id: careops_core::identity::RelationshipEventId::new(event_id),
                kind: RelationshipKind::BirthParentOf,
                from: parent,
                to: infant,
            }],
        )?;
        Ok(())
    }

    pub fn remove_birth_parent(
        &mut self,
        tick: u64,
        event_id: u64,
        parent: PersonId,
        infant: PersonId,
    ) -> Result<(), EdIdentityError> {
        self.ledger.apply_tick(
            tick,
            vec![IdentityCommand::RemoveRelationship {
                event_id: careops_core::identity::RelationshipEventId::new(event_id),
                kind: RelationshipKind::BirthParentOf,
                from: parent,
                to: infant,
            }],
        )?;
        Ok(())
    }

    pub fn ledger(&self) -> &IdentityLedger {
        &self.ledger
    }
    pub fn episode(&self, id: EpisodeId) -> Option<EpisodeSnapshot> {
        self.ledger.episode(id)
    }
    pub fn pathway_subject_key(id: EpisodeId) -> String {
        format!("episode.{}", id.value())
    }
    pub fn checkpoint(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn restore(bytes: &[u8]) -> Result<Self, EdIdentityError> {
        let adapter: Self = serde_json::from_slice(bytes)
            .map_err(|_| EdIdentityError::Ledger(IdentityError::InvalidCheckpoint))?;
        if adapter.run_namespace != adapter.ledger.run_namespace() {
            return Err(EdIdentityError::Ledger(IdentityError::InvalidCheckpoint));
        }
        adapter.ledger.validate_checkpoint()?;
        for (key, person) in &adapter.people {
            let event = ed_person_spawn_event_id(adapter.run_namespace, key)?;
            let expected = person_id_for_spawn(adapter.run_namespace, event)?;
            if adapter.spawn_keys.get(&event) != Some(key)
                || *person != expected
                || adapter.ledger.spawn_record(event)
                    != Some((expected, PersonOrigin::ExternalEntry))
            {
                return Err(EdIdentityError::Ledger(IdentityError::InvalidCheckpoint));
            }
        }
        if adapter.people.len() != adapter.ledger.people_count()
            || adapter.admissions.len() != adapter.ledger.episodes_count()
            || adapter.spawn_keys.len() != adapter.people.len()
            || adapter.episode_keys.len() != adapter.admissions.len()
        {
            return Err(EdIdentityError::Ledger(IdentityError::InvalidCheckpoint));
        }
        for (key, record) in &adapter.admissions {
            let event = ed_admission_event_id(adapter.run_namespace, key)?;
            let expected_episode = episode_id_for_entry(adapter.run_namespace, event)?;
            if adapter.episode_keys.get(&event) != Some(key)
                || adapter.people.get(&record.source_person_key) != Some(&record.person_id)
                || record.episode_id != expected_episode
                || adapter.ledger.episode_entry(event).is_none_or(|episode| {
                    episode.episode_id != expected_episode
                        || episode.person_id != record.person_id
                        || episode.service_id != record.service_id
                })
            {
                return Err(EdIdentityError::Ledger(IdentityError::InvalidCheckpoint));
            }
        }
        Ok(adapter)
    }
}
