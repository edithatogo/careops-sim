# Plan: CareOps sha2

R50.1 historical prerequisite verification and consumer inventory are accepted at snapshotecbf0ff. R50.2 exact dependency/value/build-identity qualification is accepted at sourcef1ef095. R50.3 phase review is accepted at d1bccd5; R51 bounded replacement is accepted at sourcee44946b with its closed successor guard. R52 broader acceptance remains pending.

## R50 — Evidence and scope

- [x] R50.1 Verify upstream receipt and local prerequisites; inventory direct consumers and retained/deleted paths. [Source-bound inventory](../../evidence/sha2-r501-inventory-20261104/README.md) verifies accepted Track93source5692/ancestry074 and current parent targets; no current-version/runtime adoption acceptance.
- [x] R50.2 Qualify exact version/features/licence/MSRV/security, independent baseline and cost budgets; record value decision. [Qualification](../../evidence/sha2-r502-qualification-20261104/README.md) accepts0.11.1/default alloc+oid as a bounded replacement candidate, with unchanged model-build/recovery guards; no runtime adoption or R50.3 acceptance.
- [x] R50.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision. [Review](../../evidence/sha2-r503-review-20261104/README.md) accepts R50 only and authorizes bounded R51 tests-first preparation.

## R51 — Bounded replacement

- [x] R51.1 Add failing independent contract and recovery tests before replacement. Tests-first commitc6bccc5; [baseline/controlled-red proof](../../evidence/sha2-r51-implementation-20261104/tests-first.json), real original artifact frozen before replacement.
- [x] R51.2 Replace only named machinery; delete duplicates, preserve semantic validation and version intentional changes. Sourcee44946b replaces only ED crypto machinery with existing exact0.11.1; lock unchanged, real cross-build read/resume-refusal proof and26-case compatibility pass. [Source receipt](../../evidence/sha2-r51-implementation-20261104/qualified-source.json).
- [x] R51.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision. [Phase evidence](../../evidence/sha2-r51-implementation-20261104/phase-review.json) binds the closed successor guard and review fixes; accept bounded local engineering only, R52 remains unaccepted.

## R52 — Acceptance

- [ ] R52.1 Run independent oracle, mutation/recovery controls and measure net maintenance/build/runtime cost.
- [ ] R52.2 Record accepted immutable parent/upstream commits, exact features, receipts and remaining Polars obligations.
- [ ] R52.3 Conductor — review and verify phase (workflow.md); apply accepted fixes and record next-phase decision.

## Dispatch boundary

Legacy R-prefix reservations are not write authority. Resolve narrow parent file claims; no submodule edits. External receipts are manual and unresolved.
