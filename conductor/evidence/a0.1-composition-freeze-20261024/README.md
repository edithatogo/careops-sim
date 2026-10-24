# A0.1 — Healthcare Composition Contract Freeze Evidence

**Status: A0.1 composition contract freeze verified and accepted; CC-01 through CC-08 frozen; consumed Kairos 64/66 capabilities mapped to exact evidence; Tracks 67/68/69 writer overlap resolved.**  
**Execution Timestamp:** `2026-10-24T09:53:01Z`  
**Parent Source:** `careops-sim` on branch `main`  
**Active Track:** `careops_core_domain_abstractions_20261020`  
**Active Phase:** `A0` (Small contracts, identities and extraction boundaries)

---

## 1. Executive Summary & Scope

Task A0.1 formally freezes the core composition contracts CC-01 through CC-08, binds consumed Kairos 64/66 capabilities to exact runtime evidence rather than abstract track labels, and resolves exact-file ownership across overlapping upstream domains before dispatching implementation leaves:
1. **Contract Freeze:** Freezes contracts CC-01 through CC-08 per [Cross-Service Composition Contracts](../../design/composition-contracts-20261020.md).
2. **Evidence Mapping:** Refuses whole-track completion labels for upstream Kairos Tracks 64 (resource bundles) and 66 (observation mathematics); binds only exact executed tests and cryptographic log receipts.
3. **Writer Overlap Resolution:** Resolves file ownership between upstream engine mechanics (`libs/kairos`) and healthcare domain abstractions (`crates/careops-core`) for Tracks 67 (linked entities), 68 (boundary channels), and 69 (competing risks).
4. **Compatibility Gates:** Establishes public API, checkpoint, and schema compatibility gates for the upcoming `careops-core` crate extraction.

---

## 2. Frozen Composition Contracts (CC-01 — CC-08)

| Contract | Title | Core Semantics & Invariants | Schema Version |
| :--- | :--- | :--- | :--- |
| **CC-01** | Person and episode identity | Stable synthetic `PersonId`; distinct `ReferralId`, `EncounterId`, `AdmissionId`, and `TransferId`. Typed one-to-many relationships (`Person` to multiple `ServiceEpisode`s). Recycled ECS generation/indices are strictly isolated from longitudinal clinical identities. | `careops-identity-v1` |
| **CC-02** | Transfer lifecycle | Ten explicit states: `Requested`, `Offered`, `Reserved`, `Departing`, `InTransit`, `Arrived`, `HandoverComplete`, `Rejected`, `Cancelled`, `Expired`. Deterministic `TransferOperationId` with idempotent logical effects. Originating resources retained until authoritative `HandoverComplete` event. | `careops-transfer-envelope-v1` |
| **CC-03** | Resource bundle allocation | Explicit team/room/equipment bundles. Staged sequential claims with rollback on partial failure; `AllOf`/`AnyOf` are event joins, not atomic resource allocators. Contention, expiry, and preemption policies without silent eviction. | `careops-bundle-v1` |
| **CC-04** | Independent clinical progression | Patient clinical state evolves independently while waiting, boarding, or traveling. Discrete competing risks with deterministic tie-breaking. Reassessment does not resample unrelated clocks. | `careops-progression-v1` |
| **CC-05** | Effective capacity and calendars | Eight states: `Physical`, `Equipped`, `Staffed`, `Open`, `Reserved`, `Occupied`, `Cleaning`, `Unavailable`. Maintenance, turnover, and operational calendars without silent eviction of active clinical care. | `careops-capacity-v1` |
| **CC-06** | Virtual clocks and policy isolation | Single virtual clock with explicit integer `Tick` units. Strict isolation between triage urgency, operational priority, target clock thresholds, and interruption eligibility. Decision policies cannot observe latent future outcomes. | `careops-clock-v1` |
| **CC-07** | Reproducible composition | Hierarchical random stream namespaces (`RootRng`, `PersonStream`, `ModuleStream`, `WorkerStream`). Deterministic event ordering and cross-worker equivalence (1, 2, 4 worker threads produce byte-identical or trace-identical results). | `careops-rng-stream-v1` |
| **CC-08** | Observation and evaluation | Latent state vs observed state distinction. Explicit handling of delays, missingness, interval/right censoring. Separate ledgers for activity, clinical outcomes, financial costs, and equity. | `careops-evaluation-v1` |

---

## 3. Exact Evidence Mapping for Consumed Kairos 64 / 66 Capabilities

In accordance with Conductor rules, whole-track labels are rejected. Only exact capabilities with verified receipts are admitted:

### Kairos Track 64 (Resource & Team Bundle Mechanics)
- **Consumed Capability:** Phased sequential resource acquisition (bay reservation followed by staff claim), automatic bay rollback on staff allocation failure, ATS urgency dispatch, and rejection of unsupported atomic multi-resource claims (`claim_atomic_multi_resource`).
- **Cryptographic Evidence Logs:**
  - [`conductor/evidence/r2.2-consumer-api-qualification-20261024/careops-ed-e22-dispatch-fidelity.log`](../r2.2-consumer-api-qualification-20261024/careops-ed-e22-dispatch-fidelity.log) (SHA-256: `04ed97510e96b58a3135ee09d3faf8f22178eeecf345cad658b5b66c30d21369`)
  - [`conductor/evidence/r2.2-consumer-api-qualification-20261024/scheduler-event-waits-core.log`](../r2.2-consumer-api-qualification-20261024/scheduler-event-waits-core.log) (SHA-256: `490842fe33ec90518381381dcd7fbaaa3e51dec1882b3ab6c7e082fcf8bfe56e`)

### Kairos Track 66 (Observation Mathematics & Residual Pipelines)
- **Consumed Capability:** Typed Arrow tabular outputs, sidecar pipeline adapters, and permutation-invariant residual scoring.
- **Cryptographic Evidence Logs:**
  - [`conductor/evidence/r2.2-consumer-api-qualification-20261024/calibration-c43-outputs.log`](../r2.2-consumer-api-qualification-20261024/calibration-c43-outputs.log) (SHA-256: `b18757b9708704b2112d643532241d2fd03deb957a4b8acac1c196da4e6faf4f`)
  - [`conductor/evidence/r3.1-deterministic-conformance-20261024/flow-q-c-suite.log`](../r3.1-deterministic-conformance-20261024/flow-q-c-suite.log) (SHA-256: `e9123580b2449fd9e0d86cddcbf8dd82c45486e4755e677362dcff2a10a3d722`)

---

## 4. File Writer Overlap Resolution (Tracks 67, 68, 69)

To prevent cross-repository ownership collisions and enforce the single-maintainer harness, exact file paths are partitioned:

| Track / Domain | Upstream Kairos Writers (`libs/kairos`) | CareOps Core Writers (`crates/careops-core`) | Contract Handoff Boundary |
| :--- | :--- | :--- | :--- |
| **Track 67: Linked Entities** | `crates/kairo-ecs-core/src/relationship/`<br>`crates/kairo-ecs-abm/src/spatial/` | `src/identity/person.rs`<br>`src/identity/encounter.rs`<br>`src/identity/maternal_infant.rs` | Kairos maintains neutral ECS Entity/Component relationships; CareOps Core owns healthcare person longitudinal history and maternal-infant linkage. |
| **Track 68: Boundary Channels** | `crates/kairo-ecs-des/src/channel/`<br>`crates/kairo-ecs-des/src/queue/` | `src/egress/transfer.rs`<br>`src/egress/handover.rs`<br>`src/egress/outbox.rs` | Kairos maintains discrete event queue dispatch; CareOps Core owns inter-facility transfer lifecycle and clinical handover protocols. |
| **Track 69: Competing Risks** | `crates/kairo-ecs-des/src/competing_risks/`<br>`crates/kairo-ecs-rng/` | `src/progression/stage.rs`<br>`src/progression/hazard.rs`<br>`src/progression/reassessment.rs` | Kairos maintains numeric hazard sampling and stream partitioning; CareOps Core owns clinical stage graphs, triage urgency, and disease progression models. |

---

## 5. Public Compatibility & Verification Receipts

The verification runner [`verify_a01_freeze.py`](verify_a01_freeze.py) confirmed:
- Mapped evidence logs exist on disk and match SHA-256 digests.
- Existing `careops-ed` test suite passes (39 unit tests + 8 fixture tests + integration tests).
- Task catalog integrity (`python3 tools/tasks.py check`) validated: 387 tasks, full coverage, DAG valid.
- Context integrity (`python3 tools/context.py check`) validated: 24 tracks, 105 milestones, 1,809 links, 0 errors.
- Machine-readable manifest recorded in [`composition-contracts-freeze.json`](composition-contracts-freeze.json) and execution receipts in [`receipts.json`](receipts.json).
