# Implementation plan: Multi-domain core simulation abstractions and egress boundaries

Status: planned revision; no implementation acceptance in this change.

Contract: [cross-service composition](../../design/composition-contracts-20261020.md).
Dispatch requires reviewed packets, upstream capability readback and the single-maintainer harness.

## A0 — Small contracts, identities and extraction boundaries

- [ ] A0.1 Freeze CC-01 through CC-08 from the composition design, with person/episode IDs, one virtual clock, schema versions and stream ownership; keep public APIs provisional until owner review.
- [ ] A0.2 Design two consumers and neutral fixtures before extraction; retain the ED adapter and define the minimal careops-core crate with no clinical/site defaults.
- [ ] A0.3 Write contract-negative cases and layered JurisdictionProfile/SiteProfile schema and loader tests (precedence, missing source, incompatible version); compile the smallest shared slice; bind upstream capability receipts and reserve exact paths.
- [ ] A0.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A1 — Pathways, clinical progression and policy clocks

- [ ] A1.1 Write failing cases for branching/loop bounds, invalid transitions, progression during waiting, stale events, ties and zero-target accounting.
- [ ] A1.2 Implement a bounded configurable pathway adapter with independently scheduled patient progression; retain the ED compatibility path.
- [ ] A1.3 Implement versioned classification and policy clocks with reassessment, pause semantics and separate urgency/target/priority/interruption rules; prohibit future-outcome access by decision policies.
- [ ] A1.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A2 — Effective capacity and calendars

- [ ] A2.1 Write capability, capacity conservation and calendar-boundary cases, including shared pools and daylight-saving mapping.
- [ ] A2.2 Implement separate physical/equipped/staffed/open/reserved/occupied/cleaning states and neutral capability matching.
- [ ] A2.3 Qualify closure, maintenance and turnover without silently evicting active work; distinguish unavailable capacity from zero demand.
- [ ] A2.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A3 — Team bundles and safe interruption

- [ ] A3.1 Freeze and test bundle allocation semantics with Kairos owners: all-or-none grant versus explicit staged approximation; AllOf is not an atomic allocator.
- [ ] A3.2 Implement the qualified mechanism or a labelled bounded composite-team fallback; test contention, partial failures, rollback, expiry, cancellation and starvation policy.
- [ ] A3.3 Implement staff calendars, breaks, handover, escort retention and on-call latency; test no double booking and non-interruptible procedure phases.
- [ ] A3.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A4 — Transfer ownership and minimal ward/community boundary

- [ ] A4.1 Write transfer-state and conservation oracles for request/offer/reservation/departure/transit/arrival/handover plus rejection, cancellation and expiry.
- [ ] A4.2 Implement versioned transfer envelopes with operation IDs, idempotent retries and explicit ownership/resource release; test duplicate and delayed acknowledgements.
- [ ] A4.3 Build minimal ward/community boundary models distinguishing clinically ready from actual departure, destination availability, transport delay and boarding feedback.
- [ ] A4.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A5 — Relationships and all delivery routes

- [ ] A5.1 Write identity/cardinality cases for readmission, concurrent episodes, recycled ECS handles, multiple births and operative births.
- [ ] A5.2 Implement typed one-to-many relationships and birth/entry/exit/death accounting; mother and infant can progress, recover and transfer independently.
- [ ] A5.3 Qualify fetal-to-post-birth mapping where modeled, split destinations and duplicate birth-message rejection without encoding maternal terminology in Kairos.
- [ ] A5.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A6 — Incremental ED extraction and first composition proof

- [ ] A6.1 Migrate ED through a compatibility adapter in small reviewed slices; prove unchanged fixture byte parity and document schema migration separately.
- [ ] A6.2 Run ED to minimal ward to community with rejected/expired offers, transport cancellation, delayed discharge and horizon censoring.
- [ ] A6.3 Verify conservation, stable stream namespaces, unrelated-module noninterference, 1/2/4 replication workers and interruption/recovery; logical hospital boundaries do not mandate PDES partitions.
- [ ] A6.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A7 — Two-consumer evaluation and reuse join

- [ ] A7.1 Join the ED composition evidence with B7 birthing/theatre/maternal-infant evidence before declaring reusable cross-service contracts qualified.
- [ ] A7.2 Extend existing observation/calibration contracts for delayed/missing observations, exclusions, censoring, latent demand, identifiability and held-out validation.
- [ ] A7.3 Define separate activity/resource/cost/funding/outcome/equity ledgers and versioned offline fiscal exchange; test mismatched units, periods, duplicate accounting and policy lag before optimisation.
- [ ] A7.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.
