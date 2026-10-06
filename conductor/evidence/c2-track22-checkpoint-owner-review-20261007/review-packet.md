# C2 Track22 portable checkpoint and restart owner packet

**Preparation state:** prepared for Track22 owner review; not sent and not dispositioned. This is a contract decision request, not codec/API implementation authority.

## Decision requested

Track22 owns experiment-runner checkpoint semantics. Please decide whether and how to define a versioned portable checkpoint/restart contract that can support C2, then identify the Track22/other-track owners and ADRs required before implementation. Record a bounded contract and an acceptance oracle before authorizing serializer or restore code. A rejection or narrower supported-state boundary is a valid outcome.

## Exact source context

- Parent review baseline immediately before this packet refresh: `c2cf0af66aee4164e148d64b2f7fb0e5df83b513` (this revision changes review documentation only; Kairos remains at the pin below).
- Parent source-inventory/preparation baseline: `05599280c565b4f31fcc52faae51c52ee4142e30` (historical packet baseline; later parent commits updated evidence and review context only).
- Integrated Kairos pin: `678453838f391f3dd869b79fd502077f63f3a067`.
- C2 plan keeps portable process checkpoint/restart as a separate Track22 gate; C2.5 explicitly covers same-live-Flow continuation and excludes portable serialization and process restart.
- Track22 [`agent-contract.md`](../../../libs/kairos/conductor/tracks/22-experiment-runner-scenario-management/agent-contract.md) prohibits new public Rust APIs without an ADR.
- Track22 [`handoff.md`](../../../libs/kairos/conductor/tracks/22-experiment-runner-scenario-management/handoff.md) describes `checkpoint` as writing a scaffold manifest and `resume` as path validation plus a scaffold request; it explicitly leaves real checkpoint-state and resume semantics as follow-up.
- Kairos [`ADR-0006-flow-runtime-identity.md`](../../../libs/kairos/conductor/design/calibration/ADR-0006-flow-runtime-identity.md) requires explicit rebinding because runtime identity cannot be serialized.
- The [C2 specification](../../tracks/empirical_calibration_20260925/spec.md), clause C-06, requires candidate rankings, canonical integer traces, and IDs to survive repeat runs, checkpoints, and 1/2/N workers, with numeric comparisons using declared tolerances. The [C5 plan](../../tracks/empirical_calibration_20260925/plan.md) assigns worker/resume equivalence to C5; its manual exit and [test matrix](../../tracks/empirical_calibration_20260925/test-matrix.md) add mid-study crash/resume and permuted completion-order cases. One checkpoint fixture cannot establish all C-06 requirements or complete the C5 matrix. These requirements motivate contract work; they do not establish that portable process restoration is already implemented or accepted. Current C2 evidence does not establish restoration in a new process.
- The superseding [C2.4 current readback](../c2-current-pin-rerun-rust199-20261007-6784538/current-readback-6784538.md) records the current source pin and open gates (readback SHA-256 `ffa4baabf29fc6b9e5292dfcf35fac78e126e8cac5d198cb18ae3be12c3628f7`).
- The coordinator [current-pin runtime receipt](../c2-current-pin-rerun-rust199-20261007-6784538/acceptance-rust199.json), SHA-256 `ed0b0b9bf19290d993a9aed7f0bce7df7dfa4d7a786304f021982fbe75b7f04f`, reruns the C2.4 package suite at Kairos `678453838f391f3dd869b79fd502077f63f3a067` using Rust 1.99.0 only (**455 passed, 0 failed, 5 ignored across 43 result groups**; strict all-target Clippy and formatting passed). The receipt README SHA-256 is `30972edf21df199a06d012b22938e3778d02a931d274a1b9c3a6440ed5c59ae6`. This covers local package verification only and provides no checkpoint, fresh-process restore, hosted-check, or phase-acceptance evidence.

## Current source-state inventory

The following are implementation constraints visible at Kairos `6784538`, not a proposed byte layout or a claim that every listed field must be serialized. Track owners must decide which state is encoded, rebound, regenerated under a proven invariant, or rejected:

| State area | Current code constraint | Existing owner boundary |
| --- | --- | --- |
| Scheduler frontier | Time, priority, and insertion sequence determine event order; event IDs and live/cancelled membership preserve event identity and references. Summary/preview views are not a complete restore image. | Track01 owns scheduler/entity/RNG ordering. |
| World and Flow | Allocator generations, component/resource state, requests, leases, work progress, cleanup/callback state and private registries are not exposed as a unified snapshot. | Track03 owns Flow/DES mode, policy and execution state. |
| Typed extensions | Components and contexts can be arbitrary type-erased values; handlers, hooks and factories include process-local function registrations. | Component and adapter owners must define codecs/handler rebinding or fail-closed cases. |
| Calibration streams | Logical seed identity and draw positions affect deterministic outcomes; the seed-collision registry currently lacks a snapshot interface. | Track21 owns calibration providers, streams, ledger and bridge semantics. |
| Runtime identity | `FlowRuntimeIdentity` is process-local and nonserializable; restore needs an explicit new-runtime rebind decision. | ADR-0006; Track03/21 decision with Track22 orchestration. |
| Runner state | Current `checkpoint` writes a scaffold manifest and `resume` writes a request after path validation; neither captures or restores simulation state. | Track22 owns runner/checkpoint orchestration and artifact semantics. |

Missing codecs or handlers, unobserved dispatches, in-flight callbacks/submissions, external mutable aliases, incompatible code/config/schema, unregistered streams/topologies, and corrupt/partial/oversized artifacts are candidate fail-closed cases for owner decision. The existing C2.5 same-live-Flow handoff does not provide portable serialization. C-06 states a checkpoint outcome; the broader C5 test-matrix oracle adds controlled worker scheduling and mid-study restart requirements, so a C2 checkpoint result alone must not be presented as full C-06/C5 acceptance.

## Contract questions for the owner

1. What is the supported checkpoint boundary: whole experiment-runner state, selected C2 calibration state, or a narrower explicitly rejected-on-unsupported-state boundary?
2. Which state must be captured or rebound: scenario/config/source hashes, seed manifest and each RNG stream/draw position, simulation time and queued-event ordering, world/resource state, active/suspended Flow work and identity, transit requests/arrivals, calibration provider/ledger state, and worker topology? Identify which components are owned by Track22 versus Tracks 01/03/04/21.
3. What compatibility policy applies to schema versions, code/config drift, missing plugins/adapters, changed worker count, and runtime-ID/generation changes? Which inputs must cause fail-closed rejection rather than best-effort restore?
4. What observable equivalence oracle proves a fresh process has restored correctly (including IDs, rankings, traces, event order and output hashes) versus an uninterrupted control? Which crash points and corruption/partial-write cases are required?
5. What atomicity/integrity, path handling, private-data and resource-limit requirements apply to the checkpoint artifact?
6. Which reviewed ADR/API changes and owners are prerequisites for implementation, and what exact leaf can be authorized first?

These are questions, not proposed answers. This packet intentionally does not select a serialization format, state layout, scheduler order, RNG reconstruction rule, rebinding algorithm, public API, or compatibility promise.

The coordinator prepared a separate [nonbinding contract-options proposal](../c2-track22-checkpoint-contract-options-20261007/proposal.md) to make the decision points and possible bounded first sequence reviewable. It labels fresh-process restoration as a proposed acceptance interpretation for owner review, not an existing C2.4 plan statement. It is not an owner disposition or implementation authority; Track22 and contributing owners may revise or reject it.

## Current evidence boundary

Existing C2.5 evidence establishes same-live-Flow continuation only. Track22 smoke `resume-plan` describes a planned checkpoint scenario but is not execution evidence for portable restoration. The CLI checkpoint/resume scaffold is not a codec, process restart, or accepted restore. No C2 portable checkpoint acceptance is claimed.

## Owner disposition — Track22 to complete

- Reviewer identity and Track22 role:
- Review date and exact parent/Kairos pin:
- Contract decision and supported boundary:
- Required ADR(s), contributing owners, and dependency order:
- First bounded implementation task and acceptance oracle:
- Explicit unsupported states/fail-closed cases:
- Track22 disposition (`accepted`, `accepted with follow-up`, `rejected`, or `not reviewed`):

## Separate gates

This packet does not accept the Track22 gate, C2.4 Conductor phase, full C2, Track03 owner review, hosted exact-head checks, release, or clinical acceptance. No owner response, workflow action, or external publication is recorded here.
