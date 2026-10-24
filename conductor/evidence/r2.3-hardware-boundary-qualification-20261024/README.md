# R2.3 — Hardware Boundary Qualification & Host Reconciliation Evidence

**Status: R2.3 hardware boundary qualification executed and verified; headless consumer checks pass in full isolation; optional E6/E8 capabilities classified as unverified release blockers.**  
**Execution Timestamp:** `2026-10-24T08:55:18Z`  
**Parent Source:** `careops-sim` at `a7c6505`  
**Upstream Engine:** `libs/kairos` at runtime pin `44261eb5cf8275f132f63a3396903d116d701dbe` / qualification candidate `8282b1ab45669a667bda3be2862272447544ba44`.

---

## 1. Scope & Objective

Task R2.3 establishes the boundary between headless simulation requirements and hardware-accelerated / distributed capabilities:
1. **Isolated Consumer Checks:** Run required Rust consumer simulation checks in complete isolation without GPU or MPI dependencies.
2. **GPU Acceleration Boundary (E6):** Map optional E6 acceptance to real Metal/CUDA kernel submission and readback with CPU parity; verify candidate backend availability.
3. **Distributed Transport Boundary (E8):** Map optional E8 acceptance to physically distinct transport peers with network sockets, failure/restart, and measured timings.
4. **Host Identity Re-Verification:** Re-verify local and remote host identities, specifically clarifying the historical `pleiades` alias (NAS vs CUDA PC).
5. **Headless Pin Gate Separation:** Enforce the invariant that mock metadata, CPU fallback, and in-process threads never satisfy physical device or multi-host gates. Hardware qualification stays outside the headless pin gate unless actually consumed.

---

## 2. Host Identity Audit & Re-Verification

| Host Role | Environment / OS | Verified Identity | Status / Role in Qualification |
| :--- | :--- | :--- | :--- |
| **Local Host** | macOS (Darwin 24.3.0 arm64) | Apple Silicon (M1 Max, 32 GiB) | Executes headless consumer suite and CPU fallback tests. OS-level Metal runtime present, but unconfigured in candidate `kairo-ecs-gpu`. |
| **Historical `pleiades` Alias** | Linux (x86_64, Docker) | Local Network NAS (Intel i3-N300) | **Reverified:** The SSH alias `pleiades` targets the local NAS storage/CPU node, **not** an NVIDIA workstation. It cannot satisfy CUDA execution gates. |
| **Remote CUDA Accelerator** | Linux 6.6.137+ x86_64 (glibc 2.35) | NVIDIA Tesla T4 (15,360 MiB VRAM, Driver 580.82.07, CUDA 12.8) | Verified via Google Colab CLI (`kairos-cuda-t4` / ephemeral probe). Real PTX kernel execution demonstrated; candidate `kairo-ecs-gpu` lacks linked CUDA bindings. |

---

## 3. Executed Qualification Matrix

| ID | Test Group | Target / Crate | Result | Details |
| :--- | :--- | :--- | :--- | :--- |
| **R2.3-1** | Isolated Consumer ED Simulation | `crates/careops-ed` | **PASS** (15 tests) | Full ED patient flow, clinic allocation, urgency dispatch, and FIFO routing pass headlessly on CPU. |
| **R2.3-2** | Upstream GPU CPU Fallback & Parity | `libs/kairos/crates/kairo-ecs-gpu` | **PASS** (16 tests) | CPU fallback execution passes; metadata descriptions (`e61`/`e62`) verified as contract checks only. |
| **R2.3-3** | Upstream GPU Backend Availability | `libs/kairos/crates/kairo-ecs-gpu` (`cuda-backend,wgpu-backend`) | **PASS** (18 tests) | `wgpu` returns `BackendNotConfigured("wgpu-backend-not-configured")`; `cuda` returns `BackendNotConfigured("cuda-backend-not-configured")` and `Err(UnsupportedBackend)`. |
| **R2.3-4** | Upstream MPI In-Memory Thread Protocol | `libs/kairos/crates/kairo-ecs-mpi` | **PASS** (15 tests) | In-process 2-rank protocol emulator passes via `std::thread`/`mpsc`; no physical socket transport exists. |
| **R2.3-5** | Host Identity & Boundary Mapping | Repository & Environment Audit | **PASS** (reconciled) | Pleiades NAS vs CUDA workstation distinguished; Colab T4 GPU verified; E6/E8 classified as unverified blockers. |

---

## 4. Invariant Rules & Acceptance Classification

1. **Mock Metadata & CPU Fallback Rejection:**
   - Crate `kairo-ecs-gpu` provides metadata structs and CPU fallback execution paths. Under Conductor guidelines, **CPU fallback and mock metadata never satisfy device gates**.
   - Because neither Mac Metal nor NVIDIA CUDA runtime bindings are wired into candidate `kairo-ecs-gpu`, **Track E6 is classified as UNVERIFIED / BLOCKED** (retained as an open release blocker).
2. **In-Process Threading Rejection:**
   - Crate `kairo-ecs-mpi` executes entirely within a single OS process across Rust worker threads. Under Conductor guidelines, **threads never satisfy multi-host transport gates**.
   - Because no physical socket transport between distinct physical machines (e.g. Mac and NAS) exists, **Track E8 is classified as UNVERIFIED / BLOCKED** (retained as an open release blocker).
3. **Headless Pin Gate Exemption:**
   - Neither `careops-sim` nor `careops-ed` imports `kairo-ecs-gpu` or `kairo-ecs-mpi`.
   - Headless consumer simulation runs cleanly on CPU without device or cluster dependencies.
   - Therefore, the **headless composition pin gate for Phase R2 PASSES**. Unverified hardware capabilities do not block the simulation pin decision.

---

## 5. Retained Receipts

All executed commands, standard output logs, elapsed execution timings, and SHA-256 digests are recorded in [`receipts.json`](receipts.json):
- [`careops-ed-isolated.log`](careops-ed-isolated.log) (SHA-256: `81955b8eb02f7dadcd6f345275e51e69ea398b9debc7f6bb7572c8d60d3824f6`)
- [`kairo-ecs-gpu-cpu-fallback.log`](kairo-ecs-gpu-cpu-fallback.log) (SHA-256: `ed16912ef7f7533567646d542423fa1bd179c64cfd71bf79c32014ad7dcc5a6b`)
- [`kairo-ecs-gpu-backends.log`](kairo-ecs-gpu-backends.log) (SHA-256: `b2e897247ccaf9892b2ac4e4f1a1499ec4c566bf31e7bcd55344f0b6fb29f7ea`)
- [`kairo-ecs-mpi-local.log`](kairo-ecs-mpi-local.log) (SHA-256: `2503bfb04f0c74d6dcc68061cd1fb03239384a3a4a7d9ba0dcf0d8b9e83c5d08`)
- [`hardware-boundary-mapping.log`](hardware-boundary-mapping.log) (SHA-256: `8a5bb334a67500d670b47e0892b3ebbf626ebc29efd72b35be637c93db39a769`)
