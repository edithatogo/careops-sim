# Implementation plan: Generic birthing suite simulation delivery

Status: generic synthetic engineering in progress. B0–B2 have scoped acceptance; full specification approval, clinical validation and site acceptance remain open.

Contract: [cross-service composition](../../design/composition-contracts-20261020.md).
Dispatch requires reviewed packets, upstream capability readback and the single-maintainer harness.

## B0 — Evidence and jurisdiction/site profiles

- [x] B0.1 Inventory primary perinatal evidence and source versions; distinguish measured distributions from synthetic assumptions and aggregate statistics from identifiable individual hazards. Accepted after reviewer-agent review at implementation commit `da7aa6f`; evidence: [public perinatal evidence inventory](../../evidence/b0.1-perinatal-evidence-inventory-20261027/README.md).
- [x] B0.2 Build versioned profiles for population, routes, staffing, eligibility and timing targets; verify current applicable policy rather than hardcode earlier 30/75-minute or parity/duration examples. Reference profile accepted at `a8f3e08`; metadata corrections at `6b37893`; all three reviewer-agent findings fixed and re-reviewed at `6b37893` with no further findings. Evidence: [profile and review receipt](../../evidence/b0.2-qld-reference-profile-20261027/README.md). This is public reference evidence, not clinical validation or site-policy acceptance.
- [x] B0.3 Write schema/provenance/units/range tests and retain missing evidence, uncertainty and unresolved correlations. Evidence intake can precede core implementation. Candidate implementation `76d71d1` was corrected at `ef543aa` after reviewer-agent found malformed rows were ignored; fresh reviewer-agent re-review passed all 11 focused tests and found no further findings. Evidence: [profile contract test receipt](../../evidence/b0.3-profile-contract-tests-20261027/README.md). This validates the reference input contract only, not clinical policy or model behavior.
- [x] B0.4 Conductor — review and verify phase (workflow.md). Accepted at phase-review candidate `e1fe676` with receipt consistency corrections at `03bc3fc`; fresh reviewer-agent phase re-review at `03bc3fc` found no remaining findings. Evidence: [B0 phase review](../../evidence/b0.4-phase-review-20261027/README.md). B0 closes reference-evidence and input-contract checks only; birthing remains proposed and no clinical/site/runtime acceptance follows.

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## B1 — Progression independent of service

- [x] B1.1 Write failing nominal/induction/protracted cases with competing progression, intervention and deterioration while waiting or in transit. Synthetic case-oracle fixture accepted at `6cf1a2f` after reviewer-agent review; it defines expected traces only and does not execute a birthing runtime. See [B1.1 evidence](../../evidence/b1.1-progression-cases-20261027/README.md).
- [x] B1.2 Implement explicit patient and service states using qualified core contracts; define hazard updates, censoring, ties, stale-event cancellation and random draw ownership. Bind B1.1 oracles to executable runtime tests and demonstrate red-to-green behavior; specifically schedule the stale progression event and prove it is rejected after the deterioration state change. Accepted at reviewed commit `7003413`; reviewer-agent found and prompted a hazard-reschedule correction from `5ce4036`, then fresh review found no remaining issues. See [B1.2 evidence](../../evidence/b1.2-runtime-binding-20261027/README.md). Local synthetic conformance only.
- [x] B1.3 Validate bounded trajectories and observation-only policy decisions; no claim of clinical prediction follows from synthetic conformance. Accepted at reviewed commit `56fbef6`; reviewer-agent requested full versioned-urgency provenance equality after `b8fc9ac`, which was fixed and freshly reviewed with no remaining findings. See [B1.3 evidence](../../evidence/b1.3-trajectory-observation-20261027/README.md). The 16-seed bounded run is synthetic only.
- [x] B1.4 Conductor — review and verify phase (workflow.md). Accepted at integrated candidate `d5b3720`; reviewer-agent phase review found no actionable findings. Fresh checks covered format, 7 birthing conformance tests, 13 pathway tests, 6 policy tests, 401 task records and 2,006 Conductor links. Receipt: [B1 phase review](evidence/b1.4-phase-review-20261027/README.md). Synthetic implementation conformance only; birthing remains proposed and `spec_approved=false`.

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## B2 — Birth identities and maternal-infant relationships

- [x] B2.1 Test vaginal and operative birth events, multiple infants, correlated records and duplicate event rejection. Both routes, multiple births and conflict/rollback controls are accepted at frozen runtime `871c478`; subsequent test-only controls `b01c885` are locally qualified. See [B2 acceptance](../../evidence/core-a7-prerequisites-20261102/B2.md).
- [x] B2.2 Implement one-to-many relationships and pre/post-birth state mapping where modeled; record neonatal observations with provenance. Qualified A5 identity/message contracts plus birth-time provenance are accepted at frozen runtime `871c478`; fetal identities remain unmodeled. See [B2 acceptance](../../evidence/core-a7-prerequisites-20261102/B2.md).
- [x] B2.3 Verify independent maternal/infant recovery, escalation and discharge with person/episode conservation. Per-delivery/person recovery and explicit core episode accounting are accepted at `871c478` after review fixes `d3c8fc5` and `9702fac`. See [B2 acceptance](../../evidence/core-a7-prerequisites-20261102/B2.md).
- [x] B2.4 Conductor — review and verify phase (workflow.md). Independent review found no remaining B2 runtime findings at `9702fac`; all 17 hosted checks passed at exact runtime tree `871c478` (one main-only attestation skip). Test-only `b01c885` review/13 tests are local proof; final combined-head hosted gates remain mandatory. See [B2 acceptance](../../evidence/core-a7-prerequisites-20261102/B2.md).

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
