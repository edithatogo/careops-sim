# Specification: reusable healthcare composition contracts

## Purpose

Extract the smallest reusable healthcare contracts from ED and birthing consumers,
while preserving the ED compatibility adapter and qualified baseline. Requirements
and exact invariants are defined by [CC-01–CC-08](../../design/composition-contracts-20261020.md).

## Requirements

- A-R1 / CC-04,06: bounded stage graphs and independent clinical progression; versioned urgency schemes, target clocks and reassessment; no universal clinical scoring or unsafe automatic preemption.
- A-R2 / CC-05: capability-based spaces and equipment with distinct physical/equipped/staffed/open/reserved/occupied/cleaning states and calendars.
- A-R3 / CC-03: explicitly qualified team/room/equipment allocation, rollback and safe interruption. AllOf/AnyOf do not imply atomic grants. A composite-team approximation must disclose its scope.
- A-R4 / CC-02: transfer operation IDs, states, acceptance/reservations, expiry, retry idempotency and authoritative ownership; preserve originating resources until the specified release event.
- A-R5 / CC-01: longitudinal person and episode identity and typed one-to-many relationships; vaginal/operative/multiple birth, readmission and separate recovery outcomes.
- A-R6 / CC-07: incremental ED extraction, compatibility traces, named random streams, schema migration and serial/parallel/recovery equivalence.
- A-R7 / CC-08: observed vs latent state, missingness/censoring, held-out validation and separate activity/cost/funding/outcome/equity ledgers.
- A-R8: neutral transport capability and crew-role profiles; operator names, clinical permissions, routing and targets belong in jurisdiction/site packs.
- A-R9: independent service ports for public/private hospitals, primary care, urgent care, aged/disability care and regional/external transfers; diversion depends on eligibility/access, not triage category alone.

## First consumers and boundaries

A4/A6 build ED → minimal ward → community, including discharge readiness and
actual departure. B5/B7 supply bounded theatre completion callbacks and linked
maternal/infant destinations. A7 joins both before reusable-composition acceptance.
The same single-process scheduler remains authoritative; distributed partitioning
is optional and independently qualified.

Clinical models stay in domain crates; neutral engine mechanics use existing
Kairos owners. No full ward, theatre, diagnostic or network implementation is
required for these boundary fixtures. No stable public API, hardware acceptance,
clinical deployment or fiscal optimisation is implied.

## PR173 acceptance refinements

A0 first qualifies a small experiment lifecycle extraction with existing Kairos
owners, ED/logistics parity, transactional publication and crash recovery. A3 then
qualifies generic work/allocation semantics. Two-consumer evidence and an actual
independent domain adapter precede broad adoption; clinical meaning stays here.
A1/A4/A5 explicitly test hazard-age/draw ownership, transfer logical idempotency
and relationship/lifetime separation. Whole-track 64/66 labels and broad 67–69
ownership do not replace exact capability receipts and disjoint writer packets.
See the [task mapping](../../evidence/pr173-roadmap-integration-20261021.md).
