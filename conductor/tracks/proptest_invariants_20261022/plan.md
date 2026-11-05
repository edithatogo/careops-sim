# Plan: CareOps proptest

R59.1 prerequisite and inventory accepted;exact current dependency/value qualification is next. No adoption or R60/R61 acceptance.

## R59 — Evidence and scope

- [x] R59.1 Verify upstream receipt and local prerequisites; inventory direct consumers and retained/deleted paths.
- [ ] R59.2 Qualify exact version/features/licence/MSRV/security, independent baseline and cost budgets; record value decision.
- [ ] R59.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision.

## R60 — Bounded replacement

- [ ] R60.1 Add failing independent contract and recovery tests before replacement.
- [ ] R60.2 Replace only named machinery; delete duplicates, preserve semantic validation and version intentional changes.
- [ ] R60.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision.

## R61 — Acceptance

- [ ] R61.1 Run independent oracle, mutation/recovery controls and measure net maintenance/build/runtime cost.
- [ ] R61.2 Record accepted immutable parent/upstream commits, exact features, receipts and remaining Polars obligations.
- [ ] R61.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision.

## Dispatch boundary

Legacy R-prefix reservations are not write authority. Resolve narrow parent file claims; no submodule edits. The historical Kairos96 prerequisite is accepted within the R59.1 receipt scope. Current dependency/value qualification and all R60/R61 gates remain pending.

[R59.1 inventory](../../evidence/proptest-r591-inventory-20261105/inventory.json) verifies published418b6bc/tested2e8782b source parity for three producer contract files and identifies independent parent core property consumers. Runtime074 and all conformance/fuzz tests remain unchanged.
