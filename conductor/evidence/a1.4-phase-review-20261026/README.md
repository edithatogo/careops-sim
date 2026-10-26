# A1.4 Phase Review — Pathways, Progression, and Policy Clocks

**Date:** 2026-10-26  
**Track:** `careops_core_domain_abstractions_20261020`  
**Scope:** A1.1–A1.3 and phase integration  
**Reviewed head:** `312a23811c7b1541022577d6f32eb2b283c4e8df`  
**Reviewer:** independent reviewer agent  
**Disposition:** Accepted for phase A1. Later phases A2–A7 remain open.

## Review and remediation

The reviewer examined the A1 source and tests against the plan and composition
contracts. The first review identified two A1.2 blockers:

1. The configurable pathway adapter hard-coded ED stage names and transitions.
   `c609c25` changed execution to traverse the configured `StageGraph`, added
   explicit branch choices, and tested a custom non-ED pathway.
2. Runtime progression used `hazard / 100`, did not select a competing cause,
   and logged a generic cause. `c609c25` now samples the unit-interval event
   probability `1 - exp(-total hazard)`, chooses a cause proportionally, and
   records that cause. Invalid/non-finite effective rates fail closed.

A follow-up review found that progression also ran at the inclusive censoring
horizon. `312a238` limits progression opportunities to ticks strictly before the
horizon and adds boundary tests for a one-tick horizon and an arrival at the
horizon. The reviewer confirmed this fix.

The A1.3 review separately found missing scheme/effective-date/provenance
validation. `884dd19` validates those fields and adds negative tests; the A1.3
reviewer accepted that correction. See [A1.3 review evidence](../a1.3-review-20261025/README.md).

No unresolved A1 findings remain in the reviewed source scope. The review does
not establish clinical validity, operational acceptance, or completion of the
core-abstractions track.

## Verification

Commands were run in the isolated integration worktree with the pinned Kairos
submodule at `44261eb5cf8275f132f63a3396903d116d701dbe` and Conductor extension
at `7a5c560a4fdf5297be58594cc37527eb12790272`.

| Check | Result |
| --- | --- |
| `.venv/bin/python --version` | PASS — Python 3.15.1 |
| `rustc --version` | PASS — Rust 1.99.1 |
| `cargo test --locked -p careops-core` | PASS at reviewed head |
| `cargo test --locked -p careops-ed` | PASS at reviewed head |
| Focused configurable-pathway and failing-case selectors | PASS |
| Targeted `rustfmt --check` on changed Rust files | PASS |
| `git diff --check` | PASS |
| `tools/tasks.py check` | PASS — 401 task records |
| `tools/mvp.py check` | PASS — 82 parents, 258 bounded leaves |
| `tools/context.py check` | PASS — 25 tracks, 109 milestones, no link/context errors |

Historical A1.3 package-test log links were removed from its prior receipt because
the ignored local `.artifacts/` log was not present in this checkout. The package
suites above were rerun on the reviewed source head.
