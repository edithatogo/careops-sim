# Evidence A0.2: Generic Experiment Runner Extraction & Parity Qualification

**Date:** 2026-10-24T10:39:33Z  
**Task:** `careops_core_domain_abstractions_20261020:A0.2`  
**Status:** PASS  

## 1. Scope & Objective

Extract the generic experiment lifecycle out of `careops-ed::batch_runner` and `careops-ed::api_runner` into `crates/careops-core::experiment`, while:
- Fully preserving the `careops-ed` facade and 100% backward compatibility for all existing tests and CLI commands.
- Demonstrating reuse across two distinct domain consumers:
  1. **Clinical Emergency Department Simulation** (`careops-ed`)
  2. **Non-Clinical Logistics Sorting Hub Simulation** (`crates/careops-core/tests/logistics_consumer.rs`)
- Proving **Worker Invariance**: 1, 2, and 4 workers produce bit-identical simulation results for matching seeds.
- Proving **Cancellation Safety**: Cancelled batches never publish `batch_run.json` as complete and write `batch_run.partial`.
- Proving **Crash Recovery**: Resuming an interrupted batch from checkpoint completes remaining replications without duplicates, matching unbroken baseline.
- Benchmarking worker scaling timings.

---

## 2. Verification Matrix

| Verification Scope | Check | Result | Evidence / Details |
| :--- | :--- | :---: | :--- |
| `careops-core` Lib | Deterministic seed allocation & atomic I/O | PASS | 2/2 unit tests passed |
| `logistics_consumer` | Non-clinical consumer: determinism, 1/2/4 workers, cancel, recovery | PASS | 4/4 integration tests passed |
| `careops-ed` E32 Tests | Batch execution, checkpoints, resumption, cancellation | PASS | 9/9 integration tests passed |
| `careops-ed` Workspace | Full regression suite (ED, PDES, distributed, holdout) | PASS | 83/83 test targets passed |
| ED Worker Invariance | 1 worker vs 2 workers vs 4 workers bit-equivalence | PASS | 8 replications bit-identical |
| Logistics Invariance | 1 worker vs 2 workers vs 4 workers bit-equivalence | PASS | 8 replications bit-identical |
| Cancellation Safety | Pre-dispatch and mid-dispatch cancellation | PASS | `batch_run.partial` written, `batch_run.json` absent |
| Crash Recovery | Checkpoint resumption after simulated worker crash | PASS | Bit-identical to unbroken baseline |

---

## 3. Worker Invariance & Scaling Benchmarks

Replications: 8 nominal ED scenario replications (`nominal_mvp.json`, base seed: 12345).

| Workers | Wall Clock (ms) | Mean Wait (ticks) | Std Error | Completion Rate | Bit Equivalence |
| :---: | :---: | :---: | :---: | :---: | :---: |
| 1 | 264.7 | 0.4000 | 0.0000 | 100% | Reference Baseline |
| 2 | 220.9 | 0.4000 | 0.0000 | 100% | 100% Match (Bit-Identical) |
| 4 | 198.8 | 0.4000 | 0.0000 | 100% | 100% Match (Bit-Identical) |

---

## 4. Architectural Summary

1. **`careops-core::experiment`:**
   - [`cancellation.rs`](../../../crates/careops-core/src/experiment/cancellation.rs): `CancellationToken` (thread-safe cooperative cancellation).
   - [`seed.rs`](../../../crates/careops-core/src/experiment/seed.rs): `allocate_replication_seeds` (golden-ratio stride 64-bit integer dispersion).
   - [`io.rs`](../../../crates/careops-core/src/experiment/io.rs): `atomic_write_file` (PID-staged atomic rename).
   - [`error.rs`](../../../crates/careops-core/src/experiment/error.rs): `ExperimentError` and `ExperimentErrorKind`.
   - [`runner.rs`](../../../crates/careops-core/src/experiment/runner.rs): `IndexedReplication`, `run_generic_batch`, `resume_generic_batch`, `GenericBatchCheckpoint`, `GenericBatchRunDocument`.
2. **`careops-ed` Facade:**
   - [`crates/careops-ed/src/batch_runner.rs`](../../../crates/careops-ed/src/batch_runner.rs): Implements `IndexedReplication for ReplicationResult`; delegates `run_batch_with_workers` and `resume_batch_from_checkpoint` directly to `careops-core::experiment`.
   - [`crates/careops-ed/src/api_runner.rs`](../../../crates/careops-ed/src/api_runner.rs): Re-exports `CancellationToken` and `allocate_replication_seeds` from `careops-core::experiment`.
3. **Non-Clinical Logistics Consumer:**
   - [`crates/careops-core/tests/logistics_consumer.rs`](../../../crates/careops-core/tests/logistics_consumer.rs): Proves complete domain independence of the core simulation runner abstractions.
