# Plan: CareOps thiserror

R56–R58 bounded engineering complete at source42e4b0d; all configured gates and final review pass. PR202 remains draft/frozen; separate documentation closeout.

## R56 — Evidence and scope

- [x] R56.1 Verify upstream receipt and local prerequisites; inventory direct consumers and retained/deleted paths.
- [x] R56.2 Qualify exact version/features/licence/MSRV/security, independent baseline and cost budgets; record value decision.
- [x] R56.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision.

[R56 qualification and review](../../evidence/thiserror-r562-qualification-20261105/qualification.json) binds exact2.0.21/default std, published historical prerequisite limits,125-case independent parity/negative controls, current security/graph/identity and frozen costs.

## R57 — Bounded replacement

- [x] R57.1 Add failing independent contract and recovery tests before replacement.
- [x] R57.2 Replace only named machinery; delete duplicates, preserve semantic validation and version intentional changes.
- [x] R57.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision.

## R58 — Acceptance

- [x] R58.1 Run independent oracle, mutation/recovery controls and measure net maintenance/build/runtime cost.
- [x] R58.2 Record accepted immutable parent/upstream commits, exact features, receipts and remaining Polars obligations.
- [x] R58.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision.

## Dispatch boundary

Legacy R-prefix reservations are not write authority. Resolve narrow parent file claims; no submodule edits. Published Kairos95 historical prerequisite is accepted with recorded source-chain/MSRV/features limits; it grants no runtime pin promotion. All R58 current-source gates subsequently passed; see the final acceptance receipt below.

[R57 phase review](../../evidence/thiserror-r57-implementation-20261105/phase-review.json) binds tests-first77c/939, implementation7e8 and reviewed guardb46;125actual cases,54guard regressions,10recovery tests,2explicit old-artifact consumers,1new producer and26frozen compatibility cases pass. At R57 acceptance, Linux `/dev/full` and the full R58 gates remained; the final receipt below now closes them.

[R58 final acceptance](../../evidence/thiserror-r58-acceptance-20261105/acceptance.json) binds qualified source42e4/testedmerge23db, all six workflows and Required checks,803native tests per host,793default,840+127Python,26frozen cases,88.5813% coverage and all117mutants. No merge/release/pin promotion.
