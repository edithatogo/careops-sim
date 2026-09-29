# Handoff — DES queues and preemption

## Current state

Q0.1 architecture/compatibility is accepted with local owner disposition and
`conductor/evidence/q0.1-phase-acceptance-20260929.md`. Q0.2.queue,
Q0.2.boundaries and Q0.2.strategies fixture leaves and the Q0.2.reference
comparison are integrated; see their parent evidence records. The independently
reviewed SimPy 4.1.2 table is committed in Kairos at
`34af27b76158a3feba3ae6fe26ce182c56e4e5e8`. Kairos is synced with `origin/main`
at `384e8546d69f9cbf2746fcb2ab646263256e6dec` and pinned at that reference
commit. There is no queue runtime implementation.

Q0.2 is still open. The spec now settles a configurable finite same-tick Flow
transition budget contract; its numeric default belongs to Q1 runtime configuration.
A reviewed `Q0.2.guardrails` leaf must add traces for selecting the latest admission
among equally urgent eligible victims, zero-duration completion, and exact/over
budget outcomes. Current Kairos source has caller-supplied `max_events`, but no
per-tick Flow guard yet. Then complete Q0.2 join review,
Q0.3 event IDs/lifecycle joins, and Q0.4 phase review. Q1 remains gated on Q0
closeout and D2 hosted-CI readiness.

## Next implementation action

Prepare and execute the bounded Luna-sized `Q0.2.guardrails` packet from the
reviewed recipe and accepted Q0.2.reference receipt. Cover deterministic
victim-tie selection and the explicit finite per-tick budget semantics with a
small test cap. Do not begin
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
