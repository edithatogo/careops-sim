# Plan: CareOps whole-layer Polars migration

R10–R12 retain their scoped acceptance. R13 local engineering is accepted at f7563a8; R14 engineering is accepted at258bb6b with the final hosted source, interoperability, performance, package and mutation receipts. The track is archived.

## R10 — Upstream entry and baseline

- [x] R10.1 Verify accepted upstream Track 70 receipt and immutable compatible source. Canonical manifest 5692e6a, PR #304 40 passed/3 skipped and reconciliation PR #306 25 passed are retained in [remote prerequisite evidence](../../evidence/r103-remote-20261029/prerequisites.md). Exact bridge 0741026 passed 588 native tests on each required host; old and renewed contracts are retained.
- [x] R10.2 Inventory current remote ED/core/CLI table and consumer paths, statistical definitions and supporting dependency targets; record the 26-case seed 7 baseline and input/summary parity. [Scoped inventory and evidence](../../evidence/r103-remote-20261029/prerequisites.md) explicitly excludes the unintegrated advanced identity/birthing lane. No real event-trace capture, production join cardinality or clinical acceptance is claimed.
- [x] R10.3 Add reference tests and qualify compatible Kairos pin 0741026. Accepted by code PR #188 at 0db23f1: local 358 Rust tests, 728 Python tests, full Clippy, exact pin contract and 26 benchmark parity pass; child native owner passes both hosts. Final parent hosted head ef1f0b0 passes all required checks, including 116 accounted mutations and 88.18% ED line coverage. See [bounded remote packet](../../evidence/r103-remote-20261029/README.md). Trace invocation count and capture/removal remain R11.1/R11.2.
- [x] R10.4 Conductor — phase review accepted against merged e65570b; stale coordination records reconciled, exact receipts and rejected/cross-owner work retained. [Phase review and acceptance](review-report.md#r10-phase-review-and-acceptance--30-october-2026) permits R11.1 preparation only, with fresh packet and failing behavioral oracles.

## R11 — Capture once and durable trace parts

- [x] R11.1 Add failing run-invocation-count, bounded-buffer/backpressure, disk-error and checkpoint-resume duplicate/loss tests. Retained authoring acceptance3952f51; combined local qualification at3ee7ac1:69targeted and638workspace Rust tests, policy/pin/support and774Python tests pass; independent review ready.
- [x] R11.2 Capture required tables during original replications and write bounded per-replication parts using Kairos data facilities; remove export-time reruns. Bounded local engineering accepted at3ee7ac1 after the retained PR191 source integration,638workspace Rust/774Python tests,26actual frozen/current seed7 differential cases, locked policy/pin/support checks and independent review. Full R11phase/hosted acceptance remains R11.4; R11.3 crash/durability defects stay open.
- [x] R11.3 Bind trace part IDs/watermarks to restart state and publish manifest last; preserve exact event/RNG behaviour across worker counts and restarts. Local engineering accepted at244907e after durable journal/model/envelope/attempt recovery,24real process-kill cases at1/2/4workers,646Rust/774Python tests, native CLI/PyArrow checks and independent review with nested-schema fix applied. Full phase/hosted gate remainsR11.4.
- [x] R11.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.

## R12 — Rust Polars domain processing

- [x] R12.1 Add analytical tests for weighted versus unweighted estimands, ddof/intervals, censoring, empty/singleton, null/nonfinite, skew and duplicate/missing join keys. Authoring accepted atb143bd1:14compiling baseline controls (5positive/9actual method failures), frozen numerical/key/B7 vectors and reviewed scientific contract. This is test-authoring completion only; typed DataFrame oracles and green runtime acceptance remain R12.2/R12.4.
- [x] R12.2 Replace applicable ED and birthing batch transformations, joins, summaries, repeated statistics and calibration-trace preparation with Polars expressions; keep online/domain logic authoritative.
- [x] R12.3 Use existing modules for domain definitions and Kairos for generic interchange; extract shared helpers only for actual duplicate consumers, not speculative frameworks.
- [x] R12.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.

R12 local engineering accepted at65731f0 after coherent739native/774top-level
Python/8EDmirror passes and independent review. Exact legacy R13 dispositions
and compiler/hosted distinctions are in the R12 evidence; final-head hosted
and R13/R14 consumer/FFI/performance/removal gates remain open.

## R13 — Outputs and consumers

- [x] R13.1 Add independent-reader tests for real Parquet/Zstd and IPC, explicit CSV quoting/limb fields, effective input hashes and incomplete manifest rejection. Local format/test acceptance at `b1b91ca`; exact hosted parent readers pending, production migration remains R13.2/R13.3.
- [x] R13.2 Replace JSON full-table bundles/schema DDL/manual CSV output with versioned Parquet tables plus provenance manifests; migrate CLIs and every known downstream fixture/script. Local source accepted at f7563a8 after the coherent776-test workspace pass and recorded Python/native consumer evidence; final-head hosted acceptance remains R14.
- [x] R13.3 Regenerate owned synthetic fixtures, preserve historical files, remove old alpha data APIs/readers and document clean-break run/output instructions. Owned fixture views and consumers are migrated, historical inputs/receipts preserved, 2382 obsolete production lines removed, and current guides/CI discovery reconciled; see [R13 acceptance](../../evidence/polars-r13-acceptance-20261104/README.md).
- [x] R13.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision. Local engineering accepted at f7563a8 after independent packet reviews, actual776 workspace tests,796 top-level Python tests and all127 ED cases accounted with the corrected exact E61 case. [Acceptance and remaining gates](../../evidence/polars-r13-acceptance-20261104/README.md) retain the original failures, exact release26-case receipt and final R14/hosted boundaries.

## R14 — Acceptance and removal

- [x] R14.1 Run native domain/replay/restart/numerical/interchange lanes and existing performance gates; measure small/large ensembles, output latency, peak RSS, thread contention and build size.
- [x] R14.2 Audit net maintained-code deletion and all retained candidates; remove obsolete compatibility and duplicate paths and reconcile docs without claiming clinical acceptance.
- [x] R14.3 Record accepted parent and Kairos commits, hosted/local evidence distinctions, rollback instructions and exact pin/schema/library receipts; close only after full consumer validation.
- [x] R14.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.

## External gate and scheduling

R10.1 is a manual receipt-verification gate. The local task DAG cannot certify another repository; a ready candidate is not dispatch authority. No pin/code mutation until upstream acceptance. Existing R-prefix reservations are conservative; narrow every packet to this track contract and never claim the whole submodule for parent work.

## Dependency sequencing refinement — 22 October 2026

Track 70 starts with Phase 0 inventory and migration-boundary decisions; Tracks 93/94/96 are not blanket entry or runtime-migration prerequisites. For each digest, staging or property-test target, record its path, owner and disposition: removed by Polars, retained and required for a named migration task, or independent. Before dispatching an affected task, record the exact prerequisite task/receipt and acceptance oracle in the owning plan and dispatch packet (and the CareOps task graph where representable). Unresolved required work blocks that task only; unrelated migration work may proceed under existing ownership and contract gates. Do not retrofit code scheduled for deletion. If no targets remain, record a reviewed scope-elimination decision without claiming adoption. Required integrity, durability, interoperability, replay and independent test evidence remain mandatory in Polars. Separate dependency tracks and current owner handoffs remain in force.

## Remote-baseline reconciliation

R10 retains accepted repository reconciliation R3 as its entry prerequisite. The original 44261eb retention gate was satisfied by the bounded capability-preserving 0741026 bridge, accepted in PR #188 and recorded in PR #189. The old contract and receipts remain immutable historical evidence. Advanced local candidate runtime/archival commits remain separate and are not evidence of their remote integration.

Final R14 acceptance: [source258 receipt](../../evidence/polars-r14-qualification-20261104/README.md). All20 task IDs and original R10–R13 scopes are preserved.
