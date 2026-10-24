# Evidence A1.2: Bounded Configurable Pathway Adapter & Progression

**Date:** 2026-10-24T11:29:21Z  
**Task:** `careops_core_domain_abstractions_20261020:A1.2`  
**Status:** PASS  

## 1. Scope & Objective

Implement a bounded configurable pathway adapter with independently scheduled clinical progression:
1. **Configurable Stage Graph:** Implement [`ConfigurablePathwayAdapter`](../../../crates/careops-core/src/pathway/adapter.rs) capable of executing arbitrary directed stage graphs with capacity constraints, queueing, and cycle limits.
2. **Independently Scheduled Progression (CC-04):** Prove that patient clinical progression evolves while waiting in queues or boarding in hallways, independent of server allocation.
3. **Horizon Right-Censoring:** Clamp and record in-flight work upon reaching the simulation horizon without unauthorized stage completion.
4. **ED Compatibility Path:** Retain the existing `careops-ed` triage and assessment pathway without regressions.

---

## 2. Verification Matrix

| Verification Requirement | Check | Result | Evidence / Details |
| :--- | :--- | :---: | :--- |
| Canonical ED Flow | Triage -> Assessment -> Diagnostics -> Treatment -> Discharge | PASS | Executed sequentially with start/completion ticks |
| Progression While Queued | Acuity escalation under zero capacity | PASS | Acuity deteriorated while waiting in queue (CC-04) |
| Horizon Censoring | Arrival at tick 40, planned 30, horizon 50 | PASS | Clamped to 10 ticks; marked censored without completion |
| ED Compatibility Path | Full ED pathway conformance suite | PASS | 8/8 tests passed in `pathway_conformance.rs` |

---

## 3. Key Architectural Artifacts

- [`crates/careops-core/src/pathway/adapter.rs`](../../../crates/careops-core/src/pathway/adapter.rs): `ConfigurablePathwayAdapter`, `PathwayPatientSpec`, `StageExecutionRecord`, `ClinicalProgressionLog`, `PatientPathwayOutcome`.
- [`crates/careops-core/tests/configurable_pathway_adapter_tests.rs`](../../../crates/careops-core/tests/configurable_pathway_adapter_tests.rs): Integration test suite verifying flow, independent progression, and horizon censoring.
- [`crates/careops-ed/tests/pathway_conformance.rs`](../../../crates/careops-ed/tests/pathway_conformance.rs): Retained ED compatibility pathway regression suite.
