# Same-Flow transit addition: bounded owner candidate

Root-reviewed implementation contract, 12 October 2026. Public Kairos Handoff/TransitV2 APIs are accepted at the parent pin832dfb159981f7bd5041e636a7c4d609b2bd6e9c. Foundation is still implementing: names below are proposed parent additions, not claims about its current source. Bind its ACTUAL accepted interfaces/hash before committing this contract or dispatching source. Root alone dispatches; existing heartbeat continues.

## Narrow scope and dependencies

After foundation source/gates acceptance, source ownership is at most three paths: flow_adapter.rs private module/state/builder integration, new flow_adapter/transit.rs, new tests/flow_adapter_transit_native.rs under crates/careops-ed. Protect foundation tests, lib.rs, dependencies, pathway/terminal/policies, Kairos and fixtures. Adding a child module lets it use parent-private native runtime/handles without exposing mutable Flow. No second runtime, sampling, engine API or user callbacks.

Preserve foundation EdFlowAdapter::new(plan) and FoundationBounds layouts/callers. Proposed alternate new_with_transit(plan, TransitBounds) constructs the SAME runtime with extended preflight/config BEFORE entities/steps; internal common builder may accept optional validated transit capacity. Do not create a foundation runtime, clone/rebuild it, or change budgets after the clock starts. If the accepted foundation cannot support this private builder seam within owned file, review the exact necessary path before dispatch.

## Proposed types and methods

TransitBounds { transits:u64, total_route_segments:u64, controls:u64, native_snapshot_limits:TransitContextCheckpointLimitsV1 }. Validate finite checked caps with the base plan. Native snapshot limits are native estimates, not wire budgets. There is NO W+512 translation or wire API in this packet.

TransitInput { route_id:String, graph:Arc<TransitGraphV1>, route:RoutePlan, start_tick:u64 }. Route ID ASCII1..64; exact graph.canonical_bytes_ref()==route.graph_canonical_bytes(). Existing route/graph immutable public APIs provide physical state. Reusing route_id with differing canonical graph rejects; store caller-trusted graph for later persistence rather than reconstructing trust from payload bytes. Check actual route.segments().len() against cumulative declared segment capacity before native entity creation. Native checkpoint_v2 preflight must fit declared route/graph/mode/total limits before publication; retain unchanged caller route/sample lineage.

Opaque TransitHandle carries private actual runtime identity, service WorkId and installed carrier WorkId. Methods:

- new_with_transit(ValidatedAdapterPlan, TransitBounds)->Result<EdFlowAdapter,AdapterError>.
- begin_transit(&mut self,&ServiceHandle,TransitInput)->Result<TransitHandle,AdapterError>.
- schedule_transit_control(&mut self,&TransitHandle,FlowDomainControl,at_tick:u64)->Result<EventId,AdapterError>.
- transit_snapshot(&self,&TransitHandle)->Result<TransitContextCheckpointV2,AdapterError>.
- transit_arrival(&self,&TransitHandle)->Result<Option<TransitArrivalHandoff>,AdapterError>.

Service without transit remains directly eligible. begin_transit rejects wrong identity/role, bound/non-Pending service, duplicate carrier and exhausted class caps BEFORE mutation. It uses public TransitContext::new_handoff on the existing service, fresh separate carrier actor, and create_actor_domain_context; no resource/dummy Acquire. Register fixed Transit planner through register_transit_context BEFORE any TransitContext work exists, under a fixed private label/kind. schedule_transit_start binds actual event ID into context; parent retains that returned ID. Use start/progress priority40 and controls30; source service is never consumed by transit.

The existing admit_service must reject transit-associated service until an actual accepted arrival is observed by adapter step. Then it calls the SAME foundation admission path; no new WorkId, sample, native resource or clock. Both native arrival observers return None after binding. Manual space/room claims are independent; this packet does not choose staff, release occupancy on arrival, or execute space/diagnostic policy.

## Actual dispatch and observation

Adapter step calls the public typed observers on the ACTUAL returned native FlowDispatch and installed carrier. Related event ownership starts from schedule_transit_start/control IDs and accepted callback admissions, not guessed allocator positions. Start/Progress/advancing Resume admission has requestNone and one next Domain event; paused/no-change control may have accepted-empty admissions. Match native context expected event/due/ticket with that actual receipt. Domain or frontier Resume arrival is committed with accepted-empty commands and an actual TransitArrivalHandoff receipt; record exact event/time/cause from owned start/control and native DTO witness, not an invented dispatch cause field.

No public method accepts a user-supplied FlowDispatch to manufacture observation. Error/rejected/unrelated events cannot set arrival. Duplicate observation of the SAME actual event is idempotent; native stale Domain after Resume remains observed as a no-op with scheduler/budget/batch accounting allowed. Preserve actual FlowDispatch records/callback batches and source/restored-neutral lineage; do not pretend issued IDs authenticate historical provenance.

Foundation observation adds Transit { service identity, carrier, event, at, phase, arrival:Option<TransitArrivalHandoff> }. Keep current foundation public constructors/fields compatible. Bind integration shapes to accepted foundation; no guessed fields. transit_snapshot is native structural inspection only, not checkpoint roundtrip or parent portable acceptance.

## Clock and budgets

Use the SAME Flow clock/Horizon marker. Transit while Pending has no service progress or request. An empty/zero-distance route can arrive in accepted Start at the current tick; clinical service remains positive. Handover/preemption of service does not restart/recreate its completed transit or reset logical QueueKey ready_tick. Transit elapsed/paused time remains actual RoutePlan/cursor state, not a second simulated timer.

Allow finite downstream Start40/control30 as counted native operations even after a later closure epoch, without replaying closed parent marker phases. Known native calendar boundaries remain pre-scheduled. At H consume all due transit/control/arrival effects and any downstream positive Submit50/Flush60; future transit events remain pending. Positive service granted at H is Active/observed0. A Resume at a completed arrival frontier can beat old LIVE Domain and commit arrival; the stale Domain then changes only consumed event/command plus documented scheduler/budget/next-batch counters, never service/route/lineage.

Let M=transits, L=sum admitted route segments, C=controls. Additional scheduled entries <=M+L+2C: Domain <=M+L+C plus C control events. Each Resume can arm at most one replacement Domain; stale originals still count. Additional created entities <=2M (fresh carrier actor+context Work). Add callback delivery budget M+L+2C to base global same-tick bound. No service Notify, no callback-created Submit. Fixed accepted Handoff planner emits at most one command, so foundation callback max1 remains sufficient. No arbitrary total-event multiplier/cap; combine checked totals before one Flow construction and enforce actual class counters before operations.

Only ServiceHandle binding/context origin is used; no new sampling calls or patient actor-map entries for carriers. Reject an exhausted route/control class before scheduling and preserve snapshots on invalid handles/action/times. Full parent provenance persistence later binds actual pending LIVE Submit and restored context/owner/route/sample joins; no such authenticity claim here.

## Seven exact native proof families

1. adapter_transit_actual_moving_and_service_unconsumed: real same-Flow two-edge route, actual Start/Progress, service WorkId/original sample/progress unchanged until arrival; no dummy resource.
2. adapter_transit_pause_retained_and_consumed_resume: actual Pause before original event, retained and actually consumed cuts, Resume suffix/elapsed parity, no paused service work or extra draw.
3. adapter_transit_domain_and_frontier_resume_arrival: both real Domain and Resume accepted-empty arrival; old LIVE Domain stale no-op with exact permitted accounting delta/full remaining native state equality.
4. adapter_transit_admits_once_and_suppresses: arrival-unadmitted Pending, real same-service Submit/Queued/Granted/Completed, both observers suppress; reject duplicate admission/transit without mutation.
5. adapter_transit_identity_and_owner_controls: foreign handles, wrong/bound/terminal work and actor-domain mismatch controls versus positive baseline; rejected/error/unrelated dispatch cannot create observed arrival.
6. adapter_transit_horizon_and_zero_route: actual route equality at H, pending future progress, positive service grant H Active/observed0, zero-distance arrival then finite closure, no phase replay.
7. adapter_transit_class_limits_and_callback_budget: exact/one-over M/L/C/native route limits, checked combined budget/entity overflow before mutation; actual planner <=1 command, no service Notify/new sample.

Bound source-stage full careops-ed tests, strict all-target Clippy and owned three-file formatting at actual accepted foundation base, Rust1.99. Format exact2021/skip_children first. Development ceiling3, stop each failure for root review, immutable receipt/source evidence and independent review before commit. 

## Successor trigger

Foundation acceptance opens reviewed transit packet preparation; its native acceptance plus E1 foundation migration enables composed Macro/Micro domain closure. E1 migration can proceed on pathway/parity paths while read-only transit design is reviewed, but any shared adapter writer is serial. Staff32/interrupt24/space32 binding writers begin only after the common seam/policy accepted interfaces are frozen. Parent provenance persistence/fresh-process ED proof remains a distinct mandatory successor. 

## Accepted foundation binding and review decision

Foundation source bf2d27c39e53de0dacbe005105ea214372757309 plus proof a6d148b6ff33f6ea2d2c7024a9a1a548b48dd274 accepted in conductor/evidence/e2.2-flow-foundation-completion-20261012/acceptance.json. Earlier proposal caveats above are resolved by this binding; source dispatch still requires prepared packet at current HEAD.

Actual EdFlowAdapter::new in flow_adapter.rs lines531-615 is the only Flow constructor. Factor its body into a private shared builder accepting optional validated transit extension; preserve new(plan) behavior. Validate combined event/entity u32 representation and same-tick u64 limit before FlowRuntime::with_configs. Keep max_callback_commands=1. Register fixed transit planner alongside marker hook before any carrier context. Child flow_adapter/transit.rs can access parent private fields and implement methods; store optional transit state in EdFlowAdapter. No alternate Flow or Scheduler.

Actual native APIs are in kairo-ecs-abm/src/flow_adapter.rs: TransitContext::new_handoff1333, observe_transit_arrival_handoff1746, pending_transit_arrival_handoff1776, register_transit_context1803, schedule_transit_start1887; exact source signatures must be inspected. FlowRuntime::create_actor_domain_context is in kairo-ecs-des/src/flow.rs2316. Marker hook/runtime/private handle identities remain authoritative.

Place transit eligibility guard in admit_service after runtime/service/used-service validation and before reserve_request or any mutation. Actual step consumes native dispatch; observer must use that dispatch, installed carrier and runtime. Add Transit observation variant as planned. Root explicitly accepts this source compatibility change to the doc-hidden experimental parent enum: downstream exhaustive matches may require a new arm; no stable public release compatibility is claimed. Existing foundation tests and new(plan) callers remain unchanged.

Independent GPT-6-luna source review confirms builder, child privacy, admission and public native APIs fit. Root accepts this contract for bounded implementation; GPT-6.1-sol successor planning continues separately. Use max two root-reviewed failed corrections, immutable attempts, first-failure stop; earlier development ceiling3 refers initial attempt plus those two corrections.
