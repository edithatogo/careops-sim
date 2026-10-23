# Python 3.15.1 qualification — 23 October 2026

Base `dc3aecb0c2a3fa32d2e475d9d319374945b97a47`, isolated worktree `/Volumes/PortableSSD/codex-worktrees/careops-sim-python315`, from the retained public clone at `/Volumes/PortableSSD/GitHub/careops/.local/suite-ci/careops-sim`. The original Documents checkout and retained main checkout are untouched.

Active CI, schema lock resolution, audit environments and runtime discovery now use 3.15.1. `.python-version` and README select the repo-local `.venv` (actual CPython 3.15.1). No package manifest or Docker runtime exists at the parent root. Existing submodule pins and historical evidence remain unchanged. External shared-policy workflows remain with their current owner; their internals are not certified here.

Discovery filters candidates to the 3.15 line; a second guard rejects an out-of-line candidate before environment creation. Tests exercise 3.16 exclusion, valid 3.15 preview normalization, malformed values and direct dispatch rejection. Fixed a test harness bug that replaced `python3` inside the actual interpreter path instead of the command token.

## Executed checks

- `uv venv --python /Users/doughnut/.local/share/uv/python/cpython-3.15-macos-aarch64-none/bin/python3.15 .venv`, then `.venv/bin/python --version`: Python 3.15.1.
- `uv pip compile requirements-schema-validation.txt --python-version 3.15.1 --universal --generate-hashes --no-header --no-annotate --output-file requirements-schema-validation.lock`: exit 0, lock bytes unchanged.
- `uv pip sync --python .venv/bin/python requirements-schema-validation.lock`: exit 0.
- `PATH="$PWD/.venv/bin:$PATH" .venv/bin/python -m unittest discover -s tests -v`: exit 0, 709 tests passed. The initial run exposed the interpreter-path substitution bug; this result is the corrected full run.
- After adding the direct 3.16-dispatch rejection regression, `.venv/bin/python -m unittest discover -s tests -p test_schema_canary.py -v`: exit 0, 8 tests passed.
- `.venv/bin/python tools/context.py check`, `tools/tasks.py check`, workflow YAML parse and `git diff --check`: exit 0; 387 task records remain valid.

No simulation seed applies to runtime selection. Tests retain their existing synthetic fixtures. No email, remote release or native business-service change occurred; release-rehearsal output in the logs is a test fixture. No hosted CI or native Rust qualification is claimed. Main integration requires the normal PR/checks process.
