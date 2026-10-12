# E2.2 Resource Claims and Limits Contract: Staged Ownership & Policy Validation

Status: approved-contract
Date: 2026-10-12
Authority: coordinator-accepted design for generic ED delivery track E2.2 leaves (`E2.2.claims` and `E2.2.limits`).

## 1. Context and Architectural Boundaries

Generic ED track E2.2 delivery specifies:
1. `E2.2.claims` ("Staged staff/space ownership integration"):
   - Single-unit/resource limitation is explicit. No implicit atomic multi-resource grants.
   - Flow runtime grants strictly one unit of one resource per work item.
   - Bed/space occupancy and clinical tasks must be coordinated as separate lifecycle stages rather than joint atomic acquisitions.
2. `E2.2.limits` ("Unsupported multi-role policy validation"):
   - Multi-person/multi-role procedures (e.g. sedation, resuscitation, assisted transfer) requiring concurrent multi-resource attention must be explicitly rejected with an unsupported status/error unless modelled strictly as a reviewed dedicated composite approximation.
   - Prohibits simultaneous attention assumptions, unverified multi-role inference, or implicit deterioration hazards (ST007/ST008 compliance).

## 2. Oracle Proofs

Integration test suite `crates/careops-ed/tests/e22_claims_and_limits.rs` proves:
1. `claims_single_resource_per_work_item_invariant`:
   - Every submitted `StageWork` / admission request binds to exactly one native `resource_id`.
   - Multi-resource simultaneous acquisition attempts without explicit sequencing fail closed at validation.
2. `staged_space_and_staff_sequential_coordination`:
   - Space reservation/allocation and clinical staff allocation occur via explicit sequential stages.
   - Space release is decoupled from staff task completion.
3. `limits_multi_role_concurrent_procedure_rejection`:
   - Procedures demanding multi-person attention (sedation/resus/assisted transfer) without dedicated composite resource tokens are rejected.
4. `limits_dedicated_composite_approximation_allowed`:
   - Dedicated composite approximations (single-token team representation) pass validation and schedule properly.
