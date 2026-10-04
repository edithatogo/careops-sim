# Q5.2 development qualification — 2026-10-05

**Status: Q5.2 accepted for development; Q5.3 is next.** The full runtime matrix, correctness checks and two isolated native CI comparisons pass. This acceptance advances the parent development pin and keeps compatibility, backend, security and release gates separate. The queue phase remains open.

## Immutable measured source and publication equivalence

The measured Kairos source is `e06b4aa3cabcd7a6c84cc06193f72d95b1ed763f` (clean tree, Rust 1.99.0, default features, locked release build). The final matrix receipt is `.artifacts/q52-runtime/final-matrix/20261004T153823647541Z/result.json`, SHA-256 `372899d75d89e91789286db7067794dfb2317acf9889f30bc50cdaa3a4025632`; its producer input SHA-256 is `c7962fbaa5ba273cfefd940585d3c10ca82a14639c0a35f866f9576d0ed69aa2` and binary SHA-256 is `aa73f80883a85fe900e1dba832deed898cfe62afbbe4268cafc4d24cf0e37ab8`.

The measured matrix completed all **39 cases × 5 repeats = 195 successful child processes**, with seed 42, a 30-second per-child deadline, and zero timeouts. The 13 cases at `n=100000` completed. The earlier immutable run at `d403b18ed97840effa76d06e6841c2ee333db786` remains preserved as historical evidence: 26/39 cases completed (130 successful repeats); the other 13 large cases each timed out on their first attempt (143 child attempts total). It is superseded for the final performance matrix, not erased.

The published Kairos head is `8daa0978578b8a5b5b6427e84db1a3e6c54a1123`. It retains accepted C-01 and adds the reviewed native CI qualifier and its contract. The Q5.2 runtime and collector source/build family is byte-identical to measured `e06b4aa`, independently reviewed; measurements remain bound to their actual producer heads. The C-01 acceptance itself remains synthetic invariance evidence only and is preserved independently.

## Runtime measurement limits

The measured dispatch interval covers `FlowRuntime` step, lifecycle classification/counting, and causal resource staging/commit. Correctness readbacks are outside that timer. Setup has its own timer. Peak RSS is per-child `wait4` maximum resident set, normalized to bytes (Darwin source units are bytes); it includes process setup and child output readback and is not per-claim allocation. Reported p95 uses nearest-rank on five completed samples, so p95 equals the maximum observed sample and is not a tail estimate. Matrix cases distribute total waiting population `n` across `R` resources; the interruption scenario uses `n` urgent arrivals. Collector qualification proves local source/build/binary provenance and bounded measurement integrity, not the Q5.2 acceptance threshold.

Kernel data is deliberately narrower: operations on the ordered queue primitive, key removal, replacement selector, and the legacy `Resource` FIFO primitive. Legacy comparison does not measure a matched full Flow runtime. Active-victim scan evidence uses the public/runtime path only where the recorded scenario actually invokes it; selector microbenchmarks do not establish whole-runtime speedup. No general speedup claim is made.

## Correctness and regression status

The release DES test command on `e06b4aa` exited 0: 225 passed, 0 failed, 1 ignored across 23 test-result groups. Q5.1 canonical output hash `47cfb7dca4211252` matched across 24-trace batches at worker counts 1, 2, and 4 (including ordered/reversed/permuted schedules in the retained log). This is deterministic conformance evidence, not queue performance acceptance.

The original canonical regression comparison recorded four blocking threshold failures (`create_1m_entities`, `pop_1m_events`, `schedule_1m_events`, `schedule_cancel_1m_mixed`); the two remaining rows passed. A separate controlled serial baseline/current pair completed at F baseline `f18aba1f1930aec228e830c9e58c67e4081ac4bb` and published current `0944b8198e8208f7b2a016ad9774fbb91a895d5a`. It reports two blocking failures: `component_insert_1m` +91.0063% (3% ECS threshold) and `pop_1m_events` +10.6969% (5% scheduler threshold); four rows passed. Preserve both comparisons as separate outcomes. Thresholds remain unchanged (scheduler 5%, ECS 3%, hybrid advisory 10%). The controlled pair is a real result but it fails acceptance; metadata-only threshold checks are not a native regression pass.

## Isolated CI acceptance and delivery

Both native-owner runs passed all 12 jobs at exact PR head `8daa0978578b8a5b5b6427e84db1a3e6c54a1123`: [push run 37217403302](https://github.com/edithatogo/kairos/actions/runs/37217403302) and [PR run 37217406125](https://github.com/edithatogo/kairos/actions/runs/37217406125). The qualification job checked out the exact head, compared accepted F, prebuilt both sources before timing, interleaved six serial default Criterion commands, retained 100 samples per metric, normalized real estimates and invoked the unchanged comparator. All six metrics passed in both runs. The retained receipts identify Rust 1.99.0, Python 3.14.8, clean source before/after and source/binary/log/raw hashes. No dependency or threshold was changed.

The earlier local failures remain real failed observations in the local archive. Background activity supports a contention concern, but is not causal proof. Two separate isolated CI comparisons establish this bounded qualification; results are not a general speedup claim or guaranteed performance on all hosts.

[Draft Kairos PR #214](https://github.com/edithatogo/kairos/pull/214) remains a development handoff. This parent advances its development pin to that published, qualified source and preserves accepted C-01 and C1 independent readback. Q5.3 feature-minimal/legacy/API compatibility, Q5.4 phase review, portable checkpoint integration, Metal/GPU and distributed backends, empirical/clinical calibration and broad security/release gates remain open. No full library, clinical or MVP acceptance follows from Q5.2.

## Retained evidence

`acceptance.json` records source identities, actual outcomes, measurement limits and deferred holds. The local archive preserves 1,169 files including the complete 195-run matrix, earlier timeouts, both local failures, raw Criterion outputs, command receipts and earlier native CI. A separate isolated-CI archive preserves both current native workflow artifacts and exact-head readbacks. Inventory and archive hashes are recorded alongside them; no binaries, targets or private leases are included.
