# E2.2 Resource Claims and Limits Native Verification Acceptance

- **Date:** 2026-10-12
- **Track:** `generic_ed_delivery_20260925`
- **Leaves:** `E2.2.claims` (Staged staff/space ownership integration), `E2.2.limits` (Unsupported multi-role policy validation)
- **Status:** accepted by coordinator
- **Contract:** [e2-claims-limits-contract.md](../../design/ed/e2-claims-limits-contract.md)
- **Test suite:** [e22_claims_and_limits.rs](../../../crates/careops-ed/tests/e22_claims_and_limits.rs)
- **Source commit:** `ee49cf9a70402b73c3bd8b86c312f74c1924f353`

## Oracle Verifications Passed

1. `claims_single_resource_per_work_item_invariant`:
   - Single-unit/resource limitation per work item verified.
   - Bed and nurse requests are partitioned into distinct queues; flushing bed grants bed without granting queued nurse.
2. `staged_space_and_staff_sequential_coordination`:
   - Space and staff lifecycles are decoupled.
   - Doctor consultation completes and releases active resource while space cubicle remains active throughout.
3. `limits_multi_role_concurrent_procedure_rejection`:
   - Procedures demanding multi-person attention (sedation/resus/assisted transfer) without dedicated composite representation fail closed with `AdapterError::UnknownResource`.
4. `limits_dedicated_composite_approximation_allowed`:
   - Reviewed dedicated composite approximation (`team.resus_composite`) allocates and grants cleanly under ST007/ST008.

## Gates Passed

- All 117 tests in `careops-ed` pass with 0 failures.
- `cargo clippy -p careops-ed --all-targets -- -D warnings`: 0 warnings.
- `cargo fmt --check`: Clean edition-2021 formatting.
