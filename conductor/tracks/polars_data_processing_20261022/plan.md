# Plan: CareOps whole-layer Polars migration

R10 entry and baseline are reconciled for the bounded remote packet. Runtime Polars replacement remains pending; write failing behavioural tests before replacement.

## R10 — Upstream entry and baseline

- [x] R10.1 Verify accepted upstream Track 70 receipt and immutable compatible source. Canonical manifest 5692e6a, PR #304 40 passed/3 skipped and reconciliation PR #306 25 passed are retained in [remote prerequisite evidence](../../evidence/r103-remote-20261029/prerequisites.md). Exact bridge 0741026 passed 588 native tests on each required host; old and renewed contracts are retained.
- [x] R10.2 Inventory current remote ED/core/CLI table and consumer paths, statistical definitions and supporting dependency targets; record the 26-case seed 7 baseline and input/summary parity. [Scoped inventory and evidence](../../evidence/r103-remote-20261029/prerequisites.md) explicitly excludes the unintegrated advanced identity/birthing lane. No real event-trace capture, production join cardinality or clinical acceptance is claimed.
- [x] R10.3 Add reference tests and qualify compatible Kairos pin 0741026. Accepted by code PR #188 at 0db23f1: local 358 Rust tests, 728 Python tests, full Clippy, exact pin contract and 26 benchmark parity pass; child native owner passes both hosts. Final parent hosted head ef1f0b0 passes all required checks, including 116 accounted mutations and 88.18% ED line coverage. See [bounded remote packet](../../evidence/r103-remote-20261029/README.md). Trace invocation count and capture/removal remain R11.1/R11.2.
- [x] R10.4 Conductor — phase review accepted against merged e65570b; stale coordination records reconciled, exact receipts and rejected/cross-owner work retained. [Phase review and acceptance](review-report.md#r10-phase-review-and-acceptance--30-october-2026) permits R11.1 preparation only, with fresh packet and failing behavioral oracles.

## R11 — Capture once and durable trace parts

- [ ] R11.1 Add failing run-invocation-count, bounded-buffer/backpressure, disk-error and checkpoint-resume duplicate/loss tests.
- [ ] R11.2 Capture required tables during original replications and write bounded per-replication parts using Kairos data facilities; remove export-time reruns.
- [ ] R11.3 Bind trace part IDs/watermarks to restart state and publish manifest last; preserve exact event/RNG behaviour across worker counts and restarts.
- [ ] R11.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.

## R12 — Rust Polars domain processing

- [ ] R12.1 Add analytical tests for weighted versus unweighted estimands, ddof/intervals, censoring, empty/singleton, null/nonfinite, skew and duplicate/missing join keys.
- [ ] R12.2 Replace applicable ED and birthing batch transformations, joins, summaries, repeated statistics and calibration-trace preparation with Polars expressions; keep online/domain logic authoritative.
- [ ] R12.3 Use existing modules for domain definitions and Kairos for generic interchange; extract shared helpers only for actual duplicate consumers, not speculative frameworks.
- [ ] R12.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.

## R13 — Outputs and consumers

- [ ] R13.1 Add independent-reader tests for real Parquet/Zstd and IPC, explicit CSV quoting/limb fields, effective input hashes and incomplete manifest rejection.
- [ ] R13.2 Replace JSON full-table bundles/schema DDL/manual CSV output with versioned Parquet tables plus provenance manifests; migrate CLIs and every known downstream fixture/script.
- [ ] R13.3 Regenerate owned synthetic fixtures, preserve historical files, remove old alpha data APIs/readers and document clean-break run/output instructions.
- [ ] R13.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.

## R14 — Acceptance and removal

- [ ] R14.1 Run native domain/replay/restart/numerical/interchange lanes and existing performance gates; measure small/large ensembles, output latency, peak RSS, thread contention and build size.
- [ ] R14.2 Audit net maintained-code deletion and all retained candidates; remove obsolete compatibility and duplicate paths and reconcile docs without claiming clinical acceptance.
- [ ] R14.3 Record accepted parent and Kairos commits, hosted/local evidence distinctions, rollback instructions and exact pin/schema/library receipts; close only after full consumer validation.
- [ ] R14.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.

## External gate and scheduling

R10.1 is a manual receipt-verification gate. The local task DAG cannot certify another repository; a ready candidate is not dispatch authority. No pin/code mutation until upstream acceptance. Existing R-prefix reservations are conservative; narrow every packet to this track contract and never claim the whole submodule for parent work.

## Dependency sequencing refinement — 22 October 2026

Track 70 starts with Phase 0 inventory and migration-boundary decisions; Tracks 93/94/96 are not blanket entry or runtime-migration prerequisites. For each digest, staging or property-test target, record its path, owner and disposition: removed by Polars, retained and required for a named migration task, or independent. Before dispatching an affected task, record the exact prerequisite task/receipt and acceptance oracle in the owning plan and dispatch packet (and the CareOps task graph where representable). Unresolved required work blocks that task only; unrelated migration work may proceed under existing ownership and contract gates. Do not retrofit code scheduled for deletion. If no targets remain, record a reviewed scope-elimination decision without claiming adoption. Required integrity, durability, interoperability, replay and independent test evidence remain mandatory in Polars. Separate dependency tracks and current owner handoffs remain in force.

## Remote-baseline reconciliation

R10 retains accepted repository reconciliation R3 as its entry prerequisite. The original 44261eb retention gate was satisfied by the bounded capability-preserving 0741026 bridge, accepted in PR #188 and recorded in PR #189. The old contract and receipts remain immutable historical evidence. Advanced local candidate runtime/archival commits remain separate and are not evidence of their remote integration.
