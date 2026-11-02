# Conductor status — 3 November 2026

Snapshot 2026-11-03T03:36:52+10:00, branch `codex/core-a7-prerequisites` / PR197.
Core A0–A7: 32/32 engineering tasks accepted. Exact source 99cb1c7 passed independent
logic/empirical-contract review; combined ef613be passed 17 hosted checks plus the
expected main-only attestation skip. A7 adds 16 contract controls; hosted full
Python 766 passed; critical ED coverage88.1834%,116 canonical mutants with 0 viable
survivors / 0 timeouts. Final documentation-head checks remain an integration gate;
[PR197](https://github.com/edithatogo/careops-sim/pull/197) is unmerged.

[PR194](https://github.com/edithatogo/careops-sim/pull/194) merged normally at 672d3e8.
[PR191](https://github.com/edithatogo/careops-sim/pull/191) remains a qualified,
mergeable draft at 4b0a264: 17 successful checks + expected attestation skip. Original
core/compatibility/Polars worktrees and the parent checkout retain their original
clean revisions; private ownership readback is retained outside Git.

## In-progress tracks

| Track | Phases/tasks | Remaining work or acceptance boundary |
| --- | --- | --- |
| Generic ED | E0–E8, 36/36 historically checked | Capability reconciliation remains open: disputed E6/E8 hardware claims require exact device/multi-host receipts when consumed. Historical 100% is not operational acceptance. |
| Birthing | B0–B7, 32/32 generic engineering accepted | Full specification approval remains false; clinical/site/operational acceptance and source integration remain open. Do not restart accepted generic phases. |
| Whole-layer Polars | R10, 4/20 tasks accepted; R11–R14, 16 tasks open in this branch | R11.1 test authoring is qualified on separate unmerged PR191. Original owner retains that lane; runtime capture/processing/output replacement and full migration acceptance remain pending. B7 consumer IO is not whole-layer migration acceptance. |

## Phase and task detail

### Generic ED library delivery

- **E0 — Domain contract and minimal buildable skeleton**: 4/4 checked; tasks E0.1, E0.2, E0.3, E0.4.
- **E1 — Generic patient pathway on shared DES/ABM state**: 4/4 checked; tasks E1.1, E1.2, E1.3, E1.4.
- **E2 — Staff, capacity, spatial tasks and realistic operational constraints**: 4/4 checked; tasks E2.1, E2.2, E2.3, E2.4.
- **E3 — Usable experiment API, runner, outputs and recovery**: 4/4 checked; tasks E3.1, E3.2, E3.3, E3.4.
- **E4 — Complete and harden the native ED library**: 4/4 checked; tasks E4.1, E4.2, E4.3, E4.4.
- **E5 — Interactive scenario dashboard and binding qualification**: 4/4 checked; tasks E5.1, E5.2, E5.3, E5.4.
- **E6 — Real Apple Metal acceleration through existing Track 32**: 4/4 checked; tasks E6.1, E6.2, E6.3, E6.4.
- **E7 — Within-run parallel DES through existing Track 34**: 4/4 checked; tasks E7.1, E7.2, E7.3, E7.4.
- **E8 — Real distributed execution through existing Track 35**: 4/4 checked; tasks E8.1, E8.2, E8.3, E8.4.

### Generic birthing suite simulation delivery

- **B0 — Evidence and jurisdiction/site profiles**: 4/4 checked; tasks B0.1, B0.2, B0.3, B0.4.
- **B1 — Progression independent of service**: 4/4 checked; tasks B1.1, B1.2, B1.3, B1.4.
- **B2 — Birth identities and maternal-infant relationships**: 4/4 checked; tasks B2.1, B2.2, B2.3, B2.4.
- **B3 — Spaces and effective capacity**: 4/4 checked; tasks B3.1, B3.2, B3.3, B3.4.
- **B4 — Staffing and resource bundles**: 4/4 checked; tasks B4.1, B4.2, B4.3, B4.4.
- **B5 — Escalation and bounded theatre completion**: 4/4 checked; tasks B5.1, B5.2, B5.3, B5.4.
- **B6 — Transfers and split destinations**: 4/4 checked; tasks B6.1, B6.2, B6.3, B6.4.
- **B7 — Two-consumer composition acceptance**: 4/4 checked; tasks B7.1, B7.2, B7.3, B7.4.

### CareOps whole-layer Polars migration

- **R10 — Upstream entry and baseline**: 4/4 checked; tasks R10.1, R10.2, R10.3, R10.4.
- **R11 — Capture once and durable trace parts**: 0/4 checked; tasks R11.1, R11.2, R11.3, R11.4.
  - R11.1: Add failing run-invocation-count, bounded-buffer/backpressure, disk-error and checkpoint-resume duplicate/loss tests.
  - R11.2: Capture required tables during original replications and write bounded per-replication parts using Kairos data facilities; remove export-time reruns.
  - R11.3: Bind trace part IDs/watermarks to restart state and publish manifest last; preserve exact event/RNG behaviour across worker counts and restarts.
  - R11.4: Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.
- **R12 — Rust Polars domain processing**: 0/4 checked; tasks R12.1, R12.2, R12.3, R12.4.
  - R12.1: Add analytical tests for weighted versus unweighted estimands, ddof/intervals, censoring, empty/singleton, null/nonfinite, skew and duplicate/missing join keys.
  - R12.2: Replace applicable ED and birthing batch transformations, joins, summaries, repeated statistics and calibration-trace preparation with Polars expressions; keep online/domain logic authoritative.
  - R12.3: Use existing modules for domain definitions and Kairos for generic interchange; extract shared helpers only for actual duplicate consumers, not speculative frameworks.
  - R12.4: Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.
- **R13 — Outputs and consumers**: 0/4 checked; tasks R13.1, R13.2, R13.3, R13.4.
  - R13.1: Add independent-reader tests for real Parquet/Zstd and IPC, explicit CSV quoting/limb fields, effective input hashes and incomplete manifest rejection.
  - R13.2: Replace JSON full-table bundles/schema DDL/manual CSV output with versioned Parquet tables plus provenance manifests; migrate CLIs and every known downstream fixture/script.
  - R13.3: Regenerate owned synthetic fixtures, preserve historical files, remove old alpha data APIs/readers and document clean-break run/output instructions.
  - R13.4: Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.
- **R14 — Acceptance and removal**: 0/4 checked; tasks R14.1, R14.2, R14.3, R14.4.
  - R14.1: Run native domain/replay/restart/numerical/interchange lanes and existing performance gates; measure small/large ensembles, output latency, peak RSS, thread contention and build size.
  - R14.2: Audit net maintained-code deletion and all retained candidates; remove obsolete compatibility and duplicate paths and reconcile docs without claiming clinical acceptance.
  - R14.3: Record accepted parent and Kairos commits, hosted/local evidence distinctions, rollback instructions and exact pin/schema/library receipts; close only after full consumer validation.
  - R14.4: Conductor — review and verify phase (workflow.md); apply accepted fixes, record rejected/cross-owner work, receipts, clean-tree check and explicit next-phase decision.

## Scope retained

Core completion covers bounded generic synthetic engineering and artifact/reference
oracles. Shared cross-domain scheduling, within-run durable birthing restart,
empirical fitting, clinical/site/operational, stable public API, hardware and
broader release acceptance require their separate evidence. Offline fiscal
exchange stays proposal-only. Existing protected readiness/addendum ownership
and all earlier assignments remain intact; no next code lane is dispatched here.
