# Plan: petgraph spatial topology and routing adoption

All implementation tasks are pending. Write failing behavioural tests before production replacement. Accept CareOps ECharts R28 and Kairos sampling Track 72 before upstream routing work; accept Kairos Track 73 Phase 2 before CareOps R29 pin adoption.

## R29 — Entry and qualification

- [ ] R29.1 Verify the entry receipts and ownership handoffs; reject missing/stale upstream or downstream evidence before any pin/dependency/runtime change.
- [ ] R29.2 Inventory actual replace/retain candidates and freeze baseline contracts, fixtures, costs and supported targets; qualify exact pins/features, licences/advisories and compatibility.
- [ ] R29.3 Freeze generic topology/identity/cost/tie/no-route API and CareOps mapping contract; add independent small-graph, access and multi-floor tests.
- [ ] R29.4 Conductor — review and verify phase (workflow.md); apply accepted fixes, record blockers/receipts and explicit next-phase decision.

## R30 — Replacement and consumers

- [ ] R30.1 Adopt the accepted Kairos Track 73 commit and generic routing API; implement CareOps profile-to-topology/access mappings and synthetic multi-floor fixtures. Do not build a second petgraph routing engine in CareOps.
- [ ] R30.2 Migrate actual graph consumers and synthetic CareOps adapters after accepted upstream handoff; remove superseded adjacency/algorithm code only where replaced.
- [ ] R30.3 Conductor — review and verify phase (workflow.md); apply accepted fixes, record blockers/receipts and explicit next-phase decision.

## R31 — Acceptance and handoff

- [ ] R31.1 Run every test-matrix oracle and applicable full native/browser lanes; resolve regressions and record raw evidence.
- [ ] R31.2 Audit deleted/retained production code, update compatibility/user documentation and record rollback plus exact accepted commit/pin receipts.
- [ ] R31.3 Conductor — review and verify phase (workflow.md); apply accepted fixes, record blockers/receipts and explicit next-phase decision.

## Dispatch boundary

The task catalogue expresses local ordering only. External gates are manual receipt checks, not automatic success. R-prefix reservations are legacy preparation hints and do not cover these domain/dashboard write sets accurately. Before any runtime dispatch, the coordinator must qualify explicit packet reservations against this track ownership contract and update the task reservation mapping if using automated selection. Do not use the generated broad R scope as write authority. Never dispatch writes into libs/kairos under a parent claim. All phases require Conductor review and recorded command/cwd/commit/toolchain/input hashes, exit codes, artifacts and local-versus-hosted evidence.
