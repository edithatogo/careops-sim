# ADR-0003: Additive FlowRuntime and queue API boundary

- Status: **Q0.1 architecture direction approved by Kairos owner; Q0.2/Q0.3 semantics and implementation gates remain open**
- Date: 2026-09-28
- CareOps track: `des_queue_preemption_20260925`, Q0.1
- Proposed by: CareOps Sim coordinator
- Owner direction recorded: 2026-09-29, Kairos repository owner directed adoption of the four recommended Q0.1 dispositions; see [owner decision receipt](../../evidence/q0.1-owner-decision-package-20260929.md)
- Technical review: independent Track 01/03/25 reviews; see the owner decision receipt
- Reviewed Kairos development commit: `25177e5644ecb132c4df2cba1b4a0aeed1d08cb6` (synchronized with `origin/main` `9ae5461`; reviewed runtime source trees are unchanged from the hash-bound base below)
- Parent submodule pin: `libs/kairos`

## Context

Kairos's shared core contract says resources and processes are ECS entities,
while the current public DES helper remains a FIFO `Resource` stored outside ECS.
`DESContext` owns a `Scheduler`, `World`, and resource vector. `ABMContext`
separately owns a `Scheduler`, `World`, and `ComponentRegistry`; the existing
`AgentBehavior` context borrows an event, per-agent RNG stream, and world.
`ComponentRegistry` uses Rust type erasure and has no portable encoding contract.
These current APIs must not be mistaken for an existing unified hybrid runtime.

The runtime source hashes are bound to Kairos upstream `764048a89872eae74e35bd92c5dff8a1527bd5b1` and were compared against current `origin/main` `9ae5461` and development commit `25177e56`. The reviewed Rust source trees are unchanged across those revisions; the current core source includes upstream's release-mode pending-event cleanup fix. Source hashes and differences are recorded in [the refreshed source review](../../evidence/q0.1-current-source-review-20260929.md).

This decision addresses architecture and compatibility only. Q0.2 owns queue,
tie, preemption and same-tick semantics. Q0.3 owns event-kind allocation,
lifecycle telemetry, snapshot encoding and event/transition join semantics. The
decision authorizes these settled directions to proceed into those design phases;
it does not authorize queue implementation, alter upstream task status, or waive
D2, Q0.2/Q0.3, implementation testing, or release review.

## Decision proposed

### 1. Add a new runtime; preserve the old API

Add an experimental, Rust-only `FlowRuntime` in the existing
`crates/kairo-ecs-des` root. It owns exactly one shared `Scheduler`, `World`,
and `ComponentRegistry`. New resource/request/work entities and their components
live in that shared store. The ABM adapter operates against the same runtime's
world and scheduler; it must not create a second clock, world, or scheduler.

Keep `DESContext`, its public fields and constructors, `Resource`, and their
observable FIFO behavior available and unchanged. Do not retrofit their storage
or silently change their semantics. Existing `ABMContext`, `BehaviorContext`,
`AgentBehavior`, and `BehaviorSimulation` also remain source-compatible. A new
Flow-specific adapter/context is required for shared ECS behavior; it cannot
reuse `BehaviorSimulation`, which privately owns a separate `ABMContext`.
Existing behavior that depends on standalone `ABMContext` continues to use that
context.

This is an additive change within the experimental Rust API at
`crates/kairo-ecs-des`. It does not change core scheduler order, shared type
representation, RNG derivation, Arrow schemas, C ABI, or host bindings. Track 25
must classify the actual public symbols before implementation is merged; a
release-stage claim remains subject to the protected-surface inventory and
release gate.

### 2. Keep public identity opaque and validate at the runtime boundary

The proposed surface uses typed opaque newtypes:

| Handle | Identity | Contract |
|---|---|---|
| `ResourceId` | Generational resource `EntityId` | Valid only while the exact entity generation is alive |
| `ClaimId` | Generational request `EntityId` | Identifies one request and its retained terminal record |
| `WorkId` | Generational work `EntityId` | Identifies one registered interruptible unit of work |
| `LeaseId` | `ClaimId` plus checked lease revision | Identifies one active allocation; a later grant receives a new revision |

Callers do not construct or mutate internal queue/allocation components. All
Flow-owned commands validate liveness, state, and time before state changes.
`Scheduler::schedule` currently increments sequence/index/stat counters
unchecked, while `World::despawn` wraps the entity generation. The Q0.1 runtime
proposal specifies a supported-use envelope: a private, fresh runtime rejects
any operation beyond `u32::MAX` successful scheduled events, entity creations,
or entity despawns, with every mutation routed through its facade. This bounds
the unchecked counters and generations in supported Flow use without claiming
that the underlying APIs are globally overflow-safe. Track 01 must review the
assumptions and cleanup contract before implementation; broader checked
core/state APIs would require separate upstream work. A local `FlowError`
cannot undo a wrapped counter if callers bypass the facade.

Past-time admission is a Flow-boundary error enforced before calling
`Scheduler::schedule`; the current raw scheduler accepts past times and can
regress its clock. Other Flow-owned errors may distinguish invalid/stale
handles, unknown resources, terminal requests, invalid builder state, capacity
constraints, unsupported event/handler kinds, and Flow-owned counter overflow.
Stale-handle validity across `u32` generation wrap is not promised until Track
01 resolves the existing `World` behavior. Error text is diagnostic, not a
compatibility key. Exact variant names and method signatures require Track 25
review before implementation packet binding.

An acquire builder is declarative. Its `submit` validates and records an
admission command, returning a request identity/status; it does not call user
callbacks or promise synchronous grant. Runtime state changes occur only in
deterministic dispatch. Calling code observes queue/allocation state through
read-only runtime queries and ordered lifecycle records.

### 3. Keep continuation context owned; defer portable checkpoint codecs

Interruptible work must not make a borrowed pointer, closure, Python object, or
unversioned `Any` value into a portable checkpoint contract. The Q0.1 codec
proposal selects one typed, owned, in-memory continuation context within the
shared FlowRuntime, with deterministic registration identity and explicit
lifecycle. It does not add `serde`, a codec registry, public serialization API,
or portable checkpoint claims. Handler implementations remain Rust code in
the owning model/adapter; exact handler-registration signatures remain a Track
03 review item.

The current `kairo-ecs-state::WorldSnapshot` is a deterministic view of live
entity IDs only; it omits component values, scheduler state, RNG state, Flow
queues/work, and behavior registrations. The Track 22 CLI checkpoint and
resume commands are scaffold surfaces, not complete save/restore semantics.
Track 22 owns the later versioned portable checkpoint contract, coordinated
with Tracks 01, 03, 04, and 25. No second Flow snapshot standard or
cross-version persistent save-file promise follows from this ADR.

### 4. Preserve scheduler and ownership boundaries

The core scheduler order remains `(time_ticks ASC, priority ASC, sequence ASC)`.
Resource priority is separate from scheduler event priority. Before scheduling
through the Flow facade, commands must check `at >= Scheduler::now()`; the raw
scheduler does not enforce this and can move its clock backwards.

Flow dispatch routes registered event kinds/continuation keys to specific
handlers; it must not call one behavior for every entity-associated event. A
Flow-specific behavior context must provide read-only queries over the shared
`World` and `ComponentRegistry`, the event and deterministic agent/task stream,
plus a constrained command sink. It must not expose mutable references to the
world, registry, scheduler, or internal resource/request components: handlers
could otherwise bypass liveness checks, cleanup, and arbitration. The sink
buffers typed commands and validates them at the deterministic dispatch boundary;
it cannot mutate scheduler/arbitration state reentrantly. Exact public query and
command types remain a Track 03/Q0.3 decision. The current `BehaviorContext`
lacks component access and a scheduler/command sink; `BehaviorSimulation` owns a
separate runtime. Existing public ABM types remain unchanged. Exact handler
registration, command-sink behavior and event-kind allocation are Q0.3/Track 03
review prerequisites. No Flow/ABM adapter exists today.

There is no core ordering change in this ADR.

Queue state is authoritative in ECS components; any ordered queue index is a
rebuildable/internal index of entity handles, not a duplicate owner of claim
state. No entity iteration order or hash-map iteration may decide arbitration.
The current core's entity generation behavior and RNG APIs are consumed as
they are; changes to overflow behavior, seed derivation, or event scheduling
require their owners and separate contract review.

The new runtime is single-process and single-event-queue. Per-replication local
CPU parallelism can use independent runtimes only after deterministic seed and
result-order contracts pass. A resource shared between PDES logical processes,
GPU/Metal queue execution, distributed claims, FFI/binding exposure, and
atomic multi-resource requests are outside this decision.

## Intended API shape (illustrative, not approved signatures)

```rust
let resource: ResourceId = flow.resource("treatment_bay").capacity(4).create()?;
let work: WorkId = flow.work::<TreatmentHandler>(duration, context)?;
let submitted = flow.acquire(resource)
    .with_priority(acuity_priority)
    .deadline(wait_deadline)
    .can_preempt(true)
    .preemptible(PreemptionStrategy::Suspend)
    .for_work(work)
    .submit()?;
// `submitted` is pending admission; deterministic dispatch produces grants/events.
```

Builder ownership, exact names, query shape, handler registration API, and
error variants remain open to the required owners. A change to the illustrative
syntax is not a spec failure if the compatibility and runtime invariants above
remain satisfied.

## Compatibility review

| Review item | Proposed classification |
|---|---|
| Affected crate | `crates/kairo-ecs-des` |
| Protected root | `crates/kairo-ecs-des` and `crates/kairo-ecs-abm`, now registered as experimental Rust API roots |
| Proposed surface family / stage | Rust API / experimental; Q0.1 owner-approved architecture direction, with exact-symbol review held until Q0.2/Q0.3 and implementation |
| Additive only | Yes, if existing `DESContext`, `Resource`, and ABM entry points stay source- and behavior-compatible |
| Scheduler/event ordering | No change |
| RNG algorithm or seed derivation | No change |
| C ABI | Not changed or exposed by this phase |
| Python | No binding is added; explicit N/A in Track 25 API review |
| R | No binding is added; explicit N/A in Track 25 API review |
| Julia | No binding is added; explicit N/A in Track 25 API review |
| TypeScript/Wasm | No binding is added; explicit N/A in Track 25 API review |
| C# | No binding is added; explicit N/A in Track 25 API review |
| Go | No binding is added; explicit N/A in Track 25 API review |
| Arrow schemas | Not changed; no Arrow dependency is added to DES |
| Conformance fixtures / replay | Existing fixtures remain unchanged; new deterministic Flow fixtures are required before implementation acceptance |
| Batch use | Multiple commands may be submitted before dispatch; no vectorized batch or performance guarantee is made until measured |
| Consumer migration | None required to keep using legacy API; optional migration example is Q4 |
| Release hold | **Yes.** Exact-symbol Track 25 review, implementation evidence, compatibility checks, and release signoff remain open; this ADR does not authorize release |

The Kairos owner approved registering both affected roots as experimental. The
inventory, versioning policy, compatibility matrix, release note, per-root
preimplementation dispositions, and validator now align. This records the
architecture and protected-root boundary only; it is not a completed review of
concrete public symbols or a release approval.

The normal Track 25 API review record must cover the concrete Q1/Q4 public
surface and answer its Rust, C ABI, Arrow, Python, R, Julia, TypeScript/Wasm,
C#, Go, batch, conformance and deterministic replay questions, including
explicit `not affected` results for each surface. If review finds that the additive facade
changes a protected promise or requires source migration, classify it under the
existing compatibility contract and add the required migration note. This ADR
does not declare Track 25 approval.

## Review questions for owning tracks

1. **Track 01:** Confirm a DES-owned facade can compose one scheduler, world and
   registry while consuming Track 01 APIs only; review the private-runtime
   `u32::MAX` operation caps and Flow-owned typed cleanup hooks. The caps bound
   supported facade use; they do not change global core/state guarantees.
2. **Track 03:** Approve a new Flow behavior adapter/context with read-only
   shared state, owned in-memory continuation context, and buffered checked
   commands; specify the dependency and deterministic event-kind-to-handler
   dispatch route and how it avoids reusing `BehaviorSimulation`'s separate
   runtime.
3. **Track 25:** After Q0.2/Q0.3 and implementation settle the concrete surface,
   review exact DES and ABM symbols, legacy compatibility fixtures, and any
   migration/release note. Keep release held until that review is complete.

Independent technical reviews found (a) the Track 03 adapter needs its own
read-only component-aware context and buffered checked-command dispatch
contract, (b) Track 01's
current counter and generation behavior prevents the broad checked-counter and
stale-handle claims, and (c) both DES and ABM roots must be protected. This ADR
incorporates those findings. These reviews are not upstream maintainer
signoffs. The owner-approved Q0.1 architecture dispositions are recorded in
the linked decision receipt; Q0.2/Q0.3 semantics, implementation review, and
release gates remain open.

## Source evidence at the reviewed pin

| Source | SHA-256 |
|---|---|
| `crates/kairo-ecs-des/src/lib.rs` | `15699dd5982dd83d206ae84f0e81eaa1910ccf0be0ec5424dea3a4c40efda72a` |
| `crates/kairo-ecs-abm/src/lib.rs` | `41d1e7f20860d260c6c02e8afb98aab94ac14cf8e1689acc11357216c32df7f9` |
| `crates/kairo-ecs-state/src/lib.rs` | `88cf420d5cb2b083bd76e8882a83fcc049001e905e197bba2785f72c1ec5aead` |
| `crates/kairo-ecs-core/src/lib.rs` | `cd326a9d244bfe9c949d97b5b23c3afc8249af333516e14f2b6141b8e0c40cf8` |
| `crates/kairo-ecs-types/src/lib.rs` | `9613c19c885183844ab48c157c34e9df718fc3c01363e982a847fb3e8240542b` |
| `crates/kairo-ecs-rng/src/lib.rs` | `68397e53959221c17a9705da1dd6e23a8221df9825e879222cdd9b154029a991` |
| `conductor/contracts/core-contract.md` | `6ad1804b8a5cbc1d7cd4ae5888b2527cc2864ce9b13b972f3e58fbc64622bf60` |
| `conductor/contracts/versioning-compatibility.md` | `7099dfefa5a369a39f1bdc62091cc50348560c6337d07812cf1b42298c23188a` |
| `conductor/research/careops-flow-runtime-contract-proposal-20260929.md` | `508bcb56db337279722a3837c502abb582964f6c9f71cafc1a37702a70ce3b35` |
| `conductor/research/careops-flow-context-codec-proposal-20260929.md` | `b95dfa28adf40309bcceca34a2c966139ffae6386347da008e35eaac51b3162d` |
| `conductor/research/careops-flow-api-review-gate-proposal-20260929.md` | `2d27faf5970a4c18503f050c452b58e0eddc0b4ae6800d4770994031dc89c02f` |

## Consequences and next gates

- Q0.2 writes executable semantics and boundary fixtures against an accepted ADR.
- Q0.3 reserves event kinds and agrees lifecycle/telemetry ownership with
  Tracks 01/04/12; portable checkpoint ownership remains with Track 22/Q4.
- Q0.4 closes only after owner review, Q0.2/Q0.3 outputs, and the manual tie-case
  calculations in the queue plan are independently checked.
- Q1 code work remains gated by D2 and this contract review.
- Legacy APIs stay available; a later deprecation would require a new decision.

## Revisit triggers

Reopen before changing scheduler ordering, modifying the existing `Resource` or
`DESContext` contract, unifying existing ABM storage in place, adding a serialized
checkpoint guarantee, changing RNG streams, exposing the API to bindings,
sharing a resource across PDES LPs, or claiming stable/beta compatibility.
