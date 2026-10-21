# Specification: status reconciliation and qualified Kairos integration

## Purpose

Reconcile status against actual source/runtime evidence and qualify an immutable
Kairos candidate without losing the working ED baseline. Original 145 completed
checkboxes record plan status, not proof of every hardware/clinical capability.

## Requirements

- R-R1: reconcile contradictory index/registry/resume summaries and audit exact-scope receipts; preserve historical results and unresolved gaps.
- R-R2: validate existing Python harness environment and Conductor catalogs; repair numbered task format without weakening dependency or packet checks.
- R-R3: inventory actual consumer API coverage at pinned and candidate commits. Inspect conditions, bundle grants and official format interoperability. Wave labels or named enums do not establish runtime behaviour.
- R-R4: freeze ED, Q/C, physical IO and replay/checkpoint/holdout baselines, run affected compiler/native tests and qualify a candidate in isolation before pin promotion.
- R-R5: retain the working pin if required gates fail; record missing capabilities and bounded alternatives. Advanced hardware/solver qualification remains separate from the headless composition gate.
- R-R6: prove rollback and independent output readback; reconcile dependency/source lineage and record exact commands/hashes.

## Scope and acceptance

Planning and subsequent bounded qualification only. No forced upgrade to latest,
no completed clinical validation, no automatic licence/dependency admission.
No source pin changes occur in this planning revision. R3 closes on an evidenced
candidate decision and usable baseline, including an explicit retain-pin decision;
any missing capability stays an external dispatch gate for its consumer task.

See [composition contracts](../../design/composition-contracts-20261020.md).

## PR173 acceptance refinements

R0 reconciles conflicting planning copies and contract-only E6/E8 evidence.
R2 requires a capability-preserving integration across divergent Kairos history,
including Q/C, physical IO and checkpoint equivalence. Mac Metal, separate CUDA PC
and physical Mac/NAS execution each require their own native receipts; hardware
is not a headless dependency unless consumed. R3 rejects stale/mismatched receipts.
The [C-R0–C-R7 mapping](../../evidence/pr173-roadmap-integration-20261021.md) is part
of the acceptance specification; no existing checked hardware claim is expanded.
