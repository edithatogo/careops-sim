# Handoff

## Current state

Track registered to reconcile and finish existing candidate work; no phase accepted. The [dated qualification snapshot](prior-qualification.md) and [hash manifest](prior-evidence.json) preserve prior work. Kairos catalogue PR #295 is already merged independently. The active conversation goal is armed and covers full migration, not just this registration.

## Next step

Verify inventory and candidate equivalence against current main, then adopt qualified changes under fresh exact-path claims. Do not rerun unchanged expensive qualification solely for tracking. Do not advance the CareOps Kairos pin.

## Evidence boundary

Candidate local success is not current-head, hosted, main or environment-handover acceptance. Record actual blockers and preserve existing implementation owners. No emails or native service changes.

## Registration review

Conductor review found no remaining in-scope planning issues after correcting required handoff markers and local metadata status. No implementation checkbox was completed. [Validation receipt](registration-evidence/receipt.json) retains the tested staged-tree identity, exact commands, cwd, Python runtime, exit codes and output hashes. The existing expensive candidate qualification was not repeated. The shared conversation goal remains active; main/runtime acceptance is still pending.

## Candidate reconciliation

The original `0aee0d2` migration candidate applied cleanly to the current-main-based planning branch. Its original worktree and qualification evidence remain preserved. Fresh schema canary tests (8), task catalogue and context validation pass with the explicit local Python 3.15.1 interpreter and the hash-locked schema dependencies; see [integration receipt](integration-evidence/receipt.json). The first test attempt lacked the schema dependencies in the fresh local environment and failed the required-distribution check; the locked install resolved that setup failure.

The active-use audit found that selecting 3.15.1 with setup-python does not yet bind all workflow helper execution to a repository-local environment. Complete that binding, verify actual child runtime and preserve failure reporting before claiming runtime migration acceptance. This commit preserves the earlier candidate rather than claiming the gap is closed. No implementation phase is accepted, and no Kairos pin changes.

## Local runtime binding

All seven parent Python workflows now verify a repository-local `.venv` after source checkout and invoke its exported absolute `CAREOPS_PYTHON` path. The guard verifies actual Python 3.15.1 and `sys.prefix`; unmanaged, aliased, wrong-version or symlink environments reject without replacement. The schema candidate environments moved beneath `.artifacts` in the checkout. Existing hash-locked dependency, security and full integration gates remain. See [runtime implementation evidence](local-runtime-evidence/receipt.json): six guard cases, eight schema cases, actionlint and whitespace pass locally. Current-main/hosted/handover proof and broader workflow regression reconciliation remain pending.

## Focused test and workflow regression policy

Native unittest selectors remain the default. Repository instructions now specify focused files/classes/methods during edits, broader selection for dependency/configuration/schema/fixture or unknown impact, per-worktree caches and unchanged full milestone/integration/security gates. Future pytest cache modes have explicit semantics and do not imply zero-test proof; no global retries or skip/xfail speed workaround. Seven runtime/CI-binding cases and 22 workflow contract cases pass; see [selector receipt](selector-evidence/receipt.json). Pytest plugins are not introduced into this unittest repository without a demonstrated lane and separate exact-version qualification. The full harness is running and is not yet accepted.

## Full harness reconciliation

The full 717-test local harness executed on Python 3.15.1 (342 seconds). It found one assertion and 12 subtest errors, all in three workflow fixture files that still selected/asserted `python3` instead of the verified `CAREOPS_PYTHON` command. Updating those fixture bindings preserves the actual inline receipt scripts and their first-failure/unexecuted-stage checks. All eight tests in the three affected files now pass. See [full output and focused-fix receipt](full-suite-reconciliation/receipt.json). Retained successful unaffected evidence avoids another edit-loop full run; final full hosted integration remains mandatory.
