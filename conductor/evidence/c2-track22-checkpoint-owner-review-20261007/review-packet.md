# C2 Track22 portable checkpoint and restart owner packet

**Preparation state:** prepared for Track22 owner review; not sent and not dispositioned. This is a contract decision request, not codec/API implementation authority.

## Decision requested

Track22 owns experiment-runner checkpoint semantics. Please decide whether and how to define a versioned portable checkpoint/restart contract that can support C2, then identify the Track22/other-track owners and ADRs required before implementation. Record a bounded contract and an acceptance oracle before authorizing serializer or restore code. A rejection or narrower supported-state boundary is a valid outcome.

## Exact source context

- CareOps parent source commit: `77f85cd40e50e46130e8e00ceff27393bd792f1a`.
- Integrated Kairos pin: `6303a2552257f1fcbd2aaa002d38352a2a467592`.
- C2 plan keeps portable process checkpoint/restart as a separate Track22 gate; C2.5 explicitly covers same-live-Flow continuation and excludes portable serialization and process restart.
- Track22 [`agent-contract.md`](../../../libs/kairos/conductor/tracks/22-experiment-runner-scenario-management/agent-contract.md) prohibits new public Rust APIs without an ADR.
- Track22 [`handoff.md`](../../../libs/kairos/conductor/tracks/22-experiment-runner-scenario-management/handoff.md) describes `checkpoint` as writing a scaffold manifest and `resume` as path validation plus a scaffold request; it explicitly leaves real checkpoint-state and resume semantics as follow-up.
- Kairos [`ADR-0006-flow-runtime-identity.md`](../../../libs/kairos/conductor/design/calibration/ADR-0006-flow-runtime-identity.md) requires explicit rebinding because runtime identity cannot be serialized.
- The [C2 specification](../../tracks/empirical_calibration_20260925/spec.md), clause C-06, requires candidate rankings, canonical integer traces, and IDs to survive repeat runs, checkpoints, and 1/2/N workers, with numeric comparisons using declared tolerances. This requirement motivates checkpoint-contract work; it does not establish that portable process restoration is already implemented or accepted. Current C2 evidence does not establish restoration in a new process.

## Contract questions for the owner

1. What is the supported checkpoint boundary: whole experiment-runner state, selected C2 calibration state, or a narrower explicitly rejected-on-unsupported-state boundary?
2. Which state must be captured or rebound: scenario/config/source hashes, seed manifest and each RNG stream/draw position, simulation time and queued-event ordering, world/resource state, active/suspended Flow work and identity, transit requests/arrivals, calibration provider/ledger state, and worker topology? Identify which components are owned by Track22 versus Tracks 01/03/04/21.
3. What compatibility policy applies to schema versions, code/config drift, missing plugins/adapters, changed worker count, and runtime-ID/generation changes? Which inputs must cause fail-closed rejection rather than best-effort restore?
4. What observable equivalence oracle proves a fresh process has restored correctly (including IDs, rankings, traces, event order and output hashes) versus an uninterrupted control? Which crash points and corruption/partial-write cases are required?
5. What atomicity/integrity, path handling, private-data and resource-limit requirements apply to the checkpoint artifact?
6. Which reviewed ADR/API changes and owners are prerequisites for implementation, and what exact leaf can be authorized first?

These are questions, not proposed answers. This packet intentionally does not select a serialization format, state layout, scheduler order, RNG reconstruction rule, rebinding algorithm, public API, or compatibility promise.

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
