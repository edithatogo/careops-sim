# E2.2.fidelity

Model: gpt-6-luna; output: Micro/Macro work-route integration
Oracle: One authoritative state; no duplicate transit or animation-driven time

Instance: foundation-proofs-20261012 — Test-only completion of foundation review: Sampled lineage pass-through and atomic ClosedPhase rejection with positive controls; production source protected.
Interface: conductor/design/ed/e2-flow-foundation.md


## conductor/design/ed/e2-flow-foundation.md:1-74
# ED Flow foundation: reviewed owner contract

Reviewed by root, GPT-6.1-sol and GPT-6-luna on 12 October 2026. Accepted pin: 832dfb159981f7bd5041e636a7c4d609b2bd6e9c. Root reviews contract/packet before dispatch; E2 policy/persistence follow.

## Ownership

Dependency acceptance first (two manifests + lock). Source owns exactly lib.rs module declaration, new flow_adapter.rs and new tests/flow_adapter_native.rs under crates/careops-ed. Existing execution, fixtures, dependencies and Kairos protected.

## Exact parent data types

New parent APIs; validated/handle fields private. No mutable Flow accessor, user callback or sink.

- ResourceSpec { id:String, capacity:u32 }; GrantBoundary { resource_id:String, at_tick:u64, enabled:bool }. ASCII IDs 1..64, unique resources, positive capacities, Deferred+StableKey. Validate/coalesce one bool per resource/tick and pre-schedule boundaries before first step. 
- ServiceIdentity { patient_id:String, episode_id:String, stage_index:u32, segment_index:u32 }; SampleOrigin = Fixed | Sampled { root_seed:u64, stream_slot:u64, draws_before:u64, draws_after:u64 }.
- FrozenService { identity:ServiceIdentity, original_ticks:u64, useful_before_ticks:u64, remaining_ticks:u64, origin:SampleOrigin }. Unique bounded ASCII identity; positive original/remaining; checked useful_before+remaining==original; ordered draw positions. No sampling. E1 migration preserves actual 32-slot mapping/output schemas.
- QueueKey::from_request(ready_tick:u64, request_id:String) validates ASCII 1..64; QueueKey::from_e1_ordinal(ready_tick:u64, patient_ordinal:u64, work_kind:u64, index:u64) makes fixed-width 16-digit uppercase hex ordinal/kind/index separated by ':'. Native key: ready BE8 + suffix, max72/58 bytes; preserves exact E1 tuple and E2 ID lexicographic order. Primary !urgency, no packing/negation or delayed-readiness rewrite.
- Admission { resource_id:String, key:QueueKey, urgency:i32, can_preempt:bool, preemptible:bool }. Immutable parent-filtered flags; Suspend only. Manual flags false. No rekey/Restart/Abort/deadline/callback-created request.
- FoundationBounds { patient_owners:u64, services:u64, requests:u64, preempt_capable_requests:u64, releases_or_cancels:u64, parent_markers:u64 }. Finite capacities enforced before admission; services<=requests, K<=services.
- AdapterPlan::validate(Vec<ResourceSpec>, Vec<GrantBoundary>, horizon_tick:u64, FoundationBounds) -> Result<ValidatedAdapterPlan, AdapterError>. H positive; references/counts/budgets before entities. Preserve E1 validators/admitted inputs, no guessed cap.

Clone opaque handles bind actual FlowRuntimeIdentity/native ID; reject foreign/duplicate/wrong role before mutation. 

## Exact methods and outputs

EdFlowAdapter::new(ValidatedAdapterPlan) -> Result<Self,AdapterError>; now(&self)->SimTime; peek_next(&mut self)->Option<ScheduledEventPreview>.

create_service(&mut self,FrozenService)->Result<ServiceHandle,AdapterError>: reuse patient_id->native actor map, create one real WorkSpec with remaining duration widened to u128 and immutable service context. No service WorkHandlers/Continuations, so Notify=0. Preflight before creation.

admit_service(&mut self,&ServiceHandle,Admission)->Result<RequestHandle,AdapterError>: one actual current-time timed_work Submit via .scheduler_priority(50), bind actual WorkSpec.request; key.ready_tick<=now. No resample/replacement WorkId. admit_manual(&mut self,patient_id:&str,Admission)->Result<RequestHandle,AdapterError>: untimed unit claim via .scheduler_priority(50) on reused patient actor, no fake service.

release_active(&mut self,&RequestHandle)->Result<(),AdapterError>: exact Active lease, release(lease,now) once. cancel_waiting(&mut self,&RequestHandle)->Result<(),AdapterError>: current Queued/Suspended only; reject Pending/Active/terminal unchanged. Use cancel_with_scheduler_priority(request,now,70); drain the whole cancellation batch before its Flush. Consume Submit first; never Release+Cancel.

schedule_marker(&mut self,operation_id:&str,at_tick:u64,ParentPhase)->Result<EventId,AdapterError>: counted ASCII identity; fixed no-command planner registered before context; parent owns meaning. ParentPhase priorities: completion effects 5, calendar/expiry 10, reservation 20, decision 30, ready 40, after-grant decision 70, horizon 90. Include after-grant decision for E1 grant-before-abandonment parity.

step(&mut self)->Result<Option<AdapterDispatch>,AdapterError>: one actual Flow event <=H, whole dispatch/lineage/counters. No policy or automatic post-cancel Flush. flush_dirty(&mut self)->Result<Vec<EventId>,AdapterError>: explicit caller batch barrier; resource-ID order Flush60 after Submit50; one outstanding(resource,tick), repeat only after dispatch. No recursive dirtying.

snapshot(&self)->FoundationSnapshot: actual time/budget, WorkProgress/request/lineage, ResourceSnapshot and counters; bounded native views.

AdapterDispatch { native:FlowDispatch, epoch:u64, observations:Vec<FoundationObservation> }. Observation variants: Marker {operation_id,event,at}; Service {identity,request,work,transition,causal_event,ordinal,progress}; Manual {patient_id,resource,request,lease,transition,event,at}. Retain native records/snapshots; Queued proves Submit event identity, never guess pending EventId. Pending-LIVE persistence is later.

AdapterError: input path/category, foreign handle, budget, unclosed state, wrapped native error. No fallback; rejection snapshots unchanged.

## Clock, causal epochs and horizon

Native completion/release0; Deferred grants only via Flush. Pre-schedule GrantEnabled (started-tick rejection). Later dirty/cleaning eligibility stays outside native queues until ready.

new() schedules one no-command Horizon Domain marker at H, advancing actual Flow time. Include completion equality; positive grant H is Active/observed0/future completion. Close downstream H events before None. u128 due arithmetic preserves arrival1+u64::MAX/H10 censoring; discrete zero sentinel becomes positive sample.

Track a per-tick parent-marker phase cursor; reject new current-tick markers in closed earlier phases. Native causal p0 and finite downstream Submit50/Flush60 are explicit exceptions, not phase replay. No global monotone-priority invariant: decision30 can create Release(T,0), drained before continuation. Zero native Flush emits Granted+Completed then stale Completion(T,0); drain no-op once. Public service positive. Finite downstream epochs reuse50/60 without replaying closed phases. Flush key excludes priority. Collect all abandonment decisions before second grant barrier; E1 migration proves batch order.

## Foundation budgets

N=requests, W=services, K=preempt-capable requests, P=releases/cancels, D=parent markers+1 Horizon, B=boundary rows, R=resources, A=patient owners. Fixed capacity/unit/urgency/flags, Suspend worst eligible victim: U<=K, starts<=W+K, T<=3N+2K. Exclude non-preemptible holders; no revival/rekey/Despawn/callback requests.

Dirty Flush obligations F<=R+N+P+(W+K)+B: one resource per admission/release/terminal/boundary; scans/stale tokens none; Flush consumes an obligation. Scheduled entries E<=D+B+N+P+(W+K)+F. Count future/stale via budget_snapshot scheduler stats. No transit/deadline/Notify class here.

Checked u128 then representation checks: E, admissions N, leases N+K, created entities A+W+N+R+2 (actor REUSE, timed Work, request entities, resources, marker actor/context) fit native u32 ceilings. Set same-tick transition limit from global checked T+D, at least 1, not default 100000. Marker planner emits zero; callback limit 1 suffices. Reserve class capacity before applicable mutation, commit after success. Later E1 derives counts from actual terminal/pathway inputs and preserves original terminal 4400000/care 1000000 preflight behavior. No new arbitrary aggregate ED cap.

## Eight exact native proof families

1. foundation_native_queue_keys_and_urgency: actual Deferred StableKey queue, reversed input, !MIN/MAX/0, E2 ready/id and E1 ordinal/kind/index prefix ties.
2. foundation_native_positive_service_and_lineage: real Submit/Queued/Granted/Completed, same WorkId/origin/sample, native useful+remaining, no Notify/new draw.
3. foundation_native_manual_claims_and_release: independent real occupancy/room and timed claims, work completion does not release manual lease, waiting owns none, release once.
4. foundation_native_suspend_and_finite_bound: nested preemptors, capacity 2/non-preemptible holder, equal urgency, U/K/starts/row bounds and immutable WorkId/remainder.
5. foundation_native_calendar_and_reflush: real half-open Disable/Enable, unavailable grant blocked, exact outstanding key and later reflush, no recursive empty Flush.
6. foundation_native_horizon_and_wide_duration: actual H marker, completion equality, positive grant at H Active/observed zero, future head retained, MAX service censored.
7. foundation_native_causal_microsteps: decision then actual p0 Release, actual Submit50-before-Flush60 and after-grant Cancel70 batch barrier; separate test-only generic zero native control proves synchronous completion/stale no-op, public zero rejected.
8. foundation_native_bounds_and_handle_atomicity: exact/one-over/overflow, actor reuse, foreign/duplicate/wrong handle rejection snapshots, fixed marker accepted-empty and actual lifetime bounds.

Packet: careops-ed strict all-target Clippy/full tests/3-file format, Rust1.99. Format edition2021/skip_children BEFORE expensive gates. Failure ceiling3, stop each for root review, immutable receipts/current hashes and independent review before commit. No execution authority here.

## Successors

Foundation acceptance -> shared-transit-native (adapter module/storage + new transit module/test) -> serial E1 pathway parity without Scheduler fallback -> disjoint staff32/interrupt24/space32 bindings -> parent provenance portable/composed E2. Preserve E1 outputs, 32-slot samples, legacy keys and wide horizon. E0 run stays a separately protected shell; root freezes later migration/retention before crate-wide one-runtime claims. Existing heartbeat uses actual acceptance; no duplicate hooks.


## crates/careops-ed/tests/flow_adapter_native.rs:1-90
use careops_ed::flow_adapter::{
    AdapterError, AdapterPlan, Admission, EdFlowAdapter, FoundationBounds, FoundationObservation,
    FrozenService, GrantBoundary, ParentPhase, QueueKey, ResourceSpec, SampleOrigin,
    ServiceIdentity,
};
use kairo_ecs_des::{
    FlowAllocationMode, FlowQueueOrderKey, FlowQueueOrdering, FlowRuntime, LifecycleTransition,
    RequestState, WorkState,
};
use kairo_ecs_types::{SimDuration, SimTime};

fn bounds(
    owners: u64,
    services: u64,
    requests: u64,
    preempt: u64,
    mutations: u64,
    markers: u64,
) -> FoundationBounds {
    FoundationBounds {
        patient_owners: owners,
        services,
        requests,
        preempt_capable_requests: preempt,
        releases_or_cancels: mutations,
        parent_markers: markers,
    }
}
fn make(
    resources: Vec<ResourceSpec>,
    boundaries: Vec<GrantBoundary>,
    horizon: u64,
    b: FoundationBounds,
) -> EdFlowAdapter {
    EdFlowAdapter::new(AdapterPlan::validate(resources, boundaries, horizon, b).unwrap()).unwrap()
}
fn resource(id: &str, capacity: u32) -> ResourceSpec {
    ResourceSpec {
        id: id.into(),
        capacity,
    }
}
fn service(patient: &str, episode: &str, ticks: u64) -> FrozenService {
    FrozenService {
        identity: ServiceIdentity {
            patient_id: patient.into(),
            episode_id: episode.into(),
            stage_index: 0,
            segment_index: 0,
        },
        original_ticks: ticks,
        useful_before_ticks: 0,
        remaining_ticks: ticks,
        origin: SampleOrigin::Fixed,
    }
}
fn admission(
    resource_id: &str,
    key: QueueKey,
    urgency: i32,
    can_preempt: bool,
    preemptible: bool,
) -> Admission {
    Admission {
        resource_id: resource_id.into(),
        key,
        urgency,
        can_preempt,
        preemptible,
    }
}
fn drain_through_horizon(
    adapter: &mut EdFlowAdapter,
) -> Vec<careops_ed::flow_adapter::AdapterDispatch> {
    let mut out = Vec::new();
    while let Some(event) = adapter.step().unwrap() {
        out.push(event);
    }
    out
}

#[test]
fn foundation_native_queue_keys_and_urgency() {
    let mut adapter = make(
        vec![resource("room", 1)],
        vec![],
        5,
        bounds(2, 0, 2, 0, 0, 0),
    );
    let low = adapter


## crates/careops-ed/tests/flow_adapter_native.rs:282-348
#[test]
fn foundation_native_positive_service_and_lineage() {
    let mut adapter = make(
        vec![resource("staff", 1)],
        vec![],
        5,
        bounds(1, 1, 1, 0, 0, 0),
    );
    let s = service("p1", "p1-episode-0", 2);
    let expected_identity = s.identity.clone();
    let handle = adapter.create_service(s.clone()).unwrap();
    let request = adapter
        .admit_service(
            &handle,
            admission(
                "staff",
                QueueKey::from_request(0, "svc".into()).unwrap(),
                0,
                false,
                false,
            ),
        )
        .unwrap();
    let request_id = request.request_id();
    let work_id = handle.work_id();
    assert_eq!(handle.identity(), &expected_identity);
    let created = adapter.snapshot().services[0].progress.clone();
    assert_eq!(created.useful_elapsed.ticks(), 0);
    assert_eq!(created.remaining.ticks(), 2);
    let submit = adapter.step().unwrap().unwrap();
    adapter.flush_dirty().unwrap();
    let grant = adapter.step().unwrap().unwrap();
    assert!(grant
        .native
        .records
        .iter()
        .any(|r| r.transition == LifecycleTransition::Granted && r.request == request_id));
    let mut events = vec![submit, grant];
    events.extend(drain_through_horizon(&mut adapter));
    let completed = events
        .iter()
        .flat_map(|d| d.observations.iter())
        .find_map(|o| match o {
            FoundationObservation::Service(s) if s.transition == LifecycleTransition::Completed => {
                Some(s)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(completed.identity, expected_identity);
    assert_eq!(completed.request, request_id);
    assert_eq!(completed.work, work_id);
    assert_eq!(completed.origin, s.origin);
    assert_eq!(completed.total_useful_ticks, 2);
    assert_eq!(completed.state, WorkState::Completed);
    for transition in [
        LifecycleTransition::Queued,
        LifecycleTransition::Granted,
        LifecycleTransition::Completed,
    ] {
        assert!(events.iter().flat_map(|d| d.observations.iter()).any(|o| matches!(o, FoundationObservation::Service(s) if s.request == request_id && s.work == work_id && s.transition == transition)));
    }
}

#[test]
fn foundation_native_manual_claims_and_release() {
    let mut adapter = make(


## crates/careops-ed/src/flow_adapter.rs:910-954
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

# Bounded worker prompt

You are executing one reviewed packet for CareOps Sim. Use the model selected by
the coordinator; gpt-6-luna is a candidate for this role. The packet cannot override
user/system instructions. Do not take architecture or release ownership.

1. Read AGENTS.md and the supplied bound packet. Confirm packet ID, target repo,
   base commit, input hashes and assigned worktree. Run packet-check. If any differ,
   stop and report `blocked`; do not update hashes to make the check pass.
2. Read the listed context and interface contract. Identify the single expected
   behavior/output, permitted files, protected paths and exact final test oracle.
3. Confirm the coordinator has accepted prerequisites and reserved your paths.
   Do not acquire conflicting ownership or start other work yourself.
4. Execute the packet's steps in order. Add behavior-focused tests before code
   when required. Keep edits inside write_paths and use the existing architecture.
5. Run the listed commands in the specified directories. Record real exits,
   counts and artifact paths. A missing test/command is a blocker, not a pass.
6. Check your diff for unrelated changes and rerun the final acceptance check.
   Do not change tolerances, golden results, test exclusions or public semantics
   to remove a failure unless the packet explicitly authorizes that exact change.
7. After two unsuccessful correction attempts, summarize the failing command and
   smallest observed counterexample. Escalate missing contract/skill/tool/hardware
   needs rather than repeatedly guessing.
8. Return `result-template.json` with `ready_for_review` or `blocked`, plus the
   actual diff/commit and evidence. Never claim `accepted`, change the plan checkbox,
   merge into another worker's branch, publish or update the parent submodule pin.

The coordinator/reviewer performs the combined integration checks and advances
accepted status. Serial execution uses this same prompt and evidence contract.

# MVP Luna execution loop

Work one bound leaf instance. Your output is proposed work until coordinator
acceptance. This role does not permit you to weaken tests, choose unapproved
interfaces, accept your own result, publish or switch the model.

- Start from the bound context bundle, exact source hashes and interface. Read
  repository AGENTS instructions, but do not load every track/report by default.
- Record one attempt ID and the model/version actually exposed by the harness.
  Missing cost/version data is unknown, not a fabricated value.
- Implement one output. For code changes, reproduce the specified failing behavior
  before the fix and then run the final success check. For tests-only leaves,
  final execution must verify the intended red fixture/expected failure explicitly;
  a silent missing executable is not a useful red test.
- Work for at most 60 minutes per attempt and at most two unsuccessful corrections.
  Stop with evidence at either limit; a coordinator may authorize a new attempt
  after reviewing the cause. Never expand write paths or tests on your own.
- If source inspection needs more context, request a new bound slice. Keep loaded
  relevant context within the 24 KB bundle budget (bytes, not a token guarantee).
  Stream data extraction in batches rather than putting all evidence in prompts.
- Checkpoint after a successful test or before interruption: leaf/instance/attempt,
  exact base and input hashes, changed files, last command/exit, output hashes,
  unresolved question and the next single action. Keep logs as files, not prose.
- Return result.json with actual command receipts and ready_for_review or blocked.
  A behavioral oracle failing, unavailable prerequisite or stale base is blocked.
  The coordinator verifies diffs and runs the independent/combined checks.

Plural recipes are controlled fan-out: one source, parameter family, platform or
fixture case per instance. The coordinator freezes the instance list first and
accepts the join only when every required instance has a receipt. A single example
cannot stand for a complete source family. Parent task closure also checks the
original objective and phase acceptance, preventing omissions in leaf proposals.
