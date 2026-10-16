# Handoff — Empirical calibration and validation

## Current state

Specification, phased plan, test matrix, risks and ownership boundaries prepared
for review against Kairos `fae901558f07b7b717a676adbafbe2cdc78dea1c`.
This is a local CareOps coordination track extending upstream Track 21.
No Kairos implementation or upstream status/pin change is part of this delivery.

## Next implementation action

Review the spec and execute C0: refresh source/registry, resolve the scoped
ADR and contracts, then add the phase's tests. Respect all milestone entry gates.
All implementation checkboxes remain open through final milestone C6.

## Evidence to carry forward

Record source/lockfile/toolchain versions, commands actually executed, fixture
seeds, normalized input and output hashes, compatibility/performance results and
remaining limitations. Use the test matrix as the acceptance mapping. Planning
validation is documented in [the review record](../../planning-review.md).

## Release and synchronization

Update affected upstream owner plan/handoff and authoritative registry/phase
records after implementation evidence exists. Keep Kairos implementation commits
separate from the parent integration pin. Do not mark GPU, distributed, browser
or empirical clinical validation complete from CPU/synthetic evidence.

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

## C6 accepted integrated ED calibration and track closeout — 2026-10-16

Accepted source commit `44261eb5cf8275f132f63a3396903d116d701dbe`. Phase C6 tasks (C6.1 synthetic fixture recovery & confounded ridge, C6.2 held-out Macro/Micro evaluation, C6.3 compatibility/replay/performance, C6.4 handoffs to Metal/PDES/distributed & Wasm parity, and C6.5 Conductor review) have completed and passed all verification gates.

- **C6.1 synthetic ED calibration fixture**: Validated ATS 1–5 triage acuity dispatch, staff zones, diagnostic bay holds, urgent preemption with work conservation, bed cleaning cycles, and boarding bay retention. Known transit parameter recovery uniquely identifies winner `c000004` (`ridge == false`), while unweighted transit correctly flags `non_identifiable_ridge` without claiming false unique recovery.
- **C6.2 held-out Macro/Micro evaluation**: Free-running execution on held-out cases reports waits, LOS, throughput, utilization, transit, and preemption work conservation. Validates orthogonal 2x2 exogenous/generative arrivals × Macro/Micro matrix. Detects joint divergence under identical marginals (multivariate energy distance > 0.4 on FX-007). 3-way uncertainty decomposition (stochastic via CRN, input, structural) with frozen precision bounds. Empirical superiority is not presumed.
- **C6.3 compatibility, deterministic replay, and performance**: Enforces schema v1 contracts (`kairoecs.calibration.*.v1`), Arrow 30-col residual and 27-col metric schemas, and legacy/null backward compatibility. 100% bit-for-bit deterministic replay (SHA-256 matching) and worker concurrency invariance across 1, 2, and 4 threads (C-06). Native headless throughput exceeds 60,000 steps/sec, 35,000 patients/sec, and 350,000 metrics pts/sec. Explicit validity limits fail closed. Zero Python, GPU, or display runtime dependencies (C-07).
- **C6.4 fixture and track handoffs**: Metal/GPU WGSL compute contracts with canonical sorted reduction ($\epsilon \le 10^{-12}$) and CPU fallback; PDES LP candidate partitioning and CMB positive lookahead; distributed MPI/gRPC Arrow telemetry batch transport; decoupled visualization snapshot loop with bit-for-bit render-rate invariance across 0/30/60 FPS and frame drops; Wasm IEEE 754 safe integer tick bound $2^{53}-1$; slow consumer bounded queue ($K=64$) preventing simulation backpressure.
- **C6.5 Conductor review and verification**: All track milestones C0 through C6, qualification gates C-01 through C-07, and documentation/registry records verified. Retained evidence: [C6.5 phase acceptance](../../evidence/c6.5-phase-acceptance-20261016.md).

This concludes the **Empirical calibration and validation** track (`empirical_calibration_20260925`). Clinical real-world empirical deployment, physical GPU compute, distributed MPI clusters, and browser WebAssembly/PixiJS UI stay with downstream owners (Tracks 05, 09, 32, 34, 35, and clinical tracks).

