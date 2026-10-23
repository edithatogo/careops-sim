# Plan: Python 3.15.1 migration and qualification

Prior candidate work is preserved in [the evidence snapshot](prior-qualification.md). Unchecked tasks require reconciliation/acceptance; they are not instructions to repeat proven work blindly.

## R71 — Baseline and retained evidence

- [ ] R71.1 Inventory every active Python entry point, subprocess, package, CI/container/helper path and live environment; classify historical or synthetic exceptions explicitly.
- [ ] R71.2 Reconcile the retained candidate evidence and source hashes with current main and current owners; credit completed qualification without treating it as main acceptance.
- [ ] R71.3 Freeze exact path ownership, interpreter/dependency targets, required platform checks and safe terminal handover criteria.
- [ ] R71.4 Conductor — review and verify phase (workflow.md); record evidence, unresolved targets and next-phase decision.

## R72 — Complete and qualify the candidate

- [ ] R72.1 Add or reuse meaningful failure tests for runtime mismatch, out-of-line discovery, environment preservation and native/package execution before changing affected paths.
- [ ] R72.2 Complete supported Python 3.15.1 runtime, constraints, lock, CI, container, helper and current-instruction alignment in isolated environments; remove older active runners.
- [ ] R72.3 Run the affected helper/package/binding/security and representative consumer checks with the explicit repo-local interpreter; retain raw outputs, hashes and concrete blockers.
- [ ] R72.4 Conductor — review and verify phase (workflow.md); require actual runtime evidence and resolve in-scope findings.

## R73 — Reviewed integration and current-main proof

- [ ] R73.1 Reconcile fresh main and ongoing writers, open the bounded PR, address review findings and pass every required hosted check without waivers.
- [ ] R73.2 Merge through normal protection, verify the exact main tree and inspect post-merge runtime/check evidence; preserve unrelated implementation and pins.
- [ ] R73.3 Conductor — review and verify phase (workflow.md); distinguish local qualification, hosted proof and accepted integration.

## R74 — Safe environment handover and closeout

- [ ] R74.1 Confirm old consumers have reached terminal states, hand over the qualified repo-local interpreter safely and verify its actual version in consuming commands.
- [ ] R74.2 Re-audit all active entry points and older defaults, reconcile remaining external/platform blockers, retain rollback/history and update evidence at the accepted main revision.
- [ ] R74.3 Conductor — review and verify phase (workflow.md); complete only when every requirement and handover has evidence.

## Closeout evidence

Every phase requires Conductor review, source/lock/input hashes, exact command/cwd/toolchain and exit status, retained artifacts, owner handoff, clean-tree verification and commit/PR identity. Run `.venv/bin/python tools/tasks.py check` and `.venv/bin/python tools/context.py check`; static validity is not runtime acceptance. Publication must use required PR/checks. No phase is accepted by the existence of this plan.
