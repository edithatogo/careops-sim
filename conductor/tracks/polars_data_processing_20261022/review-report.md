# Review Report: Polars migration planning

## Summary
The planning package reflects the user-approved clean break and library-replacement objective; runtime implementation remains pending.

## Verification Checks
- [x] Plan Compliance: both repositories, upstream-first gate, Rust-side processing, Parquet default, bounded floating-point comparisons.
- [x] Style Compliance: experimental maturity and explicit evidence boundaries.
- [x] New Tests: runtime test requirements specified; no runtime tests claimed implemented.
- [x] Test Coverage: planning registry/DAG/context checks only; runtime coverage not assessed.
- [x] Test Results: planning validators passed as recorded below.

## Refinements applied
Explicit tick-borrow arithmetic, duplicate-key conservation, FFI buffer ownership, effective-input provenance, trace-once capture and restart watermarks, memory/index/thread limits, manifest publication durability and code-deletion evidence. No DataFusion or duplicate backend introduced. External upstream gate remains manual because the parent task DAG cannot validate another repository's acceptance.

## Validation receipt
Date: 2026-10-22 (Australia/Brisbane). Working directory: this isolated repository worktree. Source base: 6bcbc0b947ab82b2d9e4b4e54ea51e662df8a53d. Deterministic seed/input hash: not applicable to Markdown/registry checks; runtime checks skipped because this increment changes planning only.
- `python3 tools/tasks.py build`
- `python3 tools/tasks.py check`
- `python3 tools/context.py check`
- `git diff --check`
Exit status: 0 for each listed planning command. Artifacts: this report and the generated parent task catalogue where applicable. Source commit is the containing planning commit; no hosted CI, merge, release or runtime acceptance claim. Pushed ref: none.

## Review disposition
No unresolved planning blocker. Exact dependency/feature qualification is a first-phase deliverable and a stop gate, not assumed successful. All implementation checkboxes remain unchecked.

Initial context validation failed because this new worktree had uninitialized submodules. Initialized exact recorded pins and reran successfully; no pin changed.

## Harness exception and corrective audit

The original precommit harness check returned exit 1: input hash drift for conductor/index.md. That file had been intentionally edited within the claimed conductor scope, but was mistakenly also registered as an immutable input. Commit commands nevertheless ran; the original precommit gate therefore did not pass. A postcommit diff-tree audit verified every changed path is inside conductor/. Released the original lease and acquired a fresh clean-tree claim with only unchanged workflow as immutable input, then recorded this exception and reran the scope check before the corrective commit. This correction does not retroactively turn the original failed gate into a pass.

## R10 phase review and acceptance — 30 October 2026

### Summary

R10 is accepted for the bounded remote ED/core/CLI baseline at `e65570b7a1046dafb15511ab5118b46b587f2f5d`, with Kairos `07410266bf54b6f73c59db2f8cbfbc5697c53734`. Runtime Polars replacement is not accepted. The earlier planning review above remains historical.

### Verification checks

- [x] Plan compliance: R3 entry prerequisite accepted; R10.1 Track 70 receipt verified, R10.2 inventory and baseline retained, R10.3 reference tests and compatible pin integrated.
- [x] Style compliance: existing architecture, RNG and scientific semantics preserved; no additional code-style directory exists in this checkout.
- [x] New tests: R10.3 statistical and row-reference tests retained; this phase closeout changes documentation/metadata only.
- [x] Test coverage: exact prior full local and hosted evidence reused after verifying unchanged source, lockfiles, configuration, schemas, fixtures and pin. Runtime migration oracles remain R11–R14.
- [x] Test results: accepted source checks pass; current phase/context/DAG/pin/support checks recorded below. No fresh performance, clinical or hardware claim.

### Evidence and review disposition

[Code PR #188](https://github.com/edithatogo/careops-sim/pull/188) merged as `0db23f1d5746314b33db315d1020ff47b3238f1f`, tree-identical to tested `ef1f0b0e38cdd5aae13eac562e543888db9b0ab2`. [Acceptance PR #189](https://github.com/edithatogo/careops-sim/pull/189) merged as the reviewed base `e65570b`, tree-identical to `1bca5fcd4a6bf5e9b6226beb6f47433e6034b3f5`. Provider readback on 30 October Brisbane reconfirmed #189 merged with all 14 reported checks successful, including required aggregate, both native hosts, privacy/security and session guards. [Its hosted run](https://github.com/edithatogo/careops-sim/actions/runs/40887777304) reran the full gates successfully.

The [R10.3 evidence packet](../../evidence/r103-remote-20261029/README.md), [local command hashes](../../evidence/r103-remote-20261029/local-verification.json), [26-case seed-7 parity](../../evidence/r103-remote-20261029/parity.json), [prerequisite inventory](../../evidence/r103-remote-20261029/prerequisites.md) and active `conductor/evidence/d2.4-kairos-contract.json` bind the exact inputs/source. Prior results: 358 parent Rust tests, 728 Python tests, full Clippy, 116 mutation outcomes (111 caught, two reviewed equivalent misses, three unviable, zero timeout), ED line coverage 500/567 (88.18%), and 588 Kairos tests on each native owner host. These are reused results, not newly executed tests in R10.4.

Independent reviewer `/root/acceptance_review` reviewed clean base `e65570b`, using the prior source reviews and exact receipts. No new runtime blocker. Accepted findings: reconcile stale proposed/unstarted metadata, unresolved external gate, planning-only handoff/index and old 44261eb retention instructions; record the bounded acceptance and explicit next-phase decision. Those corrections are applied here. Preserve the other lane's global A1/A2 dispatch; scoped Polars resume metadata is updated separately.

Rejected/cross-owner work: broad modern Kairos adoption and RNG changes were rejected in favor of the reviewed bridge; no golden updates or relaxed tolerances. The bridge is not for merging into modern Kairos main. Advanced A2–A6/identity/birthing source remains separately owned and unintegrated by this packet. Historical fixture retention proves exact bytes only, not current physical-schema behavior. The unavailable macOS disk-full probe, hardware, release and clinical acceptance remain unverified. Real trace capture, persisted columnar interoperability and restart durability remain required future work.

### Current verification receipt

Owner: chat `01a1876f-9b26-71e0-b55c-daeb9d9a1335`; managed worktree `careops-r103-remote/careops-sim`; exact review base `e65570b`; no native service writes. Parent commands run from the worktree root with the verified repo-local CPython 3.15.1. Upstream commands run in its initialized `libs/kairos` checkout at exact pin 0741026. Rust 1.99.1/seed 7/runtime hashes are inherited only from the linked immutable R10.3 receipts. This metadata-only closeout has no new simulation seed or input.

- `.venv/bin/python tools/python_runtime.py`: verified actual 3.15.1 interpreter.
- `.venv/bin/python tools/context.py check`, `tools/tasks.py build`, `tools/tasks.py check`, `tools/mvp.py check`, `tools/check_kairos_pin.py`: exit 0 after closeout edits.
- `.venv/bin/python tools/quality_support.py --root .`: exit 0; semver comparison remains explicitly unavailable without an owner-approved stable API baseline. An initial invocation lacking `--root` exited 2 and is not counted as a pass.
- `pwsh -NoProfile -File scripts/validate_conductor_phase_gates.ps1`: exit 0, no errors/warnings.
- `pwsh -NoProfile -File scripts/validate_conductor_dag.ps1`: exit 0, 106 tracks/47 agents, no errors/warnings.
- `pwsh -NoProfile -File scripts/validate_conductor_git_closeout.ps1 -LedgerPath ../../conductor/evidence/r103-remote-20261029/track70-closeout.yaml -RequireCleanWorkingTree`: exit 0 against the exact canonical Track 70 receipt retained from f9a19d0. Initial default-ledger runs failed for missing historical objects and then the deleted `origin/feat/tracks-70-93-94-foundation` ref in Track 70/93/94 entries. Exact objects were fetched; no source/pin/ledger or fabricated ref was changed. The default whole-ledger check remains failed for that historical cross-owner ref, not reported as passing. The corrected accepted prerequisite receipt uses 5692e6a on real origin/main and passes the unchanged validator.
- Parent harness scope check and `git diff --check`: required before the closeout commit; clean committed source required after it. The containing commit is the phase-record commit; its parent is the exact reviewed source above. Remote integration is reported separately from local acceptance.

### Next-phase decision

Permit **R11.1 preparation only** after R10.4 acceptance. Bind a fresh exact-base packet, context hashes, owned paths and claim before source edits. Name and execute failing oracles for one invocation per replication, bounded capture/backpressure, disk failure and checkpoint-resume duplicate/loss behavior before implementing capture. Retained dependency work gates only its demonstrated consumer/oracle; no blanket sha2/tempfile/proptest adoption gate. No R11 task is checked complete or dispatched by this review. Keep this track active through R14; do not archive it.
