# Implementation plan: Repository status reconciliation and Kairos upgrade

## Repository scope

This simulation plan applies to **careops-sim**, whose source checkout is
`/Users/doughnut/Documents/careops-sim` (GitHub `edithatogo/careops-sim`).
Kairos is its separately owned upstream simulation engine. The distinct
`/Volumes/PortableSSD/GitHub/careops` portfolio repository is not the target of
these C-R0–C-R7 recommendations. No change to that repository is made by this
21 October incorporation. Older shorthand “CareOps” here denotes careops-sim
unless a paragraph explicitly identifies the separate portfolio repository.

Status: R0 reconciliation and R1 local CPU/IO/recovery baseline independently accepted on the development branch; R2–R3 remain pending.
See [independent R1 review](../../evidence/r1-independent-review-20261021/README.md).
See [independent R0 review](../../evidence/r0.4-independent-review-20261021/README.md).
See [R0.1 evidence](../../evidence/r0.1-reconciliation-20261021/README.md).

Contract: [cross-service composition](../../design/composition-contracts-20261020.md).
Dispatch requires reviewed packets, upstream capability readback and the single-maintainer harness.

## R0 — Status and capability evidence reconciliation

- [x] R0.1 Reconcile registry, index and resume state against exact-scope receipts and divergent planning copies; preserve uncommitted proposals, record the reviewed integration revision and remove conflicting dispatch assumptions before binding. Retain historical acceptance without promoting it to broader capability proof.
- [x] R0.2 Inventory pinned and candidate Kairos symbols, features, Cargo dependencies, schemas, fixtures and Q/C/ED replay/checkpoint receipts; verify ancestry and missing equivalents, not just version labels. Classify contract, implemented code, exercised runtime and operational acceptance separately.
- [x] R0.3 Map PR173 C-R0–C-R7 into these tasks; audit E6/E8 receipts, retaining e62 constructed metadata/WGSL/buffer-state tests and e83 threads/mpsc with synthetic efficiency as contract evidence only. Reconcile ownership and catalog integrity; historical checked tasks do not certify real hardware.
- [x] R0.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## R1 — Harness and regression baseline

- [x] R1.1 Audit the existing harness environment and jsonschema resolution without changing global Python or dependencies.
- [x] R1.2 Run the Python harness regressions and context/task/MVP validators; record commands, hashes and failures.
- [x] R1.3 Freeze ED, Q/C, physical IO and checkpoint/recovery source/input/output baselines, replay version and rollback procedure before a candidate pin change.
- [x] R1.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## R2 — Bounded Kairos candidate qualification

- [x] R2.1 Select an immutable capability-preserving candidate with Kairos owners 03/04/21/22/25; integrate missing qualified functionality or prove a replacement. f0d30e2 is a divergent inspected revision, not an automatic upgrade target. Recheck exact source, feature set and compatibility at binding.
- [ ] R2.2 Exercise required consumer APIs: scheduler event waits, allocation/cancellation/rollback and independent official Arrow IPC/Parquet reader interoperability. Requalify native Python execution if consumed; an enum, version-only ctypes probe, custom payload or unsupported claim_atomic_multi_resource is insufficient.
- [ ] R2.3 Run required Rust consumer checks in isolation. Map optional E6/E8 acceptance to real Mac Metal kernel submission/readback and CPU parity; a separately identified CUDA PC with verified access/GPU/toolchain; and physically distinct Mac/NAS transport peers with failure/restart and measured timings. Reverify host identity: the historical pleiades alias targeted the NAS. Separate initialization/transfer/compute cost, equal-workload throughput/latency, memory limits and cancellation/recovery; mock metadata, CPU fallback and threads never satisfy device or multi-host gates. Hardware qualification stays outside the headless pin gate unless actually consumed.
- [ ] R2.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.

## R3 — Pin decision and deterministic conformance

- [ ] R3.1 Run full affected Rust tests and preserved Q/C, IO, ED replay/holdout comparisons against the frozen baseline; explain intentional versioned differences and reject unexplained drift.
- [ ] R3.2 Advance the parent pin only after required gates pass; otherwise retain the working pin and record precise missing capabilities and supported bounded alternatives.
- [ ] R3.3 Record candidate decision, rollback rehearsal and owner handoffs with command argv/cwd, immutable source/submodule/feature hashes, compiler/toolchain, seed/input hashes, host/backend/device identity, exit status and retained raw artifact checksums. Reject missing, stale or mismatched receipts; document unavailable capability without passing it.
- [ ] R3.4 Conductor — review and verify phase (workflow.md).

Exit: retain executed command/input/output hashes, negative controls and scope-specific evidence; unresolved or unavailable behaviour remains unverified.
