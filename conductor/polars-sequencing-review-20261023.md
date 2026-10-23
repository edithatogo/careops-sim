# Review Report: inventory-first Polars sequencing

## Summary

Planning-only revision follows the user decision to start Polars inventory and gate only demonstrated dependent tasks. No implementation phase is advanced.

## Verification Checks

- [x] Plan compliance: removed blanket Kairos 93/94/96 and CareOps R11 prerequisite edges; preserved ownership, existing contracts, reconciliation R3 and upstream Phase 4 receipts.
- [x] Style compliance: separate dependency tracks and explicit evidence boundaries retained.
- [x] New tests: none; no runtime change. Existing planning validators used.
- [x] Test coverage: registry/DAG and planning consistency only; no runtime or hosted acceptance claimed.
- [x] Test results: `python3 tools/tasks.py build`, `python3 tools/tasks.py check` and `python3 tools/context.py check`: exit 0; 387 tasks, 24 tracks, 105 milestones, no context errors. Initial context check failed because the new worktree submodules were uninitialised; after checking out the recorded pins it passed. No pin change. `git diff --check`: exit 0.

## Scope and evidence

Working directory: `/Volumes/PortableSSD/codex-worktrees/careops-polars-sequencing`. Base commit: `dc3aecb0c2a3fa32d2e475d9d319374945b97a47`. Review performed in the coordinator session using the Conductor review procedure; not a separate-agent review. Tool versions are recorded below. No simulation seed applies to this documentation revision. Source changes and validation are reproducible from this commit; this file is the planning receipt. Full Rust CI and runtime tests skipped because only Conductor documents/metadata changed.

## Review decisions

Each inventory target must have removal/retention rationale, owner and exact dependent task/receipt/oracle before affected dispatch. No blanket adoption is implied, and no removed target is marked implemented. Existing tests and Polars acceptance requirements remain mandatory. Historical sequencing review is explicitly superseded. Separate claims protect unrelated FMI and other runtime work. Machine readiness remains preparation eligibility, not runtime dispatch authority.

Tool versions: Python 3.15.0; PowerShell 7.6.8.

## Main-branch integration review — 23 October 2026

Publication worktree: careops-publish-sequencing; original review and its historical tool versions above are preserved. Fresh validation uses the explicitly selected repo-local .venv/bin/python, verified as Python 3.15.1. The diff remains confined to Conductor planning and preserves runtime sources, submodule pins, implementation checkboxes and acceptance status.

Reconciled against main dc3aecb0c2a3fa32d2e475d9d319374945b97a47. Exact existing submodule pins initialized without updates. Fresh tools/tasks.py check and tools/context.py check pass: 387 tasks, 24 tracks, 105 milestones. Focused unittest discovery passes test_tasks.py (34 tests) and test_context.py (11 tests). git diff --check passes. Runtime suites are not rerun for the documentation-only slice; required hosted PR checks remain mandatory.
