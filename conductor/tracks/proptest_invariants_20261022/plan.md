# Plan: CareOps proptest

R59 and bounded R60 implementation accepted. R61 full same-head engineering acceptance remains open.

## R59 — Evidence and scope

- [x] R59.1 Verify upstream receipt and local prerequisites; inventory direct consumers and retained/deleted paths.
- [x] R59.2 Qualify exact version/features/licence/MSRV/security, independent baseline and cost budgets; record value decision.
- [x] R59.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision.

## R60 — Bounded replacement

- [x] R60.1 Add failing independent contract and recovery tests before replacement.
- [x] R60.2 Replace only named machinery; delete duplicates, preserve semantic validation and version intentional changes.
- [x] R60.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision.

## R61 — Acceptance

- [ ] R61.1 Run independent oracle, mutation/recovery controls and measure net maintenance/build/runtime cost.
- [ ] R61.2 Record accepted immutable parent/upstream commits, exact features, receipts and remaining Polars obligations.
- [ ] R61.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision.

## Dispatch boundary

Legacy R-prefix reservations are not write authority. Resolve narrow parent file claims; no submodule edits. The historical Kairos96 prerequisite is accepted within the R59.1 receipt scope. R59 qualification and bounded R60 implementation are accepted. All R61 hosted/platform/integration/coverage/mutation/security/package/cost gates remain pending.

[R59.1 inventory](../../evidence/proptest-r591-inventory-20261105/inventory.json) verifies published418b6bc/tested2e8782b source parity for three producer contract files and identifies independent parent core property consumers. Runtime074 and all conformance/fuzz tests remain unchanged.

Historical R59 checkpoint: [qualification](../../evidence/proptest-r592-qualification-20261105/qualification.json) records exact1.12.0/std, independent contracts, observed costs, policy/graph/floor evidence and reviewer fixes. R60/R61 remained open at that checkpoint; current R60 disposition follows below.

R60 implementation 2911b93 and strict successor eeb4ac3 retain all prior receipts and current owners. [Local review](../../evidence/proptest-r60-implementation-20261105/README.md) records seven new properties, seven new guard tests, 52 reused historical successes plus two corrected nodes, all 26 frozen cases and actual cross-build recovery. Full same-head CI, including all 61 guard tests and Linux `/dev/full`, is mandatory in R61.
