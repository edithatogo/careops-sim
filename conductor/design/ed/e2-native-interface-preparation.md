# E2 native interface preparation

Status: reviewed design preparation, not an executable worker packet or upstream
API acceptance. E2.1 staff specifications are accepted; interruption and space
specifications must be accepted before E2.2 dispatch. Rust/Cargo 1.99.0 only.
Source examined: Kairos `6d733f489225da4bf6ceec2673b02369c7918bbe`.
Root coordinates; GPT-6.1-sol reviews architecture and bounded Luna agents inspect
source dependencies. Requested model names are not served-model attestation.

## One authoritative runtime

The parent pathway currently creates a core Scheduler and ResourceRuntime.
FlowRuntime owns another private Scheduler. Alternating those two engines would
not satisfy shared time/state ownership. E2 must use one FlowRuntime for pathway,
terminal events, staff work and ABM transit; the ABM adapter already accepts a
caller-owned mutable FlowRuntime. Preserve accepted E1 public behavior and its
native hand oracles during migration. Staff skill/zone/calendar selection and
ASCII urgency/readiness/ID ordering remain parent domain policy.

One work item claims one unit of one resource. ED occupancy, diagnostic room
reservation and work claims remain explicit separate lifecycles. No atomic
staff-plus-space request or hidden multi-role approximation is introduced.

## Resource grant gate: smallest proposed engine change

Existing domain callbacks run after the allocation plan commits. Pause/Resume
domain controls are notifications, not allocation gates. Setting capacity to zero
cannot represent a closed calendar while active work drains: shrink below active
claims rejects. Cancel/resubmit would change Suspend lineage and accounting.

Propose a scheduled per-resource enabled flag, default true. A disabled resource
blocks new grants, preemption selection and suspended-work resumption. Active
work, completion, release, cancellation and waiting deadlines retain their current
semantics. Re-enabling wakes its eligible queue. Calendar closure does not alter
configured capacity or resample work.

Provisional API shape for Track03 review:
`schedule_resource_grant_enabled(resource, enabled, at, scheduler_priority)`
returns the scheduled EventId. Exact Rust signature, error variant names and
read-only inspection API must be frozen in the owner ADR before implementation.

### Same-tick closure invariant

For every allocation transaction on resource R at tick T:

1. Settle R's due completion boundaries, as the existing planner does.
2. Treat R as closed when its current flag is false OR an accepted pending
   Disable(R,T) exists, regardless of that command's scheduler priority or
   insertion order.
3. If closed, perform no free grant, preemption or resume. Other lifecycle
   transitions still commit; there is no transient unavailable claim.

The disable remains a real scheduled command. A pending-boundary index must agree
with command and scheduler identities; do not discard its command or infer a
second clock. Enabling is never applied by lookahead. Its explicit command settles
due completions, enables the resource, then wakes allocation.

Reject a second gate command for the same resource and tick, even if identical.
The parent normalizes calendar net availability first: adjacent shifts produce
one effective boundary, not a same-tick disable/enable pair. Predeclare calendar
boundaries before execution reaches that tick; reject changes at or before a
started execution frontier. Exact transaction and rejected-admission invariants
need native tests before API acceptance.

### Parent admission ordering

The parent must expose all ready-at-T work before an Enable(R,T) event allocates.
Define explicit scheduler phases for fixed domain ready events, completion
notifications and their same-tick continuations, then enabling. For example,
notifications at priority 0 and work submission at -1 can precede enable at 10;
these values remain a proposed adapter contract, not engine-wide reserved values.
Queue insertion follows the parent urgency/readiness/ASCII-ID policy.

This guarantees the frozen ED cases only under the adapter's closed admission
phase. It does not promise that arbitrary external callers can introduce a future
same-tick urgent request after a grant and retrospectively reorder that grant.
Do not replace step semantics with a general tick-batching API without a new
demonstrated requirement and review. No gate means legacy runtime behavior.

### Resource lifetime and transactional admission

Further GPT-6.1-sol source review identified these rules for the future owner ADR:

- Reject resource removal while gate commands remain pending, at admission and
  dispatch. Conversely reject new gate commands while removal is queued. No
  implicit cancellation or silent loss of scheduled toggles. An idle disabled
  resource with no pending toggles may be removed; a new generational resource
  identity starts enabled and inherits no old gate/index state.
- Validate resource identity, time/frontier, duplicate boundary, pending removal
  and all command/event/index/counter budgets before publishing any mutation.
  Rejected admission leaves gate state, queues, scheduler counters and event
  identities unchanged; a valid retry must match an untouched control.
- At initial time zero, now()==0 does not distinguish unstarted from started
  execution. Use an explicit optional started frontier or a verified equivalent.
  A gate at zero is allowed before the first committed dispatch, then rejected.
  A precommit budget rejection does not start the frontier; a dispatched command
  with a semantic rejection does. V2 persists this distinction; V1 restore must
  derive the legacy frontier from actual scheduler dispatch state.

These add native oracles for remove/toggle admission in both orders, disabled
idle removal and generational reuse, duplicate/time/budget rejection atomicity,
and tick-zero behavior before/after dispatch and restore. They do not authorize
engine edits before the owner ADR and bounded packet review.

## Checkpoint compatibility

Flow V1 encodes resource capacity as a bare u32, has command tags 0 through 11,
and rejects trailing bytes and unsupported versions. Adding a flag or command
silently to that format is incompatible.

Propose explicit Flow checkpoint V2 DTO/wire/capture/restore, preserving V1
layouts and decode. V1 restore means all resources enabled and no gate commands.
V1 capture must visibly reject any disabled state OR pending gate command,
including a future command while currently enabled. Never silently downconvert.
V2 retains gate state, pending commands, event identity/time/priority, uniqueness
and index invariants; restore validates resource references and the one-to-one
scheduler/command join before mutation. Preserve existing V1 golden bytes.

Actual downstream consumers requiring owner review:

| Source under Kairos crates | Dependency |
| --- | --- |
| `kairo-ecs-calibration/src/c2_portable_checkpoint.rs` | Embeds Flow bytes and explicitly decodes V1; dispatch by inner version while preserving the outer envelope if sufficient |
| `kairo-ecs-calibration/src/flow_bridge/checkpoint_wire.rs` | Coherent-cut validation takes V1 and examines registrations, context stores, scheduler entries and command variants; use a lossless version-neutral view |
| `kairo-ecs-calibration/src/flow_bridge.rs` | In-memory snapshot tests and typed helpers need compatible coverage |
| `kairo-ecs-calibration/src/shadow_native.rs` | Captures a snapshot to inspect the next scheduler tick; review a common accessor rather than lossy conversion |
| `kairo-ecs-calibration/src/c5_native.rs` | Same scheduler inspection dependency |

ABM transit persists its own registered context; no direct full Flow checkpoint
consumer was found in ABM source. Its context/coherent-cut validation remains
required. Nested FlowDispatch wire and the ABM transit schema do not automatically
need a version change. An unchanged outer envelope does not mean old software
can read a newly embedded Flow V2 image.

## Required native oracles

- Completion at 3 with break [3,5): Completed at 3, no grant/resume at 3, grant at 5.
- Low work 10 from 1; urgent work 2 ready at 3 during break [3,5): low [1,5),
  urgent [5,7), low [7,13), conserved original sample.
- Low suspended at 4; urgent finishes at 6 during break [6,8): resume at 8,
  preserving remaining work, never at 6.
- Enable/release and urgent readiness at 5 under the parent admission contract:
  urgent first, no gratuitous low resume/preempt pair.
- Enable at H may grant with zero observed work; no invented completion.
- Closure works for completion-first and disable-first insertion, and i32::MIN
  competing submissions. Adjacent calendar intervals normalize deterministically.
- V1 legacy restore, V2 disabled state and pending-toggle round trips; exact
  uninterrupted/restored claims, malformed references/tags/versions rejection,
  restore atomicity, and no lossy V1 capture.
- Full accepted E1 parity and actual native bindings for staff32, interrupt24 and
  space32. Specification checker success is not evidence for these bindings.

## Ownership, dependency order and parallel opportunities

1. After E2.1 acceptance, root obtains Track03 owner ADR/public contract review
   and records downstream Track22/calibration dependencies in the owner handoff.
2. One isolated Track03 writer implements the grant gate and focused native
   collision tests. It owns flow.rs and its declared test paths. During this
   intermediate work, legacy V1 capture must already fail closed for disabled
   state or pending gate commands. Focused success freezes a dependency interface;
   combined gate API acceptance waits for the checkpoint and consumer join.
3. Once runtime types are frozen, a checkpoint writer implements V2 and compatible
   V1 behavior. Runtime/checkpoint changes are serialized while they share types;
   disjoint review and consumer preparation can proceed in parallel.
4. A separately owned calibration packet updates coherent-cut and portable image
   consumers. No unreviewed conversion that drops gate state is allowed.
5. Independently qualify combined Kairos under Rust 1.99, commit there, then
   update the parent pin under a separate claim. Bind parent E2.2 dispatch,
   fidelity, claims and limits packets to actual accepted APIs.
6. Integrate the single-runtime parent adapter, preserve E1, then run E2.3 composed
   MVP and E2.4 acceptance. Continue later delivery phases; this document closes
   no plan checkbox, empirical/clinical gate or release gate.

The coordinator starts the next dependency-ready packet after acceptance. The
existing quiet continuation heartbeat supports recovery; no new hook, broad
unattended agent, publication or external side effect is authorized here.
