# R3.2 — Submodule Pin Decision & Bounded Alternatives Evidence

**Status: R3.2 submodule pin decision executed and verified; working pin 44261eb retained with full deterministic conformance; candidate 8282b1ab qualified as a milestone.**  
**Execution Timestamp:** `2026-10-24T09:16:30Z`  
**Parent Source:** `careops-sim` at `ef0ba59`  
**Retained Runtime Pin:** `libs/kairos` at `44261eb5cf8275f132f63a3396903d116d701dbe`  
**Qualified Candidate:** `libs/kairos` at `8282b1ab45669a667bda3be2862272447544ba44`

---

## 1. Executive Summary & Decision

In accordance with Task R3.2 and track specification [`spec.md`](../../tracks/repo_reconciliation_kairos_upgrade_20261020/spec.md) (line 22: *"No source pin changes occur in this planning revision. R3 closes on an evidenced candidate decision and usable baseline, including an explicit retain-pin decision"*), the CareOps coordinator records an **evidenced retain-pin decision**:

- **Submodule Pin Action:** **RETAIN WORKING PIN** `44261eb5cf8275f132f63a3396903d116d701dbe`.
- **Candidate Evaluation:** Candidate `8282b1ab` is qualified as an immutable capability-preserving revision. Inspection shows it is exactly 1 commit ahead of `44261eb`, modifying only `crates/kairo-ecs-gpu` (adding `numeric_batch.rs` CPU profiling oracle). Core scheduling, arrow-io, calibration, and ED simulation layers are byte-identical.
- **Hardware Separation:** Because real GPU execution (Metal/CUDA) and multi-host physical networking (MPI) remain unconfigured in the candidate, advancing the parent pin provides no additional capabilities to the headless simulation engine.
- **Stability Guarantee:** Retaining the proven working pin eliminates regression risk, guarantees 100% deterministic conformance with the frozen R1 baseline, and satisfies all composition contracts.

---

## 2. Decision Matrix

| Option | Candidate Commit | Consumer Suitability | Hardware Readiness | Decision |
| :--- | :--- | :--- | :--- | :--- |
| **A. Advance Pin to Candidate** | `8282b1ab` | Passes all headless suites (296 workspace, 969 Q/C). | Metal/CUDA and MPI remain unconfigured (`BackendNotConfigured`). | **REJECTED:** Violates `spec.md` line 22; introduces no new capability for headless ED simulation. |
| **B. Advance to Divergent Main** | `f0d30e2` | **FAIL:** Missing `kairo-ecs-arrow-io` and `kairo-ecs-calibration`. | Divergent history; breaking path changes. | **REJECTED:** Breaks 321 pin-contract paths. |
| **C. Retain Working Pin (Selected)** | `44261eb` | **PASS (100%):** All 296 workspace, 969 Q/C, Arrow I/O, C2 replay, and ED runs match baseline with zero drift. | Advanced hardware remains tracked as external dispatch gates. | **ACCEPTED:** Conforms strictly to specification and composition contracts. |

---

## 3. Missing Capabilities & External Dispatch Gates

Under Conductor rules, missing or unavailable hardware capabilities must not be converted into passes or assumed to be delivered:

1. **Track E6 — GPU Acceleration (Metal / CUDA):**
   - **Current State:** Crate `kairo-ecs-gpu` provides metadata descriptors and CPU fallback execution. Runtime device backends (`wgpu` and `cuda`) return `BackendNotConfigured`.
   - **Classification:** **UNVERIFIED / RELEASE BLOCKER**.
   - **Handling:** Tracked as an external dispatch gate for optional track E6. Headless discrete event simulation does not consume GPU acceleration.
2. **Track E8 — Multi-Host Distributed Transport (MPI):**
   - **Current State:** Crate `kairo-ecs-mpi` is an in-process thread emulator using `std::thread` and `mpsc` channels within a single OS process.
   - **Classification:** **UNVERIFIED / RELEASE BLOCKER**.
   - **Handling:** Tracked as an external dispatch gate for optional track E8. Physical cluster networking between distinct machines (e.g. Mac and NAS) requires dedicated wire protocol qualification.

---

## 4. Supported Bounded Alternatives

For all operations within the current scope of `careops-sim`, robust, qualified alternatives are fully operational:

1. **Deterministic Single-Node Simulation:** CPU-native DES/ABM execution with independent RNG stream and seed partitioning across concurrent replications.
2. **Phased Resource Allocation:** Phased sequential resource claims with automatic rollback on partial failure, strictly rejecting unsupported atomic multi-resource claims (`claim_atomic_multi_resource`).
3. **Sealed Process Checkpoint & Replay:** Byte-identical replay save/restore journals under schema `kairos-c2-replay-journal-v1`, protected by fail-closed sentinel destination collision rejection.
4. **Interoperable Arrow I/O:** Physical IPC streams and Parquet tabular outputs adhering to Arrow 60 / Parquet 60 schema standards with strict row and column budget guards.

---

## 5. Submodule State Verification

- **Submodule Path:** `libs/kairos`
- **Submodule Git Commit:** `44261eb5cf8275f132f63a3396903d116d701dbe`
- **Parent Gitlink Status:** Clean, verified matching `HEAD detached at 44261eb`.
- **Parent Repository:** Working tree clean on `main`.
- **Decision Record:** Retained in [`decision.json`](decision.json).
