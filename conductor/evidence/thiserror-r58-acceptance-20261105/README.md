# thiserror R56–R58 acceptance

Bounded engineering is complete at source `42e4b0d713fa66b3e0f3053935077eaba35a3add`. GitHub tested merge `23db74af9768c10f7766ed52cb6f7b519fe2e7b3` has the identical complete tree `f3645448c3b3d3e007d9d635eaed7704a104e07d`. This later documentation closeout is separate from that hosted qualification. [Structured receipt](acceptance.json) retains exact artifacts, hashes, costs and review.

## Accepted behavior and gates

Exact thiserror 2.0.21/default std replaces eight nontrivial error implementations. All public variants, messages/Debug, From conversions, serde, source(None), code/FFI mappings and domain behavior remain compatible. Seven trivial wrappers remain. Runtime pin `07410266` and all historical Polars, SHA-2 and Tempfile receipts remain unchanged.

All six applicable workflows and Required checks pass: 803 native tests on each Linux/macOS host; 793 default-feature and regenerated-lock tests; full Python suites of 840 + 127 tests; independent IPC/Parquet readers; strict Clippy, format, doctests, harness, privacy and configured root/fuzz audits. No native failures, ignored or skipped tests. The main-only attestation skip is expected for a draft PR.

All 26 frozen input/summary pairs and 14 adversarial controls pass with the original budgets, including Linux `/dev/full`, preflight limits and slow/blocked stdout. The actual packaged Linux consumer passes; all nine SPDX entry hashes, package checksum, lock and explicit notices verify. Explicit notices are not a complete binary dependency closure.

Bounded ED-library coverage is 512/578 lines (88.5813%, floor 80%). All 117 source/catalogue-bound mutants are accounted for: 111 caught, two existing reviewed equivalents, four known unviable and zero timeouts. The terminal actual catalogue `45357…` and source `16be…` were independently revalidated from the downloaded artifact; the earlier synthetic unittest record was excluded.

## Maintenance and costs

The eight production files lose 115 formatted lines. Qualification tools, Rust contracts and guard support produce net +1,106 tracked Rust/Python lines overall; fixture/lock/documentation growth is separate. Reviewed test plumbing was reduced 474 lines before adoption. No overall code reduction is claimed.

Frozen error-format measurements are 57.04→91.30 ns/error, ratio 1.60045: the ratio arm fails, while the original absolute +34.25 ns ≤500 ns OR arm passes. The independent isolated Mach-O grows 112 bytes; actual full Linux ELF is 109,598,080 bytes (232 bytes smaller than Tempfile). Release compilation of 350 s versus the prior 545 s is an observation, not a paired or causal speedup claim.

## Review and disposition

Reviewer agent `/root/tempfile_phase_review` gives final R58 GO with no findings. The strict 29-file successor preserves all earlier receipt checks and 15 shared bindings. Actual old-artifact readers and foreign-model recovery refusal remain source-bound R57 proof. All 405 task IDs/dependencies remain; only accepted phase flags advance.

[PR202](https://github.com/edithatogo/careops-sim/pull/202) remains draft at the frozen qualified head. PR200/201 and upstream ownership/pins remain intact. No merge, stable release or clinical/site/operational acceptance is implied. Existing Track 70 physical-durability obligations remain separate. No new track is dispatched.
