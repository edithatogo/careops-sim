# Handoff — DES queues and preemption

## Current state

Q0.1 architecture/compatibility is accepted with local owner disposition and
`conductor/evidence/q0.1-phase-acceptance-20260929.md`. Q0.2.queue,
Q0.2.boundaries and Q0.2.strategies fixture leaves and the Q0.2.reference
comparison are integrated; see their parent evidence records. The independently
reviewed SimPy 4.1.2 table is committed in Kairos at
`a7e3397d8f18f1f856da155263b6cc9533b8d097`. Kairos is synced with `origin/main`
at `384e8546d69f9cbf2746fcb2ab646263256e6dec` and pinned at that reference
commit. There is no queue runtime implementation.

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

Execute `Q0.2.join_followups` from its reviewed Luna packet, then rerun the full
Q0.2 parent join against spec, plan, test matrix, report 27 reconciliation, and
all artifact hashes. If accepted, proceed to Q0.3 event-kind/lifecycle joins and
Q0.4 phase review. Q1 remains gated on Q0 closeout and D2 hosted-CI readiness.
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
