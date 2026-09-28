---
name: empirical-calibration-review
description: Review empirical calibration and validation changes for ED simulation data, leakage, censoring, identifiability, units, and residual metrics. Use when mapping EHR-derived observations to Kairos inputs or evaluating calibration/replay code.
---

# Empirical calibration review

Review an exact proposal against its empirical-data and statistical contract. This is not clinical validation and does not authorize use of private patient data.

## Inputs

- Exact repository/base/head or immutable patch identity.
- Versioned data schema, source/derivation record, parameter mapping, and analysis plan.
- Synthetic or approved de-identified fixture references and exact validation commands.

Stop with `hold_for_evidence` if provenance, units, observation window, or cohort definitions are missing. Never request or copy identifiable records into the repository.

## Review

1. Trace each observed timestamp/event to its source field, timezone, unit, inclusion rule, and transformation; distinguish observed, derived, censored, missing, and synthetic values.
2. Check cohort/time splits and trace-replay clamps for leakage. A clamp must not silently force the outcome used to claim fit; report what remains identifiable under the clamp.
3. Check whether distributions match the observation process, including censoring, truncation, ties, and sampling granularity. Separate macro service-time calibration from micro movement/behavior assumptions.
4. Verify metric direction, sample pairing, finite sample handling, confidence/uncertainty reporting, and declared tolerances. Do not choose thresholds after seeing held-out results.
5. Run the exact focused checks on synthetic/approved fixtures; record commit, toolchain, input/schema hashes, commands, exit status, and output location.

## Output

Return `ready_for_review`, `revise`, or `hold_for_evidence`; include provenance and mapping coverage, leakage/identifiability findings, metric checks, exact evidence, limitations, and unresolved data requirements. Do not describe statistical fit as clinical validity or generalize beyond the evaluated population and period.
