//! Bounded ED facade over one authoritative native Kairos Flow runtime.
//!
//! This module only binds supplied synthetic service facts and explicit parent
//! operations. It does not sample, select policy, or persist native identities.
use kairo_ecs_core::ScheduledEventPreview;
use kairo_ecs_des::{
    FlowAllocationMode, FlowBudgetSnapshot, FlowCallbackConfig, FlowCallbackSnapshot,
    FlowCommandSink, FlowConfig, FlowDispatch, FlowError, FlowQueueOrderKey, FlowQueueOrdering,
    FlowRuntime, FlowRuntimeIdentity, FlowWorldView, LifecycleTransition, PreemptionStrategy,
    RequestId, RequestState, ResourceAllocationAdmissionError, ResourceGrantAdmissionError,
    ResourceId, ResourceSnapshot, WorkId, WorkProgress, WorkState,
};
use kairo_ecs_types::{EntityId, EventId, EventKind, SimDuration, SimTime};
use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU64, NonZeroUsize};

const SERVICE_CONTEXT: &str = "careops-ed-frozen-service-v1";
const MARKER_CONTEXT: &str = "careops-ed-parent-marker-v1";
const MARKER_KIND: EventKind = EventKind::custom(4100);
const HORIZON_OPERATION: &str = "__ed_horizon__";
const U32_MAX: u128 = u32::MAX as u128;

#[doc(hidden)]
mod transit;
pub use transit::{TransitBounds, TransitHandle, TransitInput};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterError {
    InvalidInput(&'static str),
    UnknownResource,
    DuplicateIdentity,
    ForeignHandle,
    WrongHandleRole,
    WrongNativeState,
    BudgetExceeded(&'static str),
    ClosedPhase,
    DuplicateOperation,
    UnclosedCancellationBatch,
    UnclosedGrantPhase,
    Native(FlowError),
    NativeGrant(ResourceGrantAdmissionError),
    NativeAllocation(ResourceAllocationAdmissionError),
    NativeDispatchFailed {
        error: FlowError,
        dispatch: Box<FlowDispatch>,
    },
}
impl From<FlowError> for AdapterError {
    fn from(value: FlowError) -> Self {
        Self::Native(value)
    }
}
impl From<ResourceGrantAdmissionError> for AdapterError {
    fn from(value: ResourceGrantAdmissionError) -> Self {
        Self::NativeGrant(value)
    }
}
impl From<ResourceAllocationAdmissionError> for AdapterError {
    fn from(value: ResourceAllocationAdmissionError) -> Self {
        Self::NativeAllocation(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ServiceIdentity {
    pub patient_id: String,
    pub episode_id: String,
    pub stage_index: u32,
    pub segment_index: u32,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SampleOrigin {
    Fixed,
    Sampled {
        root_seed: u64,
        stream_slot: u64,
        draws_before: u64,
        draws_after: u64,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenService {
    pub identity: ServiceIdentity,
    pub original_ticks: u64,
    pub useful_before_ticks: u64,
    pub remaining_ticks: u64,
    pub origin: SampleOrigin,
}
impl FrozenService {
    fn validate(&self) -> Result<(), AdapterError> {
        check_id(&self.identity.patient_id)?;
        if self.identity.episode_id.is_empty()
            || self.identity.episode_id.len() > 128
            || !self.identity.episode_id.is_ascii()
        {
            return Err(AdapterError::InvalidInput(
                "episode identity must be 1..128 ASCII bytes",
            ));
        }
        if self.original_ticks == 0 || self.remaining_ticks == 0 {
            return Err(AdapterError::InvalidInput(
                "service duration must be positive",
            ));
        }
        if self.useful_before_ticks.checked_add(self.remaining_ticks) != Some(self.original_ticks) {
            return Err(AdapterError::InvalidInput(
                "useful_before + remaining must equal original",
            ));
        }
        if let SampleOrigin::Sampled {
            draws_before,
            draws_after,
            ..
        } = &self.origin
        {
            if draws_before > draws_after {
                return Err(AdapterError::InvalidInput("sample draw order"));
            }
        }
        Ok(())
    }
}
fn check_id(value: &str) -> Result<(), AdapterError> {
    if value.is_empty() || value.len() > 64 || !value.is_ascii() {
        return Err(AdapterError::InvalidInput(
            "identifier must be 1..64 ASCII bytes",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceSpec {
    pub id: String,
    pub capacity: u32,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantBoundary {
    pub resource_id: String,
    pub at_tick: u64,
    pub enabled: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FoundationBounds {
    pub patient_owners: u64,
    pub services: u64,
    pub requests: u64,
    pub preempt_capable_requests: u64,
    pub releases_or_cancels: u64,
    pub parent_markers: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterPlan {
    pub resources: Vec<ResourceSpec>,
    pub grant_boundaries: Vec<GrantBoundary>,
    pub horizon_tick: u64,
    pub bounds: FoundationBounds,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedAdapterPlan {
    resources: Vec<ResourceSpec>,
    grant_boundaries: Vec<GrantBoundary>,
    horizon_tick: u64,
    bounds: FoundationBounds,
    transition_limit: u64,
    event_limit: u64,
    entity_limit: u64,
    flush_limit: u64,
}
impl AdapterPlan {
    pub fn validate(
        resources: Vec<ResourceSpec>,
        grant_boundaries: Vec<GrantBoundary>,
        horizon_tick: u64,
        bounds: FoundationBounds,
    ) -> Result<ValidatedAdapterPlan, AdapterError> {
        Self {
            resources,
            grant_boundaries,
            horizon_tick,
            bounds,
        }
        .validate_inner()
    }
    fn validate_inner(mut self) -> Result<ValidatedAdapterPlan, AdapterError> {
        if self.horizon_tick == 0 {
            return Err(AdapterError::InvalidInput("horizon must be positive"));
        }
        if self.bounds.services > self.bounds.requests
            || self.bounds.preempt_capable_requests > self.bounds.services
        {
            return Err(AdapterError::InvalidInput(
                "services <= requests and preempt-capable <= services",
            ));
        }
        let mut resources = BTreeMap::<String, ResourceSpec>::new();
        for r in self.resources.drain(..) {
            check_id(&r.id)?;
            if r.capacity == 0 {
                return Err(AdapterError::InvalidInput(
                    "resource capacity must be positive",
                ));
            }
            if resources.insert(r.id.clone(), r).is_some() {
                return Err(AdapterError::DuplicateIdentity);
            }
        }
        if resources.len() as u128 > U32_MAX {
            return Err(AdapterError::BudgetExceeded("resource entities"));
        }
        let mut boundaries = BTreeMap::<(String, u64), bool>::new();
        for b in self.grant_boundaries.drain(..) {
            check_id(&b.resource_id)?;
            if !resources.contains_key(&b.resource_id) {
                return Err(AdapterError::UnknownResource);
            }
            if b.at_tick > self.horizon_tick {
                return Err(AdapterError::InvalidInput("grant boundary exceeds horizon"));
            }
            match boundaries.get(&(b.resource_id.clone(), b.at_tick)) {
                Some(previous) if *previous != b.enabled => {
                    return Err(AdapterError::InvalidInput(
                        "conflicting duplicate grant boundary",
                    ))
                }
                Some(_) => {}
                None => {
                    boundaries.insert((b.resource_id, b.at_tick), b.enabled);
                }
            }
        }
        let grants: Vec<GrantBoundary> = boundaries
            .into_iter()
            .map(|((resource_id, at_tick), enabled)| GrantBoundary {
                resource_id,
                at_tick,
                enabled,
            })
            .collect();
        let n = self.bounds.requests as u128;
        let w = self.bounds.services as u128;
        let k = self.bounds.preempt_capable_requests as u128;
        let p = self.bounds.releases_or_cancels as u128;
        let d = self.bounds.parent_markers as u128 + 1;
        let b = grants.len() as u128;
        let r = resources.len() as u128;
        let a = self.bounds.patient_owners as u128;
        let transition = 3u128
            .checked_mul(n)
            .and_then(|x| x.checked_add(2u128.checked_mul(k)?))
            .and_then(|x| x.checked_add(d))
            .ok_or(AdapterError::BudgetExceeded("same-tick transitions"))?
            .max(1);
        let flushes = r
            .checked_add(n)
            .and_then(|x| x.checked_add(p))
            .and_then(|x| x.checked_add(w.checked_add(k)?))
            .and_then(|x| x.checked_add(b))
            .ok_or(AdapterError::BudgetExceeded("flushes"))?;
        let events = d
            .checked_add(b)
            .and_then(|x| x.checked_add(n))
            .and_then(|x| x.checked_add(p))
            .and_then(|x| x.checked_add(w.checked_add(k)?))
            .and_then(|x| x.checked_add(flushes))
            .ok_or(AdapterError::BudgetExceeded("scheduled events"))?;
        let entities = a
            .checked_add(w)
            .and_then(|x| x.checked_add(n))
            .and_then(|x| x.checked_add(r))
            .and_then(|x| x.checked_add(2))
            .ok_or(AdapterError::BudgetExceeded("created entities"))?;
        if events > U32_MAX
            || entities > U32_MAX
            || n.checked_add(k)
                .ok_or(AdapterError::BudgetExceeded("leases"))?
                > U32_MAX
            || transition > u64::MAX as u128
        {
            return Err(AdapterError::BudgetExceeded(
                "native u32/u64 representation",
            ));
        }
        Ok(ValidatedAdapterPlan {
            resources: resources.into_values().collect(),
            grant_boundaries: grants,
            horizon_tick: self.horizon_tick,
            bounds: self.bounds,
            transition_limit: transition as u64,
            event_limit: events as u64,
            entity_limit: entities as u64,
            flush_limit: flushes as u64,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct QueueKey(Vec<u8>);
impl QueueKey {
    pub fn from_request(ready_tick: u64, request_id: String) -> Result<Self, AdapterError> {
        check_id(&request_id)?;
        let mut bytes = ready_tick.to_be_bytes().to_vec();
        bytes.extend_from_slice(request_id.as_bytes());
        if bytes.len() > 72 {
            return Err(AdapterError::InvalidInput("request queue key too long"));
        }
        Ok(Self(bytes))
    }
    pub fn from_e1_ordinal(
        ready_tick: u64,
        patient_ordinal: u64,
        work_kind: u64,
        index: u64,
    ) -> Self {
        let suffix = format!("{patient_ordinal:016X}:{work_kind:016X}:{index:016X}");
        let mut bytes = ready_tick.to_be_bytes().to_vec();
        bytes.extend_from_slice(suffix.as_bytes());
        Self(bytes)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    fn native(&self) -> Result<FlowQueueOrderKey, AdapterError> {
        FlowQueueOrderKey::new(&self.0)
            .map_err(|_| AdapterError::InvalidInput("native queue key bound"))
    }
    fn ready_tick(&self) -> u64 {
        u64::from_be_bytes(self.0[..8].try_into().expect("8 byte prefix"))
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Admission {
    pub resource_id: String,
    pub key: QueueKey,
    pub urgency: i32,
    pub can_preempt: bool,
    pub preemptible: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ParentPhase {
    CompletionEffects,
    CalendarExpiry,
    Reservation,
    Decision,
    Ready,
    AfterGrantDecision,
    Horizon,
}
impl ParentPhase {
    pub const fn priority(self) -> i32 {
        match self {
            Self::CompletionEffects => 5,
            Self::CalendarExpiry => 10,
            Self::Reservation => 20,
            Self::Decision => 30,
            Self::Ready => 40,
            Self::AfterGrantDecision => 70,
            Self::Horizon => 90,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceHandle {
    runtime: FlowRuntimeIdentity,
    work: WorkId,
    identity: ServiceIdentity,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RequestRole {
    Service,
    Manual,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestHandle {
    runtime: FlowRuntimeIdentity,
    request: RequestId,
    role: RequestRole,
    resource_id: String,
    patient_id: String,
}
impl ServiceHandle {
    pub fn work_id(&self) -> WorkId {
        self.work
    }
    pub fn identity(&self) -> &ServiceIdentity {
        &self.identity
    }
}
impl RequestHandle {
    pub fn request_id(&self) -> RequestId {
        self.request
    }
    pub fn patient_id(&self) -> &str {
        &self.patient_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceObservation {
    pub identity: ServiceIdentity,
    pub request: RequestId,
    pub work: WorkId,
    pub origin: SampleOrigin,
    pub original_ticks: u64,
    pub useful_before_ticks: u64,
    pub useful_elapsed_ticks: u128,
    pub total_useful_ticks: u128,
    pub remaining_ticks: u128,
    pub state: WorkState,
    pub transition: LifecycleTransition,
    pub causal_event: EventId,
    pub ordinal: u32,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualObservation {
    pub patient_id: String,
    pub resource_id: String,
    pub request: RequestId,
    pub lease: Option<kairo_ecs_des::LeaseId>,
    pub transition: LifecycleTransition,
    pub causal_event: EventId,
    pub at: SimTime,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FoundationObservation {
    Marker {
        operation_id: String,
        event: EventId,
        at: SimTime,
    },
    Service(ServiceObservation),
    Manual(ManualObservation),
    Transit {
        identity: ServiceIdentity,
        carrier: WorkId,
        event: EventId,
        at: SimTime,
        phase: kairo_ecs_abm::TransitPhase,
        arrival: Option<kairo_ecs_abm::TransitArrivalHandoff>,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterDispatch {
    pub native: FlowDispatch,
    pub epoch: u64,
    pub observations: Vec<FoundationObservation>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestSnapshot {
    pub request: RequestId,
    pub state: RequestState,
    pub lease: Option<kairo_ecs_des::LeaseId>,
    pub work: Option<WorkId>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceSnapshot {
    pub identity: ServiceIdentity,
    pub request: Option<RequestId>,
    pub progress: WorkProgress,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundationSnapshot {
    pub now: SimTime,
    pub horizon_tick: u64,
    pub requests: Vec<RequestSnapshot>,
    pub services: Vec<ServiceSnapshot>,
    pub resources: BTreeMap<String, ResourceSnapshot>,
    pub counters: FoundationCounters,
    pub native_budget: FlowBudgetSnapshot,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FoundationCounters {
    pub owners: u64,
    pub services: u64,
    pub requests: u64,
    pub preempt_capable_requests: u64,
    pub releases_or_cancels: u64,
    pub parent_markers: u64,
    pub flushes: u64,
    pub scheduled_adapter_events: u64,
}

#[derive(Clone, Debug)]
struct RequestMeta {
    role: RequestRole,
    patient_id: String,
    resource_id: String,
    work: Option<WorkId>,
}
#[derive(Clone, Debug)]
enum NativeEvent {
    Marker {
        operation_id: String,
        phase: ParentPhase,
    },
    Horizon,
    Boundary {
        resource: ResourceId,
    },
    Flush {
        resource: ResourceId,
        at_tick: u128,
    },
}
pub struct EdFlowAdapter {
    runtime_identity: FlowRuntimeIdentity,
    flow: FlowRuntime,
    horizon_tick: u64,
    bounds: FoundationBounds,
    resources: BTreeMap<String, ResourceId>,
    owners: BTreeMap<String, EntityId>,
    services: BTreeMap<WorkId, FrozenService>,
    service_identities: BTreeSet<ServiceIdentity>,
    used_services: BTreeSet<WorkId>,
    requests: BTreeMap<RequestId, RequestMeta>,
    events: BTreeMap<EventId, NativeEvent>,
    dirty: BTreeSet<ResourceId>,
    pending_flushes: BTreeSet<(ResourceId, u128)>,
    pending_mutations: BTreeSet<RequestId>,
    pending_cancellations: BTreeSet<RequestId>,
    marker_ids: BTreeSet<String>,
    phase_cursor: BTreeMap<u128, i32>,
    counters: FoundationCounters,
    event_limit: u64,
    entity_limit: u64,
    entity_used: u64,
    marker_work_id: WorkId,
    flush_limit: u64,
    transit: Option<transit::TransitAdapterState>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MarkerContext;
fn no_command_marker_plan(
    context: &MarkerContext,
    _snapshot: &FlowCallbackSnapshot,
    _view: FlowWorldView<'_>,
    _sink: &mut FlowCommandSink,
) -> Result<MarkerContext, FlowError> {
    Ok(context.clone())
}
fn no_command_marker_receipt(
    _context: &mut MarkerContext,
    _receipt: &kairo_ecs_des::FlowBatchReceipt,
) {
}
impl EdFlowAdapter {
    pub fn new(plan: ValidatedAdapterPlan) -> Result<Self, AdapterError> {
        Self::build(plan, None)
    }
    fn build(
        mut plan: ValidatedAdapterPlan,
        transit_bounds: Option<TransitBounds>,
    ) -> Result<Self, AdapterError> {
        let transit = if let Some(bounds) = transit_bounds {
            let operations = bounds
                .transits
                .checked_add(bounds.total_route_segments)
                .and_then(|n| n.checked_add(bounds.controls.checked_mul(2)?))
                .ok_or(AdapterError::BudgetExceeded("transit operation bounds"))?;
            let entities = bounds
                .transits
                .checked_mul(2)
                .ok_or(AdapterError::BudgetExceeded("transit entity bounds"))?;
            let added_events = operations;
            plan.event_limit = plan
                .event_limit
                .checked_add(added_events)
                .filter(|n| *n <= u32::MAX as u64)
                .ok_or(AdapterError::BudgetExceeded("native event representation"))?;
            plan.entity_limit = plan
                .entity_limit
                .checked_add(entities)
                .filter(|n| *n <= u32::MAX as u64)
                .ok_or(AdapterError::BudgetExceeded("native entity representation"))?;
            plan.transition_limit = plan
                .transition_limit
                .checked_add(operations)
                .ok_or(AdapterError::BudgetExceeded("same-tick transition bound"))?;
            Some(transit::TransitAdapterState::new(bounds))
        } else {
            None
        };
        let flow = FlowRuntime::with_configs(
            FlowConfig {
                max_same_tick_flow_transitions: NonZeroU64::new(plan.transition_limit)
                    .expect("validated nonzero"),
            },
            FlowCallbackConfig {
                max_callback_commands: NonZeroUsize::new(1).expect("positive callback limit"),
            },
        );
        let mut flow = flow;
        flow.register_domain_plan_hook_with_receipt::<MarkerContext>(
            MARKER_CONTEXT,
            MARKER_KIND,
            no_command_marker_plan,
            no_command_marker_receipt,
        )?;
        if transit.is_some() {
            transit::TransitAdapterState::register(&mut flow)?;
        }
        let mut resources = BTreeMap::new();
        for spec in &plan.resources {
            let id = flow.create_resource_with_queue_ordering(
                spec.capacity,
                FlowAllocationMode::Deferred,
                FlowQueueOrdering::StableKey,
            )?;
            resources.insert(spec.id.clone(), id);
        }
        let marker_actor = flow.spawn_actor()?;
        let marker_work = flow.create_work(
            marker_actor,
            SimDuration::ZERO,
            MARKER_CONTEXT,
            MarkerContext,
        )?;
        let mut adapter = Self {
            runtime_identity: flow.identity(),
            flow,
            horizon_tick: plan.horizon_tick,
            bounds: plan.bounds,
            resources,
            owners: BTreeMap::new(),
            services: BTreeMap::new(),
            service_identities: BTreeSet::new(),
            used_services: BTreeSet::new(),
            requests: BTreeMap::new(),
            events: BTreeMap::new(),
            dirty: BTreeSet::new(),
            pending_flushes: BTreeSet::new(),
            pending_mutations: BTreeSet::new(),
            pending_cancellations: BTreeSet::new(),
            marker_ids: BTreeSet::new(),
            phase_cursor: BTreeMap::new(),
            counters: FoundationCounters::default(),
            event_limit: plan.event_limit,
            entity_limit: plan.entity_limit,
            entity_used: plan.resources.len() as u64 + 2,
            marker_work_id: marker_work,
            flush_limit: plan.flush_limit,
            transit,
        };
        for boundary in plan.grant_boundaries {
            let resource = *adapter
                .resources
                .get(&boundary.resource_id)
                .ok_or(AdapterError::UnknownResource)?;
            adapter.reserve_event()?;
            let event = adapter.flow.schedule_resource_grant_enabled(
                resource,
                boundary.enabled,
                SimTime::from_ticks(boundary.at_tick as u128),
                ParentPhase::CalendarExpiry.priority(),
            )?;
            adapter
                .events
                .insert(event, NativeEvent::Boundary { resource });
            adapter.counters.scheduled_adapter_events += 1;
        }
        adapter.reserve_event()?;
        let horizon = adapter.flow.schedule_domain(
            marker_work,
            MARKER_KIND,
            SimTime::from_ticks(plan.horizon_tick as u128),
            ParentPhase::Horizon.priority(),
        )?;
        adapter.events.insert(horizon, NativeEvent::Horizon);
        adapter.counters.scheduled_adapter_events += 1;
        Ok(adapter)
    }
    fn reserve_event(&self) -> Result<(), AdapterError> {
        let scheduled = self.flow.budget_snapshot().scheduler.scheduled_events;
        if scheduled >= self.event_limit {
            Err(AdapterError::BudgetExceeded("scheduled events"))
        } else {
            Ok(())
        }
    }
    fn reserve_entity(&self, additional: u64) -> Result<(), AdapterError> {
        if self
            .entity_used
            .checked_add(additional)
            .filter(|n| *n <= self.entity_limit)
            .is_none()
        {
            Err(AdapterError::BudgetExceeded("created entities"))
        } else {
            Ok(())
        }
    }
    pub fn now(&self) -> SimTime {
        self.flow.now()
    }
    pub fn peek_next(&mut self) -> Option<ScheduledEventPreview> {
        self.flow.peek_next()
    }
    pub fn create_service(
        &mut self,
        service: FrozenService,
    ) -> Result<ServiceHandle, AdapterError> {
        service.validate()?;
        if self.counters.services >= self.bounds.services {
            return Err(AdapterError::BudgetExceeded("services"));
        }
        if self.service_identities.contains(&service.identity) {
            return Err(AdapterError::DuplicateIdentity);
        }
        let new_owner = !self.owners.contains_key(&service.identity.patient_id);
        if new_owner && self.counters.owners >= self.bounds.patient_owners {
            return Err(AdapterError::BudgetExceeded("patient owners"));
        }
        self.reserve_entity(u64::from(new_owner) + 1)?;
        let owner = if let Some(owner) = self.owners.get(&service.identity.patient_id) {
            *owner
        } else {
            self.flow.spawn_actor()?
        };
        let work = self.flow.create_work(
            owner,
            SimDuration::from_ticks(service.remaining_ticks as u128),
            SERVICE_CONTEXT,
            service.clone(),
        )?;
        if new_owner {
            self.owners
                .insert(service.identity.patient_id.clone(), owner);
            self.counters.owners += 1;
        }
        self.entity_used += u64::from(new_owner) + 1;
        self.counters.services += 1;
        self.service_identities.insert(service.identity.clone());
        self.services.insert(work, service.clone());
        Ok(ServiceHandle {
            runtime: self.runtime_identity.clone(),
            work,
            identity: service.identity,
        })
    }
    fn validate_admission(&self, admission: &Admission) -> Result<ResourceId, AdapterError> {
        check_id(&admission.resource_id)?;
        if admission.key.ready_tick() as u128 > self.now().ticks() {
            return Err(AdapterError::InvalidInput("ready tick is in the future"));
        }
        admission.key.native()?;
        self.resources
            .get(&admission.resource_id)
            .copied()
            .ok_or(AdapterError::UnknownResource)
    }
    fn reserve_request(&self, can_preempt: bool, manual: bool) -> Result<(), AdapterError> {
        if self.counters.requests >= self.bounds.requests {
            return Err(AdapterError::BudgetExceeded("requests"));
        }
        if can_preempt
            && self.counters.preempt_capable_requests >= self.bounds.preempt_capable_requests
        {
            return Err(AdapterError::BudgetExceeded("preempt-capable requests"));
        }
        self.reserve_entity(1)?;
        self.reserve_event()?;
        if manual && can_preempt {
            return Err(AdapterError::InvalidInput("manual claims cannot preempt"));
        }
        Ok(())
    }
    pub fn admit_service(
        &mut self,
        handle: &ServiceHandle,
        admission: Admission,
    ) -> Result<RequestHandle, AdapterError> {
        if handle.runtime != self.runtime_identity {
            return Err(AdapterError::ForeignHandle);
        }
        let service = self
            .services
            .get(&handle.work)
            .ok_or(AdapterError::ForeignHandle)?;
        if service.identity != handle.identity {
            return Err(AdapterError::ForeignHandle);
        }
        if self.used_services.contains(&handle.work) {
            return Err(AdapterError::WrongHandleRole);
        }
        if self
            .transit
            .as_ref()
            .is_some_and(|state| state.blocks_admission(handle.work))
        {
            return Err(AdapterError::WrongNativeState);
        }
        let resource = self.validate_admission(&admission)?;
        self.reserve_request(admission.can_preempt, false)?;
        let owner = *self
            .owners
            .get(&service.identity.patient_id)
            .ok_or(AdapterError::ForeignHandle)?;
        let now = self.now();
        let mut acquire = self
            .flow
            .acquire(resource)
            .owner(owner)
            .timed_work(handle.work)
            .at(now)
            .priority(!admission.urgency)
            .can_preempt(admission.can_preempt)
            .scheduler_priority(50)
            .queue_order_key(admission.key.native()?);
        if admission.preemptible {
            acquire = acquire.preemptible(PreemptionStrategy::Suspend);
        }
        let request = acquire.submit()?;
        self.used_services.insert(handle.work);
        self.requests.insert(
            request,
            RequestMeta {
                role: RequestRole::Service,
                patient_id: service.identity.patient_id.clone(),
                resource_id: admission.resource_id.clone(),
                work: Some(handle.work),
            },
        );
        self.entity_used += 1;
        self.counters.requests += 1;
        self.counters.preempt_capable_requests += u64::from(admission.can_preempt);
        self.counters.scheduled_adapter_events += 1;
        self.dirty.insert(resource);
        Ok(RequestHandle {
            runtime: self.runtime_identity.clone(),
            request,
            role: RequestRole::Service,
            resource_id: admission.resource_id,
            patient_id: service.identity.patient_id.clone(),
        })
    }
    pub fn admit_manual(
        &mut self,
        patient_id: &str,
        admission: Admission,
    ) -> Result<RequestHandle, AdapterError> {
        check_id(patient_id)?;
        if admission.can_preempt || admission.preemptible {
            return Err(AdapterError::InvalidInput("manual claims cannot preempt"));
        }
        let resource = self.validate_admission(&admission)?;
        self.reserve_request(false, true)?;
        let new_owner = !self.owners.contains_key(patient_id);
        if new_owner && self.counters.owners >= self.bounds.patient_owners {
            return Err(AdapterError::BudgetExceeded("patient owners"));
        }
        self.reserve_entity(u64::from(new_owner) + 1)?;
        let owner = if let Some(id) = self.owners.get(patient_id) {
            *id
        } else {
            self.flow.spawn_actor()?
        };
        let now = self.now();
        let request = self
            .flow
            .acquire(resource)
            .owner(owner)
            .at(now)
            .priority(!admission.urgency)
            .scheduler_priority(50)
            .queue_order_key(admission.key.native()?)
            .submit()?;
        if new_owner {
            self.owners.insert(patient_id.to_owned(), owner);
            self.counters.owners += 1;
        }
        self.entity_used += u64::from(new_owner) + 1;
        self.counters.requests += 1;
        self.counters.scheduled_adapter_events += 1;
        self.dirty.insert(resource);
        self.requests.insert(
            request,
            RequestMeta {
                role: RequestRole::Manual,
                patient_id: patient_id.to_owned(),
                resource_id: admission.resource_id.clone(),
                work: None,
            },
        );
        Ok(RequestHandle {
            runtime: self.runtime_identity.clone(),
            request,
            role: RequestRole::Manual,
            resource_id: admission.resource_id,
            patient_id: patient_id.to_owned(),
        })
    }
    fn validate_handle(
        &self,
        handle: &RequestHandle,
        role: RequestRole,
    ) -> Result<(RequestId, RequestMeta), AdapterError> {
        if handle.runtime != self.runtime_identity {
            return Err(AdapterError::ForeignHandle);
        }
        if handle.role != role {
            return Err(AdapterError::WrongHandleRole);
        }
        let meta = self
            .requests
            .get(&handle.request)
            .ok_or(AdapterError::ForeignHandle)?
            .clone();
        if meta.role != role
            || meta.resource_id != handle.resource_id
            || meta.patient_id != handle.patient_id
        {
            return Err(AdapterError::ForeignHandle);
        }
        Ok((handle.request, meta))
    }
    pub fn release_active(&mut self, handle: &RequestHandle) -> Result<(), AdapterError> {
        let (request, meta) = self.validate_handle(handle, handle.role)?;
        if self.pending_mutations.contains(&request) {
            return Err(AdapterError::WrongNativeState);
        }
        let native = self.flow.request(request)?;
        if native.state != RequestState::Active {
            return Err(AdapterError::WrongNativeState);
        }
        if let Some(work) = meta.work {
            if self.flow.work_progress(work)?.remaining.ticks() == 0 {
                return Err(AdapterError::WrongNativeState);
            }
        }
        let lease = native.lease.ok_or(AdapterError::WrongNativeState)?;
        if self.counters.releases_or_cancels >= self.bounds.releases_or_cancels {
            return Err(AdapterError::BudgetExceeded("releases or cancels"));
        }
        self.reserve_event()?;
        let now = self.now();
        self.flow.release(lease, now)?;
        self.pending_mutations.insert(request);
        self.counters.releases_or_cancels += 1;
        self.counters.scheduled_adapter_events += 1;
        Ok(())
    }
    pub fn cancel_waiting(&mut self, handle: &RequestHandle) -> Result<(), AdapterError> {
        let (request, meta) = self.validate_handle(handle, handle.role)?;
        if self.pending_mutations.contains(&request) {
            return Err(AdapterError::WrongNativeState);
        }
        let native = self.flow.request(request)?;
        if !matches!(native.state, RequestState::Queued | RequestState::Suspended) {
            return Err(AdapterError::WrongNativeState);
        }
        let resource = *self
            .resources
            .get(&meta.resource_id)
            .ok_or(AdapterError::UnknownResource)?;
        if self
            .pending_flushes
            .contains(&(resource, self.now().ticks()))
        {
            return Err(AdapterError::UnclosedGrantPhase);
        }
        if self.counters.releases_or_cancels >= self.bounds.releases_or_cancels {
            return Err(AdapterError::BudgetExceeded("releases or cancels"));
        }
        self.reserve_event()?;
        let now = self.now();
        self.flow.cancel_with_scheduler_priority(request, now, 70)?;
        self.pending_mutations.insert(request);
        self.pending_cancellations.insert(request);
        self.counters.releases_or_cancels += 1;
        self.counters.scheduled_adapter_events += 1;
        Ok(())
    }
    pub fn schedule_marker(
        &mut self,
        operation_id: &str,
        at_tick: u64,
        phase: ParentPhase,
    ) -> Result<EventId, AdapterError> {
        check_id(operation_id)?;
        if operation_id == HORIZON_OPERATION || self.marker_ids.contains(operation_id) {
            return Err(AdapterError::DuplicateOperation);
        }
        if at_tick > self.horizon_tick {
            return Err(AdapterError::InvalidInput("marker exceeds horizon"));
        }
        let at = at_tick as u128;
        if at == self.now().ticks()
            && self
                .phase_cursor
                .get(&at)
                .is_some_and(|cursor| phase.priority() < *cursor)
        {
            return Err(AdapterError::ClosedPhase);
        }
        if self.counters.parent_markers >= self.bounds.parent_markers {
            return Err(AdapterError::BudgetExceeded("parent markers"));
        }
        self.reserve_event()?;
        let work = self.marker_work_id;
        let event = self.flow.schedule_domain(
            work,
            MARKER_KIND,
            SimTime::from_ticks(at),
            phase.priority(),
        )?;
        self.marker_ids.insert(operation_id.to_owned());
        self.events.insert(
            event,
            NativeEvent::Marker {
                operation_id: operation_id.to_owned(),
                phase,
            },
        );
        self.counters.parent_markers += 1;
        self.counters.scheduled_adapter_events += 1;
        Ok(event)
    }
    pub fn flush_dirty(&mut self) -> Result<Vec<EventId>, AdapterError> {
        let resource_names: Vec<String> = self.resources.keys().cloned().collect();
        let resource_ids: Vec<&str> = resource_names.iter().map(String::as_str).collect();
        self.flush_dirty_selected(&resource_ids)
    }
    pub fn flush_dirty_selected(
        &mut self,
        resource_ids: &[&str],
    ) -> Result<Vec<EventId>, AdapterError> {
        let mut selected = BTreeSet::new();
        for resource_id in resource_ids {
            if !self.resources.contains_key(*resource_id) {
                return Err(AdapterError::UnknownResource);
            }
            if !selected.insert(*resource_id) {
                return Err(AdapterError::DuplicateIdentity);
            }
        }
        let at_tick = self.now().ticks();
        if !self.pending_cancellations.is_empty() {
            return Err(AdapterError::UnclosedCancellationBatch);
        }
        let resources: Vec<(&str, ResourceId)> = selected
            .into_iter()
            .filter_map(|name| {
                let resource = *self.resources.get(name)?;
                self.dirty.contains(&resource).then_some((name, resource))
            })
            .filter(|(_, resource)| !self.pending_flushes.contains(&(*resource, at_tick)))
            .collect();
        let count =
            u64::try_from(resources.len()).map_err(|_| AdapterError::BudgetExceeded("flushes"))?;
        if self
            .counters
            .flushes
            .checked_add(count)
            .filter(|x| *x <= self.flush_limit)
            .is_none()
        {
            return Err(AdapterError::BudgetExceeded("flushes"));
        }
        if self
            .flow
            .budget_snapshot()
            .scheduler
            .scheduled_events
            .checked_add(count)
            .filter(|x| *x <= self.event_limit)
            .is_none()
        {
            return Err(AdapterError::BudgetExceeded("scheduled events"));
        }
        for (_, resource) in &resources {
            self.flow.resource(*resource)?;
            if self.flow.resource_allocation_mode(*resource)? != FlowAllocationMode::Deferred {
                return Err(AdapterError::NativeAllocation(
                    ResourceAllocationAdmissionError::NotDeferred,
                ));
            }
            if SimTime::from_ticks(at_tick) < self.flow.now() {
                return Err(AdapterError::Native(FlowError::PastCommand));
            }
        }
        let mut scheduled = Vec::new();
        for (_, resource) in resources {
            let key = (resource, at_tick);
            let event =
                self.flow
                    .schedule_allocation_flush(resource, SimTime::from_ticks(at_tick), 60)?;
            self.pending_flushes.insert(key);
            self.events
                .insert(event, NativeEvent::Flush { resource, at_tick });
            self.counters.scheduled_adapter_events += 1;
            self.counters.flushes += 1;
            scheduled.push(event);
        }
        Ok(scheduled)
    }
    pub fn step(&mut self) -> Result<Option<AdapterDispatch>, AdapterError> {
        let Some(preview) = self.peek_next() else {
            return Ok(None);
        };
        if preview.at.ticks() > self.horizon_tick as u128 {
            return Ok(None);
        }
        let transit_owned = self.transit.as_ref().map_or_else(Vec::new, |transit| {
            transit.owned_carriers(&self.flow, preview.id)
        });
        let native = self
            .flow
            .step()?
            .ok_or(AdapterError::Native(FlowError::InvalidState))?;
        if let Some(error) = native.error {
            return Err(AdapterError::NativeDispatchFailed {
                error,
                dispatch: Box::new(native),
            });
        }
        if self.flow.budget_snapshot().scheduler.scheduled_events > self.event_limit {
            return Err(AdapterError::BudgetExceeded("scheduled events"));
        }
        let at = native.at.ticks();
        let mut observations = Vec::new();
        if let Some(transit) = self.transit.as_mut() {
            observations.extend(transit.observe(&self.flow, &native, &transit_owned)?);
        }
        let event_meta = self.events.get(&native.event).cloned();
        let is_flush = matches!(&event_meta, Some(NativeEvent::Flush { .. }));
        for record in &native.records {
            if let Some(meta) = self.requests.get(&record.request).cloned() {
                if matches!(
                    record.transition,
                    LifecycleTransition::Released | LifecycleTransition::Cancelled
                ) {
                    self.pending_mutations.remove(&record.request);
                    if record.transition == LifecycleTransition::Cancelled {
                        self.pending_cancellations.remove(&record.request);
                    }
                }
                match meta.role {
                    RequestRole::Service => {
                        let work = meta
                            .work
                            .ok_or(AdapterError::Native(FlowError::InvalidWork))?;
                        let frozen = self
                            .services
                            .get(&work)
                            .ok_or(AdapterError::ForeignHandle)?;
                        if let Some(progress) = record.snapshot.progress.as_ref() {
                            let useful = progress.useful_elapsed.ticks();
                            let total = (frozen.useful_before_ticks as u128)
                                .checked_add(useful)
                                .ok_or(AdapterError::BudgetExceeded("useful elapsed"))?;
                            observations.push(FoundationObservation::Service(ServiceObservation {
                                identity: frozen.identity.clone(),
                                request: record.request,
                                work,
                                origin: frozen.origin.clone(),
                                original_ticks: frozen.original_ticks,
                                useful_before_ticks: frozen.useful_before_ticks,
                                useful_elapsed_ticks: useful,
                                total_useful_ticks: total,
                                remaining_ticks: progress.remaining.ticks(),
                                state: progress.state,
                                transition: record.transition,
                                causal_event: record.causal_event_id,
                                ordinal: record.transition_ordinal,
                            }));
                        }
                    }
                    RequestRole::Manual => {
                        observations.push(FoundationObservation::Manual(ManualObservation {
                            patient_id: meta.patient_id,
                            resource_id: meta.resource_id,
                            request: record.request,
                            lease: record.lease,
                            transition: record.transition,
                            causal_event: record.causal_event_id,
                            at: record.at,
                        }))
                    }
                }
                if !is_flush {
                    self.dirty.insert(record.resource);
                }
            }
        }
        match event_meta {
            Some(NativeEvent::Marker {
                operation_id,
                phase,
            }) => {
                self.phase_cursor
                    .entry(at)
                    .and_modify(|p| *p = (*p).max(phase.priority()))
                    .or_insert(phase.priority());
                observations.push(FoundationObservation::Marker {
                    operation_id,
                    event: native.event,
                    at: native.at,
                });
            }
            Some(NativeEvent::Horizon) => {
                self.phase_cursor
                    .entry(at)
                    .and_modify(|p| *p = (*p).max(ParentPhase::Horizon.priority()))
                    .or_insert(ParentPhase::Horizon.priority());
                observations.push(FoundationObservation::Marker {
                    operation_id: HORIZON_OPERATION.to_owned(),
                    event: native.event,
                    at: native.at,
                });
            }
            Some(NativeEvent::Boundary { resource }) => {
                self.dirty.insert(resource);
            }
            Some(NativeEvent::Flush { resource, at_tick }) => {
                self.pending_flushes.remove(&(resource, at_tick));
                self.dirty.remove(&resource);
            }
            None => {}
        }
        self.events.remove(&native.event);
        Ok(Some(AdapterDispatch {
            native,
            epoch: self.flow.budget_snapshot().scheduler.dispatched_events,
            observations,
        }))
    }
    pub fn snapshot(&self) -> FoundationSnapshot {
        let requests = self
            .requests
            .keys()
            .filter_map(|id| {
                self.flow.request(*id).ok().map(|r| RequestSnapshot {
                    request: *id,
                    state: r.state,
                    lease: r.lease,
                    work: r.work,
                })
            })
            .collect();
        let services = self
            .services
            .iter()
            .map(|(work, frozen)| ServiceSnapshot {
                identity: frozen.identity.clone(),
                request: self.flow.work(*work).ok().and_then(|w| w.request),
                progress: self
                    .flow
                    .work_progress(*work)
                    .expect("adapter-owned work remains live"),
            })
            .collect();
        let resources = self
            .resources
            .iter()
            .map(|(name, id)| {
                (
                    name.clone(),
                    self.flow
                        .resource(*id)
                        .expect("adapter-owned resource remains live"),
                )
            })
            .collect();
        FoundationSnapshot {
            now: self.flow.now(),
            horizon_tick: self.horizon_tick,
            requests,
            services,
            resources,
            counters: self.counters,
            native_budget: self.flow.budget_snapshot(),
        }
    }
}

#[cfg(test)]
mod selected_flush_atomicity_tests {
    use super::*;
    use kairo_ecs_des::{
        FlowCallbackCodeV1, FlowCheckpointCodecError, FlowCheckpointCodecs, FlowCheckpointLimits,
    };

    fn test_codecs() -> FlowCheckpointCodecs {
        let mut codecs = FlowCheckpointCodecs::new();
        codecs
            .register_context::<MarkerContext>(
                MARKER_CONTEXT,
                1,
                |_, _| Ok(Vec::new()),
                |bytes, _| {
                    if bytes.is_empty() {
                        Ok(MarkerContext)
                    } else {
                        Err(FlowCheckpointCodecError("invalid marker bytes".into()))
                    }
                },
            )
            .unwrap();
        codecs
            .register_domain_plan_hook_with_receipt::<MarkerContext>(
                MARKER_CONTEXT,
                MARKER_KIND,
                no_command_marker_plan,
                no_command_marker_receipt,
                FlowCallbackCodeV1 {
                    stable_id: "careops-ed.marker.plan".into(),
                    version: 1,
                },
                FlowCallbackCodeV1 {
                    stable_id: "careops-ed.marker.receipt".into(),
                    version: 1,
                },
            )
            .unwrap();
        codecs
    }

    fn native_image(adapter: &EdFlowAdapter) -> String {
        format!(
            "{:?}",
            adapter
                .flow
                .capture_checkpoint_v4(&test_codecs(), FlowCheckpointLimits::default())
                .unwrap()
        )
    }

    fn private_state(adapter: &EdFlowAdapter) -> String {
        format!(
            "{}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
            native_image(adapter),
            adapter.counters,
            adapter.dirty,
            adapter.pending_flushes,
            adapter.events,
            adapter.pending_mutations,
            adapter.pending_cancellations,
        )
    }

    fn dirty_adapter() -> EdFlowAdapter {
        let plan = AdapterPlan::validate(
            vec![
                ResourceSpec {
                    id: "c".into(),
                    capacity: 1,
                },
                ResourceSpec {
                    id: "b".into(),
                    capacity: 1,
                },
                ResourceSpec {
                    id: "a".into(),
                    capacity: 1,
                },
            ],
            vec![],
            4,
            FoundationBounds {
                patient_owners: 3,
                services: 0,
                requests: 3,
                preempt_capable_requests: 0,
                releases_or_cancels: 0,
                parent_markers: 0,
            },
        )
        .unwrap();
        let mut adapter = EdFlowAdapter::new(plan).unwrap();
        for name in ["a", "b", "c"] {
            adapter
                .admit_manual(
                    name,
                    Admission {
                        resource_id: name.into(),
                        key: QueueKey::from_request(0, name.to_owned()).unwrap(),
                        urgency: 0,
                        can_preempt: false,
                        preemptible: false,
                    },
                )
                .unwrap();
        }
        adapter
    }

    #[test]
    fn selected_flush_full_multi_resource_wrapper_parity_and_order() {
        let mut selected = dirty_adapter();
        let mut wrapper = dirty_adapter();
        let selected_ids = selected.flush_dirty_selected(&["c", "b", "a"]).unwrap();
        let wrapper_ids = wrapper.flush_dirty().unwrap();
        assert_eq!(selected_ids, wrapper_ids);
        assert_eq!(private_state(&selected), private_state(&wrapper));
        assert_eq!(selected.snapshot(), wrapper.snapshot());
        assert_eq!(selected.dirty.len(), 3);
        assert_eq!(wrapper.dirty.len(), 3);

        let flush_map = |adapter: &EdFlowAdapter| {
            adapter
                .events
                .iter()
                .filter_map(|(event, meta)| match meta {
                    NativeEvent::Flush { resource, .. } => Some((*event, *resource)),
                    _ => None,
                })
                .collect::<BTreeMap<_, _>>()
        };
        let selected_flushes = flush_map(&selected);
        let wrapper_flushes = flush_map(&wrapper);
        assert_eq!(selected_flushes, wrapper_flushes);
        let names = |adapter: &EdFlowAdapter, flushes: &BTreeMap<EventId, ResourceId>| {
            flushes
                .values()
                .map(|resource| {
                    adapter
                        .resources
                        .iter()
                        .find_map(|(name, id)| (id == resource).then(|| name.clone()))
                        .unwrap()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(names(&selected, &selected_flushes), vec!["a", "b", "c"]);
        assert_eq!(names(&wrapper, &wrapper_flushes), vec!["a", "b", "c"]);
        let selected_expected = selected.requests.keys().copied().collect::<BTreeSet<_>>();
        let wrapper_expected = wrapper.requests.keys().copied().collect::<BTreeSet<_>>();
        assert_eq!(selected_expected, wrapper_expected);

        let run_to_flushes =
            |adapter: &mut EdFlowAdapter, flushes: &BTreeMap<EventId, ResourceId>| {
                let mut dispatches = Vec::new();
                let mut flush_order = Vec::new();
                let mut grants = BTreeSet::new();
                while !adapter.pending_flushes.is_empty() {
                    let preview = adapter.peek_next().unwrap();
                    let dirty_before = adapter.dirty.clone();
                    let flush_resource = flushes.get(&preview.id).copied();
                    let dispatch = adapter.step().unwrap().unwrap();
                    assert_eq!(dispatch.native.event, preview.id);
                    if let Some(resource) = flush_resource {
                        assert!(dirty_before.contains(&resource));
                        let mut expected_dirty = dirty_before;
                        expected_dirty.remove(&resource);
                        assert_eq!(adapter.dirty, expected_dirty);
                        flush_order.push(resource);
                    } else {
                        assert_eq!(adapter.dirty, dirty_before);
                    }
                    grants.extend(
                        dispatch
                            .native
                            .records
                            .iter()
                            .filter(|record| record.transition == LifecycleTransition::Granted)
                            .map(|record| record.request),
                    );
                    dispatches.push(dispatch);
                }
                (dispatches, flush_order, grants)
            };
        let (selected_dispatches, selected_order, selected_grants) =
            run_to_flushes(&mut selected, &selected_flushes);
        let (wrapper_dispatches, wrapper_order, wrapper_grants) =
            run_to_flushes(&mut wrapper, &wrapper_flushes);
        assert_eq!(
            selected_order,
            selected_flushes.values().copied().collect::<Vec<_>>()
        );
        assert_eq!(
            wrapper_order,
            wrapper_flushes.values().copied().collect::<Vec<_>>()
        );
        let resource_name = |adapter: &EdFlowAdapter, resource: ResourceId| {
            adapter
                .resources
                .iter()
                .find_map(|(name, id)| (id == &resource).then(|| name.clone()))
                .unwrap()
        };
        assert_eq!(
            selected_order
                .iter()
                .map(|resource| resource_name(&selected, *resource))
                .collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
        assert_eq!(
            wrapper_order
                .iter()
                .map(|resource| resource_name(&wrapper, *resource))
                .collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
        assert_eq!(selected_dispatches, wrapper_dispatches);
        assert_eq!(selected_grants, selected_expected);
        assert_eq!(wrapper_grants, wrapper_expected);
        assert_eq!(private_state(&selected), private_state(&wrapper));
        assert_eq!(selected.snapshot(), wrapper.snapshot());
        assert_eq!(selected.counters, wrapper.counters);
    }

    #[test]
    fn selected_flush_pending_cancellation_global_guard_is_atomic() {
        let plan = AdapterPlan::validate(
            vec![
                ResourceSpec {
                    id: "c".into(),
                    capacity: 1,
                },
                ResourceSpec {
                    id: "b".into(),
                    capacity: 1,
                },
                ResourceSpec {
                    id: "a".into(),
                    capacity: 1,
                },
            ],
            vec![],
            4,
            FoundationBounds {
                patient_owners: 2,
                services: 0,
                requests: 2,
                preempt_capable_requests: 0,
                releases_or_cancels: 1,
                parent_markers: 0,
            },
        )
        .unwrap();
        let mut adapter = EdFlowAdapter::new(plan).unwrap();
        let cancelled = adapter
            .admit_manual(
                "a",
                Admission {
                    resource_id: "a".into(),
                    key: QueueKey::from_request(0, "a".into()).unwrap(),
                    urgency: 0,
                    can_preempt: false,
                    preemptible: false,
                },
            )
            .unwrap();
        let other = adapter
            .admit_manual(
                "b",
                Admission {
                    resource_id: "b".into(),
                    key: QueueKey::from_request(0, "b".into()).unwrap(),
                    urgency: 0,
                    can_preempt: false,
                    preemptible: false,
                },
            )
            .unwrap();
        let expected_queued = [cancelled.request_id(), other.request_id()]
            .into_iter()
            .collect::<BTreeSet<_>>();
        let mut queued = BTreeSet::new();
        let mut submit_dispatches = Vec::new();
        while queued != expected_queued {
            let dispatch = adapter.step().unwrap().unwrap();
            for record in &dispatch.native.records {
                if expected_queued.contains(&record.request)
                    && record.transition == LifecycleTransition::Queued
                {
                    queued.insert(record.request);
                }
                assert!(
                    !expected_queued.contains(&record.request)
                        || record.transition != LifecycleTransition::Granted
                );
            }
            submit_dispatches.push(dispatch);
        }
        assert_eq!(queued, expected_queued);
        for request in &expected_queued {
            assert_eq!(
                adapter
                    .snapshot()
                    .requests
                    .iter()
                    .find(|snapshot| snapshot.request == *request)
                    .unwrap()
                    .state,
                RequestState::Queued
            );
        }
        assert!(submit_dispatches.iter().all(|dispatch| dispatch
            .native
            .records
            .iter()
            .all(|record| !expected_queued.contains(&record.request)
                || record.transition != LifecycleTransition::Granted)));
        assert_eq!(adapter.counters.flushes, 0);
        assert!(adapter
            .events
            .values()
            .all(|event| !matches!(event, NativeEvent::Flush { .. })));
        adapter.cancel_waiting(&cancelled).unwrap();
        let pending = private_state(&adapter);
        for selection in [&[][..], &["c"][..], &["b"][..]] {
            assert!(matches!(
                adapter.flush_dirty_selected(selection),
                Err(AdapterError::UnclosedCancellationBatch)
            ));
            assert_eq!(private_state(&adapter), pending);
        }
        loop {
            let dispatch = adapter.step().unwrap().unwrap();
            if dispatch.native.records.iter().any(|record| {
                record.request == cancelled.request_id()
                    && record.transition == LifecycleTransition::Cancelled
            }) {
                break;
            }
        }
        let consumed = private_state(&adapter);
        assert!(adapter.flush_dirty_selected(&[]).unwrap().is_empty());
        assert!(adapter.flush_dirty_selected(&["c"]).unwrap().is_empty());
        assert_eq!(private_state(&adapter), consumed);

        let before = adapter.snapshot();
        let scheduled = adapter.flush_dirty_selected(&["b"]).unwrap();
        assert_eq!(scheduled.len(), 1);
        let after = adapter.snapshot();
        assert_eq!(
            after.native_budget.scheduler.scheduled_events,
            before.native_budget.scheduler.scheduled_events + 1
        );
        assert_eq!(after.counters.flushes, before.counters.flushes + 1);
        assert_eq!(
            after.counters.scheduled_adapter_events,
            before.counters.scheduled_adapter_events + 1
        );
    }

    #[test]
    fn selected_flush_selection_atomicity() {
        let mut adapter = dirty_adapter();
        let before = private_state(&adapter);
        assert!(matches!(
            adapter.flush_dirty_selected(&["a", "b", "unknown"]),
            Err(AdapterError::UnknownResource)
        ));
        assert_eq!(private_state(&adapter), before);
        assert!(matches!(
            adapter.flush_dirty_selected(&["a", "b", "a"]),
            Err(AdapterError::DuplicateIdentity)
        ));
        assert_eq!(private_state(&adapter), before);

        let mut flush_budget = dirty_adapter();
        assert_eq!(flush_budget.flush_dirty_selected(&["a"]).unwrap().len(), 1);
        flush_budget.flush_limit = flush_budget.counters.flushes + 1;
        let before = private_state(&flush_budget);
        assert!(matches!(
            flush_budget.flush_dirty_selected(&["b", "c"]),
            Err(AdapterError::BudgetExceeded("flushes"))
        ));
        assert_eq!(private_state(&flush_budget), before);

        let mut event_budget = dirty_adapter();
        assert_eq!(event_budget.flush_dirty_selected(&["a"]).unwrap().len(), 1);
        event_budget.event_limit = event_budget
            .flow
            .budget_snapshot()
            .scheduler
            .scheduled_events
            + 1;
        let before = private_state(&event_budget);
        assert!(matches!(
            event_budget.flush_dirty_selected(&["b", "c"]),
            Err(AdapterError::BudgetExceeded("scheduled events"))
        ));
        assert_eq!(private_state(&event_budget), before);
    }
}
