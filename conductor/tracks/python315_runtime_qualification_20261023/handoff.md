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

## First hosted findings and corrections

PR #178 executes actual Python 3.15.1 and local prefixes on Linux/macOS, with native Linux/macOS tests and native CLI package smoke passing on the first head. The first hosted checks exposed a missing root `.venv` ignore rule (Rust source snapshot rejected fresh environment files), an older shared wheel-only audit lock incompatible with 3.15, and one verified source-digest false positive. The exact digest was recomputed from the named source file; only its historical fingerprint is recorded. No scanner rule or broad path waiver.

The root ignore and real Git fixture fix source cleanliness; parent policy references now use owner-qualified `30df4953ea87c8915b4a525fc671a4c9857c3f7f` (3.15.1/current audit lock). Seven focused runtime cases, actionlint and full-history gitleaks pass locally. See [hosted-fix evidence](hosted-fix-evidence/receipt.json). The shared security workflow still needs independent repo-local prefix proof, and updated-head checks/main/handover remain open.

## Purpose-specific audit environment

The second hosted schema audit still failed because the owner-qualified shared CI bundle includes PyYAML 6.0.3 without a 3.15 wheel. That bundle is valid for other root tooling but overbroad for the existing pip-audit consumer. The two schema audit lanes now use a committed purpose-specific exact input/hash lock for the already adopted pip-audit 2.10.1 (independently confirmed published stable), retaining `--require-hashes --only-binary=:all:`. Its resolved graph contains no PyYAML, and two audit-only policy checkouts are removed. Shared discovery/security policy ownership is retained. This is an optimization of existing tooling, with no new library-adoption track.

Fresh repo-local 3.15.1 macOS ARM64 wheel installation succeeds, the real five-package schema audit passes, and an isolated read-only known-vulnerable requests 2.31.0 control returns exit 1 with six findings. Three actual receipt regressions now bind the audit-lock hash; eight canary regressions and actionlint pass. [Qualification receipt](audit-qualification/receipt.json) preserves commands, first failures and runtime/package identity. No source-build relaxation, downgrade, retry or audit waiver. Hosted Linux/final-head proof remains required.

To review a future auditor update, update `requirements-schema-audit.txt`, resolve with `uv pip compile requirements-schema-audit.txt --python-version 3.15.1 --universal --generate-hashes --no-header --no-annotate --output-file requirements-schema-audit.lock`, qualify wheel-only installation and the audit controls, then publish through normal checks. The input and lock remain explicit reviewed source. Inventory path/hash records retain all prior values while avoiding credential-like keys across squash commits.

## Main integration and POSIX bootstrap correction — 2026-10-24

PR #178 merged through protection as `839cb443ce5cd9170f63bab972d64951faf2d9eb` after all current-head checks passed, including full context, coverage and 116 ED mutation cases (111 caught, 3 unviable, 2 preclassified equivalent misses). Both review findings were resolved with actual inline-script controls. The qualified reusable security MAIN revision `5cf006bce6c37c0c15db5d35055e302701a500ad` is adopted by both caller and policy; the actual PR and main shared-security jobs passed.

The clean retained main checkout was fast-forwarded with an identical tree. Its first fresh `.venv` setup exposed a local uv/POSIX shim issue: `venv` copied the shim executable and lost its standard-library path. The failed, newly created environment was preserved under `.artifacts/python315-main-verification/failed-stdlib-copy-env`; no prior consumer environment was replaced. A real external-symlink regression failed on the merged implementation. The candidate now uses POSIX executable symlinks, retaining Windows copy behavior and all existing prefix/version/symlink-directory refusal rules. All eight runtime cases pass, including actual child standard-library imports. This correction requires its own protected PR and main handover; migration closeout remains pending.

## Accepted parent runtime closeout — 2026-10-24

PR #179 merged as `43ea6902c5cb0ac7bf6089e986d73aa4ac91d55f` after every final-head check passed. All three triggered main workflows passed, including full context/coverage/mutation gates, native tests, both hosted session guards and shared security. The retained primary main is clean, matches the reviewed tree, and actually reports final Python 3.15.1 with its own `.venv` prefix. All eight final-main runtime controls passed, including a real external-symlink bootstrap and stdlib imports. The qualified existing environment was reused without replacement.

All R71–R74 requirements are accepted within parent-owned runtime scope. The current source inventory and immutable hosted handles are in [closeout evidence](closeout-evidence.json); [review](review-report.md) distinguishes dated candidate, final PR, main and handover proof. The existing Kairos pin and original private source/candidate remain preserved. Upstream Kairos's separately observed provider/CI work is not represented as completed by this parent closeout. No clinical, production or new platform acceptance follows.
