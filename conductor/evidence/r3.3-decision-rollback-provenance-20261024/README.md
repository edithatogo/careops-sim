# R3.3 — Candidate Decision, Rollback Rehearsal, and Provenance Ledger Evidence

**Status: R3.3 candidate decision, rollback rehearsal, owner handoffs, and provenance ledger verified; receipts complete.**  
**Execution Timestamp:** `2026-10-24T09:20:10Z`  
**Parent Source:** `careops-sim` at `0e1e425`  
**Submodule Retained Pin:** `libs/kairos` at `44261eb5cf8275f132f63a3396903d116d701dbe`  
**Submodule Qualified Candidate:** `libs/kairos` at `8282b1ab45669a667bda3be2862272447544ba44`

---

## 1. Candidate Decision & Rollback Rehearsal

Task R3.3 records the immutable candidate decision and proves that the submodule pin can be safely rolled back in an emergency without disrupting the parent checkout:

1. **Candidate Decision:**
   - Evaluated upstream candidate `8282b1ab`. Proved that `8282b1ab` is 1 commit ahead of working pin `44261eb`, modifying only `crates/kairo-ecs-gpu` (adding `numeric_batch.rs` CPU profiling oracle).
   - In accordance with track specification [`spec.md`](../../tracks/repo_reconciliation_kairos_upgrade_20261020/spec.md) (line 22: *"No source pin changes occur in this planning revision. R3 closes on an evidenced candidate decision and usable baseline, including an explicit retain-pin decision"*), the coordinator retains working pin `44261eb5cf8275f132f63a3396903d116d701dbe`.
2. **Rollback Rehearsal:**
   - Rehearsal executed via [`run_r33_rollback_rehearsal.py`](run_r33_rollback_rehearsal.py) in an isolated detached git worktree (`/private/tmp/careops-r33-rollback-rehearsal-run`).
   - Verified initial pin: `44261eb5cf8275f132f63a3396903d116d701dbe`.
   - Simulated candidate advancement to `8282b1ab45669a667bda3be2862272447544ba44` in a parent commit.
   - Executed documented recovery procedure: `git revert HEAD` followed by `git submodule update --init libs/kairos`.
   - Verified that both the parent gitlink and the checked-out submodule `HEAD` restored cleanly to `44261eb5cf8275f132f63a3396903d116d701dbe`.
   - Cleaned up the isolated worktree with zero side-effects on the primary repository checkout.
   - Full execution logs recorded in [`rollback-rehearsal.log`](rollback-rehearsal.log) and [`rollback-rehearsal-receipt.json`](rollback-rehearsal-receipt.json).

---

## 2. Owner Handoff Matrix

In accordance with [Composition Contracts](../../design/composition-contracts-20261020.md) and [Agent Engineering](../../agent-engineering.md):

| Domain / Scope | Responsible Owner | Contract Obligations & Boundary Limits |
| :--- | :--- | :--- |
| **CareOps Coordinator** | CareOps Top-Level Coordinator | Retains registry consistency, submodule pin coordination, phase closeout reviews, and evidence audits. |
| **Scheduling & DES** | Kairos Tracks 01, 03, 59, 64 | Reusable DES/ABM scheduling, event ordering, priority queue previews, and resource bundle contention. |
| **Interoperable I/O** | Kairos Tracks 04, 62 | Official Arrow 60 / Parquet 60 physical readers, stream writers, and budget limit enforcement. |
| **Calibration & Replay** | Kairos Tracks 21, 22, 66 | Observation mathematics, calibration C43 sidecar adapters, and byte-identical C2 replay journals. |
| **Public API** | Kairos Track 25 | Public API semver stability, export visibility, and backward compatibility across versions. |
| **Device / Cluster Acceleration** | Kairos Tracks 32–35 | GPU/Metal/CUDA and distributed MPI execution; **retained as open external gates (E6/E8 unverified blockers)**. |
| **Core Person Identity** | CareOps Core Domain | Linked patient, referral, encounter, and admission identity (CC-01). |
| **ED Clinical Pathway** | CareOps ED Domain | ATS triage urgency categories, physician/nurse staffing ratios, and macro/micro transit fidelity. |

---

## 3. Environment & Provenance Ledger

| Parameter | Value / Hash |
| :--- | :--- |
| **Parent Repository Commit** | `0e1e425` on branch `main` |
| **Submodule Retained Pin** | `libs/kairos` at `44261eb5cf8275f132f63a3396903d116d701dbe` |
| **Submodule Qualified Candidate** | `libs/kairos` at `8282b1ab45669a667bda3be2862272447544ba44` |
| **Compiler / Toolchain** | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` (Homebrew) |
| **Cargo Version** | `cargo 1.99.1 (5f94df478 2026-08-27)` (Homebrew) |
| **Host Environment** | Apple M1 Max (32 GiB, macOS Darwin 24.3.0 arm64) |
| **Backend Execution** | CPU Native (zero GPU device or cluster network runtime linked) |
| **Simulation Scenario Input** | `crates/careops-ed/examples/nominal_mvp.json` (SHA-256: `71215b7ff4a2b47b38ace339cadac15a68c1c49c9738856ff742e9400c2ff9ae`) |
| **Simulation Seed & Horizon** | Seed `42`, Horizon `20` ticks |
| **Retained ED Output Artifact** | [`ed-nominal-run.json`](../r3.1-deterministic-conformance-20261024/ed-nominal-run.json) (SHA-256: `2ef0b777a876a3eb79be7a873832d2012d92555776a3e6bb69c02580ecadbb3d`) |
| **Retained Replay Checkpoint** | [`c2-checkpoint.json`](../r3.1-deterministic-conformance-20261024/c2-checkpoint.json) (SHA-256: `532acb205e99099300e1311160550308740885c0c52cfcab6db4b76d79c7244e`) |

---

## 4. Retained Negative Controls & Receipt Rejection Policy

In strict conformance with Conductor standards, missing, stale, or mismatched receipts are rejected:
- **Negative Control 1:** `claim_atomic_multi_resource()` fails closed with `OperationalError::AtomicMultiResourceUnsupported`.
- **Negative Control 2:** Composite procedures without multi-role teams fail with `OperationalError::UnsupportedMultiRole`.
- **Negative Control 3:** Replay save to an existing sentinel destination fails with exit `1` and preserves sentinel bytes without corruption.
- **Negative Control 4:** Inaccessible transit edges and unreachable routes fail closed.
- **Negative Control 5:** Mock metadata and CPU fallback are rejected as device execution proof; E6 remains an unverified release blocker.
- **Negative Control 6:** In-process threads and mpsc channels are rejected as multi-host cluster transport proof; E8 remains an unverified release blocker.

All structured records are preserved in [`provenance-manifest.json`](provenance-manifest.json).
