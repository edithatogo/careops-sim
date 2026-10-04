# Automated test matrix — calibration and validation

All tests are planned; phase C0 freezes exact tolerances and generator versions.
Correctness and semantic assertions take precedence over aggregate coverage.

| ID / phase | Layer | Cases and automated oracle | Acceptance |
| --- | --- | --- | --- |
| trace_arrow_interop / C1 | Integration | Actual IPC/Parquet round trips; independent generated/read fixtures; physical schema and null/ID/tick preservation | C-01/07 |
| trace_normalization / C1 | Unit + property | ns/us/ms/s, timezone/DST, nonrepresentable/overflow ticks, wide/long maps, equal times, duplicate keys, source order permutations | C-01 |
| trace_batch_invariance / C1 | Determinism | Different batch/row-group sizes and spill thresholds produce identical normalized rows/hash; bounded configured memory | C-01/06 |
| trace_quality / C1 | Unit/integration | Partial-order violation, missing anchors, observation-window censoring, repeated events, occupancy overflow; exact reason/count conservation | C-01/03 |
| fidelity_policy / C2 | Unit | Override precedence, mode recorded at admission, active/suspended switches rejected, Macro no transit | C-02 |
| fidelity_paired_runs / C2 | Integration/determinism | Zero-transit Micro equals Macro on controlled fixture; service draws unchanged by enabling transit; mixed subsystem state conserved | C-02/06 |
| spatial_routes / C2 | Unit/property | Equal-length routes tie by stable IDs, unit conversion/ceil, invalid speed, unreachable nodes, graph hash; interrupted edge progress restored | C-02 |
| shadow_unclamped_residual / C3 | Integration | Predicted target before/at/after anchor; ledger stays fixed, residual has expected sign/magnitude, no time reversal | C-03 |
| shadow_isolation / C3 | Integration | Late probe continues in isolation; no future-state leak; infeasible occupancy strict rejection; unknown actor assumptions labelled | C-03 |
| shadow_resume / C3 | Determinism | Restore active and overdue probes plus ledger; exact IDs/anchors/integer residuals, no duplicate endpoints | C-03/06 |
| metrics_analytic / C4 | Unit | [0,2] vs [1,3]: W1=1, KS=.5; identical=0; separated point masses=1; unequal samples/weights/ties | C-04 |
| metrics_validity / C4 | Unit/property | Empty/NaN/Inf/negative weights invalid; large ticks precision guard; censor/missing counts visible; symmetry, translation/scale relations | C-04 |
| metrics_reference / C4 | Differential | Independent pinned implementation generates reviewed golden cases; tolerance documented; default no unjustified KS p-value | C-04 |
| calibration_recovery / C5–C6 | Integration | Known noiseless speed on candidate grid recovered exactly; seeded noisy case within declared tolerance; confounded case reports ambiguity | C-05 |
| calibration_split_integrity / C5 | Unit/integration | Patient/episode overlap and future leakage rejected; fitted distributions use training only; frozen parameters for final test | C-05 |
| calibration_worker_resume / C5 | Determinism | 1/2/N workers, permuted finish times and crash/resume; same candidate ordering/budgets/seeds and canonical outputs | C-06 |
| calibration_failure_penalty / C5 | Integration | Late/missing/unreachable probes cannot improve objective through silent exclusion; typed failures and fixed penalties/thresholds | C-03/05 |
| ed_holdout_comparison / C6 | End-to-end | Synthetic staffing/cleaning/boarding and urgent interruption; Macro/Micro free-run report agrees with fixture expectations; uncertainty/cost disclosed | C-05 |
| calibration_compatibility / C6 | Regression/build | Legacy manifest/event_log fixtures, minimal core, optional IO features, supported MSRV, no Python/GPU/renderer required | C-07 |

Integer scheduling/IDs/canonical rows require exact equality. W1/objectives use
fixed reduction order and declared absolute/relative tolerance; C0 must justify
that tolerance relative to time units and parameter ranking resolution. A backend
that changes candidate selection outside the tie tolerance fails compatibility.

Benchmark ingestion peak memory/throughput, shadow probe overhead, metric sizes
and 1/2/N candidate throughput. Keep CPU baseline and compiler/hardware recorded;
no fixed speedup is assumed. Browser/Metal/PDES gates remain separate owner work.

## Additional research-derived cases

Include the task-specific cases in [reports 9–12 integration](../../research/ed-research-incorporation-9-12-20260927.md)
when preparing executable packets. These are proposed oracles, not recorded passes.

## C-01 executed synthetic qualification — 5 October 2026

[Acceptance and boundaries](../../evidence/c01-closeout-20261005/README.md).
Three profiles independently vary IPC batches {1, 2}, Parquet row groups {1, 3},
physical order {forward, reverse}, formats {IPC file, IPC stream, Parquet} and
writer limits {1, 2, 3}: 216 actual reader-derived sources. Each covers chunk
rows {1, 2, 64}, run rows {1, 2, 64} and run bytes {4096, 1048576}.
All 3,888 points pass with 108 actual executions and 3,780 explicit byte-equivalent
aliases. Exact bytes, ordered records, rows, byte lengths and SHA-256 match for
all four populations. Observed spills, single-run controls, independent C0
checks, schema-valid negative mutations and six retained actual representative
output sets are verified. This accepts C-01 only; C1.4 remains open.
