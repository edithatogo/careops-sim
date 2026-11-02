# Implementation plan: Multi-domain core simulation abstractions and egress boundaries

## Repository scope

This simulation plan applies to **careops-sim**, whose source checkout is
`${PRIVATE_HOME}/Documents/careops-sim` (GitHub `edithatogo/careops-sim`).
Kairos is its separately owned upstream simulation engine. The distinct
`${PRIVATE_VOLUME}/GitHub/careops` portfolio repository is not the target of
these C-R0–C-R7 recommendations. No change to that repository is made by this
21 October incorporation. Older shorthand “CareOps” here denotes careops-sim
unless a paragraph explicitly identifies the separate portfolio repository.

Status: implementation in progress. A0–A6 are accepted locally. A7.1–A7.4 remain incomplete; A7.1 depends on birthing B7.4.

Contract: [cross-service composition](../../design/composition-contracts-20261020.md).
Dispatch requires reviewed packets, upstream capability readback and the single-maintainer harness.

## A0 — Small contracts, identities and extraction boundaries

- [x] A0.1 Freeze CC-01 through CC-08, stable identities, virtual clock, schema versions and stream ownership. Map each consumed Kairos 64/66 capability to exact evidence rather than whole-track completion; resolve 67 core/state/ABM overlap and 68/69 DES overlap to exact-file writers and contract handoffs before dispatch. Public API/checkpoint/schema compatibility remains an acceptance gate.
- [x] A0.2 Qualify the first small extraction from batch_runner/api_runner using existing Kairos experiment interfaces before resource extraction: seed partitioning, worker invariance, progress/cancel, manifests, resume and recovery. Preserve the CareOps facade; prove ED plus a nonclinical logistics consumer, 1/2/4 worker and restart parity. Cancellation cannot publish partial output as complete; recover a crash between output write and completion record. Benchmark further movement before broad extraction; bind separate upstream owner packets.
- [x] A0.3 Define the minimal careops-core slice with clinical/site defaults outside Kairos; write contract-negative cases and layered JurisdictionProfile/SiteProfile schema and loader tests (precedence, missing source, incompatible version). Retain old adapters until compatibility and recovery pass; qualify a separate real domain adapter before broad generic adoption.
- [x] A0.4 Conductor — review and verify phase (workflow.md); the later integration finding was addressed at `88135e4` with [review evidence](../../evidence/pr-183-review-fix-20261026/README.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A1 — Pathways, clinical progression and policy clocks

- [x] A1.1 Write failing cases for branching/loop bounds, invalid transitions, progression during waiting, elapsed-age/residual-time and covariate changes, stale events, ties and zero-target accounting. Require analytic exponential competing-risk and zero-hazard controls, unchanged-covariate invariance, time-varying/censoring references and explicit draw ownership; failed calibration cannot resample unrelated clocks.
- [x] A1.2 Implement a bounded configurable pathway adapter with independently scheduled patient progression; retain the ED compatibility path.
- [x] A1.3 Implement versioned classification and policy clocks with reassessment, pause semantics and separate urgency/target/priority/interruption rules; prohibit future-outcome access by decision policies. Accepted at `f46b369` per [A1.3 review evidence](../../evidence/a1.3-review-20261025/README.md).
- [x] A1.4 Conductor — review and verify phase (workflow.md). Accepted at
  `312a238` per [A1 phase review](../../evidence/a1.4-phase-review-20261026/README.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A2 — Effective capacity and calendars

- [x] A2.1 Write capability, capacity conservation and calendar-boundary cases, including shared pools and daylight-saving mapping. Contract cases frozen in [ADR-0010](../../decisions/ADR-0010-effective-capacity-and-calendar-boundaries.md); reviewer-agent acceptance and packet checks are recorded in [A2.1 evidence](../../evidence/a2.1-capacity-contract-20261026/README.md).
- [x] A2.2 Implement separate physical/equipped/staffed/open/reserved/occupied/cleaning states and neutral capability matching. Integrated at `df0612a`; focused tests, package tests, clippy, and reviewer-agent findings are recorded in [A2.2 evidence](../../evidence/a2.2-capacity-core-20261027/README.md).
- [x] A2.3 Qualify closure, maintenance and turnover without silently evicting active work; distinguish unavailable capacity from zero demand. Accepted at `9f0327c`; see [A2 phase review](../../evidence/a2.4-phase-review-20261027/README.md).
- [x] A2.4 Conductor — review and verify phase (workflow.md). Accepted at `9f0327c` per [A2 phase review](../../evidence/a2.4-phase-review-20261027/README.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A3 — Team bundles and safe interruption

- [x] A3.1 After the A0 experiment-lifecycle gate, freeze generic allocation leases, held work, continuation, eligibility/dispatch hooks and safe interruption against the pinned Kairos capability; distinguish all-or-none grant from staged acquisition. AllOf is not an atomic allocator; unsupported claim_atomic_multi_resource is not a capability. Accepted in [ADR-0011](../../decisions/ADR-0011-staged-resource-claims-and-interruption.md) after reviewer-agent review; see [A3.1 evidence](../../evidence/a3.1-allocation-contract-20261027/README.md).
- [x] A3.2 Implement staged single-resource claim coordination against the pinned Kairos grant/release surface; test capacity-one contention, timeout-at-grant ties, partial failure, cancellation, expiry, rollback, interruption and stale completion. Conserve claims, retain release failures and test bounded FIFO issuance. Accepted at `34c83d2`; see [A3.2 evidence](../../evidence/a3.2-staged-allocation-core-20261027/README.md). Atomic multi-resource grant and portable checkpoint remain unverified.
- [x] A3.3 Implement staff calendars, breaks, handover, escort retention and on-call latency; test no double booking and non-interruptible procedure phases. Accepted at `1a5b620` per [A3.3 evidence](../../evidence/a3.3-staff-calendar-and-handover-20261027/README.md).
- [x] A3.4 Conductor — review and verify phase (workflow.md). Accepted at `3bce4b2` per [A3 phase review](../../evidence/a3.4-phase-review-20261027/README.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A4 — Transfer ownership and minimal ward/community boundary

- [x] A4.1 Write transfer-state and conservation oracles for request/offer/reservation/departure/transit/arrival/handover plus rejection, cancellation and expiry. Accepted at `54a01fa` per [A4.1 contract evidence](../../evidence/a4.1-transfer-contract-20261027/README.md) and [ADR-0012](../../decisions/ADR-0012-transfer-lifecycle-and-conservation.md).
- [x] A4.2 Implement versioned transfer envelopes with operation IDs, idempotent logical effects and explicit ownership/resource release on one scheduler. Test duplicate/reordered/delayed acknowledgement, cancellation after reservation, expiry-at-arrival and checkpoint/restart; exactly-once domain effects do not imply network exactly-once delivery. Accepted at `ecb24f0` after 24 focused contract tests, core Clippy, formatting and final reviewer-agent pass; see [A4.2 evidence](../../evidence/a4.2-transfer-core-20261027/README.md).
- [x] A4.3 Build minimal ward/community boundary models distinguishing clinically ready from actual departure, destination availability, transport delay and boarding feedback. Accepted at `3bfaf0e` after 34 transfer/boundary tests, core Clippy, formatting and agent review; see [A4.3 evidence](../../evidence/a4.3-ward-community-contract-20261027/README.md) and [ADR-0013](../../decisions/ADR-0013-ward-community-boundary.md).
- [x] A4.4 Conductor — review and verify phase (workflow.md). Accepted at `095f79b` per [A4 phase review](../../evidence/a4.4-phase-review-20261027/README.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A5 — Relationships and all delivery routes

- [x] A5.1 Write identity/cardinality cases for readmission, concurrent episodes, stale/recycled ECS handles, cycles, duplicate spawn, deterministic identity/streams, multiple/operative births and checkpoint restore; separate typed relationships from destruction policy. Accepted at `0c9f5a9` per [A5.1 contract evidence](../../evidence/a5.1-identity-contract-20261027/README.md).
- [x] A5.2 Implement typed one-to-many relationships and birth/entry/exit/death accounting; mother and infant progress, recover and transfer independently. Removing a parent relationship must not implicitly destroy a surviving dependent; test neutral and clinical adapters. Accepted at `bc1f88b` after independent reviewer-agent re-review found no issues; see [A5.2 evidence](../../evidence/a5.2-identity-ledger-20261027/README.md).
- [x] A5.3 Qualify fetal-to-post-birth mapping where modeled, split destinations and duplicate birth-message rejection without encoding maternal terminology in Kairos. Accepted at `887e967` after independent reviewer-agent review found no remaining issues; fetal identity is not modeled in current CareOps source, see [A5.3 evidence](../../evidence/a5.3-birth-message-20261027/README.md).
- [x] A5.4 Conductor — review and verify phase (workflow.md). Accepted at `79570f5` after independent reviewer-agent phase review and full workspace/Conductor gates; see [A5.4 phase evidence](../../evidence/a5.4-phase-review-20261027/README.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A6 — Incremental ED extraction and first composition proof

- [x] A6.1 Migrate ED through a compatibility adapter in small reviewed slices; prove unchanged fixture byte parity and document schema migration separately. Accepted at `e70a35d` with whitespace correction `e547b62` after independent reviewer-agent re-review found no remaining issues; see [A6.1 evidence](../../evidence/a6.1-ed-compatibility-20261027/README.md).
- [x] A6.2 Run ED to minimal ward to community with rejected/expired offers, transport cancellation, delayed discharge and horizon censoring. Accepted at `24f55b1` after independent reviewer-agent review found no actionable findings; see [A6.2 evidence](../../evidence/a6.2-ed-ward-community-20261027/README.md).
- [x] A6.3 Verify conservation, stable stream namespaces, unrelated-module noninterference, 1/2/4 replication workers and interruption/recovery; logical hospital boundaries do not mandate PDES partitions. Accepted at implementation `b814bfb` and scope correction `deba8a2` after reviewer-agent re-review found no remaining wording issues; see [A6.3 evidence](../../evidence/a6.3-composition-scheduler-parity-20261027/README.md). The evidence is sequential composition within one process and does not establish a shared cross-domain event loop.
- [x] A6.4 Conductor — review and verify phase (workflow.md). Accepted at source `f2e09a3` after independent reviewer-agent review and full workspace/Conductor gates; see [A6 phase review](../../evidence/a6.4-phase-review-20261027/README.md). The phase closes local synthetic conformance only; A7.1 remains gated on birthing B7.4.

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## A7 — Two-consumer evaluation and reuse join

- [ ] A7.1 Join the ED composition evidence with B7 birthing/theatre/maternal-infant evidence before declaring reusable cross-service contracts qualified.
- [ ] A7.2 Extend existing observation/calibration contracts for delayed/missing observations, exclusions, censoring, latent demand, identifiability and held-out validation.
- [ ] A7.3 Define separate activity/resource/cost/funding/outcome/equity ledgers and versioned offline fiscal exchange; test mismatched units, periods, duplicate accounting and policy lag before optimisation.
- [ ] A7.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.
