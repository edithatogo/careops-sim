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
