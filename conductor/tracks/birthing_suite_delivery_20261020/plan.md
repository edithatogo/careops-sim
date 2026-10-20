# Implementation plan: Generic birthing suite simulation delivery

Status: planned revision; no implementation acceptance in this change.

Contract: [cross-service composition](../../design/composition-contracts-20261020.md).
Dispatch requires reviewed packets, upstream capability readback and the single-maintainer harness.

## B0 — Evidence and jurisdiction/site profiles

- [ ] B0.1 Inventory primary perinatal evidence and source versions; distinguish measured distributions from synthetic assumptions and aggregate statistics from identifiable individual hazards.
- [ ] B0.2 Build versioned profiles for population, routes, staffing, eligibility and timing targets; verify current applicable policy rather than hardcode earlier 30/75-minute or parity/duration examples.
- [ ] B0.3 Write schema/provenance/units/range tests and retain missing evidence, uncertainty and unresolved correlations. Evidence intake can precede core implementation.
- [ ] B0.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## B1 — Progression independent of service

- [ ] B1.1 Write failing nominal/induction/protracted cases with competing progression, intervention and deterioration while waiting or in transit.
- [ ] B1.2 Implement explicit patient and service states using qualified core contracts; define hazard updates, censoring, ties, stale-event cancellation and random draw ownership.
- [ ] B1.3 Validate bounded trajectories and observation-only policy decisions; no claim of clinical prediction follows from synthetic conformance.
- [ ] B1.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## B2 — Birth identities and maternal-infant relationships

- [ ] B2.1 Test vaginal and operative birth events, multiple infants, correlated records and duplicate event rejection.
- [ ] B2.2 Implement one-to-many relationships and pre/post-birth state mapping where modeled; record neonatal observations with provenance.
- [ ] B2.3 Verify independent maternal/infant recovery, escalation and discharge with person/episode conservation.
- [ ] B2.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## B3 — Spaces and effective capacity

- [ ] B3.1 Test capability-based room matching, reservation, cleaning, closure and equipment shortages.
- [ ] B3.2 Implement LDR/pool/monitoring/stabilisation profiles through neutral capabilities, with staffing/opening constraints.
- [ ] B3.3 Verify boarding holds, surge demand, occupancy and horizon censoring without premature release.
- [ ] B3.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## B4 — Staffing and resource bundles

- [ ] B4.1 Test configured staffing ratios, shared-team contention, breaks, shift handover and call-in latency.
- [ ] B4.2 Implement qualified bundle or explicitly bounded approximation with role eligibility and no double booking.
- [ ] B4.3 Verify continuous responsibility and safe handover under absence, overtime limits and escort commitments defined by the profile.
- [ ] B4.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## B5 — Escalation and bounded theatre completion

- [ ] B5.1 Test urgency reassessment, queue displacement and refusal to interrupt unsafe active phases; targets may be breached.
- [ ] B5.2 Implement a bounded theatre service with acceptance, procedure-start and birth/completion callbacks; transfer alone is not delivery.
- [ ] B5.3 Measure decision-to-transfer, incision and delivery separately against versioned policy; preserve delay causes and breached targets.
- [ ] B5.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## B6 — Transfers and split destinations

- [ ] B6.1 Test ward/theatre/neonatal/retrieval transfer rejection, reservation expiry, cancellation and delayed acknowledgements.
- [ ] B6.2 Integrate transfer lifecycle and destination models, including maternal/infant split destinations and safe fallback after departure.
- [ ] B6.3 Verify unique ownership, person/episode conservation and origin/destination/escort resource accounting.
- [ ] B6.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## B7 — Two-consumer composition acceptance

- [ ] B7.1 Implement batch CLI and interoperable Arrow/Parquet output using qualified routes; inspect actual files with an independent reader.
- [ ] B7.2 Run multi-day birthing to bounded theatre to maternal/infant destinations, with multiple birth, split destinations, unavailable resources and target breaches.
- [ ] B7.3 Verify replay, named streams, worker invariance, retry/recovery and performance budgets; supply evidence to A7 without declaring clinical validation.
- [ ] B7.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.
