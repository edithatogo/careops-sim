# Plan: rand deterministic RNG adoption

All implementation tasks are pending. Write failing behavioural tests before production replacement. Both Kairos Track 70 Phase 4 and CareOps Polars R14 must be accepted before upstream RNG implementation; then accept Kairos Track 71 Phase 2 before CareOps R20 pin adoption.

## R20 — Entry and qualification

- [ ] R20.1 Verify the entry receipts and ownership handoffs; reject missing/stale upstream or downstream evidence before any pin/dependency/runtime change.
- [ ] R20.2 Inventory actual replace/retain candidates and freeze baseline contracts, fixtures, costs and supported targets; qualify exact pins/features, licences/advisories and compatibility.
- [ ] R20.3 Freeze canonical stream derivation, versioned checkpoint/manifest API and reviewed MSRV change; add failing known-answer, isolation and malformed-checkpoint tests.
- [ ] R20.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record blockers/receipts and explicit next-phase decision.

## R21 — Replacement and consumers

- [ ] R21.1 Adopt the accepted Kairos Track 71 commit and RNG API in all CareOps consumers; migrate parent provenance/checkpoints and remove any parent duplicate generator. Do not implement a second ChaCha wrapper or alter upstream code in this parent track.
- [ ] R21.2 Update consumers and independent baseline receipts; remove legacy generator code and document intentional trajectory break.
- [ ] R21.3 Conductor — review and verify phase (workflow.md); apply accepted fixes, record blockers/receipts and explicit next-phase decision.

## R22 — Acceptance and handoff

- [ ] R22.1 Run every test-matrix oracle and applicable full native/browser lanes; resolve regressions and record raw evidence.
- [ ] R22.2 Audit deleted/retained production code, update compatibility/user documentation and record rollback plus exact accepted commit/pin receipts.
- [ ] R22.3 Conductor — review and verify phase (workflow.md); apply accepted fixes, record blockers/receipts and explicit next-phase decision.

## Dispatch boundary

The task catalogue expresses local ordering only. External gates are manual receipt checks, not automatic success. R-prefix reservations are legacy preparation hints and do not cover these domain/dashboard write sets accurately. Before any runtime dispatch, the coordinator must qualify explicit packet reservations against this track ownership contract and update the task reservation mapping if using automated selection. Do not use the generated broad R scope as write authority. Never dispatch writes into libs/kairos under a parent claim. All phases require Conductor review and recorded command/cwd/commit/toolchain/input hashes, exit codes, artifacts and local-versus-hosted evidence.

## Dependency sequencing refinement — 22 October 2026

R20 additionally requires parent thiserror R58 and the existing upstream rand receipt. Optional tracing, Loom and state-machine testing never block this sequence.
