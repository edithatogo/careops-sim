//! Experimental CareOps coordinator for staged single-resource claims.
//!
//! This state machine records requests and acknowledgements from a single-
//! resource allocator. It does not itself arbitrate capacity and does not
//! provide an atomic multi-resource grant.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ResourceId(u64);

impl ResourceId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OperationId(u64);

impl OperationId {
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Engine request identity copied from public request readback.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EngineRequestId {
    pub index: u64,
    pub generation: u32,
}

impl EngineRequestId {
    pub const fn new(index: u64, generation: u32) -> Self {
        Self { index, generation }
    }
}

/// Engine resource identity copied from the adapter's resource mapping.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EngineResourceId {
    pub index: u64,
    pub generation: u32,
}

impl EngineResourceId {
    pub const fn new(index: u64, generation: u32) -> Self {
        Self { index, generation }
    }
}

/// Opaque engine lease identity, retaining the request and revision pair.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LeaseId {
    request: EngineRequestId,
    revision: u64,
}

impl LeaseId {
    pub const fn new(request: EngineRequestId, revision: u64) -> Self {
        Self { request, revision }
    }

    pub const fn request_id(self) -> EngineRequestId {
        self.request
    }

    pub const fn revision(self) -> u64 {
        self.revision
    }
}

/// Token for one outstanding single-resource request. A late grant for a
/// superseded token is retained as an orphan release instead of satisfying a
/// newer operation revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClaimRequest {
    operation: OperationId,
    revision: u64,
    resource: ResourceId,
}

impl ClaimRequest {
    pub const fn operation_id(self) -> OperationId {
        self.operation
    }

    pub const fn revision(self) -> u64 {
        self.revision
    }

    pub const fn resource(self) -> ResourceId {
        self.resource
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GrantReceipt {
    pub claim_request: ClaimRequest,
    pub careops_resource: ResourceId,
    pub engine_resource: EngineResourceId,
    pub lease: LeaseId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EngineBinding {
    request: EngineRequestId,
    resource: EngineResourceId,
    lease_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocationState {
    Queued,
    Acquiring,
    Ready,
    Active,
    Suspended,
    RollingBack,
    Releasing,
    Failed,
    Cancelled,
    TimedOut,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrantDisposition {
    Accepted,
    /// The grant arrived after cancellation, expiry, completion, or its
    /// deadline and must be released as an orphan claim.
    ReleaseLateGrant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocationError {
    EmptyResourceSet,
    DuplicateResource,
    InvalidWorkUnits,
    UnknownOperation,
    InvalidState,
    StaleWorkRevision,
    InvalidProgress,
    NotInterruptible,
    UnknownLease,
    DuplicateLease,
    CounterOverflow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationSnapshot {
    pub state: AllocationState,
    pub progress_units: u64,
    pub total_work_units: u64,
    pub work_revision: u64,
    pub claimed: Vec<(ResourceId, LeaseId)>,
}

#[derive(Clone, Debug)]
struct Operation {
    sequence: u64,
    resources: Vec<ResourceId>,
    next_resource: usize,
    next_request_revision: u64,
    pending: Option<ClaimRequest>,
    engine_binding: Option<EngineBinding>,
    claims: BTreeMap<ResourceId, LeaseId>,
    acquisition_order: Vec<LeaseId>,
    deadline: Option<u64>,
    total_work_units: u64,
    progress_units: u64,
    work_revision: u64,
    interruptible: bool,
    state: AllocationState,
    terminal_after_release: Option<AllocationState>,
}

/// Owns visible CareOps-side state for a bounded staged operation.
#[derive(Clone, Debug, Default)]
pub struct StagedAllocator {
    operations: BTreeMap<OperationId, Operation>,
    orphan_releases: BTreeSet<LeaseId>,
    next_operation: u64,
    next_sequence: u64,
}

impl StagedAllocator {
    /// Add a staged operation. Resource IDs are normalized into a deterministic
    /// order; duplicates are rejected because quantity semantics are separate.
    pub fn submit(
        &mut self,
        resources: impl IntoIterator<Item = ResourceId>,
        deadline: Option<u64>,
        total_work_units: u64,
        interruptible: bool,
    ) -> Result<OperationId, AllocationError> {
        if total_work_units == 0 {
            return Err(AllocationError::InvalidWorkUnits);
        }
        let mut resources: Vec<_> = resources.into_iter().collect();
        if resources.is_empty() {
            return Err(AllocationError::EmptyResourceSet);
        }
        resources.sort_unstable();
        if resources.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(AllocationError::DuplicateResource);
        }
        let id = OperationId(
            self.next_operation
                .checked_add(1)
                .ok_or(AllocationError::CounterOverflow)?,
        );
        let sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(AllocationError::CounterOverflow)?;
        self.next_operation = id.0;
        self.next_sequence = sequence;
        self.operations.insert(
            id,
            Operation {
                sequence,
                resources,
                next_resource: 0,
                next_request_revision: 0,
                pending: None,
                engine_binding: None,
                claims: BTreeMap::new(),
                acquisition_order: Vec::new(),
                deadline,
                total_work_units,
                progress_units: 0,
                work_revision: 0,
                interruptible,
                state: AllocationState::Queued,
                terminal_after_release: None,
            },
        );
        Ok(id)
    }

    /// Issue or return the existing outstanding claim token for each eligible
    /// operation, in FIFO admission order. This defines CareOps issuance only;
    /// the downstream engine still controls grant arbitration and availability.
    pub fn next_claim_requests(&mut self) -> Result<Vec<ClaimRequest>, AllocationError> {
        let mut order: Vec<_> = self
            .operations
            .iter()
            .map(|(id, operation)| (operation.sequence, *id))
            .collect();
        order.sort_unstable();
        let mut issued = Vec::new();
        for (_, id) in order {
            let operation = self
                .operations
                .get_mut(&id)
                .ok_or(AllocationError::UnknownOperation)?;
            if !matches!(
                operation.state,
                AllocationState::Queued | AllocationState::Acquiring
            ) {
                continue;
            }
            if let Some(pending) = operation.pending {
                issued.push(pending);
                continue;
            }
            let Some(resource) = operation.resources.get(operation.next_resource).copied() else {
                continue;
            };
            operation.next_request_revision = operation
                .next_request_revision
                .checked_add(1)
                .ok_or(AllocationError::CounterOverflow)?;
            let request = ClaimRequest {
                operation: id,
                revision: operation.next_request_revision,
                resource,
            };
            operation.pending = Some(request);
            operation.engine_binding = None;
            operation.state = AllocationState::Acquiring;
            issued.push(request);
        }
        Ok(issued)
    }

    /// Bind a claim token to the engine request/resource after submission has
    /// returned its request identity and before accepting its grant readback.
    pub fn bind_engine_request(
        &mut self,
        request: ClaimRequest,
        engine_request: EngineRequestId,
        engine_resource: EngineResourceId,
    ) -> Result<(), AllocationError> {
        let operation = self
            .operations
            .get_mut(&request.operation)
            .ok_or(AllocationError::UnknownOperation)?;
        if operation.pending != Some(request) || operation.engine_binding.is_some() {
            return Err(AllocationError::InvalidState);
        }
        operation.engine_binding = Some(EngineBinding {
            request: engine_request,
            resource: engine_resource,
            lease_revision: None,
        });
        Ok(())
    }

    /// Bind the exact revision observed in the engine's current request/lease
    /// readback before recording the grant.
    pub fn bind_engine_lease_revision(
        &mut self,
        request: ClaimRequest,
        engine_request: EngineRequestId,
        lease_revision: u64,
    ) -> Result<(), AllocationError> {
        let operation = self
            .operations
            .get_mut(&request.operation)
            .ok_or(AllocationError::UnknownOperation)?;
        if operation.pending != Some(request) {
            return Err(AllocationError::InvalidState);
        }
        let binding = operation
            .engine_binding
            .as_mut()
            .ok_or(AllocationError::InvalidState)?;
        if binding.request != engine_request || binding.lease_revision.is_some() {
            return Err(AllocationError::InvalidState);
        }
        binding.lease_revision = Some(lease_revision);
        Ok(())
    }

    /// Record a single-resource grant. The engine request, resource and lease
    /// revision must match the bound request. Deadline ties lose to timeout.
    pub fn grant(
        &mut self,
        receipt: GrantReceipt,
        at: u64,
    ) -> Result<GrantDisposition, AllocationError> {
        let request = receipt.claim_request;
        let lease = receipt.lease;
        if self.lease_exists(lease) {
            return Err(AllocationError::DuplicateLease);
        }
        let pending_matches = self
            .operations
            .get(&request.operation)
            .is_some_and(|operation| {
                operation.pending == Some(request)
                    && matches!(
                        operation.state,
                        AllocationState::Acquiring | AllocationState::Queued
                    )
            });
        if !pending_matches {
            self.orphan_releases.insert(lease);
            return Ok(GrantDisposition::ReleaseLateGrant);
        }
        let binding_matches = self
            .operations
            .get(&request.operation)
            .and_then(|operation| operation.engine_binding)
            .is_some_and(|binding| {
                binding.request == lease.request_id()
                    && binding.resource == receipt.engine_resource
                    && request.resource == receipt.careops_resource
                    && binding.lease_revision == Some(lease.revision())
            });
        let operation = self
            .operations
            .get_mut(&request.operation)
            .ok_or(AllocationError::UnknownOperation)?;
        if !binding_matches {
            operation.pending = None;
            operation.engine_binding = None;
            Self::begin_release(operation, AllocationState::Failed);
            self.orphan_releases.insert(lease);
            return Ok(GrantDisposition::ReleaseLateGrant);
        }
        if operation.deadline.is_some_and(|deadline| at >= deadline) {
            operation.pending = None;
            operation.engine_binding = None;
            Self::begin_release(operation, AllocationState::TimedOut);
            self.orphan_releases.insert(lease);
            return Ok(GrantDisposition::ReleaseLateGrant);
        }
        if operation.resources.get(operation.next_resource) != Some(&request.resource) {
            return Err(AllocationError::InvalidState);
        }
        operation.claims.insert(request.resource, lease);
        operation.acquisition_order.push(lease);
        operation.pending = None;
        operation.engine_binding = None;
        operation.next_resource += 1;
        operation.state = if operation.next_resource == operation.resources.len() {
            AllocationState::Ready
        } else {
            AllocationState::Acquiring
        };
        Ok(GrantDisposition::Accepted)
    }

    /// Reject one currently outstanding single-resource request and compensate
    /// every previously granted lease.
    pub fn reject(&mut self, request: ClaimRequest, at: u64) -> Result<(), AllocationError> {
        let operation = self
            .operations
            .get_mut(&request.operation)
            .ok_or(AllocationError::UnknownOperation)?;
        if operation.pending != Some(request) {
            return Err(AllocationError::InvalidState);
        }
        operation.pending = None;
        operation.engine_binding = None;
        let terminal = if operation.deadline.is_some_and(|deadline| at >= deadline) {
            AllocationState::TimedOut
        } else {
            AllocationState::Failed
        };
        Self::begin_release(operation, terminal);
        Ok(())
    }

    /// Cancel an operation before work starts. Repeating cancellation after a
    /// cancelled or timed-out outcome is harmless.
    pub fn cancel(&mut self, id: OperationId, at: u64) -> Result<(), AllocationError> {
        let operation = self
            .operations
            .get_mut(&id)
            .ok_or(AllocationError::UnknownOperation)?;
        if matches!(
            operation.state,
            AllocationState::Cancelled | AllocationState::TimedOut | AllocationState::RollingBack
        ) {
            return Ok(());
        }
        if !matches!(
            operation.state,
            AllocationState::Queued | AllocationState::Acquiring | AllocationState::Ready
        ) {
            return Err(AllocationError::InvalidState);
        }
        operation.pending = None;
        operation.engine_binding = None;
        let terminal = if operation.deadline.is_some_and(|deadline| at >= deadline) {
            AllocationState::TimedOut
        } else {
            AllocationState::Cancelled
        };
        Self::begin_release(operation, terminal);
        Ok(())
    }

    /// Expire all not-yet-active operations whose exclusive deadline has been
    /// reached. `deadline <= now` makes grant/timeout ties deterministic.
    pub fn expire(&mut self, now: u64) -> Result<Vec<OperationId>, AllocationError> {
        let mut expired = Vec::new();
        for (id, operation) in &mut self.operations {
            if operation.deadline.is_some_and(|deadline| now >= deadline)
                && matches!(
                    operation.state,
                    AllocationState::Queued | AllocationState::Acquiring | AllocationState::Ready
                )
            {
                operation.pending = None;
                operation.engine_binding = None;
                Self::begin_release(operation, AllocationState::TimedOut);
                expired.push(*id);
            }
        }
        Ok(expired)
    }

    /// Return release work that still needs acknowledgement. A failed external
    /// release is represented by not acknowledging it; the claim stays visible.
    pub fn releases_due(&self, id: OperationId) -> Vec<LeaseId> {
        self.operations
            .get(&id)
            .filter(|operation| {
                matches!(
                    operation.state,
                    AllocationState::RollingBack | AllocationState::Releasing
                )
            })
            .map(|operation| {
                operation
                    .acquisition_order
                    .iter()
                    .rev()
                    .copied()
                    .filter(|lease| operation.claims.values().any(|held| held == lease))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn orphan_releases_due(&self) -> Vec<LeaseId> {
        self.orphan_releases.iter().copied().collect()
    }

    pub fn release_failed(&self, id: OperationId, lease: LeaseId) -> Result<(), AllocationError> {
        let operation = self
            .operations
            .get(&id)
            .ok_or(AllocationError::UnknownOperation)?;
        if matches!(
            operation.state,
            AllocationState::RollingBack | AllocationState::Releasing
        ) && operation.claims.values().any(|held| *held == lease)
        {
            Ok(())
        } else {
            Err(AllocationError::UnknownLease)
        }
    }

    pub fn ack_release(&mut self, id: OperationId, lease: LeaseId) -> Result<(), AllocationError> {
        let operation = self
            .operations
            .get_mut(&id)
            .ok_or(AllocationError::UnknownOperation)?;
        if !matches!(
            operation.state,
            AllocationState::RollingBack | AllocationState::Releasing
        ) {
            return Err(AllocationError::InvalidState);
        }
        let resource = operation
            .claims
            .iter()
            .find_map(|(resource, held)| (*held == lease).then_some(*resource))
            .ok_or(AllocationError::UnknownLease)?;
        operation.claims.remove(&resource);
        operation.acquisition_order.retain(|held| *held != lease);
        Self::finish_release_if_empty(operation);
        Ok(())
    }

    pub fn ack_orphan_release(&mut self, lease: LeaseId) -> Result<(), AllocationError> {
        if !self.orphan_releases.remove(&lease) {
            return Err(AllocationError::UnknownLease);
        }
        Ok(())
    }

    pub fn start(&mut self, id: OperationId) -> Result<u64, AllocationError> {
        let operation = self
            .operations
            .get_mut(&id)
            .ok_or(AllocationError::UnknownOperation)?;
        if operation.state != AllocationState::Ready {
            return Err(AllocationError::InvalidState);
        }
        operation.work_revision = operation
            .work_revision
            .checked_add(1)
            .ok_or(AllocationError::CounterOverflow)?;
        operation.state = AllocationState::Active;
        Ok(operation.work_revision)
    }

    /// Suspend only at a caller-declared work-segment boundary. There is no
    /// urgency or priority input to this operation.
    pub fn suspend_at_segment_boundary(
        &mut self,
        id: OperationId,
        revision: u64,
        completed_units: u64,
    ) -> Result<u64, AllocationError> {
        let operation = self
            .operations
            .get_mut(&id)
            .ok_or(AllocationError::UnknownOperation)?;
        if operation.state != AllocationState::Active {
            return Err(AllocationError::InvalidState);
        }
        if operation.work_revision != revision {
            return Err(AllocationError::StaleWorkRevision);
        }
        if !operation.interruptible {
            return Err(AllocationError::NotInterruptible);
        }
        if completed_units <= operation.progress_units
            || completed_units > operation.total_work_units
        {
            return Err(AllocationError::InvalidProgress);
        }
        operation.work_revision = operation
            .work_revision
            .checked_add(1)
            .ok_or(AllocationError::CounterOverflow)?;
        operation.progress_units = completed_units;
        operation.state = AllocationState::Suspended;
        Ok(operation.work_revision)
    }

    pub fn resume(&mut self, id: OperationId) -> Result<u64, AllocationError> {
        let operation = self
            .operations
            .get_mut(&id)
            .ok_or(AllocationError::UnknownOperation)?;
        if operation.state != AllocationState::Suspended {
            return Err(AllocationError::InvalidState);
        }
        operation.work_revision = operation
            .work_revision
            .checked_add(1)
            .ok_or(AllocationError::CounterOverflow)?;
        operation.state = AllocationState::Active;
        Ok(operation.work_revision)
    }

    pub fn complete(&mut self, id: OperationId, revision: u64) -> Result<(), AllocationError> {
        let operation = self
            .operations
            .get_mut(&id)
            .ok_or(AllocationError::UnknownOperation)?;
        if operation.work_revision != revision {
            return Err(AllocationError::StaleWorkRevision);
        }
        if operation.state != AllocationState::Active {
            return Err(AllocationError::InvalidState);
        }
        operation.progress_units = operation.total_work_units;
        Self::begin_release(operation, AllocationState::Completed);
        Ok(())
    }

    pub fn state(&self, id: OperationId) -> Option<AllocationState> {
        self.operations.get(&id).map(|operation| operation.state)
    }

    pub fn progress(&self, id: OperationId) -> Option<u64> {
        self.operations
            .get(&id)
            .map(|operation| operation.progress_units)
    }

    pub fn claims(&self, id: OperationId) -> Vec<(ResourceId, LeaseId)> {
        self.operations
            .get(&id)
            .map(|operation| {
                operation
                    .claims
                    .iter()
                    .map(|(resource, lease)| (*resource, *lease))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn snapshot(&self, id: OperationId) -> Option<OperationSnapshot> {
        let operation = self.operations.get(&id)?;
        Some(OperationSnapshot {
            state: operation.state,
            progress_units: operation.progress_units,
            total_work_units: operation.total_work_units,
            work_revision: operation.work_revision,
            claimed: self.claims(id),
        })
    }

    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Count all leases still known locally, including late grants awaiting
    /// release. A failed release never disappears from this conservation view.
    pub fn claim_count(&self) -> usize {
        self.operations
            .values()
            .map(|operation| operation.claims.len())
            .sum::<usize>()
            + self.orphan_releases.len()
    }

    fn lease_exists(&self, lease: LeaseId) -> bool {
        self.orphan_releases.contains(&lease)
            || self
                .operations
                .values()
                .any(|operation| operation.claims.values().any(|held| *held == lease))
    }

    fn begin_release(operation: &mut Operation, terminal: AllocationState) {
        operation.terminal_after_release = Some(terminal);
        operation.state = if terminal == AllocationState::Completed {
            AllocationState::Releasing
        } else {
            AllocationState::RollingBack
        };
        Self::finish_release_if_empty(operation);
    }

    fn finish_release_if_empty(operation: &mut Operation) {
        if operation.claims.is_empty() && operation.pending.is_none() {
            if let Some(terminal) = operation.terminal_after_release.take() {
                operation.state = terminal;
            }
        }
    }
}
