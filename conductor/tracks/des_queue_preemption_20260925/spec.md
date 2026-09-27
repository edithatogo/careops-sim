# Specification: first-class DES queues and preemption

**ID:** `des_queue_preemption_20260925` · **Status:** proposed
**Upstream owner:** 03 (Flow), with 01/04/12/25 review.
**Baseline:** Kairos `fae901558f07b7b717a676adbafbe2cdc78dea1c`.

## 1. Objective and scope

Make resources, queued claims and interrupted work first-class ECS data with
concise Flow APIs. Support capacity, deterministic priority/FIFO arbitration,
waiting deadlines, explicit release/cancel, and Suspend/Abort/Restart preemption.
Use the existing scheduler, world, generational handles and component registry.

The first release supports **one capacity unit per request, one resource per
work item**, with many units per resource. Atomic multi-resource acquisition,
multi-unit gang requests, dynamic priority aging, priority inheritance and
cross-LP shared resources are deferred. Sequential multi-resource claims can
deadlock; the API documents this and does not imply atomic acquisition.

Inputs: registered resources, actor/work entities, duration/context descriptors,
checked virtual-time commands, priorities, run configuration and seed identity.
Outputs: stable request/allocation handles, lifecycle notifications, observable
queue/allocation snapshots and versioned resource telemetry.

## 2. Integration and ownership

Current `Resource` is a FIFO helper outside ECS storage. `DESContext` exposes
public scheduler/world/resources fields and has no component registry. Preserve
its existing constructors, fields and behavior. Introduce additive
`FlowRuntime` in `kairo-ecs-des`, owning exactly one Scheduler, World and
ComponentRegistry. Resource definitions, requests and resumable work use these
stores. Queue indexes contain entity handles, not a second authoritative copy of
claim state.

Adapters let existing DES trajectories and ABM behavior hooks operate against
that runtime. Do not synchronize two independent DES/ABM worlds or clocks. The
legacy resource helper remains available; an explicit migration example shows
creation of the new resource entities. Using the new runtime is required for new
preemption semantics. It must enforce checked admission even though the raw core
scheduler currently accepts past timestamps.

Owned paths: DES modules and tests, coordinated ABM adapter, `examples/flow/`.
Other owners: core/state/types/RNG (01), Arrow (04), conformance (12), API review
(25). See [agent contract](agent-contract.md). No core ordering change, unrelated
backend rewrite, binding expansion or ED triage policy is authorized by this spec.

## 3. Component and identifier schemas

Names and Rust shapes below are proposed public design, subject to Q0 API review.
Time values use existing `SimTime`/`SimDuration` nonnegative integer ticks. IDs
wrap generational `EntityId`; allocation identity also includes an incrementing
lease revision. Revision/sequence overflow returns an error, never wraps silently.

| Component / type | Required fields and meaning |
| --- | --- |
| `ResourceCapacity` on resource entity | `total: u32`; zero allowed to represent closure. Available = total − active allocations; never an independently mutable counter |
| `PriorityKey` | `level: i32`, `enqueue_sequence: u64`, `request: EntityId`; lexicographic ascending, including `(index,generation)` for the final tie-break |
| `ClaimQueue<PriorityKey>` | Ordered request index with explicit insert/remove/rekey; monotonically assigned admission sequence per run. No floating priority or user comparator in v1 |
| `ActiveAllocations` on resource | Allocations indexed by lease identity; each records request, owner, work, granted_at, segment_started_at, completion_at if timed, priority and lease revision |
| `ResourceRequest` on request entity | resource, owner, optional work; priority_level, original_enqueue_sequence, submitted_at, optional waiting_deadline, `can_preempt: bool`, `on_preempt: Option<PreemptionStrategy>`, state, revision |
| `RequestState` | Queued, Active, Suspended, Completed, Released, TimedOut, Cancelled, Aborted; terminal states have reason/time and no active lease |
| `PreemptionStrategy` | `Suspend`, `Abort`, `Restart`; policy of the work being interrupted |
| `WorkSpec` on work entity | Original sampled duration, registered continuation/context reference and codec version; optional purpose-specific RNG identity |
| `InterruptedWork` on work entity | elapsed_ticks (useful work in current attempt), remaining_ticks, original_duration, cumulative_busy_ticks, attempt, interruption_count, interrupted_at, suspended_context, pending_resume_policy |
| `WorkSchedule` on work entity | completion_event: Option<EventId>, lease_revision, work_revision; stale scheduled events must not match a replacement attempt |
| `SuspendedContext` | Versioned owned Rust task data or a typed handle into runtime-owned data with registered encode/decode hooks; no borrowed pointers, closures, Python objects or anonymous `Any` snapshot promises |

Checkpoint schemas explicitly encode component types. ComponentRegistry's
in-memory type erasure is not a portable serialization format. Validate entity
liveness/generation on every command; despawn cleans associated claims, leases
and task references before recycling the actor handle.

## 4. Queue and capacity rules

- Lower numeric priority is more urgent. Within a priority, original admission
  sequence is FIFO. Reprioritization retains that sequence; cancel+resubmit gets
  a new sequence. Default behavior is no preemption in either direction.
- Drain the best eligible claims after a capacity change, release, completion,
  timeout, cancellation or priority change. Suspended/restarting claims share the
  same queue and retain their original sequence. They do not bypass more urgent
  waiters. No implicit aging or fairness guarantee against endless urgent work.
- A waiting deadline is absolute and exclusive: a queued claim may grant only
  while `now < deadline`. It applies until the **first grant**, then is cleared.
  Requeued interrupted work has no implicit timeout. A separate explicit cancel
  command can terminate it; future total-task deadlines require a new contract.
- Requests with an already reached deadline become TimedOut immediately, with no
  allocation. Reprioritizing active work changes its eviction priority and may
  allow a queued higher-priority preemptor to evict it.
- Capacity growth drains the queue. Shrinking below the active count fails
  atomically; there is no implicit eviction. Shrinking to zero succeeds only
  when no allocations remain. Resource removal requires no claims/allocations,
  or an explicit administrative cancellation operation with a reason.
- Manual untimed leases permit explicit release. Preemptible leases must be
  bound to registered interruptible work/context so interruption cannot silently
  discard state. Non-preemptible manual leases remain valid.
- Cancel/release handles are idempotent: return `AlreadyTerminal` for a known
  terminal claim and `StaleHandle` for a recycled entity/generation; never free
  capacity twice. Keep terminal records until an explicit prune/checkpoint
  boundary; stale handles remain invalid afterwards.

## 5. Preemption contract

`can_preempt` means the incoming claim may evict others. `preemptible(strategy)`
means this claim's work consents to interruption and selects its recovery policy.
These are independent flags. SimPy's request `preempt` flag concerns the former;
this API does not conflate the two.

When capacity is full, consider waiting claims in PriorityKey order. For a claim
with `can_preempt=true`, a victim must be Active, explicitly preemptible, and have
**strictly worse priority**. Equal priorities never evict. Select the least urgent
victim; tie-break by latest original admission sequence, then largest request
ID. Re-evaluate after each atomic replacement. One-unit claims need one victim;
never evict several holders speculatively.

Before eviction, account for actual busy time since segment_started_at. Waiting
and suspended time are excluded. If a timed victim has zero remaining duration
at this tick, complete it instead of interrupting it, then arbitrate the queue.
Completion is emitted once; any later dispatch of its old event is stale.

| Strategy | State transition on eviction | Resume behavior |
| --- | --- | --- |
| Suspend | Active → Suspended; store progress/context; requeue same request | Remaining duration; resume handler restores saved context |
| Abort | Active → Aborted; remove lease; cancel completion | Terminal; cancellation handler exactly once, no automatic requeue |
| Restart | Active → Suspended with Restart policy; store audit/progress; requeue | New attempt, elapsed=0, remaining=original duration, initial context restored |

Restart reuses the original sampled duration by default, so interruption does not
silently consume new random draws. `cumulative_busy_ticks` retains discarded
attempt effort. Resampling-on-restart is outside v1. Useful elapsed for Suspend
accumulates across segments; for Restart it is per attempt. Application handlers
must define restartable effects: completed external side effects cannot be
rolled back by the runtime. Core work/notifications remain simulation-local.

Eviction cancels the old completion token and invalidates its revisions before
publishing notifications. Cancellation failure because the event is already
dispatched cannot resurrect work: revision/state guards are mandatory. One
allocation release and one interruption notification result from one eviction.

## 6. System execution order and same-time events

The core dispatch order remains **(time ticks, scheduler priority, insertion
sequence)**. Resource priority controls claims, not the scheduler's priority.
Flow-generated events default to scheduler priority 0. Any advanced override is
explicit and recorded in the run manifest. No global phase-priority lanes or
hidden reordering of unrelated events are introduced.

For each dispatched Flow command/event, execute a non-reentrant transaction:

1. Validate target generation, command revision, timestamp (`at >= now`) and
   checked arithmetic; stale completion/timeout events are observable no-ops.
2. Expire affected waiting claims whose deadline has been reached, in PriorityKey
   order. For each affected resource, materialize timed allocations with
   completion_at <= now as completed, in `(completion_at, request ID)` order.
   These boundary transitions precede the explicit operation and arbitration.
3. Apply the event's explicit operation (arrival/release/completion/cancel/
   reprioritize/capacity change). A cancellation of work already completed in step
   2 returns AlreadyTerminal. Unrelated cancel/repriority commands at equal time
   otherwise take their existing scheduler order.
4. Arbitrate the affected resource queues in resource-ID order. Grant free units,
   or perform one eligible eviction+grant transaction, and repeat until stable.
5. Commit state, schedule completion/timeout events with fresh revisions, then
   append lifecycle records in transition order with a local transition ordinal.
6. Enqueue continuation notifications in the same deterministic order. Callbacks
   run as later dispatched events and may submit checked commands; they cannot
   mutate an arbitration transaction reentrantly.

This defines local deadline/completion boundary semantics, not a modification of
Scheduler ordering. A task's busy interval is `[segment_start, completion_at)`;
work finished exactly at T is not a preemption victim at T. A claim with deadline
T cannot grant at T even if release dispatches before its timeout token. Commands
with competing effects not covered by these two boundary rules resolve by core
order. Tests must cover both insertion orders and multiple scheduler priorities.

Zero-duration work receives a grant and one same-tick completion. Per-tick
transition/notification limits and the existing max-events limit terminate
zero-time feedback loops with a structured limit result, never an unbounded loop.
Raw scheduler events for other domains are dispatched through registered domain
handlers; a Flow wrapper does not silently ignore or reinterpret them.

## 7. Public API additions

Illustrative intended API (not code available at the reviewed baseline):

```rust
let bed = flow.resource("treatment_bay").capacity(4).spawn()?;
let work = ctx.work(duration, patient_context, continuation)?;
let claim = ctx.acquire(bed)
    .with_priority(acuity_priority)
    .deadline(wait_until)
    .can_preempt(true)
    .preemptible(PreemptionStrategy::Suspend)
    .for_work(work)
    .submit()?;

ctx.reprioritize(claim, updated_priority)?;
ctx.cancel(claim, CancelReason::PatientLeft)?;
// ctx.release(claim)? is available for explicit/manual leases.
```

`submit()` attaches ECS components and schedules an admission command; it does
not synchronously run arbitrary callbacks. The request ID allows querying Pending
admission and subsequent state via an API result (Pending is an API admission
status, not a second queue state). Builder validation returns typed errors without
partially inserted components. Runtime changes occur on deterministic dispatch.

Notifications: Queued, Granted, Preempted(strategy, by_request, progress), Resumed,
Restarted(attempt), Completed, Released, TimedOut, Cancelled and Aborted. Each
carries resource/request/work/owner IDs, tick, causal event ID and transition
ordinal. Only lifecycle-supported notifications fire; no Released in addition to
Completed merely to signal the same capacity release. Handlers observe committed
state. Restart emits Preempted on eviction and Restarted on subsequent grant.

## 8. Determinism, replay and telemetry

- Canonicalize queue/resource/notification iteration; ComponentStore dense order
  and HashMap iteration cannot determine outcomes.
- Checkpoint world IDs, registered contexts, queue/admission sequences, active
  leases, revisions, remaining work, pending commands/notifications, deadlines,
  clock/scheduler order and RNG states. Compare resumed and uninterrupted runs.
  Implement via Track 01/22 contracts; do not invent a second snapshot format.
- Derive task/attempt/purpose streams under a versioned Track 01 rule. No wall
  clock, worker identity or thread order enters seeds. Reuse original draws for
  Suspend/Restart. DESContext::new(seed) alone currently provides no such proof.
- Preserve canonical `event_log.v1` fields and encoding. Propose a Track 04
  `resource_lifecycle.v1` sidecar: schema_version, run_id, causal_event_id,
  transition_ordinal, ticks, resource/request/owner/work IDs, lease_revision,
  transition, strategy?, preemptor_request?, priority, queue_len, active_count,
  capacity, elapsed/remaining/cumulative_busy ticks?, reason?. Tick and ID Arrow
  encodings match the existing machine schema. Nullable fields are explicitly
  typed; a record is uniquely keyed by run/event/ordinal.
- Event-log dispatch and resource transition records are distinct: a cancelled
  stale scheduler event is not another model completion. Roll-up telemetry must
  not double-count them.

## 9. Compatibility and backend boundaries

Local CPU replications own independent runtimes. Results sorted by replication
and stable event/transition key must match across 1/2/N workers. Within one LP,
queue transitions are serial. Global resource sharing across PDES LPs is rejected
until Track 34 defines the owner/message and zero-lookahead behavior. Compare
PDES final observable state per its existing contract, not identical cross-LP
trace order. No lock-based shared global queue or GPU event scheduler is added.

Add APIs without removing legacy Resource/DESContext behavior. Public enums must
follow Track 25 compatibility rules; feature flags and FFI expansion are separate
reviewed work. Rust-only components cannot accidentally appear in a stable C ABI.

## 10. Acceptance and quality gates

- Q-01 Capacity is conserved; every active lease has one live request/owner;
  requests never occur in both active and waiting indexes.
- Q-02 Priority/FIFO order, rekeying, timeout, cancellation and stale-handle behavior
  match sections 4–6 under both unit and integration tests.
- Q-03 Low-priority work of duration 10 starts at 0; urgent duration-2 work arrives
  at 3. Urgent completes at 5. Low completes at 12 (Suspend), 15 (Restart), or
  never (Abort), with correct effort accounting and single notifications.
- Q-04 Same-time completion/preemption, timeout/release, repriority/cancel and
  zero-duration cases satisfy the declared tie contract without time reversal.
- Q-05 Byte-identical canonical integer lifecycle outputs across repeated seeded
  serial runs, worker counts and checkpoint/resume. Backend float metrics have
  separate tolerance contracts; queue outputs do not.
- Q-06 Legacy fixtures and public compatibility gates still pass; users can build
  a resource/preemptible task without manual component attachment.
- Q-07 Queue insert/remove/rekey scales logarithmically in waiting-claim count;
  initial victim selection may scan active allocations. Benchmarks disclose that
  cost and retain upstream performance gates. No unmeasured speedup claim.

See [test matrix](test-matrix.md) and [plan](plan.md). Release requires contract/API
review, implementation evidence, docs/migration notes, feature/version policy,
upstream registry/handoff updates and applicable phase gates.

## Sources

- [Kairos core contract](../../../libs/kairos/conductor/contracts/core-contract.md)
  is normative for scheduler order and ECS concepts.
- [SimPy resources reference](https://simpy.readthedocs.io/en/latest/api_reference/simpy.resources.html)
  informs ergonomics and the distinction between priority and permission to
  preempt. This design deliberately uses explicit victim policies and FIFO equal
  priorities; it does not promise exact SimPy API or interrupt equivalence.

## Domain research review boundary

Q0 reviews assisted transport, co-working, interruption ancestry and ED reselection needs. Clinical task selection belongs in the ED adapter and must conform to approved Suspend/Abort/Restart and deterministic ordering. Atomic bundles, fractional multitasking and new scheduler precedence remain deferred unless explicitly redesigned and tested.

See [reports 5–8 incorporation](../../research/ed-research-incorporation-20260927.md).
