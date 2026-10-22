# Plan: CareOps whole-layer Polars migration

All implementation work is pending. Write failing behavioural tests before replacement.

## R10 — Upstream entry and baseline

- [ ] R10.1 Verify Kairos Track 70 Phase 4 accepted immutable commit, checks and receipt manifest; refuse pin adoption on status labels alone. This external gate remains unresolved at registration.
- [ ] R10.2 Inventory ED/birthing/shared/script table paths, join cardinality, metrics/denominators, fixtures and consumers; record baseline traces and resource measurements.
- [ ] R10.3 Add failing reference tests for existing statistical definitions and trace-once exports; update parent Kairos pin and lockfiles only to the accepted compatible upstream commit.
- [ ] R10.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.

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

R10 upstream entry remains. Before R11 starts require parent sha2 R52, tempfile R55 and proptest R61 accepted receipts. These parent prerequisite tracks have no dependency on completed Polars, preventing a cycle. R11/R13 retain real-format interruption, durability and resume tests; prerequisite acceptance is not output acceptance.

## Remote-baseline reconciliation

R10 additionally requires accepted repository reconciliation R3. Preserve the reviewed 44261eb Kairos pin until the existing capability-preserving upgrade gates pass. Local candidate runtime/archival commits are preserved separately and are not acceptance evidence.
