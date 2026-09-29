# Handoff — DES queues and preemption

## Current state

Q0.1 architecture/compatibility is accepted with local owner disposition and
`conductor/evidence/q0.1-phase-acceptance-20260929.md`. Q0.2.queue,
Q0.2.boundaries and Q0.2.strategies synthetic fixture leaves are integrated;
see their parent evidence records. The SimPy 4.1.2 reference/difference table is
committed in Kairos at `4811af0d4a9e3e392232c95e62f09980e85316be`; its independent
review is pending. Kairos remains synced with `origin/main` at
`384e8546d69f9cbf2746fcb2ab646263256e6dec` and is pinned at the reference-table
commit. There is no queue runtime implementation.

Q0.2 is still open. Before its joined review, add a trace for selecting the
latest admission among equally urgent eligible victims and settle a configurable
same-tick Flow transition budget. Current Kairos source has caller-supplied
`max_events`, but no per-tick Flow transition/notification budget; prose alone
does not satisfy the guardrail requirement. Then complete Q0.2 join review,
Q0.3 event IDs/lifecycle joins, and Q0.4 phase review. Q1 remains gated on Q0
closeout and D2 hosted-CI readiness.

## Next implementation action

Prepare a bounded Luna-sized Q0.2 guardrail packet after reference review and
Kairos source inspection are recorded. Freeze a deterministic victim-tie trace
and explicit finite per-tick budget semantics with a small test cap. Do not begin
Q1 until the Q0.2 fixture joins and Q0.3/Q0.4 are accepted. All runtime phases
Q1-Q5 remain open.

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
