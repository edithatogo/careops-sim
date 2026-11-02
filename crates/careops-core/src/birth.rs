//! Idempotent CareOps birth-message registration over the reusable identity ledger.
//!
//! This module records modeled birth events and their newborn identities. It
//! does not create pre-birth fetal entities or perform downstream transfers.

use crate::identity::{
    person_id_for_spawn, BirthEventId, IdentityCommand, IdentityError, IdentityLedger, PersonId,
    PersonOrigin, Relationship, RelationshipEventId, RelationshipKind, SpawnEventId,
};
use crate::transfer::ServiceId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const BIRTH_MESSAGE_CHECKPOINT_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct BirthMessageId(u64);

impl BirthMessageId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BirthInfantInput {
    pub infant_ordinal: u16,
    pub spawn_event_id: SpawnEventId,
    pub relationship_event_id: RelationshipEventId,
    pub destination_service: ServiceId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BirthMessageInput {
    pub message_id: BirthMessageId,
    pub birth_event_id: BirthEventId,
    pub mother_person_id: PersonId,
    pub maternal_destination_service: ServiceId,
    pub infants: Vec<BirthInfantInput>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BirthRecipient {
    Mother {
        person_id: PersonId,
        destination_service: ServiceId,
    },
    Infant {
        person_id: PersonId,
        infant_ordinal: u16,
        destination_service: ServiceId,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BirthMessageResult {
    pub birth_event_id: BirthEventId,
    pub recipients: Vec<BirthRecipient>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BirthMessageError {
    InvalidIdentifier,
    EmptyInfantSet,
    DuplicateInfantOrdinal,
    DuplicateEventKey,
    BirthEventAlreadyRegistered,
    MessagePayloadConflict,
    UnknownMother,
    Identity(IdentityError),
    InvalidCheckpoint,
}

impl std::fmt::Display for BirthMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidIdentifier => {
                "birth message identifiers and destinations must be non-zero"
            }
            Self::EmptyInfantSet => "birth event must contain at least one infant",
            Self::DuplicateInfantOrdinal => "infant ordinals must be unique within a birth event",
            Self::DuplicateEventKey => "birth event keys must be unique within the message",
            Self::BirthEventAlreadyRegistered => {
                "birth event is already registered by another message"
            }
            Self::MessagePayloadConflict => "birth message ID was reused with different content",
            Self::UnknownMother => "birth message references an unknown maternal person",
            Self::Identity(error) => return error.fmt(f),
            Self::InvalidCheckpoint => "invalid or unsupported birth-message checkpoint",
        };
        f.write_str(message)
    }
}
impl std::error::Error for BirthMessageError {}
impl From<IdentityError> for BirthMessageError {
    fn from(value: IdentityError) -> Self {
        Self::Identity(value)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct BirthMessageRecord {
    input: BirthMessageInput,
    result: BirthMessageResult,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BirthMessageLedger {
    checkpoint_version: u16,
    messages: BTreeMap<BirthMessageId, BirthMessageRecord>,
}

impl Default for BirthMessageLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl BirthMessageLedger {
    pub fn new() -> Self {
        Self {
            checkpoint_version: BIRTH_MESSAGE_CHECKPOINT_VERSION,
            messages: BTreeMap::new(),
        }
    }

    /// Register a complete birth atomically with its people and parent links.
    /// Infant input order is normalized by ordinal before replay comparison.
    pub fn register(
        &mut self,
        identities: &mut IdentityLedger,
        tick: u64,
        input: BirthMessageInput,
    ) -> Result<BirthMessageResult, BirthMessageError> {
        let input = canonical_input(input)?;
        self.validate(identities)?;
        if let Some(previous) = self.messages.get(&input.message_id) {
            return if previous.input == input {
                Ok(previous.result.clone())
            } else {
                Err(BirthMessageError::MessagePayloadConflict)
            };
        }
        if self
            .messages
            .values()
            .any(|record| record.input.birth_event_id == input.birth_event_id)
        {
            return Err(BirthMessageError::BirthEventAlreadyRegistered);
        }
        if identities.person(input.mother_person_id).is_none() {
            return Err(BirthMessageError::UnknownMother);
        }

        let mut candidate_identities = identities.clone();
        let mut commands = Vec::with_capacity(input.infants.len() * 2);
        let mut recipients = vec![BirthRecipient::Mother {
            person_id: input.mother_person_id,
            destination_service: input.maternal_destination_service,
        }];
        for infant in &input.infants {
            let person_id =
                person_id_for_spawn(candidate_identities.run_namespace(), infant.spawn_event_id)?;
            commands.push(IdentityCommand::Spawn {
                event_id: infant.spawn_event_id,
                origin: PersonOrigin::Birth {
                    birth_event_id: input.birth_event_id,
                    infant_ordinal: infant.infant_ordinal,
                },
            });
            commands.push(IdentityCommand::AddRelationship {
                event_id: infant.relationship_event_id,
                kind: RelationshipKind::BirthParentOf,
                from: input.mother_person_id,
                to: person_id,
            });
            recipients.push(BirthRecipient::Infant {
                person_id,
                infant_ordinal: infant.infant_ordinal,
                destination_service: infant.destination_service,
            });
        }
        candidate_identities.apply_tick(tick, commands)?;
        let result = BirthMessageResult {
            birth_event_id: input.birth_event_id,
            recipients,
        };
        let mut candidate = self.clone();
        candidate.messages.insert(
            input.message_id,
            BirthMessageRecord {
                input,
                result: result.clone(),
            },
        );
        *identities = candidate_identities;
        *self = candidate;
        Ok(result)
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn checkpoint(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn restore(bytes: &[u8], identities: &IdentityLedger) -> Result<Self, BirthMessageError> {
        let ledger: Self =
            serde_json::from_slice(bytes).map_err(|_| BirthMessageError::InvalidCheckpoint)?;
        ledger.validate(identities)?;
        Ok(ledger)
    }

    pub fn validate_checkpoint(
        &self,
        identities: &IdentityLedger,
    ) -> Result<(), BirthMessageError> {
        self.validate(identities)
    }

    fn validate(&self, identities: &IdentityLedger) -> Result<(), BirthMessageError> {
        if self.checkpoint_version != BIRTH_MESSAGE_CHECKPOINT_VERSION {
            return Err(BirthMessageError::InvalidCheckpoint);
        }
        identities
            .validate_checkpoint()
            .map_err(|_| BirthMessageError::InvalidCheckpoint)?;
        let mut birth_events = BTreeSet::new();
        let mut spawn_events = BTreeSet::new();
        let mut relationship_events = BTreeSet::new();
        for (message_id, record) in &self.messages {
            let input = canonical_input(record.input.clone())
                .map_err(|_| BirthMessageError::InvalidCheckpoint)?;
            if message_id.value() == 0
                || *message_id != input.message_id
                || input != record.input
                || !birth_events.insert(input.birth_event_id)
            {
                return Err(BirthMessageError::InvalidCheckpoint);
            }
            let mut expected = vec![BirthRecipient::Mother {
                person_id: input.mother_person_id,
                destination_service: input.maternal_destination_service,
            }];
            if identities.person(input.mother_person_id).is_none() {
                return Err(BirthMessageError::InvalidCheckpoint);
            }
            for infant in &input.infants {
                if !spawn_events.insert(infant.spawn_event_id)
                    || !relationship_events.insert(infant.relationship_event_id)
                {
                    return Err(BirthMessageError::InvalidCheckpoint);
                }
                let person_id =
                    person_id_for_spawn(identities.run_namespace(), infant.spawn_event_id)
                        .map_err(|_| BirthMessageError::InvalidCheckpoint)?;
                let origin = PersonOrigin::Birth {
                    birth_event_id: input.birth_event_id,
                    infant_ordinal: infant.infant_ordinal,
                };
                let relation = Relationship {
                    kind: RelationshipKind::BirthParentOf,
                    from: input.mother_person_id,
                    to: person_id,
                };
                if identities.spawn_record(infant.spawn_event_id) != Some((person_id, origin))
                    || identities.relationship_add_record(infant.relationship_event_id)
                        != Some(relation)
                {
                    return Err(BirthMessageError::InvalidCheckpoint);
                }
                expected.push(BirthRecipient::Infant {
                    person_id,
                    infant_ordinal: infant.infant_ordinal,
                    destination_service: infant.destination_service,
                });
            }
            if record.result.birth_event_id != input.birth_event_id
                || record.result.recipients != expected
            {
                return Err(BirthMessageError::InvalidCheckpoint);
            }
        }
        Ok(())
    }
}

fn canonical_input(mut input: BirthMessageInput) -> Result<BirthMessageInput, BirthMessageError> {
    if input.message_id.value() == 0
        || !input.birth_event_id.is_valid()
        || !input.mother_person_id.is_valid()
        || input.maternal_destination_service.value() == 0
    {
        return Err(BirthMessageError::InvalidIdentifier);
    }
    if input.infants.is_empty() {
        return Err(BirthMessageError::EmptyInfantSet);
    }
    input.infants.sort_by_key(|infant| infant.infant_ordinal);
    let mut ordinals = BTreeSet::new();
    let mut spawn_events = BTreeSet::new();
    let mut relationship_events = BTreeSet::new();
    for infant in &input.infants {
        if !infant.spawn_event_id.is_valid()
            || !infant.relationship_event_id.is_valid()
            || infant.destination_service.value() == 0
        {
            return Err(BirthMessageError::InvalidIdentifier);
        }
        if !ordinals.insert(infant.infant_ordinal) {
            return Err(BirthMessageError::DuplicateInfantOrdinal);
        }
        if !spawn_events.insert(infant.spawn_event_id)
            || !relationship_events.insert(infant.relationship_event_id)
        {
            return Err(BirthMessageError::DuplicateEventKey);
        }
    }
    Ok(input)
}
