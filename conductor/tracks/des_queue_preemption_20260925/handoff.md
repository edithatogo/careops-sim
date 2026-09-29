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

All five planned Q0.2 leaves have accepted artifacts: priority/FIFO,
deadline/capacity, Suspend/Abort/Restart, the SimPy 4.1.2 comparison, and the
victim-tie/zero-time guardrail fixture. The current fixture remains synthetic;
there is no queue runtime implementation, and the per-tick budget's numeric
default is a Q1 configuration decision.

## Next implementation action

Run the Q0.2 parent join review across all five artifacts and the phase exit
criteria. If accepted, close Q0.2 and continue with Q0.3 event-kind/lifecycle
joins, then Q0.4 phase review. Q1 remains gated on Q0 closeout and D2 hosted-CI
readiness. All runtime phases Q1-Q5 remain open.

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
