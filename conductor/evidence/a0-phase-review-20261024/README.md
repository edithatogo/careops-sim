# A0 Phase Review — Small Contracts, Identities and Extraction Boundaries

**Phase Disposition: ACCEPTED**  
**Review Date:** `2026-10-24`  
**Active Track:** `careops_core_domain_abstractions_20261020`  
**Active Phase:** `A0` (Small contracts, identities and extraction boundaries)  
**Parent Workspace:** `careops-sim` (`crates/careops-core`, `crates/careops-ed`)  
**Upstream Engine:** `libs/kairos` (retained pin `44261eb5cf8275f132f63a3396903d116d701dbe`)

---

## 1. Executive Summary

Phase `A0` of Track `careops_core_domain_abstractions_20261020` has fulfilled all requirements defined in its specification and implementation plan:
1. **Composition Freeze (Task A0.1):** Froze cross-service composition contracts CC-01 through CC-08 per [Cross-Service Composition Contracts](../../design/composition-contracts-20261020.md). Refused whole-track completion labels for upstream Kairos Tracks 64 (resource bundles) and 66 (observation mathematics), binding only exact runtime execution receipts. Resolved writer overlap for pending Kairos Tracks 67–69 to disjoint file boundaries before dispatch.
2. **Small Experiment Runner Extraction (Task A0.2):** Successfully extracted the generic multi-worker experiment lifecycle from `crates/careops-ed/src/batch_runner.rs` into new workspace crate [`crates/careops-core`](../../../crates/careops-core). Preserved the `careops-ed` facade and 100% backward compatibility for all 83 test targets. Proved two independent consumers (Clinical ED Simulation and Non-Clinical Logistics Hub Simulation), verified strict bit-identical worker invariance across 1, 2, and 4 workers, verified cancellation safety (writes `batch_run.partial` and suppresses `batch_run.json`), and verified crash recovery without duplicate work.
3. **Domain Defaults, Profiles & Contract-Negative Cases (Task A0.3):** Established the clinical and site defaults in `careops-core` outside Kairos. Implemented [`JurisdictionProfile`](../../../crates/careops-core/src/profile/jurisdiction.rs), [`SiteProfile`](../../../crates/careops-core/src/profile/site.rs), and [`ProfileLoader`](../../../crates/careops-core/src/profile/layered.rs) enforcing strict layering precedence (`Base Defaults < JurisdictionProfile < SiteProfile < RuntimeOverrides`). Validated failure modes (missing source files, incompatible schema versions, jurisdiction mismatches, malformed JSON). Implemented and verified the contract-negative invariant test suite across contracts CC-01 through CC-08.

---

## 2. Phase A0 Task Audit

| Task | Objective | Status | Evidence Document | Key Verification |
| :--- | :--- | :---: | :--- | :--- |
| **A0.1** | Freeze CC-01–CC-08, Capability Mapping, & Writer Overlap | **ACCEPTED** | [`a0.1-composition-freeze-20261024/README.md`](../a0.1-composition-freeze-20261024/README.md) | CC-01–CC-08 frozen; Kairos 64/66 capabilities mapped to exact logs; Tracks 67/68/69 writer overlap resolved to disjoint file paths. |
| **A0.2** | Small Runner Extraction, 2-Consumer Proof, & Invariance | **ACCEPTED** | [`a0.2-small-runner-extraction-20261024/README.md`](../a0.2-small-runner-extraction-20261024/README.md) | Generic experiment runner in `careops-core`; CareOps facade preserved in `careops-ed`; non-clinical logistics consumer proven; 1/2/4 worker invariance bit-identical; cancellation & crash recovery verified. |
| **A0.3** | Domain Defaults, Layered Profiles, & Contract-Negative Suite | **ACCEPTED** | [`a0.3-domain-defaults-profiles-20261024/README.md`](../a0.3-domain-defaults-profiles-20261024/README.md) | Clinical/site defaults outside Kairos; `JurisdictionProfile`/`SiteProfile` schema & loader tests; precedence verified; CC-01–CC-08 contract-negative suite passing (26/26 tests). |
| **A0.4** | Conductor Phase Review | **ACCEPTED** | [`a0-phase-review-20261024/README.md`](README.md) | Phase A0 audit, regression verification, and readiness gates verified. |

---

## 3. Specification Conformance Matrix

| Specification Requirement | Plan Mapping | Implementation Evidence | Disposition |
| :--- | :--- | :--- | :--- |
| **A-R1 / CC-04,06:** Bounded stage graphs, clinical progression, policy clocks | Task A0.1, A0.3 | [`crates/careops-core/src/profile/jurisdiction.rs`](../../../crates/careops-core/src/profile/jurisdiction.rs), [`tests/contract_negative_tests.rs`](../../../crates/careops-core/tests/contract_negative_tests.rs) | **SATISFIED (A0 SCOPE)** |
| **A-R2 / CC-05:** Capability-based spaces, equipment, effective capacity | Task A0.1, A0.3 | [`crates/careops-core/src/profile/site.rs`](../../../crates/careops-core/src/profile/site.rs), [`tests/contract_negative_tests.rs`](../../../crates/careops-core/tests/contract_negative_tests.rs) | **SATISFIED (A0 SCOPE)** |
| **A-R3 / CC-03:** Staged resource bundle allocation and rollback | Task A0.1, A0.3 | [`tests/contract_negative_tests.rs`](../../../crates/careops-core/tests/contract_negative_tests.rs) | **SATISFIED (A0 SCOPE)** |
| **A-R4 / CC-02:** Transfer states, operation IDs, resource retention | Task A0.1, A0.3 | [`tests/contract_negative_tests.rs`](../../../crates/careops-core/tests/contract_negative_tests.rs) | **SATISFIED (A0 SCOPE)** |
| **A-R5 / CC-01:** Longitudinal person & episode identity, one-to-many | Task A0.1, A0.3 | [`tests/contract_negative_tests.rs`](../../../crates/careops-core/tests/contract_negative_tests.rs) | **SATISFIED (A0 SCOPE)** |
| **A-R6 / CC-07:** Incremental extraction, facade preservation, 1/2/4 invariance | Task A0.1, A0.2 | [`crates/careops-core/src/experiment/`](../../../crates/careops-core/src/experiment/), [`crates/careops-ed/src/batch_runner.rs`](../../../crates/careops-ed/src/batch_runner.rs), [`receipts.json`](../a0.2-small-runner-extraction-20261024/receipts.json) | **SATISFIED (A0 SCOPE)** |
| **A-R7 / CC-08:** Latent vs observed state, decision policy isolation | Task A0.1, A0.3 | [`tests/contract_negative_tests.rs`](../../../crates/careops-core/tests/contract_negative_tests.rs) | **SATISFIED (A0 SCOPE)** |
| **PR173 Acceptance:** Small experiment lifecycle extraction before resource extraction | Task A0.2 | [`crates/careops-core/src/experiment/runner.rs`](../../../crates/careops-core/src/experiment/runner.rs), [`crates/careops-core/tests/logistics_consumer.rs`](../../../crates/careops-core/tests/logistics_consumer.rs) | **SATISFIED** |

---

## 4. Phase A0 Readiness & Exit Gates

- [x] **Contract Freeze:** CC-01 through CC-08 formally frozen with schema versions and invariant assertions.
- [x] **Upstream Mapping:** Consumed Kairos 64/66 capabilities verified with cryptographic evidence logs.
- [x] **Writer Disjointness:** Overlapping Kairos Tracks 67–69 resolved to disjoint file boundaries.
- [x] **Small Extraction:** Experiment runner extracted into `crates/careops-core` with CareOps facade preserved in `careops-ed`.
- [x] **Two-Consumer Proof:** Verified on clinical ED simulation and non-clinical logistics hub consumer.
- [x] **Worker Invariance:** Verified bit-identical outputs across 1, 2, and 4 workers.
- [x] **Cancellation & Crash Recovery:** Verified cancellation marker writes and checkpoint resumption without duplicate work.
- [x] **Profiles & Precedence:** Layered `JurisdictionProfile`/`SiteProfile` loader tested for strict precedence and fail-closed error handling.
- [x] **Contract Negatives:** Verified fail-closed behavior for all CC-01–CC-08 contract violations.
- [x] **Workspace Integrity:** 26/26 tests passing in `careops-core`, 83/83 test targets passing in `careops-ed`. Conductor context and tasks DAG checks passing with zero errors.

---

## 5. Phase Disposition & Hand-off

Phase `A0` is **ACCEPTED**.  
Track `careops_core_domain_abstractions_20261020` is ready to proceed to Phase `A1` (Pathways, clinical progression and policy clocks).
