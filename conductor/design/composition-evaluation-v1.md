# Composition evaluation companion v1

This implements A7.2/A7.3 as versioned artifact contracts and executable reference
oracles. It extends [C0.2](calibration/c0.2-contract.md) and the hash-bound P3.1/P3.2
synthetic contracts. Their source bytes, canonical trace schema and Rust fitting
owners remain unchanged. New invented synthetic-second values are not numeric
imports or unit conversions of P3.1/P3.2. No empirical fitting, parameter promotion,
clinical validity, site acceptance or fiscal optimisation follows.

The [schema](composition-evaluation-v1.schema.json) validates the
[synthetic companion](../../model-inputs/core/a7-evaluation.synthetic.json).
Run `.venv/bin/python tools/composition_contract.py model-inputs/core/a7-evaluation.synthetic.json`
with the qualified repo-local Python3.15.1/jsonschema4.26.0 tooling. This is an
artifact validator, not a Python simulation/calibration runtime.

## Observation and inference boundary

Declare units, origin, cohort/inclusion rule, half-open window and historical
materialization cutoff. Preserve occurrence and knowledge availability separately.
Observed, right-censored, delayed, unknown, missing and excluded endpoints remain
distinct. Last contact alone does not establish event-free follow-up. Censoring
needs an explicit reason, verified event-free interval `[risk_start,event_free_until)`
and evidence available by cutoff. Claims extending past an actual endpoint reject.
A delayed endpoint before cutoff cannot be hidden as censoring even when an older
verified event-free lower bound exists. Its endpoint follow-up remains unresolved;
the verified lower bound is reported separately. Missing/unknown never become zero.
An event at window end is outside the observation window; an explicitly verified
exclusive event-free interval may support administrative censoring at that boundary.

Delayed status here is a retrospective audit classification. It never grants a
policy-time feature or reveals a future endpoint to a decision. Every used feature
binds its member and actual prediction cutoff. V1 predicts at member entry `at`;
a different landmark needs a separately frozen contract. Feature availability
must precede that member cutoff, independently of later audit availability.
Latent requests/unmet demand retain model-only lineage and separate denominators;
observed arrivals cannot establish unseen demand or imply zero unmet need.

Known synthetic family/person membership defines grouped splits. Longitudinal
people and linked families cannot cross train/selection/locked-test stages;
chronological windows cannot overlap. Unknown cluster evidence is a hold, not
independence. Freeze the test plan before opening it; test-based selection or
retuning rejects. These are artifact/plan constraints, not proof of a new fitted
model. The existing held-out P3.2 result is preserved, not rerun or reselected.
Total-only observations retain every finite-grid decomposition; no micro hazard
or causal parameter is identified from an aggregate. No new calibration family,
empirical parameter value or operating range is selected.

## Separate accounting and offline policy

Activity, resource, economic cost, funding/payment, outcome and equity are
separate ledgers. Record identity, source component, units, observation period,
valuation basis, currency/price year and perspective. Monetary quantities use
explicit integer minor units. A resource-use cost is not a payment. The same
source component may appear in different ledgers but cannot be charged twice
within one ledger/period. No automatic cost/funding net or monetary conversion.
Access numerator/denominator pairs remain explicit; undefined denominators reject
and distinct cohort ratios are not silently summed.

The versioned offline fiscal proposal binds the exact evidence period/units and
valuation. Calendar decision/effective periods carry an explicit positive lag;
early/same-period feedback rejects. Capacity units must match declared resource
units. Partial observation windows are not silently scaled to a month. Activation
stays `proposal_only`; no budget, payment, operational capacity or native object is
changed. Future operational policy adapters and optimisation need their own
qualification. All illustrative amounts and time scales are invented fixtures.
