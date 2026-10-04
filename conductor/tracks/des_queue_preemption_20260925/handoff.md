# Handoff — DES queues and preemption

## Current development state — 2026-10-04

Q0–Q3 and Q4.1–Q4.4 have accepted bounded implementation evidence. The queue
runtime, priority/preemption strategies, typed Flow continuations, shared DES/ABM
world, immutable lifecycle snapshots and opt-in Arrow lifecycle sidecar exist.
The synthetic staff/bed/cleaning example is runnable and compares exact live
state/output across `step` and bounded `run_for` execution.

Q4.5 is accepted for bounded development; parent publication is pending. Canonical
current stable Rust 1.99.0 has passed local Q4 tests and exact-source Linux/macOS
native-owner CI at `1123ad4bd0c9121a4a8f5f0be1229fbafc9861f6`, run
[37192093779](https://github.com/edithatogo/kairos/actions/runs/37192093779).
Final governance head `1125b5268bb349a5befe12f5789045042faab3e3` passed
[37192692770](https://github.com/edithatogo/kairos/actions/runs/37192692770).
The parent pin/contract is updated; parent PR checks/merge remain pending. The
qualified parent pin is controlled by `conductor/current-state.json` and the D2.4 contract;
this handoff does not advance it.

After Q4.5 acceptance, the next queue task is Q5.1: deterministic conformance
fixtures across boundary and strategy cases. Q5 benchmarks, worker invariance
and compatibility review remain required. Portable checkpoint restore belongs
to Track 22; the public lifecycle snapshot extension remains experimental under
Track 25. Strict priority has no starvation guarantee, and staged single-resource
claims do not provide atomic multi-resource acquisition. No complete ED MVP,
clinical calibration, advanced backend or release readiness is claimed here.

## Historical Q0 handoff

The following records the earlier Q0 planning checkpoint and its then-current
implementation status; it is superseded by the development state above.

## Current state

Q0.1 architecture/compatibility is accepted with local owner disposition and
`conductor/evidence/q0.1-phase-acceptance-20260929.md`. Q0.2.queue,
Q0.2.boundaries and Q0.2.strategies fixture leaves and the Q0.2.reference
comparison are integrated; see their parent evidence records. The SimPy 4.1.2 comparison and Q0.2 design fixtures are committed in Kairos on
the owner-approved development branch; the current parent pin is recorded in
`conductor/current-state.json`. Kairos remains synced with `origin/main` at
`384e8546d69f9cbf2746fcb2ab646263256e6dec`. There is no queue runtime
implementation.

The original Q0.2 queue/FIFO, deadline/capacity, Suspend/Abort/Restart,
SimPy-reference, and victim-tie/zero-time guardrail artifacts are integrated.
An independent parent-join review found four follow-ups before Q0.2 closeout:
use a distinct request identity after cancellation; add same-tick cancel/reprioritize
ordering cases across insertion order and scheduler priority; count a delivered
notification in the finite-budget oracle; update stale reference-note coverage
language. The bounded `Q0.2.join_followups` leaf is now in the reviewed MVP DAG.
The per-tick budget remains proposed design; its numeric runtime default belongs
to Q1. There is no queue runtime implementation.

## Next implementation action

Q0.2 design criteria are accepted. The next queue-track milestone is Q0.3
event-kind/lifecycle joins, followed by Q0.4 phase review. The global serial
scheduler currently selects P0.2; runtime Q1 remains gated on Q0 closeout and D2. Q1 remains gated on Q0 closeout and D2 hosted-CI readiness.
All runtime phases Q1-Q5 remain open.

## Evidence to carry forward

Record source/lockfile/toolchain versions, commands actually executed, fixture
seeds, normalized input and output hashes, compatibility/performance results and
remaining limitations. Use the test matrix as the acceptance mapping. Planning
validation is documented in [the review record](../../planning-review.md).

## Release and synchronization

Update affected upstream owner plan/handoff and authoritative registry/phase
records after implementation evidence exists. Keep Kairos implementation commits
separate from the parent integration pin. Do not mark GPU, distributed, browser
or empirical clinical validation complete from CPU/synthetic evidence.
