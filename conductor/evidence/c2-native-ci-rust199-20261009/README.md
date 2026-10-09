# C2 native CI and current Rust policy — 9 October 2026

Root integrated workflow and policy packets at Kairos `c16936f08c4e3df6a7ccd7b0e41f20318f2e2b7f`.
ADR-0015 records the user-selected Rust1.99-only current development profile.
The original Cargo minimum declarations and historical toolchain receipts remain;
this profile does not test them or assert upstream discontinuation.

## Implemented

- Replace older core/Wasm/Arrow qualification lanes with pinned1.99 while retaining
  compilation/feature coverage and the `Rust core quality` aggregate check.
- Retain both native OS runners and four Arrow feature selections per runner.
- Add the full DES/ABM/calibration flow suite, then build the sealed example once,
  invoke separate save/restore/restore processes, compare trace bytes, require
  existing-target save exit1 and unchanged sentinel bytes.
- Declare exactly the generated native/replay artifact prefixes before tests and
  upload only synthetic C2 evidence. Existing permissions and action pins remain.
- Replace stale1.98/beta toolchain matrix expectations with a single1.99.0 entry;
  preserve static lane mapping and wrong-installed-version rejection.

## Independent executed local gates

Root executed the literal YAML C2 run block in an isolated child worktree, with
complete Rustup1.99 PATH/RUSTC/RUSTDOC bindings and a separate target. Full native
suite: **468 passed,0 failed,5 ignored across43 groups**. Example build, save,
two separate restores and exact sentinel checks pass. Static matrix, installed
Rust1.99 validator, actionlint and diff-check pass; the deliberately wrong1.98
expectation rejects the installed1.99 compiler (no older compiler executed).
The tested staged tree equals the resulting commit tree. Commands, environment,
logs, hashes, actual checkpoint and output traces are retained here.

The initial root aggregate readback expected the wrong sentinel-summary key and
raised an assertion after the actual workflow succeeded. Corrected postcommit
readback checks the actual `sentinel_save_exit=1` key and commit tree; acceptance
JSON preserves this correction. Policy worker's pre-integration static failure
and initial component-version probe failure are retained separately.

## Review and hosted handoff

Three Luna workers supported requirements/manual review, internal DES/ABM review,
CI/policy implementation and independent negative-oracle review. The latter caught
an any-nonzero sentinel assertion; the integrated workflow now requires exactly1.
Internal engineering review makes no external-maintainer or model-quality claim.

Draft [Kairos PR243](https://github.com/edithatogo/kairos/pull/243) is stacked on
PR229's exact public C2 foundation. This avoids the broad divergent diff against
remote main. Source is published for review, with no merge/release action.
Initial hosted readback at c16936f: all eight Arrow1.99 feature jobs succeeded;
Linux's actual C2 run step succeeded, native Linux/macOS overall jobs still running.
Changelog policy failed and is delegated for a bounded correction. These are
partial hosted results, not final workflow or phase acceptance.

C2.4, general Track22 portable checkpointing, stable API/release and clinical/ED
gates remain open. The native replay API supports one sealed recipe and one
executable identity. Parent512 tests and learning14 tests are earlier executed
receipts; this change only updates documentation, evidence and child pin.
