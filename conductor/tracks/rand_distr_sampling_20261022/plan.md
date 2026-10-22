# Plan: rand_distr statistical sampling adoption

All implementation tasks are pending. Write failing behavioural tests before production replacement. Accept Kairos Track 71 and CareOps rand R22 before upstream sampling work; accept Kairos Track 72 Phase 2 before CareOps R23 pin adoption.

## R23 — Entry and qualification

- [ ] R23.1 Verify the entry receipts and ownership handoffs; reject missing/stale upstream or downstream evidence before any pin/dependency/runtime change.
- [ ] R23.2 Inventory actual replace/retain candidates and freeze baseline contracts, fixtures, costs and supported targets; qualify exact pins/features, licences/advisories and compatibility.
- [ ] R23.3 Inventory production samplers; specify replace/retain decisions, units/support and deliberate corrections; add independent statistical and hazard tests before replacement.
- [ ] R23.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record blockers/receipts and explicit next-phase decision.

## R24 — Replacement and consumers

- [ ] R24.1 Adopt the accepted Kairos Track 72 commit and sampling APIs; replace applicable parent production samplers with these facilities or direct library distributions only where parent-owned policy requires them. Do not duplicate generic upstream samplers.
- [ ] R24.2 Migrate checkpoint/provenance and consumers; establish new sampling baselines and document target-specific reproducibility limits.
- [ ] R24.3 Conductor — review and verify phase (workflow.md); apply accepted fixes, record blockers/receipts and explicit next-phase decision.

## R25 — Acceptance and handoff

- [ ] R25.1 Run every test-matrix oracle and applicable full native/browser lanes; resolve regressions and record raw evidence.
- [ ] R25.2 Audit deleted/retained production code, update compatibility/user documentation and record rollback plus exact accepted commit/pin receipts.
- [ ] R25.3 Conductor — review and verify phase (workflow.md); apply accepted fixes, record blockers/receipts and explicit next-phase decision.

## Dispatch boundary

The task catalogue expresses local ordering only. External gates are manual receipt checks, not automatic success. R-prefix reservations are legacy preparation hints and do not cover these domain/dashboard write sets accurately. Before any runtime dispatch, the coordinator must qualify explicit packet reservations against this track ownership contract and update the task reservation mapping if using automated selection. Do not use the generated broad R scope as write authority. Never dispatch writes into libs/kairos under a parent claim. All phases require Conductor review and recorded command/cwd/commit/toolchain/input hashes, exit codes, artifacts and local-versus-hosted evidence.
