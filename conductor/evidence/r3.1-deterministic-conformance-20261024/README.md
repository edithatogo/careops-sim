# R3.1 — Deterministic Conformance & Baseline Comparison Evidence

**Status: R3.1 deterministic conformance executed and verified; zero unexplained drift detected against the frozen R1 baseline.**  
**Execution Timestamp:** `2026-10-24T09:13:19Z`  
**Parent Source:** `careops-sim` at `e8fb77b`  
**Upstream Engine:** `libs/kairos` at runtime pin `44261eb5cf8275f132f63a3396903d116d701dbe` / qualification candidate `8282b1ab45669a667bda3be2862272447544ba44`.

---

## 1. Scope & Objective

Task R3.1 requires running all affected Rust test suites, queue/calibration contracts, physical Arrow I/O, and ED simulation replays against the frozen baseline established in [`conductor/evidence/r1-native-baseline-20261021/`](../r1-native-baseline-20261021/README.md):
1. **Parent Workspace Suite:** Run the full root suite (ED models, synthetic holdouts, CLI, and integration tests) and verify conformance with the 296 passed baseline.
2. **Upstream Q/C Suite:** Run DES, ABM, and Calibration with Flow and test-support features, verifying conformance with the 969 passed / 6 ignored baseline.
3. **Physical Arrow I/O & Lifecycle Telemetry:** Run typed Arrow IPC and Parquet roundtrips and C43 calibration sidecar pipelines.
4. **C2 Process Replay:** Verify process-level save, two byte-identical restores, and sentinel collision rejection against the `kairos-c2-replay-journal-v1` contract.
5. **ED Nominal Simulation Comparison:** Replay nominal ED scenario (`nominal_mvp.json`, seed 42, horizon 20 ticks), verify repeat byte-identity, and perform an exact comparison against frozen baseline `first.json`.

---

## 2. Conformance Comparison Matrix

| Suite / Test Group | Frozen R1 Baseline (`2026-10-21`) | Current Run (`2026-10-24`) | Conformance Status | Notes / Invariant Checks |
| :--- | :--- | :--- | :--- | :--- |
| **Parent Workspace Suite** | 296 passed, 0 failed | **296 passed, 0 failed** | **PASS (Exact Match)** | Full ED simulation, synthetic holdout evaluation, CLI integration. |
| **Upstream Q/C Flow Suite** | 969 passed, 6 ignored | **969 passed, 6 ignored** | **PASS (Exact Match)** | Queue, scheduler, preemption, and calibration mechanics. |
| **Arrow I/O (IPC & Parquet)** | 31 passed, 0 failed | **31 passed, 0 failed** | **PASS (Exact Match)** | Arrow 60 / Parquet 60 schema roundtrip and limits. |
| **Arrow Lifecycle Telemetry** | 31 passed, 0 failed | **31 passed, 0 failed** | **PASS (Exact Match)** | Lifecycle event log schemas and signed span traces. |
| **Calibration C43 Outputs** | 22 passed, 0 failed | **22 passed, 0 failed** | **PASS (Exact Match)** | Sidecar adapters and residual pipeline invariants. |
| **C2 Replay Demo Save/Restore** | Byte-identical restore logs | **Byte-identical restore logs** | **PASS (Exact Match)** | Restore 1 matches Restore 2 byte-for-byte (`diff -u` exit 0). |
| **C2 Sentinel Safety** | Exit 1, destination preserved | **Exit 1, destination preserved** | **PASS (Exact Match)** | Sentinel collision rejected; sentinel bytes uncorrupted. |
| **ED Nominal Run Repeat** | Byte-identical repeat | **Byte-identical repeat** | **PASS (Exact Match)** | Run 1 output matches Run 2 output byte-for-byte. |
| **ED Frozen Baseline Summary** | 5 arr, 5 started, 5 completed | **5 arr, 5 started, 5 completed** | **PASS (Exact Match)** | Every patient arrival, start, and completion tick matches `r1-native-baseline-20261021/ed/first.json`. |

---

## 3. Detailed ED Baseline Conformance Check

Comparison between [`conductor/evidence/r1-native-baseline-20261021/ed/first.json`](../r1-native-baseline-20261021/ed/first.json) and [`ed-nominal-run.json`](ed-nominal-run.json):

```json
{
  "summary": {
    "schema_version": 1,
    "scenario_id": "nominal-ed-mvp",
    "seed": 42,
    "time_unit": "tick",
    "horizon_ticks": 20,
    "arrivals": 5,
    "started": 5,
    "completed": 5,
    "unfinished": 0,
    "patients": [
      {"patient_id": "patient-001", "arrival_tick": 1, "start_tick": 1, "completion_tick": 4, "work_ticks": 3, "wait_ticks": 0, "elapsed_ticks": 3, "status": "completed"},
      {"patient_id": "patient-002", "arrival_tick": 1, "start_tick": 1, "completion_tick": 5, "work_ticks": 4, "wait_ticks": 0, "elapsed_ticks": 4, "status": "completed"},
      {"patient_id": "patient-003", "arrival_tick": 3, "start_tick": 4, "completion_tick": 7, "work_ticks": 3, "wait_ticks": 1, "elapsed_ticks": 4, "status": "completed"},
      {"patient_id": "patient-004", "arrival_tick": 5, "start_tick": 5, "completion_tick": 7, "work_ticks": 2, "wait_ticks": 0, "elapsed_ticks": 2, "status": "completed"},
      {"patient_id": "patient-005", "arrival_tick": 6, "start_tick": 7, "completion_tick": 10, "work_ticks": 3, "wait_ticks": 1, "elapsed_ticks": 4, "status": "completed"}
    ]
  }
}
```

- **Manifest:** Scenario `nominal-ed-mvp`, seed `42`, horizon `20` ticks identical.
- **Validation Taxonomy:** Verification scope `software_verification`, provenance class `synthetic` identical.
- **Summary Metrics:** Exact 100% equivalence across all patient records and timelines.

---

## 4. Drift Analysis

- **Intentional Versioned Differences:** None introduced. The simulation engine and upstream dependencies maintain strict backward compatibility.
- **Unexplained Drift:** **ZERO**. All test counts, execution codes, and output payloads match the frozen baseline with full determinism.

---

## 5. Retained Receipts

All executed commands, standard output logs, elapsed execution timings, and SHA-256 digests are recorded in [`receipts.json`](receipts.json):
- [`parent-workspace-full.log`](parent-workspace-full.log) (SHA-256: `4958e01222af58e083b6ed4a2fbf7f723abc07ce1e130bd44007601c2596d636`)
- [`flow-q-c-suite.log`](flow-q-c-suite.log) (SHA-256: `e9123580b2449fd9e0d86cddcbf8dd82c45486e4755e677362dcff2a10a3d722`)
- [`arrow-io-ipc-parquet.log`](arrow-io-ipc-parquet.log) (SHA-256: `2092323a78c3c126d4ae7eb230a133f990a07153a8123286b245e3ea339dae7e`)
- [`arrow-telemetry-lifecycle.log`](arrow-telemetry-lifecycle.log) (SHA-256: `18507204918e9505f013d87b322daefee5a32b679b360b97361fc37f1917f300`)
- [`calibration-c43-outputs.log`](calibration-c43-outputs.log) (SHA-256: `da6e75a89f3680e6e76cf0d859fa424bc4fa672d0ea62283a216f40445d070b4`)
- [`c2-demo-save.log`](c2-demo-save.log), [`c2-demo-restore-1.log`](c2-demo-restore-1.log), [`c2-demo-restore-2.log`](c2-demo-restore-2.log), [`c2-demo-sentinel-rejection.log`](c2-demo-sentinel-rejection.log)
- [`ed-run-first.log`](ed-run-first.log), [`ed-run-second.log`](ed-run-second.log)
- [`ed-nominal-run.json`](ed-nominal-run.json) (SHA-256: `2ef0b777a876a3eb79be7a873832d2012d92555776a3e6bb69c02580ecadbb3d`)
- [`c2-checkpoint.json`](c2-checkpoint.json) (SHA-256: `532acb205e99099300e1311160550308740885c0c52cfcab6db4b76d79c7244e`)
