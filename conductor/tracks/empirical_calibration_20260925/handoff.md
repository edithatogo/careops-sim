# Handoff — Empirical calibration and validation

## Current state

C0/C1/C2/C3/C4 are accepted for their recorded bounded synthetic development
scopes. C3 acceptance below supersedes the original planning-only handoff.
Kairos is pinned to the qualified C3 development commit; the wider track remains
in progress, with clinical/ED, public API and release gates separate.

## Next implementation action

Prepare C5.1 fixture/contracts and bounded candidate, CLI and CPU worker packets.
Preserve deterministic seeds, complete attempt accounting and existing C3 policies.
C5/C6 remain unimplemented; do not infer full calibration or ED readiness.

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

## C3 accepted synthetic shadow implementation — 10 October 2026

[C3 closeout](../../evidence/c3-completion-20261010/README.md) accepts C3.1–C3.4/C-03
at `da327cf263ff6e4507f48f597c201bb269680d51`. Rust 1.99-only local native/Arrow,
recovery, isolation, residual/feasibility and identifiability oracles pass. Final
hosted checks: 47 success/two conditional skips; both actual native host trees and
checkpoint/results independently reconcile. Earlier pending C3 records retain
historical scope. Next C5; stable API, clinical/ED, whole MVP and release remain open.

## C5 acceptance handoff — 11 October 2026

[C5 acceptance](../../evidence/c5-completion-20261011/README.md) binds Kairos6d733f4 and Rust1.99 to actual local/hosted
results: bounded search, split/censor/CRN/precision fixtures, known native recovery,
legacy CLI,1/2/4 worker parity and cooperative/kill resume. Fixed confirmation
remains MC-indeterminate; C-05 ED holdout/C6, clinical, public API and release
claims remain open. Next preparation candidate is E1.1; completed C5 is not
authority to bypass ED/MVP dispatch gates. Prior scope/CI failures are retained.
