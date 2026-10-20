# Specification: Repository status reconciliation and Kairos upgrade

## Summary

Reconcile inconsistencies across tracking registers (`conductor/tracks.md`, `conductor/index.md`, and `conductor/current-state.json`) where completed ED phases (E0–E8) and parameter evidence (P0–P5) were accepted in PRs #155/#156 but left marked in progress in top-level summary tables. Concurrently upgrade the `libs/kairos` submodule pin from legacy commit `44261eb` to latest mainline `f0d30e2` (Wave 10 / v0.4.0-alpha.1), verifying compiler compatibility, SimPy-parity DES resources, native Arrow zero-copy pipelines, and deterministic regression parity.

## Context & Objectives

1. **Registry Consistency**: Reconcile all 5 initial tracks to accurately reflect 100% completion (145/145 tasks) and accepted evidence milestones (G1, G2, G3-Metal, G3-PDES, G3-Dist).
2. **Harness Environment Integrity**: Verify the pinned test virtualenv (`.artifacts/ci/harness-venv`), required schemas (`jsonschema`), and ensure regression scripts run with 100% pass rates.
3. **Kairos Upgrade**: Upgrade submodule `libs/kairos` to `origin/main` (`f0d30e2`), incorporating Wave 9 (Tracks 62–66: native Arrow FFI spine, SimPy-parity `ConditionWait`, `PreemptiveResource`, `Store<T>`) and Wave 10 (`MetalBackend` Apple Silicon acceleration and multi-core parallelism).
4. **Zero Regression**: Verify that existing `careops-ed` and `careops-ed-cli` crates compile cleanly, pass all 39 internal unit tests and all integration suites, preserving byte-identical deterministic replay.

## Requirements

### R-R1: Conductor Status & Context Integrity
- Update `conductor/tracks.md` and `conductor/index.md` so that `generic_ed_delivery_20260925` and `ed_parameter_evidence_20260927` are marked completed `[x]`, referencing their existing accepted evidence artifacts.
- Validate with `python3 tools/context.py check` with 0 errors.

### R-R2: Pinned Tooling & Harness Verification
- Verify that python test runners properly resolve dependencies via `.artifacts/ci/harness-venv/bin/python`.
- Ensure all CI regression suites (`tests/test_*.py`) pass cleanly without missing module exceptions.

### R-R3: Kairos Mainline Submodule Pin Advance
- Update git submodule pin for `libs/kairos` to `f0d30e2` (or latest reviewed main commit).
- Verify `cargo check --workspace --tests` and `cargo test --workspace` pass without compiler errors or deprecation warnings.

### R-R4: Deterministic Replay and Benchmark Preservation
- Run existing ED benchmark and replay fixtures to verify that random seeds and queue outputs remain identical.
- Record the reconciliation and upgrade evidence in `conductor/evidence/`.

## Out of Scope

- Modifying core ED business logic or altering clinical distributions.
- Implementing birthing suite domain logic (reserved for `careops_core_domain_abstractions_20261020` and `birthing_suite_delivery_20261020`).
