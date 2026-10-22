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
