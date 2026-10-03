# Conservative CI scope qualification — 3 October 2026

[PR #18](https://github.com/edithatogo/careops-sim/pull/18) qualified the classifier and aggregate at checked head `3c9fedacb29a6459ebfcc85458a242dbef8f27af`; every required hosted lane succeeded in [CI run 37113260676](https://github.com/edithatogo/careops-sim/actions/runs/37113260676). The implementation merged as `059ec79724357897b66ab1c402bfc0be154c682f` after independent source review. Locally, 39 focused tests passed, including real Git changes to tracked inputs and both gitlinks, both rename directions, mixed changes, type changes, invalid refs and an exercised missed-native oracle. The NUL-framing regression failed before its parser fix.

The first optimization has two execution classes. Reviewed regular, non-executable Markdown/JSON under Conductor evidence, design, tracks or execution keeps context and policy checks while skipping native Rust, fuzz and Miri. Unknown paths, malformed diffs, invalid refs, executable/symlink planning files, code, tools, tests, workflows, dependencies and gitlinks select every lane. Empty diffs select none. The aggregate requires selected jobs to succeed and unselected jobs to be explicitly skipped; missing/invalid selectors or results fail.

## Planning-only hosted oracle

This document is the sole source change in branch `codex/ci-planning-skip-proof`, based on the merged classifier. Its hosted qualification requires:

- scope, context, policy and Required checks: success;
- fmt, clippy, both native test hosts, fuzz, Miri and doctest: skipped;
- session-guard checks: success where triggered.

These are required observations, not a claim of an already executed planning-only run. The coordinator retains the actual run/head/conclusions after readback. Native Rust source, Cargo inputs, submodule pins, workflow and classifier remain unchanged by this document. This scheduling qualification does not complete D3.3, release acceptance or any unfinished runtime phase.
