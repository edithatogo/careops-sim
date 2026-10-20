# Implementation Plan: Repository status reconciliation and Kairos upgrade

## R0 — Status Registry Alignment

- [ ] Task: Reconcile `conductor/tracks.md` to reflect full completion `[x]` of `generic_ed_delivery_20260925` (E0–E8) and `ed_parameter_evidence_20260927` (P0–P5)
- [ ] Task: Reconcile `conductor/index.md` status table and milestone links to match accepted evidence
- [ ] Task: Verify context integrity via `python3 tools/context.py check`
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## R1 — Harness Environment & Tooling Verification

- [ ] Task: Audit harness virtualenv `.artifacts/ci/harness-venv` and ensure `jsonschema` is resolved in default test runners
- [ ] Task: Run full Python regression test suite (`tests/test_*.py`) with harness environment
- [ ] Task: Confirm `python3 tools/tasks.py check` and `python3 tools/mvp.py check` pass
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## R2 — Kairos Mainline Submodule Pin Advance

- [ ] Task: Advance `libs/kairos` submodule pin to `origin/main` (`f0d30e2`, Wave 10 / v0.4.0-alpha.1)
- [ ] Task: Validate Rust workspace compilation (`cargo check --workspace --tests`) against upgraded Kairos crates
- [ ] Task: Verify SimPy-parity DES resource availability (`PriorityResource`, `ConditionWait`, `Store<T>`) for subsequent clinical tracks
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## R3 — Deterministic Replay & Conformance Gate

- [ ] Task: Execute full workspace Rust test suite (`cargo test --workspace`)
- [ ] Task: Run ED replay and macro/micro holdout fixtures to verify zero regression in output traces
- [ ] Task: Generate and record R3 phase evidence document in `conductor/evidence/`
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)
