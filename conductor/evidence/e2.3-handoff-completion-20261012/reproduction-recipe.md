# E2.3 Generic ED MVP Reproduction Recipe & Shared Fixture Handoff

## 1. Overview & Objective

This document captures the reproduction recipe, exact commands, inputs, configurations, seeds, and execution receipts for the Generic ED MVP end-to-end milestone (`E2.3`).

Per the bounds defined in `conductor/tracks/generic_ed_delivery_20260925/agent-contract.md` and `conductor/execution/mvp/recipes.json` for leaf `E2.3.handoff`:
- Exact commands/config/seed/hash/results are recorded.
- Limitations and bounded scopes are explicitly declared.
- **No full v1 production completion or clinical validation claim is made.**

---

## 2. Tested Configurations & Inputs

1. **Nominal Integrated ED Scenario**:
   - Path: `configs/scenarios/e23_nominal.json`
   - Horizon: 20 ticks (1200 seconds / 20 minutes)
   - Configuration: 2 treatment bays (`loc.bay-1`, `loc.bay-2`), 1 doctor (`staff.doc-1`), 2 arrivals (`patient-1`, `patient-2`).
   - Expected Output: 2 departures, 0 queue at horizon, complete pathway progression.

2. **Constrained Integrated ED Scenario**:
   - Path: `configs/scenarios/e23_constrained.json`
   - Horizon: 10 ticks (600 seconds / 10 minutes)
   - Configuration: 1 treatment bay (`loc.bay-1`), 1 doctor (`staff.doc-1`), 2 arrivals (`patient-1` at t=0, `patient-2` at t=1).
   - Expected Output: `patient-1` occupies bay and receives care; `patient-2` experiences physical space delay and wait queue; horizon censoring cleanly preserves in-flight state without resource leaks.

---

## 3. Reproduction Commands

To reproduce test execution and verification locally:

### A. Run Integrated Fixture & Boundary Test Suites
```bash
cargo test -p careops-ed --test e23_integrated_fixture
cargo test -p careops-ed --test e23_boundary_cases
```

### B. Run CLI Runner & Summary Integration Tests
```bash
cargo test -p careops-ed-cli --test e23_runner_summary
```

### C. Direct Execution of CLI Scenarios
```bash
# Nominal execution
cargo run -p careops-ed-cli -- run configs/scenarios/e23_nominal.json

# Constrained execution
cargo run -p careops-ed-cli -- run configs/scenarios/e23_constrained.json
```

### D. Strict Toolchain Lint & Formatting Verification
```bash
cargo clippy -p careops-ed -p careops-ed-cli --all-targets -- -D warnings
cargo fmt --check
```

---

## 4. Test Results & Conservation Proofs

All 129 native unit and integration tests across `careops-ed` and `careops-ed-cli` pass with 0 failures, 0 ignored:
- **E2.3 Integrated Fixture (`e23_integrated_fixture.rs`)**:
  - `nominal_integrated_ed_conserves_patients_and_capacity`: PASS (asserts exact departure counts equal arrival counts; bay and staff occupancies return to 0).
  - `simultaneous_arrivals_compete_deterministically`: PASS (deterministic tie-breaking across repeated runs).
  - `horizon_censoring_preserves_in_flight_state_and_accounting`: PASS (in-flight state preserved; zero leakage).
  - `repeatable_execution_under_identical_inputs`: PASS (byte-for-byte identical output metrics).
- **E2.3 Boundary Cases (`e23_boundary_cases.rs`)**:
  - `st001_st002_compatible_and_expired_offers`: PASS.
  - `st003_st004_occupancy_retention_and_care`: PASS.
  - `st005_st006_offload_and_receiver_staff_sensitivity`: PASS.
  - `st007_st009_multi_role_limits_and_ward_boundary_conservation`: PASS.
- **E2.3 Runner & Summary (`e23_runner_summary.rs`)**:
  - `e23_cli_runs_nominal_scenario_without_error`: PASS.
  - `e23_cli_runs_constrained_scenario_without_error`: PASS.
  - `e23_cli_rejects_malformed_input_with_clear_error`: PASS.
  - `e23_cli_runs_repeatable_under_identical_seed`: PASS.

---

## 5. Explicit Limitations & Boundaries

1. **Scope Boundary**: These fixtures represent the Generic ED MVP delivery milestone (`E2`). They exercise single-role task claims, staged space/staff resource acquisition, FCFS queues, offer consumption/expiration, and terminal disposition transitions.
2. **Multi-Role & Complex Shared Teams**: Multi-role concurrent procedures remain explicitly unsupported per ADR and fail-closed policies in `crates/careops-ed/src/flow_adapter.rs`.
3. **Clinical Validation**: Does not represent calibrated clinical profiles or Cairns ED site parameters; empirical calibration track (`C2`/`C6`) retains authority over empirical parameter fitting and clinical acceptance.
