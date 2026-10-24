# Evidence A1.1: Pathway Progression, Policy Clocks & Invariant Failing Cases

**Date:** 2026-10-24T11:23:16Z  
**Task:** `careops_core_domain_abstractions_20261020:A1.1`  
**Status:** PASS  

## 1. Scope & Objective

Establish the comprehensive failing test suite and mathematical oracles for Phase A1:
1. **Branching & Loop Bounds:** Enforce circuit breaking on configurable stage cycles exceeding repetition limits.
2. **Invalid Transitions:** Reject undeclared transitions and disjoint stage jumps with deterministic errors.
3. **Progression During Waiting (CC-04):** Prove that patient clinical state evolves while waiting in queues or boarding in hallways, independent of server allocation.
4. **Elapsed-Age / Residual-Time & Covariates:** Prove proportional hazard adjustment of residual survival times when covariates change without resetting elapsed hazard.
5. **Stale Events & Deterministic Ties:** Invalidate stale events on state generation advance; enforce multi-key deterministic event tie-breaking `(tick, priority, event_id, patient_id)`.
6. **Zero-Target Accounting (CC-06):** Safely compute immediate-service zero-duration targets without division-by-zero or NaN.
7. **Analytic Competing Risks & Zero Hazards:** Prove exact theoretical fraction matching for competing exponential risks; enforce zero-hazard halts.
8. **Unchanged-Covariate Invariance:** Verify invariant hazard calculation when covariates are unchanged.
9. **Time-Varying & Right-Censoring:** Verify right-censoring at simulation horizon $T$.
10. **Explicit Draw Ownership (CC-07):** Verify decoupled stream namespaces (`ProgressionStream`, `ServiceStream`, `RoutingStream`) preventing draw leakage.

---

## 2. Verification Matrix

| Verification Requirement | Check | Result | Evidence / Details |
| :--- | :--- | :---: | :--- |
| Loop Bounds | Exceeding `max_repetitions` | PASS | Fails closed with `PathwayErrorKind::LoopBoundExceeded` |
| Invalid Transitions | Unregistered edges and disjoint hops | PASS | Fails closed with `PathwayErrorKind::InvalidTransition` |
| Progression Waiting | Queue wait with positive hazard | PASS | Clinical hazard accumulates during queue wait (CC-04) |
| Covariate Updates | Residual time rescaling | PASS | Doubled hazard halves remaining residual time |
| Stale Events | Event generation < current generation | PASS | Rejected with `PathwayErrorKind::StaleEvent` |
| Deterministic Ties | Identical tick event collisions | PASS | Multi-key ordering: `tick -> priority -> id` |
| Zero Targets | Immediate service 0-tick target | PASS | Evaluates to 0.0 or $\infty$ without division-by-zero (CC-06) |
| Competing Risks | $\lambda_1 = 0.02, \lambda_2 = 0.03$ | PASS | Matches analytic fractions 0.40 and 0.60 |
| Zero Hazard | $\lambda = 0.0$ quiescent state | PASS | Fails closed with `ZeroTargetError`; no event occurs |
| Covariate Invariance | Constant covariates | PASS | Strict identical hazard rate evaluation |
| Right-Censoring | Event tick > horizon | PASS | Clamped and right-censored; no unauthorized transition |
| Draw Ownership | Isolated subsystem streams | PASS | Progression draws bit-identical despite 100 service draws (CC-07) |
| Calibration Isolation | Retrying service duration draws | PASS | Progression stream draw count unchanged (CC-07) |

---

## 3. Key Architectural Artifacts

- [`crates/careops-core/src/pathway/stage.rs`](../../../crates/careops-core/src/pathway/stage.rs): `StageGraph`, `StageNode`, `StageId`.
- [`crates/careops-core/src/pathway/progression.rs`](../../../crates/careops-core/src/pathway/progression.rs): `CompetingRiskModel`, `HazardRate`.
- [`crates/careops-core/src/pathway/clock.rs`](../../../crates/careops-core/src/pathway/clock.rs): `PolicyClock`, `ScheduledEvent`.
- [`crates/careops-core/src/pathway/streams.rs`](../../../crates/careops-core/src/pathway/streams.rs): `PathwayRngStreams`, `SubsystemStream`.
- [`crates/careops-core/src/pathway/error.rs`](../../../crates/careops-core/src/pathway/error.rs): `PathwayError`, `PathwayErrorKind`.
- [`crates/careops-core/tests/pathway_failing_cases_tests.rs`](../../../crates/careops-core/tests/pathway_failing_cases_tests.rs): Comprehensive failing cases and mathematical oracles test suite (13 integration tests).
