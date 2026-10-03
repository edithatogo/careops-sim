# Proposed fixture semantics

Consume the existing calibration specification rather than creating a new runtime API. W1 is the exact integral of absolute difference of independently normalized CDFs; units match input. KS D is the supremum CDF difference after advancing all ties together. Optional weighted D is descriptive, not a classical KS test. No p-value is requested.

Use exact rational arithmetic for reference construction. Runtime floating results require a declared tolerance and algorithm version. Preserve integer event ordering. For absolute large ticks, subtract one common documented origin in exact integer arithmetic before conversion; reject conversion that loses required resolution. Different independently chosen origins can erase a real location shift and are prohibited. Duration scaling multiplies W1 and leaves D unchanged.

Empty samples are insufficient, invalid numbers/weights are invalid; neither becomes zero distance. Weights must be finite, nonnegative with positive finite mass; retain counts and weight provenance. Zero weight observations may remain in source counts but carry no CDF mass. Compare censored and missing rates separately; an observed-only distance requires coverage warnings and is not a survival correction. Keep patient/task cluster IDs for later inference; no iid inference from repeated tasks.

The equal-marginal fixture deliberately has W1=D=0 for each marginal but opposite paired dependence. A proposed conditional diagnostic groups by the first coordinate and compares the second: both group W1 distances are 1. Record grouping rules, eligible counts and empty groups; this does not establish a general multivariate calibration objective.

## Consuming test packets

1. Arithmetic: bind actual metric API and test targets; demonstrate a behavioral red failure, then exact analytic cases, ties, unequal counts, weights and unit scaling. Independent reference generator must be evaluated and pinned separately; hand calculation alone does not satisfy the full C4.1 independent-reference requirement.
2. Validity: typed empty/invalid statuses, large-origin conversion, overflow/scale policy, coverage and missing outcomes. Test that dropping censored records cannot silently improve a reported objective. No automatic correction or p-value.
3. Integration: after C0/C1 acceptance, bind actual calibration_residual.v1/calibration_metric.v1 and Arrow IO; verify counts, units, validity, grouping/window policies, stable ordering and schema compatibility with an independent reader.

Reserve one writer per path. Shared schema/Cargo/lockfile changes require the owner integration barrier. No proposal is dispatched before current prerequisites and source hashes are verified. C4 closes only after all plan requirements and integrated native/IO evidence; the cases here are a starting subset.
