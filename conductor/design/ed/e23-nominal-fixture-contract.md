# E2.3 Integrated Nominal ED Fixture Contract

Status: approved-contract
Date: 2026-10-12
Authority: coordinator contract for Generic ED leaf `E2.3.fixture` (integrated nominal fixture, conservation, simultaneous events, restart/resume).

## 1. Scope & Objective

Leaf `E2.3.fixture` establishes:
- Integrated nominal ED synthetic scenario composed across arrival, triage, consultation/care, disposition, boarding, and departure.
- Simultaneous events and restart/resume edge cases.
- Invariants:
  - Strict conservation of patients and work: zero leaked or duplicate allocations.
  - No movement or state transitions driven by animation or cosmetic clocks.
  - Proper censoring and interval accounting at the simulation horizon.
  - Serves as the foundation for ST001–ST009 boundary acceptance and C6 shared fixture.

## 2. Oracle Verification

The integration suite `crates/careops-ed/tests/e23_integrated_fixture.rs` proves:
1. `nominal_integrated_ed_conserves_patients_and_capacity`:
   - Multi-patient simultaneous arrivals across distinct acuity and disposition intents (Discharge vs Admission).
   - Validates patient conservation and complete work interval accounting.
2. `simultaneous_arrivals_and_resource_competition_deterministic`:
   - Multiple patients arriving on the same tick with equal/distinct acuities.
   - Resource allocation follows deterministic priority and StableKey tie-breaking without race conditions or starvation.
3. `horizon_censoring_preserves_in_flight_state_and_accounting`:
   - Patients in mid-treatment at the horizon tick are accurately categorized as Censored without state or capacity corruption.
4. `repeatable_execution_under_identical_inputs`:
   - Exact run repeatability across independent instances.
