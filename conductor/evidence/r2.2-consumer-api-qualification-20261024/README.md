# R2.2 — Consumer API Qualification Evidence

**Status: R2.2 consumer API qualification executed and verified; candidate 8282b1ab and pin 44261eb conform to all required consumer contracts.**  
**Execution Timestamp:** `2026-10-24T08:40:34Z`  
**Parent Source:** `careops-sim` at `7b61ba9`  
**Upstream Engine:** `libs/kairos` at runtime pin `44261eb5cf8275f132f63a3396903d116d701dbe` / qualification candidate `8282b1ab45669a667bda3be2862272447544ba44`.

---

## 1. Scope & Objective

Task R2.2 requires exercising all consumer APIs demanded by `careops-sim` against the capability-preserving Kairos candidate:
1. **Scheduler event waits & priority queues:** Ordered time, priority, and insertion tie-breaking; non-consuming queue preview; queue checkpoints and cancellations.
2. **Allocation, cancellation, and rollback:** Resource contention, phased space/staff acquisition, rollback on partial failure, urgent preemption, and process-level replay save/restore.
3. **Official Arrow IPC / Parquet interoperability:** Pinned Arrow 60 / Parquet 60 schemas, IPC file/stream roundtrips, row/column budget limits, and official PyArrow fixture readback.
4. **Negative controls & rejection:** Explicit rejection of unsupported atomic multi-resource claims (`claim_atomic_multi_resource`), uncoordinated multi-role procedures, and checkpoint destination collision.

---

## 2. Executed Qualification Matrix

| ID | Test Group | Target / Crate | Result | Details |
| :--- | :--- | :--- | :--- | :--- |
| **R2.2-1** | Scheduler event waits & queue preview | `libs/kairos/crates/kairo-ecs-core` | **PASS** (49 tests) | Event queues, time-priority order, zero-delay guards, scheduler checkpoint v1, and preview v1. |
| **R2.2-2** | Official Arrow IPC & Parquet I/O | `libs/kairos/crates/kairo-ecs-arrow-io` | **PASS** (31 tests) | Typed schemas, IPC file/stream writers, Parquet roundtrip, row caps, and `interop_v1` fixture readback. |
| **R2.2-3** | Resource lifecycle Arrow telemetry | `libs/kairos/crates/kairo-ecs-arrow` | **PASS** (31 tests) | Event log schema stability, resource lifecycle streams, and signed span trace times. |
| **R2.2-4** | Calibration telemetry & sidecar pipeline | `libs/kairos/crates/kairo-ecs-calibration` | **PASS** (22 tests) | C43 Arrow outputs, sidecar adapter, and permutation-invariant residual pipelines. |
| **R2.2-5** | Process save/restore & rollback | `kairo-ecs-calibration/c2_replay_demo` | **PASS** (4 steps) | Saved checkpoint, two byte-identical restores (`diff -u` exit 0), and sentinel collision rejected. |
| **R2.2-6** | Consumer dispatch & fidelity constraints | `crates/careops-ed/tests/e22_dispatch_fidelity_constraints.rs` | **PASS** (4 tests) | ATS urgency dispatch, FIFO order, Macro vs Micro transit routing, and phased allocation. |
| **R2.2-7** | Parent workspace suite | `careops-sim` (all crates) | **PASS** (296 tests) | Full ED simulation, synthetic holdout evaluation, CLI, and integration tests. |

---

## 3. Negative Controls & Invariant Proofs

1. **Atomic Multi-Resource Rejection:**
   - Calling `claim_atomic_multi_resource()` explicitly fails closed and returns `Err(OperationalError::AtomicMultiResourceUnsupported)`.
   - Phased sequential claim (`execute_phased_claim`) reserves/occupies the bay first, then attempts staff allocation. If staff is unavailable, the bay is rolled back cleanly.
2. **Procedural Multi-Role Staffing:**
   - High-acuity procedures (sedation, resuscitation, assisted transfer) without an explicit composite team fail with `Err(OperationalError::UnsupportedMultiRole)`.
3. **Checkpoint Sentinel Collision:**
   - Invoking `c2_replay_demo save` against an existing sentinel file exits with code `1` and emits `"checkpoint destination already exists"`. The sentinel file bytes are preserved without corruption.
4. **Transit Graph Accessibility:**
   - Inaccessible edges reject with `Err(OperationalError::EdgeInaccessible)` and disconnected routes reject with `Err(OperationalError::RouteUnreachable)`.

---

## 4. Retained Receipts

All executed commands, standard output logs, elapsed execution timings, and SHA-256 digests are recorded in [`receipts.json`](receipts.json):
- [`scheduler-event-waits-core.log`](scheduler-event-waits-core.log)
- [`arrow-io-ipc-parquet.log`](arrow-io-ipc-parquet.log)
- [`arrow-telemetry-lifecycle.log`](arrow-telemetry-lifecycle.log)
- [`calibration-c43-outputs.log`](calibration-c43-outputs.log)
- [`c2-demo-save.log`](c2-demo-save.log), [`c2-demo-restore-1.log`](c2-demo-restore-1.log), [`c2-demo-sentinel-rejection.log`](c2-demo-sentinel-rejection.log)
- [`careops-ed-e22-dispatch-fidelity.log`](careops-ed-e22-dispatch-fidelity.log)
- [`careops-ed-full-workspace.log`](careops-ed-full-workspace.log)
