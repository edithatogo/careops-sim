# A1.3 Review — Versioned Classification and Policy Clocks

**Date:** 2026-10-25  
**Track:** `careops_core_domain_abstractions_20261020`  
**Task:** A1.3  
**Reviewed commit:** `f46b36907f7af97c6fb4e66dbc341faed6ecb540`  
**Disposition:** Accepted for the A1.3 task; A1.4 phase review remains open.

## Review result

The implementation meets the bounded CC-06 task. Urgency assessments retain the
jurisdiction, scheme ID/version, assessment tick, effective date and source
provenance used. Reassessment appends a new immutable history item. The new
policy clock has explicit lifecycle methods and typed events that retain the
policy version and target in force; elapsed policy time is preserved across
pause and reassessment. Event history is exposed read-only. Zero-target breach
behavior and the existing profile schema/defaults remain unchanged.

Decision policies receive an `ObservedPolicyState` with observed tick, recorded
urgency and elapsed waiting only. Outputs use distinct types for urgency,
service-target status, dispatch priority and interruption eligibility. No ATS
priority mapping or automatic interruption rule was introduced. Existing ED
pathway consumers were not changed.

The final source review against A1.3, CC-06, product guidelines, source and tests
found no remaining findings. Review identified that clock events also needed to
record the target duration; this was corrected and verified before acceptance.

## Verification

| Check | Result |
| --- | --- |
| Focused `careops-core` selectors (classification, pathway, profiles, contract negatives) | PASS — 38 tests |
| `careops-ed` pathway conformance | PASS — 8 tests |
| Full `careops-core` and `careops-ed` package suites at `f46b369` | Historically recorded PASS. The referenced log was stored under the ignored local `.artifacts/` directory and is not present in this source checkout; this historical run is not independently reproducible from this checkout. |
| Targeted `rustfmt --check` on all A1.3-owned Rust paths | PASS |
| `cargo fmt --all -- --check` | FAIL — existing formatting differences in unrelated workspace files; none were retained as changes |
| `git diff --check` | PASS |
| `tools/tasks.py check` | PASS — 387 task records |
| `tools/context.py check` | PASS — no link/context errors |

Python 3.15.1 and Rust 1.99.1 were selected. Focused logs, formatter logs,
package-test logs, and the worker result contract are under
`.artifacts/a1.3-policy-clocks/`.

## Review checklist

- [x] Plan compliance for A1.3
- [x] Style compliance on changed paths
- [x] New behavior tests
- [x] Full selected-package tests passed
- [x] No unresolved A1.3 findings
- [ ] A1.4 phase closeout and later track tasks

This accepts A1.3 only. It does not establish clinical validity, operational
acceptance or completion of the core-abstractions track.

## Follow-up review — 26 October 2026

A reviewer agent re-examined the implementation at the integrated source branch
and found that blank urgency scheme ID/version, effective date, source authority,
or citation could be copied into an assessment. Commit `884dd19` adds nonblank
validation and a negative test for each field. The reviewer confirmed the fix and
reported no remaining finding in that implementation/test scope.

The subsequent A1 phase review also identified A1.2 defects and their fixes.
The final A1 phase verification is recorded in [A1.4 phase review](../a1.4-phase-review-20261026/README.md). This A1.3 receipt remains scoped to A1.3 and does not close the full track.
