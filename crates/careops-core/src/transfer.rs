//! Versioned, single-scheduler transfer lifecycle accounting.
//!
//! The ledger owns logical transfer state and unit-level resource claims. It is
//! not a network transport, clinical eligibility engine, or multi-resource
//! allocator. Callers serialize access through one deterministic simulation
//! scheduler and provide virtual ticks, never wall-clock time.

pub use crate::identity::{EpisodeId, PersonId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const TRANSFER_ENVELOPE_VERSION: u16 = 1;
const MAX_PENDING_MESSAGES_PER_OPERATION: usize = 256;

macro_rules! id_type {
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

            const fn is_valid(self) -> bool {
                self.0 != 0
            }
        }
    };
}

id_type!(TransferOperationId);
id_type!(ServiceId);
id_type!(ResourceId);
id_type!(MessageId);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferState {
    Requested,
    Offered,
    Reserved,
    Departing,
    InTransit,
    Arrived,
    HandoverComplete,
    Rejected,
    Cancelled,
    Expired,
}

impl TransferState {
    const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::HandoverComplete | Self::Rejected | Self::Cancelled | Self::Expired
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "location", content = "service", rename_all = "snake_case")]
pub enum TransferLocation {
    Service(ServiceId),
    Transport { from: ServiceId, to: ServiceId },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TransferRequest {
    pub person_id: PersonId,
    pub episode_id: EpisodeId,
    pub origin_service: ServiceId,
    pub destination_service: ServiceId,
    /// Deadline for a request to receive an offer, measured in virtual ticks.
    pub request_expires_at: Option<u64>,
    /// Unit-level origin claims already occupied by this transfer episode.
    pub origin_resources: Vec<ResourceId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompensationRequest {
    pub operation_id: TransferOperationId,
    pub destination_service: ServiceId,
    pub request_expires_at: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransferCommand {
    Request {
        request: TransferRequest,
    },
    Offer {
        expires_at: Option<u64>,
    },
    Reserve {
        resources: Vec<ResourceId>,
        expires_at: Option<u64>,
    },
    BeginDeparture,
    Depart,
    Arrive,
    CompleteHandover,
    Reject,
    Cancel,
    DestinationRejected {
        compensation: CompensationRequest,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TransferEnvelope {
    pub schema_version: u16,
    pub operation_id: TransferOperationId,
    pub message_id: MessageId,
    /// The operation revision this command follows. A request starts at 0.
    pub expected_revision: u64,
    /// Event time from the single simulation scheduler.
    pub tick: u64,
    pub command: TransferCommand,
}

impl TransferEnvelope {
    pub const fn new(
        operation_id: TransferOperationId,
        message_id: MessageId,
        expected_revision: u64,
        tick: u64,
        command: TransferCommand,
    ) -> Self {
        Self {
            schema_version: TRANSFER_ENVELOPE_VERSION,
            operation_id,
            message_id,
            expected_revision,
            tick,
            command,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimRole {
    Origin,
    Destination,
    ServiceOccupancy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimDisposition {
    Occupied,
    Held,
    Reserved,
    InFlightHold,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceClaimSnapshot {
    pub resource_id: ResourceId,
    pub operation_id: Option<TransferOperationId>,
    pub owner_service: ServiceId,
    pub role: ClaimRole,
    pub disposition: ClaimDisposition,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersonSnapshot {
    pub person_id: PersonId,
    pub episodes: BTreeSet<EpisodeId>,
    pub owner_service: ServiceId,
    pub location: TransferLocation,
    pub active_operation: Option<TransferOperationId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TransferSnapshot {
    pub operation_id: TransferOperationId,
    pub person_id: PersonId,
    pub episode_id: EpisodeId,
    pub origin_service: ServiceId,
    pub destination_service: ServiceId,
    pub state: TransferState,
    pub revision: u64,
    pub owner_service: ServiceId,
    pub location: TransferLocation,
    pub origin_resources: Vec<ResourceId>,
    pub destination_resources: Vec<ResourceId>,
    pub compensation_parent: Option<TransferOperationId>,
    pub compensation_children: Vec<TransferOperationId>,
    pub active_compensation: Option<TransferOperationId>,
    pub actual_handover_service: Option<ServiceId>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferEffect {
    Requested,
    Offered,
    Reserved,
    DeparturePrepared,
    Departed,
    Arrived,
    HandoverCompleted,
    Rejected,
    Cancelled,
    Expired,
    CompensationCreated,
    CompensationCompleted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TransferEventRecord {
    pub sequence: u64,
    pub tick: u64,
    pub operation_id: TransferOperationId,
    pub related_operation: Option<TransferOperationId>,
    pub revision: u64,
    pub message_id: Option<MessageId>,
    pub state: TransferState,
    pub effect: TransferEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplyOutcome {
    Applied,
    Duplicate,
    Buffered,
    StaleIgnored,
}

#[derive(thiserror::Error, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferError {
    #[error("unsupported transfer envelope version")]
    UnsupportedEnvelopeVersion,
    #[error("transfer identifiers must be non-zero")]
    InvalidIdentifier,
    #[error("invalid transfer request")]
    InvalidRequest,
    #[error("unknown transfer operation")]
    UnknownOperation,
    #[error("unknown transfer person")]
    UnknownPerson,
    #[error("transfer command is invalid in the current state")]
    InvalidTransition,
    #[error("transfer operation revision overflow")]
    RevisionOverflow,
    #[error("transfer revision gap is too large")]
    RevisionGapOverflow,
    #[error("message ID was reused with different content")]
    MessageConflict,
    #[error("too many pending transfer messages")]
    TooManyPendingMessages,
    #[error("resource is already claimed or conflicts with this transfer")]
    ResourceConflict,
    #[error("transfer event time regresses within an operation")]
    TimeRegression,
    #[error("transfer event sequence overflow")]
    SequenceOverflow,
    #[error("invalid or unsupported transfer checkpoint")]
    InvalidCheckpoint,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct TransferOperation {
    request: TransferRequest,
    state: TransferState,
    revision: u64,
    last_event_tick: u64,
    offer_expires_at: Option<u64>,
    reservation_expires_at: Option<u64>,
    origin_resources: Vec<ResourceId>,
    destination_resources: Vec<ResourceId>,
    compensation_parent: Option<TransferOperationId>,
    compensation_children: Vec<TransferOperationId>,
    active_compensation: Option<TransferOperationId>,
    actual_handover_service: Option<ServiceId>,
    applied_messages: BTreeMap<MessageId, TransferEnvelope>,
    pending_messages: BTreeMap<u64, TransferEnvelope>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PersonRecord {
    episodes: BTreeSet<EpisodeId>,
    owner_service: ServiceId,
    location: TransferLocation,
    active_operation: Option<TransferOperationId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ResourceClaim {
    operation_id: Option<TransferOperationId>,
    owner_service: ServiceId,
    role: ClaimRole,
    disposition: ClaimDisposition,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct TransferLedger {
    checkpoint_version: u16,
    now: u64,
    next_sequence: u64,
    operations: BTreeMap<TransferOperationId, TransferOperation>,
    people: BTreeMap<PersonId, PersonRecord>,
    claims: BTreeMap<ResourceId, ResourceClaim>,
    events: Vec<TransferEventRecord>,
}

impl Default for TransferLedger {
    fn default() -> Self {
        Self {
            checkpoint_version: 1,
            now: 0,
            next_sequence: 0,
            operations: BTreeMap::new(),
            people: BTreeMap::new(),
            claims: BTreeMap::new(),
            events: Vec::new(),
        }
    }
}

impl TransferLedger {
    /// Apply one versioned envelope atomically, including clock advancement and
    /// expiry processing. The ledger is intended to be called by one scheduler
    /// owner; callers must not apply envelopes concurrently.
    pub fn apply(&mut self, envelope: TransferEnvelope) -> Result<ApplyOutcome, TransferError> {
        let mut candidate = self.clone();
        let outcome = candidate.apply_inner(envelope)?;
        *self = candidate;
        Ok(outcome)
    }

    fn apply_inner(&mut self, envelope: TransferEnvelope) -> Result<ApplyOutcome, TransferError> {
        if envelope.schema_version != TRANSFER_ENVELOPE_VERSION {
            return Err(TransferError::UnsupportedEnvelopeVersion);
        }
        if !envelope.operation_id.is_valid() || !envelope.message_id.is_valid() {
            return Err(TransferError::InvalidIdentifier);
        }

        let target_tick = self.now.max(envelope.tick);
        self.advance_to(target_tick)?;

        if matches!(envelope.command, TransferCommand::Request { .. }) {
            if let Some(operation) = self.operations.get(&envelope.operation_id) {
                return match operation.applied_messages.get(&envelope.message_id) {
                    Some(previous) if previous == &envelope => Ok(ApplyOutcome::Duplicate),
                    Some(_) => Err(TransferError::MessageConflict),
                    None => Err(TransferError::InvalidRequest),
                };
            }
            let mut candidate = self.clone();
            candidate.create_request(envelope)?;
            *self = candidate;
            return Ok(ApplyOutcome::Applied);
        }

        let operation = self
            .operations
            .get(&envelope.operation_id)
            .ok_or(TransferError::UnknownOperation)?;
        if let Some(previous) = operation.applied_messages.get(&envelope.message_id) {
            return if previous == &envelope {
                Ok(ApplyOutcome::Duplicate)
            } else {
                Err(TransferError::MessageConflict)
            };
        }
        if let Some(previous) = operation
            .pending_messages
            .values()
            .find(|pending| pending.message_id == envelope.message_id)
        {
            return if previous == &envelope {
                Ok(ApplyOutcome::Buffered)
            } else {
                Err(TransferError::MessageConflict)
            };
        }
        if operation.state.is_terminal() {
            return Err(TransferError::InvalidTransition);
        }
        if envelope.expected_revision < operation.revision {
            return Ok(ApplyOutcome::StaleIgnored);
        }
        operation
            .revision
            .checked_add(1)
            .ok_or(TransferError::RevisionOverflow)?;
        if envelope.expected_revision > operation.revision {
            if operation.pending_messages.len() >= MAX_PENDING_MESSAGES_PER_OPERATION {
                return Err(TransferError::TooManyPendingMessages);
            }
            if operation
                .pending_messages
                .contains_key(&envelope.expected_revision)
            {
                return Err(TransferError::MessageConflict);
            }
            let mut candidate = self.clone();
            candidate
                .operations
                .get_mut(&envelope.operation_id)
                .expect("operation was checked above")
                .pending_messages
                .insert(envelope.expected_revision, envelope);
            *self = candidate;
            return Ok(ApplyOutcome::Buffered);
        }

        let operation_id = envelope.operation_id;
        let mut candidate = self.clone();
        candidate.apply_next(envelope)?;
        *self = candidate;
        self.drain_ready_messages(operation_id);
        Ok(ApplyOutcome::Applied)
    }

    /// Advance the one scheduler's virtual clock atomically. Expiry runs before
    /// commands at the same tick and uses stable (deadline, operation ID) ordering.
    pub fn advance_to(&mut self, tick: u64) -> Result<(), TransferError> {
        let mut candidate = self.clone();
        candidate.advance_to_inner(tick)?;
        *self = candidate;
        Ok(())
    }

    fn advance_to_inner(&mut self, tick: u64) -> Result<(), TransferError> {
        if tick < self.now {
            return Err(TransferError::TimeRegression);
        }
        self.now = tick;
        let mut due = self
            .operations
            .iter()
            .filter_map(|(id, operation)| {
                operation
                    .deadline_for_state()
                    .filter(|deadline| *deadline <= tick)
                    .map(|deadline| (deadline, *id))
            })
            .collect::<Vec<_>>();
        due.sort_unstable();

        for (deadline, operation_id) in due {
            let Some(mut operation) = self.operations.get(&operation_id).cloned() else {
                continue;
            };
            if operation
                .deadline_for_state()
                .is_none_or(|current| current > tick)
            {
                continue;
            }
            let revision = operation
                .revision
                .checked_add(1)
                .ok_or(TransferError::RevisionOverflow)?;
            self.detach_origin_claims(operation_id, &operation.origin_resources)?;
            self.release_destination_claims(operation_id, &operation.destination_resources)?;
            operation.state = TransferState::Expired;
            operation.revision = revision;
            operation.last_event_tick = deadline;
            operation.destination_resources.clear();
            operation.offer_expires_at = None;
            operation.reservation_expires_at = None;
            operation.pending_messages.clear();
            self.restore_person_after_predeparture_terminal(operation_id, &operation)?;
            self.operations.insert(operation_id, operation.clone());
            self.record_event(
                operation_id,
                None,
                deadline,
                revision,
                None,
                operation.state,
                TransferEffect::Expired,
            )?;
        }
        Ok(())
    }

    pub fn snapshot(&self, operation_id: TransferOperationId) -> Option<TransferSnapshot> {
        let operation = self.operations.get(&operation_id)?;
        let person = self.people.get(&operation.request.person_id)?;
        Some(TransferSnapshot {
            operation_id,
            person_id: operation.request.person_id,
            episode_id: operation.request.episode_id,
            origin_service: operation.request.origin_service,
            destination_service: operation.request.destination_service,
            state: operation.state,
            revision: operation.revision,
            owner_service: person.owner_service,
            location: person.location,
            origin_resources: operation.origin_resources.clone(),
            destination_resources: operation.destination_resources.clone(),
            compensation_parent: operation.compensation_parent,
            compensation_children: operation.compensation_children.clone(),
            active_compensation: operation.active_compensation,
            actual_handover_service: operation.actual_handover_service,
        })
    }

    pub fn person(&self, person_id: PersonId) -> Option<PersonSnapshot> {
        let person = self.people.get(&person_id)?;
        Some(PersonSnapshot {
            person_id,
            episodes: person.episodes.clone(),
            owner_service: person.owner_service,
            location: person.location,
            active_operation: person.active_operation,
        })
    }

    pub fn claims(&self) -> Vec<ResourceClaimSnapshot> {
        self.claims
            .iter()
            .map(|(resource_id, claim)| ResourceClaimSnapshot {
                resource_id: *resource_id,
                operation_id: claim.operation_id,
                owner_service: claim.owner_service,
                role: claim.role,
                disposition: claim.disposition,
            })
            .collect()
    }

    pub fn events(&self) -> &[TransferEventRecord] {
        &self.events
    }

    pub const fn now(&self) -> u64 {
        self.now
    }

    /// Serialize a deterministic checkpoint image, including deduplication and
    /// pending revision-gap state needed for safe replay after restart.
    pub fn checkpoint(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Restore only supported schema v1 images and validate cross-ledger
    /// references before exposing the state to the scheduler.
    pub fn restore(bytes: &[u8]) -> Result<Self, TransferError> {
        let ledger: Self =
            serde_json::from_slice(bytes).map_err(|_| TransferError::InvalidCheckpoint)?;
        ledger.validate_checkpoint()?;
        Ok(ledger)
    }

    fn create_request(&mut self, envelope: TransferEnvelope) -> Result<(), TransferError> {
        if envelope.expected_revision != 0 || envelope.tick < self.now {
            return Err(TransferError::InvalidRequest);
        }
        let TransferCommand::Request { request } = &envelope.command else {
            return Err(TransferError::InvalidRequest);
        };
        validate_request(request, envelope.tick)?;
        if self.operations.contains_key(&envelope.operation_id) {
            return Err(TransferError::InvalidRequest);
        }
        if request.origin_resources.iter().any(|resource| {
            self.claims.get(resource).is_some_and(|claim| {
                claim.operation_id.is_some()
                    || claim.owner_service != request.origin_service
                    || claim.role != ClaimRole::ServiceOccupancy
                    || claim.disposition != ClaimDisposition::Occupied
            })
        }) {
            return Err(TransferError::ResourceConflict);
        }
        if let Some(person) = self.people.get(&request.person_id) {
            if person.active_operation.is_some()
                || person.owner_service != request.origin_service
                || person.location != TransferLocation::Service(request.origin_service)
            {
                return Err(TransferError::InvalidRequest);
            }
        }

        for resource_id in &request.origin_resources {
            self.claims.insert(
                *resource_id,
                ResourceClaim {
                    operation_id: Some(envelope.operation_id),
                    owner_service: request.origin_service,
                    role: ClaimRole::Origin,
                    disposition: ClaimDisposition::Occupied,
                },
            );
        }
        let mut episodes = self
            .people
            .get(&request.person_id)
            .map(|person| person.episodes.clone())
            .unwrap_or_default();
        episodes.insert(request.episode_id);
        let owner_service = self
            .people
            .get(&request.person_id)
            .map(|person| person.owner_service)
            .unwrap_or(request.origin_service);
        self.people.insert(
            request.person_id,
            PersonRecord {
                episodes,
                owner_service,
                location: TransferLocation::Service(request.origin_service),
                active_operation: Some(envelope.operation_id),
            },
        );
        let operation = TransferOperation {
            request: request.clone(),
            state: TransferState::Requested,
            revision: 1,
            last_event_tick: envelope.tick,
            offer_expires_at: None,
            reservation_expires_at: None,
            origin_resources: request.origin_resources.clone(),
            destination_resources: Vec::new(),
            compensation_parent: None,
            compensation_children: Vec::new(),
            active_compensation: None,
            actual_handover_service: None,
            applied_messages: BTreeMap::from([(envelope.message_id, envelope.clone())]),
            pending_messages: BTreeMap::new(),
        };
        self.operations.insert(envelope.operation_id, operation);
        self.record_event(
            envelope.operation_id,
            None,
            envelope.tick,
            1,
            Some(envelope.message_id),
            TransferState::Requested,
            TransferEffect::Requested,
        )
    }

    fn apply_next(&mut self, envelope: TransferEnvelope) -> Result<(), TransferError> {
        let mut operation = self
            .operations
            .get(&envelope.operation_id)
            .cloned()
            .ok_or(TransferError::UnknownOperation)?;
        if envelope.expected_revision != operation.revision {
            return Err(TransferError::InvalidTransition);
        }
        if envelope.tick < operation.last_event_tick {
            return Err(TransferError::TimeRegression);
        }
        if operation.state.is_terminal() {
            return Err(TransferError::InvalidTransition);
        }
        let next_revision = operation
            .revision
            .checked_add(1)
            .ok_or(TransferError::RevisionOverflow)?;
        let effect = match &envelope.command {
            TransferCommand::Request { .. } => return Err(TransferError::InvalidTransition),
            TransferCommand::Offer { expires_at } => {
                if operation.state != TransferState::Requested
                    || expires_at.is_some_and(|deadline| deadline <= envelope.tick)
                {
                    return Err(TransferError::InvalidTransition);
                }
                operation.state = TransferState::Offered;
                operation.offer_expires_at = *expires_at;
                TransferEffect::Offered
            }
            TransferCommand::Reserve {
                resources,
                expires_at,
            } => {
                if operation.state != TransferState::Offered
                    || resources.is_empty()
                    || expires_at.is_some_and(|deadline| deadline <= envelope.tick)
                    || !unique_valid_resources(resources)
                    || resources
                        .iter()
                        .any(|resource| operation.origin_resources.contains(resource))
                    || resources
                        .iter()
                        .any(|resource| self.claims.contains_key(resource))
                {
                    return Err(TransferError::ResourceConflict);
                }
                for resource_id in resources {
                    self.claims.insert(
                        *resource_id,
                        ResourceClaim {
                            operation_id: Some(envelope.operation_id),
                            owner_service: operation.request.destination_service,
                            role: ClaimRole::Destination,
                            disposition: ClaimDisposition::Reserved,
                        },
                    );
                }
                operation.destination_resources = resources.clone();
                operation.reservation_expires_at = *expires_at;
                operation.state = TransferState::Reserved;
                TransferEffect::Reserved
            }
            TransferCommand::BeginDeparture => {
                if operation.state != TransferState::Reserved {
                    return Err(TransferError::InvalidTransition);
                }
                operation.state = TransferState::Departing;
                TransferEffect::DeparturePrepared
            }
            TransferCommand::Depart => {
                if operation.state != TransferState::Departing {
                    return Err(TransferError::InvalidTransition);
                }
                if let Some(parent_id) = operation.compensation_parent {
                    self.prepare_compensation_departure(
                        parent_id,
                        envelope.operation_id,
                        &mut operation,
                    )?;
                } else {
                    self.set_claim_disposition(
                        envelope.operation_id,
                        &operation.origin_resources,
                        ClaimRole::Origin,
                        ClaimDisposition::Held,
                    )?;
                }
                self.set_claim_disposition(
                    envelope.operation_id,
                    &operation.destination_resources,
                    ClaimRole::Destination,
                    ClaimDisposition::InFlightHold,
                )?;
                let person = self
                    .people
                    .get_mut(&operation.request.person_id)
                    .ok_or(TransferError::UnknownPerson)?;
                person.location = TransferLocation::Transport {
                    from: operation.request.origin_service,
                    to: operation.request.destination_service,
                };
                person.active_operation = Some(envelope.operation_id);
                operation.state = TransferState::InTransit;
                TransferEffect::Departed
            }
            TransferCommand::Arrive => {
                if operation.state != TransferState::InTransit {
                    return Err(TransferError::InvalidTransition);
                }
                self.set_claim_disposition(
                    envelope.operation_id,
                    &operation.destination_resources,
                    ClaimRole::Destination,
                    ClaimDisposition::Occupied,
                )?;
                let person = self
                    .people
                    .get_mut(&operation.request.person_id)
                    .ok_or(TransferError::UnknownPerson)?;
                person.location = TransferLocation::Service(operation.request.destination_service);
                operation.state = TransferState::Arrived;
                TransferEffect::Arrived
            }
            TransferCommand::CompleteHandover => {
                if operation.state != TransferState::Arrived
                    || operation.active_compensation.is_some()
                {
                    return Err(TransferError::InvalidTransition);
                }
                self.complete_handover(envelope.operation_id, &mut operation, envelope.tick)?;
                operation.state = TransferState::HandoverComplete;
                operation.actual_handover_service = Some(operation.request.destination_service);
                TransferEffect::HandoverCompleted
            }
            TransferCommand::Reject => {
                if !matches!(
                    operation.state,
                    TransferState::Requested
                        | TransferState::Offered
                        | TransferState::Reserved
                        | TransferState::Departing
                ) {
                    return Err(TransferError::InvalidTransition);
                }
                self.detach_origin_claims(envelope.operation_id, &operation.origin_resources)?;
                self.release_destination_claims(
                    envelope.operation_id,
                    &operation.destination_resources,
                )?;
                operation.destination_resources.clear();
                self.restore_person_after_predeparture_terminal(envelope.operation_id, &operation)?;
                operation.state = TransferState::Rejected;
                TransferEffect::Rejected
            }
            TransferCommand::Cancel => {
                if !matches!(
                    operation.state,
                    TransferState::Requested
                        | TransferState::Offered
                        | TransferState::Reserved
                        | TransferState::Departing
                ) {
                    return Err(TransferError::InvalidTransition);
                }
                self.detach_origin_claims(envelope.operation_id, &operation.origin_resources)?;
                self.release_destination_claims(
                    envelope.operation_id,
                    &operation.destination_resources,
                )?;
                operation.destination_resources.clear();
                self.restore_person_after_predeparture_terminal(envelope.operation_id, &operation)?;
                operation.state = TransferState::Cancelled;
                TransferEffect::Cancelled
            }
            TransferCommand::DestinationRejected { compensation } => {
                // V1 supports one compensation level; nested diversions need an
                // explicit chain-completion contract before they can be accepted.
                if operation.state != TransferState::Arrived
                    || operation.compensation_parent.is_some()
                    || !self.compensation_can_start(&operation)
                {
                    return Err(TransferError::InvalidTransition);
                }
                self.create_compensation(
                    envelope.operation_id,
                    &operation,
                    compensation,
                    envelope.tick,
                )?;
                operation
                    .compensation_children
                    .push(compensation.operation_id);
                operation.active_compensation = Some(compensation.operation_id);
                TransferEffect::CompensationCreated
            }
        };
        operation.revision = next_revision;
        operation.last_event_tick = envelope.tick;
        operation
            .applied_messages
            .insert(envelope.message_id, envelope.clone());
        self.operations
            .insert(envelope.operation_id, operation.clone());
        self.record_event(
            envelope.operation_id,
            match &envelope.command {
                TransferCommand::DestinationRejected { compensation } => {
                    Some(compensation.operation_id)
                }
                _ => None,
            },
            envelope.tick,
            operation.revision,
            Some(envelope.message_id),
            operation.state,
            effect,
        )?;
        Ok(())
    }

    fn drain_ready_messages(&mut self, operation_id: TransferOperationId) {
        loop {
            let Some(operation) = self.operations.get(&operation_id) else {
                return;
            };
            let Some(envelope) = operation.pending_messages.get(&operation.revision).cloned()
            else {
                return;
            };
            let mut candidate = self.clone();
            candidate
                .operations
                .get_mut(&operation_id)
                .expect("operation was checked above")
                .pending_messages
                .remove(&operation.revision);
            if candidate.apply_next(envelope).is_err() {
                return;
            }
            *self = candidate;
        }
    }

    fn complete_handover(
        &mut self,
        operation_id: TransferOperationId,
        operation: &mut TransferOperation,
        tick: u64,
    ) -> Result<(), TransferError> {
        if let Some(parent_id) = operation.compensation_parent {
            let mut parent = self
                .operations
                .get(&parent_id)
                .cloned()
                .ok_or(TransferError::InvalidTransition)?;
            if parent.active_compensation != Some(operation_id)
                || parent.state.is_terminal()
                || parent.request.person_id != operation.request.person_id
            {
                return Err(TransferError::InvalidTransition);
            }
            self.release_origin_claims(parent_id, &parent.origin_resources)?;
            self.release_origin_claims(operation_id, &operation.origin_resources)?;
            self.detach_destination_claims(
                operation_id,
                &operation.destination_resources,
                operation.request.destination_service,
            )?;
            parent.state = TransferState::HandoverComplete;
            parent.revision = parent
                .revision
                .checked_add(1)
                .ok_or(TransferError::RevisionOverflow)?;
            parent.last_event_tick = tick;
            parent.active_compensation = None;
            parent.actual_handover_service = Some(operation.request.destination_service);
            let parent_revision = parent.revision;
            self.operations.insert(parent_id, parent.clone());
            self.record_event(
                parent_id,
                Some(operation_id),
                tick,
                parent_revision,
                None,
                TransferState::HandoverComplete,
                TransferEffect::CompensationCompleted,
            )?;
        } else {
            self.release_origin_claims(operation_id, &operation.origin_resources)?;
            self.detach_destination_claims(
                operation_id,
                &operation.destination_resources,
                operation.request.destination_service,
            )?;
        }
        let person = self
            .people
            .get_mut(&operation.request.person_id)
            .ok_or(TransferError::UnknownPerson)?;
        person.owner_service = operation.request.destination_service;
        person.location = TransferLocation::Service(operation.request.destination_service);
        person.active_operation = None;
        Ok(())
    }

    fn prepare_compensation_departure(
        &mut self,
        parent_id: TransferOperationId,
        child_id: TransferOperationId,
        child: &mut TransferOperation,
    ) -> Result<(), TransferError> {
        let mut parent = self
            .operations
            .get(&parent_id)
            .cloned()
            .ok_or(TransferError::InvalidTransition)?;
        if parent.active_compensation != Some(child_id)
            || !matches!(
                parent.state,
                TransferState::InTransit | TransferState::Arrived
            )
        {
            return Err(TransferError::InvalidTransition);
        }
        for resource_id in &parent.destination_resources {
            let claim = self
                .claims
                .get_mut(resource_id)
                .ok_or(TransferError::ResourceConflict)?;
            if claim.operation_id != Some(parent_id) || claim.role != ClaimRole::Destination {
                return Err(TransferError::ResourceConflict);
            }
            claim.operation_id = Some(child_id);
            claim.role = ClaimRole::Origin;
            claim.disposition = ClaimDisposition::Held;
            child.origin_resources.push(*resource_id);
        }
        parent.destination_resources.clear();
        self.operations.insert(parent_id, parent);
        Ok(())
    }

    fn compensation_can_start(&self, operation: &TransferOperation) -> bool {
        if operation.active_compensation.is_some() {
            return false;
        }
        operation.compensation_children.iter().all(|child_id| {
            self.operations
                .get(child_id)
                .is_some_and(|child| child.state.is_terminal())
        })
    }

    fn create_compensation(
        &mut self,
        parent_id: TransferOperationId,
        parent: &TransferOperation,
        compensation: &CompensationRequest,
        tick: u64,
    ) -> Result<(), TransferError> {
        if !compensation.operation_id.is_valid()
            || !compensation.destination_service.is_valid()
            || compensation.operation_id == parent_id
            || compensation.destination_service == parent.request.origin_service
            || compensation.destination_service == parent.request.destination_service
            || compensation
                .request_expires_at
                .is_some_and(|deadline| deadline <= tick)
            || self.operations.contains_key(&compensation.operation_id)
        {
            return Err(TransferError::InvalidRequest);
        }
        let request = TransferRequest {
            person_id: parent.request.person_id,
            episode_id: parent.request.episode_id,
            origin_service: parent.request.destination_service,
            destination_service: compensation.destination_service,
            request_expires_at: compensation.request_expires_at,
            origin_resources: Vec::new(),
        };
        let operation = TransferOperation {
            request,
            state: TransferState::Requested,
            revision: 1,
            last_event_tick: tick,
            offer_expires_at: None,
            reservation_expires_at: None,
            origin_resources: Vec::new(),
            destination_resources: Vec::new(),
            compensation_parent: Some(parent_id),
            compensation_children: Vec::new(),
            active_compensation: None,
            actual_handover_service: None,
            applied_messages: BTreeMap::new(),
            pending_messages: BTreeMap::new(),
        };
        self.operations.insert(compensation.operation_id, operation);
        Ok(())
    }

    fn set_claim_disposition(
        &mut self,
        operation_id: TransferOperationId,
        resources: &[ResourceId],
        role: ClaimRole,
        disposition: ClaimDisposition,
    ) -> Result<(), TransferError> {
        for resource_id in resources {
            let claim = self
                .claims
                .get_mut(resource_id)
                .ok_or(TransferError::ResourceConflict)?;
            if claim.operation_id != Some(operation_id) || claim.role != role {
                return Err(TransferError::ResourceConflict);
            }
            claim.disposition = disposition;
        }
        Ok(())
    }

    fn release_destination_claims(
        &mut self,
        operation_id: TransferOperationId,
        resources: &[ResourceId],
    ) -> Result<(), TransferError> {
        for resource_id in resources {
            let claim = self
                .claims
                .get(resource_id)
                .ok_or(TransferError::ResourceConflict)?;
            if claim.operation_id != Some(operation_id) || claim.role != ClaimRole::Destination {
                return Err(TransferError::ResourceConflict);
            }
        }
        for resource_id in resources {
            self.claims.remove(resource_id);
        }
        Ok(())
    }

    fn detach_origin_claims(
        &mut self,
        operation_id: TransferOperationId,
        resources: &[ResourceId],
    ) -> Result<(), TransferError> {
        for resource_id in resources {
            let claim = self
                .claims
                .get(resource_id)
                .ok_or(TransferError::ResourceConflict)?;
            if claim.operation_id != Some(operation_id) || claim.role != ClaimRole::Origin {
                return Err(TransferError::ResourceConflict);
            }
        }
        for resource_id in resources {
            let claim = self
                .claims
                .get_mut(resource_id)
                .expect("claim was checked above");
            claim.operation_id = None;
            claim.role = ClaimRole::ServiceOccupancy;
            claim.disposition = ClaimDisposition::Occupied;
        }
        Ok(())
    }

    fn release_origin_claims(
        &mut self,
        operation_id: TransferOperationId,
        resources: &[ResourceId],
    ) -> Result<(), TransferError> {
        for resource_id in resources {
            let claim = self
                .claims
                .get(resource_id)
                .ok_or(TransferError::ResourceConflict)?;
            if claim.operation_id != Some(operation_id) || claim.role != ClaimRole::Origin {
                return Err(TransferError::ResourceConflict);
            }
        }
        for resource_id in resources {
            self.claims.remove(resource_id);
        }
        Ok(())
    }

    fn detach_destination_claims(
        &mut self,
        operation_id: TransferOperationId,
        resources: &[ResourceId],
        service: ServiceId,
    ) -> Result<(), TransferError> {
        for resource_id in resources {
            let claim = self
                .claims
                .get(resource_id)
                .ok_or(TransferError::ResourceConflict)?;
            if claim.operation_id != Some(operation_id) || claim.role != ClaimRole::Destination {
                return Err(TransferError::ResourceConflict);
            }
        }
        for resource_id in resources {
            let claim = self
                .claims
                .get_mut(resource_id)
                .expect("claim was checked above");
            claim.operation_id = None;
            claim.owner_service = service;
            claim.role = ClaimRole::ServiceOccupancy;
            claim.disposition = ClaimDisposition::Occupied;
        }
        Ok(())
    }

    fn restore_person_after_predeparture_terminal(
        &mut self,
        operation_id: TransferOperationId,
        operation: &TransferOperation,
    ) -> Result<(), TransferError> {
        let person = self
            .people
            .get_mut(&operation.request.person_id)
            .ok_or(TransferError::UnknownPerson)?;
        if let Some(parent_id) = operation.compensation_parent {
            person.active_operation = Some(parent_id);
            let parent = self
                .operations
                .get_mut(&parent_id)
                .ok_or(TransferError::InvalidTransition)?;
            if parent.active_compensation == Some(operation_id) {
                parent.active_compensation = None;
            }
        } else {
            person.location = TransferLocation::Service(operation.request.origin_service);
            person.active_operation = None;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn record_event(
        &mut self,
        operation_id: TransferOperationId,
        related_operation: Option<TransferOperationId>,
        tick: u64,
        revision: u64,
        message_id: Option<MessageId>,
        state: TransferState,
        effect: TransferEffect,
    ) -> Result<(), TransferError> {
        let next = self
            .next_sequence
            .checked_add(1)
            .ok_or(TransferError::SequenceOverflow)?;
        self.events.push(TransferEventRecord {
            sequence: self.next_sequence,
            tick,
            operation_id,
            related_operation,
            revision,
            message_id,
            state,
            effect,
        });
        self.next_sequence = next;
        Ok(())
    }

    fn validate_checkpoint(&self) -> Result<(), TransferError> {
        if self.checkpoint_version != 1
            || self.next_sequence != self.events.len() as u64
            || self
                .events
                .iter()
                .enumerate()
                .any(|(index, event)| event.sequence != index as u64 || event.tick > self.now)
        {
            return Err(TransferError::InvalidCheckpoint);
        }
        for (operation_id, operation) in &self.operations {
            if !operation_id.is_valid()
                || operation.revision == 0
                || !operation.request.person_id.is_valid()
                || !operation.request.episode_id.is_valid()
                || !operation.request.origin_service.is_valid()
                || !operation.request.destination_service.is_valid()
                || operation.request.origin_service == operation.request.destination_service
                || !unique_valid_resources(&operation.origin_resources)
                || !unique_valid_resources(&operation.destination_resources)
                || operation
                    .applied_messages
                    .iter()
                    .any(|(message_id, envelope)| {
                        !message_id.is_valid()
                            || envelope.operation_id != *operation_id
                            || envelope.schema_version != TRANSFER_ENVELOPE_VERSION
                            || envelope.expected_revision > operation.revision
                    })
                || operation.pending_messages.len() > MAX_PENDING_MESSAGES_PER_OPERATION
                || operation
                    .pending_messages
                    .iter()
                    .any(|(revision, envelope)| {
                        *revision != envelope.expected_revision
                            || envelope.operation_id != *operation_id
                            || envelope.schema_version != TRANSFER_ENVELOPE_VERSION
                            || *revision <= operation.revision
                    })
            {
                return Err(TransferError::InvalidCheckpoint);
            }
        }
        for (resource_id, claim) in &self.claims {
            if !resource_id.is_valid() || !claim.owner_service.is_valid() {
                return Err(TransferError::InvalidCheckpoint);
            }
            if let Some(operation_id) = claim.operation_id {
                let operation = self
                    .operations
                    .get(&operation_id)
                    .ok_or(TransferError::InvalidCheckpoint)?;
                if operation.state.is_terminal() {
                    return Err(TransferError::InvalidCheckpoint);
                }
                let matching = match claim.role {
                    ClaimRole::Origin => operation.origin_resources.contains(resource_id),
                    ClaimRole::Destination => operation.destination_resources.contains(resource_id),
                    ClaimRole::ServiceOccupancy => false,
                };
                if !matching {
                    return Err(TransferError::InvalidCheckpoint);
                }
            } else if claim.role != ClaimRole::ServiceOccupancy
                || claim.disposition != ClaimDisposition::Occupied
            {
                return Err(TransferError::InvalidCheckpoint);
            }
        }
        for (person_id, person) in &self.people {
            if !person_id.is_valid()
                || !person.owner_service.is_valid()
                || person.episodes.iter().any(|episode| !episode.is_valid())
                || !matches!(person.location, TransferLocation::Service(service) if service == person.owner_service)
                    && person.active_operation.is_none()
            {
                return Err(TransferError::InvalidCheckpoint);
            }
            if let Some(operation_id) = person.active_operation {
                let operation = self
                    .operations
                    .get(&operation_id)
                    .ok_or(TransferError::InvalidCheckpoint)?;
                if operation.request.person_id != *person_id
                    || operation.state.is_terminal()
                    || !person.episodes.contains(&operation.request.episode_id)
                {
                    return Err(TransferError::InvalidCheckpoint);
                }
                let root_operation = self.root_operation(operation_id)?;
                let expected_owner = root_operation.request.origin_service;
                let expected_location = match operation.state {
                    TransferState::Requested
                    | TransferState::Offered
                    | TransferState::Reserved
                    | TransferState::Departing => {
                        TransferLocation::Service(operation.request.origin_service)
                    }
                    TransferState::InTransit => TransferLocation::Transport {
                        from: operation.request.origin_service,
                        to: operation.request.destination_service,
                    },
                    TransferState::Arrived => {
                        TransferLocation::Service(operation.request.destination_service)
                    }
                    TransferState::HandoverComplete
                    | TransferState::Rejected
                    | TransferState::Cancelled
                    | TransferState::Expired => return Err(TransferError::InvalidCheckpoint),
                };
                if person.owner_service != expected_owner || person.location != expected_location {
                    return Err(TransferError::InvalidCheckpoint);
                }
            } else {
                let terminal_operation_id =
                    self.events
                        .iter()
                        .rev()
                        .find(|event| {
                            event.state.is_terminal()
                                && self.operations.get(&event.operation_id).is_some_and(
                                    |operation| operation.request.person_id == *person_id,
                                )
                        })
                        .map(|event| event.operation_id)
                        .ok_or(TransferError::InvalidCheckpoint)?;
                let terminal_operation = self
                    .operations
                    .get(&terminal_operation_id)
                    .ok_or(TransferError::InvalidCheckpoint)?;
                let (expected_owner, expected_location) = match terminal_operation.state {
                    TransferState::HandoverComplete => (
                        terminal_operation.request.destination_service,
                        TransferLocation::Service(terminal_operation.request.destination_service),
                    ),
                    TransferState::Rejected | TransferState::Cancelled | TransferState::Expired => {
                        (
                            self.root_operation(terminal_operation_id)?
                                .request
                                .origin_service,
                            TransferLocation::Service(terminal_operation.request.origin_service),
                        )
                    }
                    _ => return Err(TransferError::InvalidCheckpoint),
                };
                if person.owner_service != expected_owner || person.location != expected_location {
                    return Err(TransferError::InvalidCheckpoint);
                }
            }
        }
        for (operation_id, operation) in &self.operations {
            let person = self
                .people
                .get(&operation.request.person_id)
                .ok_or(TransferError::InvalidCheckpoint)?;
            if !person.episodes.contains(&operation.request.episode_id) {
                return Err(TransferError::InvalidCheckpoint);
            }
            if operation.state.is_terminal() {
                continue;
            }
            let active_operation = person
                .active_operation
                .ok_or(TransferError::InvalidCheckpoint)?;
            if !self.active_chain_contains(active_operation, *operation_id) {
                return Err(TransferError::InvalidCheckpoint);
            }
            for resource_id in &operation.origin_resources {
                let claim = self
                    .claims
                    .get(resource_id)
                    .ok_or(TransferError::InvalidCheckpoint)?;
                let expected_disposition = if matches!(
                    operation.state,
                    TransferState::InTransit | TransferState::Arrived
                ) {
                    ClaimDisposition::Held
                } else {
                    ClaimDisposition::Occupied
                };
                if claim.operation_id != Some(*operation_id)
                    || claim.role != ClaimRole::Origin
                    || claim.disposition != expected_disposition
                    || claim.owner_service != operation.request.origin_service
                {
                    return Err(TransferError::InvalidCheckpoint);
                }
            }
            for resource_id in &operation.destination_resources {
                let claim = self
                    .claims
                    .get(resource_id)
                    .ok_or(TransferError::InvalidCheckpoint)?;
                let expected_disposition = match operation.state {
                    TransferState::Requested | TransferState::Offered => {
                        return Err(TransferError::InvalidCheckpoint)
                    }
                    TransferState::Reserved | TransferState::Departing => {
                        ClaimDisposition::Reserved
                    }
                    TransferState::InTransit => ClaimDisposition::InFlightHold,
                    TransferState::Arrived => ClaimDisposition::Occupied,
                    TransferState::HandoverComplete
                    | TransferState::Rejected
                    | TransferState::Cancelled
                    | TransferState::Expired => return Err(TransferError::InvalidCheckpoint),
                };
                if claim.operation_id != Some(*operation_id)
                    || claim.role != ClaimRole::Destination
                    || claim.disposition != expected_disposition
                    || claim.owner_service != operation.request.destination_service
                {
                    return Err(TransferError::InvalidCheckpoint);
                }
            }
        }
        for (operation_id, operation) in &self.operations {
            if let Some(parent_id) = operation.compensation_parent {
                let parent = self
                    .operations
                    .get(&parent_id)
                    .ok_or(TransferError::InvalidCheckpoint)?;
                if parent.request.person_id != operation.request.person_id
                    || parent.request.episode_id != operation.request.episode_id
                    || !parent.compensation_children.contains(operation_id)
                {
                    return Err(TransferError::InvalidCheckpoint);
                }
            }
            if operation.compensation_children.iter().any(|child| {
                self.operations
                    .get(child)
                    .is_none_or(|item| item.compensation_parent != Some(*operation_id))
            }) || operation.active_compensation.is_some_and(|child| {
                self.operations.get(&child).is_none_or(|item| {
                    item.compensation_parent != Some(*operation_id) || item.state.is_terminal()
                })
            }) {
                return Err(TransferError::InvalidCheckpoint);
            }
        }
        Ok(())
    }

    fn root_operation(
        &self,
        operation_id: TransferOperationId,
    ) -> Result<&TransferOperation, TransferError> {
        let mut operation = self
            .operations
            .get(&operation_id)
            .ok_or(TransferError::InvalidCheckpoint)?;
        let mut visited = BTreeSet::new();
        visited.insert(operation_id);
        while let Some(parent_id) = operation.compensation_parent {
            if !visited.insert(parent_id) {
                return Err(TransferError::InvalidCheckpoint);
            }
            operation = self
                .operations
                .get(&parent_id)
                .ok_or(TransferError::InvalidCheckpoint)?;
        }
        Ok(operation)
    }

    fn active_chain_contains(
        &self,
        active_operation: TransferOperationId,
        target_operation: TransferOperationId,
    ) -> bool {
        let mut current = Some(active_operation);
        let mut visited = BTreeSet::new();
        while let Some(operation_id) = current {
            if !visited.insert(operation_id) {
                return false;
            }
            if operation_id == target_operation {
                return true;
            }
            current = self
                .operations
                .get(&operation_id)
                .and_then(|operation| operation.active_compensation);
        }
        false
    }
}

impl TransferOperation {
    fn deadline_for_state(&self) -> Option<u64> {
        match self.state {
            TransferState::Requested => self.request.request_expires_at,
            TransferState::Offered => self.offer_expires_at,
            TransferState::Reserved | TransferState::Departing => self.reservation_expires_at,
            TransferState::InTransit
            | TransferState::Arrived
            | TransferState::HandoverComplete
            | TransferState::Rejected
            | TransferState::Cancelled
            | TransferState::Expired => None,
        }
    }
}

fn validate_request(request: &TransferRequest, tick: u64) -> Result<(), TransferError> {
    if !request.person_id.is_valid()
        || !request.episode_id.is_valid()
        || !request.origin_service.is_valid()
        || !request.destination_service.is_valid()
        || request.origin_service == request.destination_service
        || request
            .request_expires_at
            .is_some_and(|deadline| deadline <= tick)
        || !unique_valid_resources(&request.origin_resources)
    {
        return Err(TransferError::InvalidRequest);
    }
    Ok(())
}

fn unique_valid_resources(resources: &[ResourceId]) -> bool {
    let mut unique = BTreeSet::new();
    resources
        .iter()
        .all(|resource| resource.is_valid() && unique.insert(*resource))
}

/// Public single-scheduler owner for CareOps transfer envelopes.
#[derive(Clone, Debug, Default)]
pub struct TransferManager(TransferLedger);

impl TransferManager {
    pub fn apply(&mut self, envelope: TransferEnvelope) -> Result<ApplyOutcome, TransferError> {
        self.0.apply(envelope)
    }

    pub fn advance_to(&mut self, tick: u64) -> Result<(), TransferError> {
        self.0.advance_to(tick)
    }

    pub fn snapshot(&self, operation_id: TransferOperationId) -> Option<TransferSnapshot> {
        self.0.snapshot(operation_id)
    }

    pub fn person(&self, person_id: PersonId) -> Option<PersonSnapshot> {
        self.0.person(person_id)
    }

    pub fn claims(&self) -> Vec<ResourceClaimSnapshot> {
        self.0.claims()
    }

    pub fn events(&self) -> &[TransferEventRecord] {
        self.0.events()
    }

    pub const fn now(&self) -> u64 {
        self.0.now()
    }

    pub fn checkpoint(&self) -> Result<Vec<u8>, serde_json::Error> {
        self.0.checkpoint()
    }

    pub fn restore(bytes: &[u8]) -> Result<Self, TransferError> {
        TransferLedger::restore(bytes).map(Self)
    }
}
