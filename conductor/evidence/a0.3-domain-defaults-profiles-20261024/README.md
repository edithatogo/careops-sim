# Evidence A0.3: Domain Defaults, Layered Profiles & Contract-Negative Qualification

**Date:** 2026-10-24T10:55:04Z  
**Task:** `careops_core_domain_abstractions_20261020:A0.3`  
**Status:** PASS  

## 1. Scope & Objective

Establish the healthcare domain abstraction layer in `careops-core` outside Kairos:
1. **Clinical & Site Defaults:** Defined entirely in `careops-core`, never in `libs/kairos`.
2. **Layered Profiles:** Implement [`JurisdictionProfile`](../../../crates/careops-core/src/profile/jurisdiction.rs), [`SiteProfile`](../../../crates/careops-core/src/profile/site.rs), and [`ProfileLoader`](../../../crates/careops-core/src/profile/layered.rs).
3. **Precedence Hierarchy:** Verify resolution order: `Base Defaults < JurisdictionProfile < SiteProfile < RuntimeOverrides`.
4. **Failure Modes:** Verify fail-closed handling on missing source files, incompatible schema versions, jurisdiction mismatches, and malformed JSON.
5. **Contract-Negative Invariant Tests:** Prove that violations of composition contracts CC-01 through CC-08 fail closed.

---

## 2. Verification Matrix

| Verification Scope | Check | Result | Evidence / Details |
| :--- | :--- | :---: | :--- |
| Layered Precedence | Base < Jurisdiction < Site < Runtime Override | PASS | Verified cleaning ticks, boarding targets, and parameter maps |
| Missing Source File | Non-existent jurisdiction/site file | PASS | Rejected with `ProfileErrorKind::MissingSource` |
| Incompatible Schema | Unsupported version (e.g. `schema_version = 2`) | PASS | Rejected with `ProfileErrorKind::IncompatibleVersion` |
| Jurisdiction Mismatch | Site referencing non-matching jurisdiction | PASS | Rejected with `ProfileErrorKind::JurisdictionMismatch` |
| Malformed JSON | Syntax error in profile document | PASS | Rejected with `ProfileErrorKind::MalformedFormat` |
| Acuity Targets | Non-monotonic ATS waiting times | PASS | Rejected with `ProfileErrorKind::ValidationFailed` |
| CC-01 Identity | Recycled ECS handles as Patient IDs | PASS | Rejected; distinct synthetic `PersonId` conserved |
| CC-01 Relationships | Dependent infant survival on parent discharge | PASS | Verified; infant remains active with linkage |
| CC-02 Transfer | Premature origin resource release | PASS | Rejected prior to `HandoverComplete` |
| CC-02 State Flow | Skipping required transfer states | PASS | Direct transition to `InTransit` rejected |
| CC-03 Bundles | Partial bundle allocation failure | PASS | Acquired bay rolled back cleanly on staff deficit |
| CC-03 Preemption | Triage urgency interrupting surgery | PASS | Non-interruptible procedure protected |
| CC-05 Capacity | `Cleaning` transitioning directly to `Occupied` | PASS | Direct jump rejected; must pass `Open`/`Staffed` |
| CC-05 Eviction | Maintenance closure evicting active patient | PASS | Rejected; active work conserved |
| CC-06 Latent State | Decision policy inspecting latent future outcome | PASS | Prohibited; policy sees only observable state |
| CC-06 Zero Target | Immediate-service zero duration target clock | PASS | Handled without division by zero |
| CC-06 Hazard Clock | Admin clock pause freezing progression | PASS | Rejected; clinical hazard advances independently |

---

## 3. Layered Precedence Order

```text
+-------------------------------------------------------------+
| RuntimeOverrides (Scenario/CLI)                             |  <-- Highest precedence
|   e.g., cleaning_ticks = 12, max_boarding_ticks = 180       |
+-------------------------------------------------------------+
                              |
+-------------------------------------------------------------+
| SiteProfile (Facility-specific)                             |
|   e.g., turnover_cleaning_ticks = 20, capacity profile      |
+-------------------------------------------------------------+
                              |
+-------------------------------------------------------------+
| JurisdictionProfile (Regional policy)                       |
|   e.g., ATS 1-5 targets, default_cleaning_ticks = 30        |
+-------------------------------------------------------------+
                              |
+-------------------------------------------------------------+
| Base Defaults (careops-core engine defaults)                |  <-- Lowest precedence
+-------------------------------------------------------------+
```

---

## 4. Key Architectural Artifacts

- [`crates/careops-core/src/profile/jurisdiction.rs`](../../../crates/careops-core/src/profile/jurisdiction.rs): `JurisdictionProfile`, `UrgencyScheme`, `AcuityTarget`, `JurisdictionPolicies`.
- [`crates/careops-core/src/profile/site.rs`](../../../crates/careops-core/src/profile/site.rs): `SiteProfile`, `SiteCapacityProfile`, `SiteType`.
- [`crates/careops-core/src/profile/layered.rs`](../../../crates/careops-core/src/profile/layered.rs): `LayeredProfile`, `ProfileLoader`, `RuntimeOverrides`.
- [`crates/careops-core/src/profile/error.rs`](../../../crates/careops-core/src/profile/error.rs): `ProfileError`, `ProfileErrorKind`.
- [`crates/careops-core/tests/profile_loader_tests.rs`](../../../crates/careops-core/tests/profile_loader_tests.rs): Schema validation and precedence tests.
- [`crates/careops-core/tests/contract_negative_tests.rs`](../../../crates/careops-core/tests/contract_negative_tests.rs): CC-01 through CC-08 contract-negative invariant suite.
