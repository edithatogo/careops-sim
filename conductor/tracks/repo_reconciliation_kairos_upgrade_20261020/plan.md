# Implementation plan: Repository status reconciliation and Kairos upgrade

Status: planned revision; no implementation acceptance in this change.

Contract: [cross-service composition](../../design/composition-contracts-20261020.md).
Dispatch requires reviewed packets, upstream capability readback and the single-maintainer harness.

## R0 — Status and capability evidence reconciliation

- [ ] R0.1 Reconcile registry, index and resume state against exact-scope receipts; retain historical acceptance and expose contradictions rather than declare all capabilities verified.
- [ ] R0.2 Inventory the pinned Kairos consumer APIs and candidate mainline; distinguish contract, implemented code, exercised runtime and operational acceptance. Record parallel audit findings and exact revisions.
- [ ] R0.3 Review numbered plan tasks, ownership, external capability gates and task catalog; require context/catalog integrity before dispatch.
- [ ] R0.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## R1 — Harness and regression baseline

- [ ] R1.1 Audit the existing harness environment and jsonschema resolution without changing global Python or dependencies.
- [ ] R1.2 Run the Python harness regressions and context/task/MVP validators; record commands, hashes and failures.
- [ ] R1.3 Freeze ED, Q/C, physical IO and checkpoint/recovery source/input/output baselines, replay version and rollback procedure before a candidate pin change.
- [ ] R1.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## R2 — Bounded Kairos candidate qualification

- [ ] R2.1 Select an immutable candidate after reviewing code, compatibility and current hosted evidence; f0d30e2 is an inspected candidate, not an automatic upgrade target.
- [ ] R2.2 Exercise required consumer APIs: event conditions, allocation semantics and official Arrow IPC/Parquet interoperability with an independent reader; an enum or custom serialization is insufficient.
- [ ] R2.3 Run relevant Rust consumer checks against the candidate in isolation. Keep Metal, distributed transport and advanced solvers separate unless required and exercised by this scope.
- [ ] R2.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## R3 — Pin decision and deterministic conformance

- [ ] R3.1 Run full affected Rust tests and preserved Q/C, IO, ED replay/holdout comparisons against the frozen baseline; explain intentional versioned differences and reject unexplained drift.
- [ ] R3.2 Advance the parent pin only after required gates pass; otherwise retain the working pin and record precise missing capabilities and supported bounded alternatives.
- [ ] R3.3 Record candidate decision, rollback rehearsal, limitations and owner handoffs; do not mark an unavailable capability passed.
- [ ] R3.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.
