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

## C1 executed independent manual readback — 5 October 2026

[Readback and count acceptance](../../evidence/c1-independent-readback-20261005/README.md).
666 actual retained files pass direct PyArrow schema/type/unit/null/payload reads;
8 in-memory negatives reject. A separate standard-library verifier reconciles
3 C-01 profiles and all51 C1.1 request partitions, including failures/unresolved
units and distinct observed/right-censored outcomes. Null, absent and empty
observations are recorded on logical paths. This closes the manual check only;
C1.4 remains open and the accepted Kairos pin is unchanged.

## C1.4 accepted synthetic ingestion phase — 2026-10-05

Reviewed integrated development pin `34e1d776e0ab4e2c84eb07f7c45779220df7e9b1` preserves accepted Q5.2 and the
90 exact tested C1 source files. Fresh native tests and C-01 runs, 666 independent
IPC/Parquet file reads, schema/units/nulls, negative controls and input/accepted/
excluded/censored count reconciliation pass. Final source has 48 successful
hosted checks and two documented conditional skips; phase, DAG, strict-clean and
evidence-boundary checks pass. See [C1.4 completion](../../evidence/c1.4-completion-20261005/README.md).
Historical failed attempts remain retained. C2, public API review, clinical
validation and release remain open. Earlier proposed/planning statements describe
their original delivery, superseded for C0/C1 only by recorded acceptance.

## C4.1 fixture preparation — 5 October 2026

Accepted source `6a17578a16d4bb13969961dfd116b3afaf7c8c6c`, evidence successor `a2cdeab33286e14db59f377449816702e79b2a6a`: 42 synthetic fixtures, eight native tests on Rust 1.99/1.88, ten Python tests, 23 pinned SciPy crosschecks and 28 independent exact numeric readbacks. Archive independently verified (137 members). Exact-head Linux/macOS native owner CI passed. [Evidence](../../evidence/c4.1-completion-20261005/README.md). Actual runtime candidate gate remains expected red; C4.2, C4.3, C4.4/C-04, clinical and release acceptance remain open.

## C4.2 private runtime qualification — 5 October 2026

Actual runtime source `59d7dbb0c004654e3da90951d42e2b983ef23993`; reviewed development head `21e48b257b5c89dd9756b647d9ee17f698f9496b` in Kairos draft PR221. All 42 frozen cases pass in Rust on 1.99/1.88 and an independent exact Python comparator. Full calibration: 119 pass/4 named ignores on each toolchain; strict current Clippy and formatting pass. False commit/toolchain and mutated metric fail as required. Exact-head Linux/macOS native owner CI and all 42 non-skipped child checks pass. Archive142 members independently read back. [Evidence and limits](../../evidence/c4.2-completion-20261005/README.md). Private API only; C4.3 Arrow, C4.4/C-04, clinical/release gates remain open. Historical C1/Q5/C4.1 records remain intact; parent publication is recorded separately.

## C4.4 accepted synthetic statistical phase — 2026-10-05

Compiled source `79fac7ea52b8c21650759ef2d137104fa421c48d`, evidence publication `fd211888a4f435bc3679fd40589e40b482b92cf7`. Source-derived exact tie counts and censor/coverage warnings are independently reconstructed from actual IPC/Parquet source and join manifests. All 42 frozen cases pass C0/C4.1 tolerances on explicitly pinned Rust 1.99/1.88; full calibration 146 pass/four named ignores each, strict current Clippy, framing readback and actual supplemental cases pass. Six independent actual-output tampering controls and strict manifest/statistical controls reject inconsistent evidence. Archive592 safe members independently verified. Exact-head hosted CI: 48 success/two conditional skips; native Linux/macOS and all Rust 1.88 feature lanes pass. [C4.4 evidence and limits](../../evidence/c4.4-completion-20261005/README.md). C-04 closes for synthetic implementation conformance. C2, public API, release, clinical and full MVP acceptance remain open. Earlier phase records remain historical snapshots.
