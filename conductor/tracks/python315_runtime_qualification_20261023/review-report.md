# Review Report: Python 3.15 migration and qualification

## Summary

The retained CareOps Sim runtime requirements are accepted at main `43ea690`; upstream engine and domain acceptance remain separate.

## Verification Checks

- [x] **Plan Compliance**: Yes — R71 inventory/ownership, R72 qualification, R73 protected integration and R74 terminal handover have retained evidence.
- [x] **Style Compliance**: Pass — existing standard-library helpers and native unittest selectors; no new pytest/retry framework.
- [x] **New Tests**: Yes — actual environment/version/prefix/refusal, out-of-line discovery, all six runtime-independent terminal recorders, and POSIX shim/stdlib controls.
- [x] **Test Coverage**: Yes for changed contracts — final PR and main checks include full context, native/coverage/mutation gates and Linux/macOS session lanes.
- [x] **Test Results**: Passed — PRs #178/#179 final checks and all triggered final-main workflows; actual final-main Python 3.15.1/local-prefix and eight runtime controls.

The original review findings were fixed and their threads resolved with executed workflow evidence. Actual retained-main setup then exposed the POSIX shim copy failure; the failed new environment was preserved, a real regression reproduced it, and PR #179 qualified the correction before acceptance. Existing environments remain protected against replacement. No in-scope findings remain.

See [closeout evidence](closeout-evidence.json) for immutable main/run identities and current source hashes, and the dated qualification receipts for exact commands, first failures and raw output hashes. No unchanged expensive tests were repeated merely to update checkboxes. The original private checkout/candidate and Kairos pin remain preserved. No Windows/container, clinical, package-release or portfolio-wide acceptance is claimed.
