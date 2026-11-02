# R12 statistical and join contract

This is the intentional v2 correction contract for the existing alpha processing
layer. R11 is accepted; R12 implementation and runtime acceptance remain open.
The user requested implementation and application of review fixes. Independent
review recommends these corrections instead of perpetuating known methodological
defects in golden output. Scheduler, RNG, clinical policy and event capture stay
authoritative and unchanged. This is engineering evidence, not clinical approval.

## Estimands and uncertainty

| Metric | Point estimate | Sample unit / weighting | Uncertainty |
|---|---|---|---|
| Mean wait | Equal-weight mean of replication means | Replication; retain the documented zero convention for a zero-start replication | Sample variance, ddof1, SE and explicitly labelled normal-approximation interval when at least two replications exist |
| Completion | Sum completed / sum arrivals | Pooled count ratio | Null with `ratio_method_unqualified`; the old unweighted-rate interval describes another estimand |
| Throughput | Equal-weight mean of completed / horizon per replication | Replication | ddof1 as above; invalid zero horizons reject |
| Censoring | Equal-weight mean of unfinished / arrivals per replication | Replication; explicit zero-arrival convention | ddof1 as above; do not label this as the pooled ratio |
| Elapsed | Mean of completed-patient durations | Completed patients pooled across replications | Null with `cluster_method_unqualified`; patient independence has not been qualified |

Each metric row declares its units, estimand, sample unit, weighting, eligible, included and
total replication counts, numerator/denominator where applicable, censor and
unfinished counts, ddof, uncertainty method and status. No samples or one sample
has null variance/SE/interval with an explicit reason. Two identical samples have
defined zero variance, not null. Native no-data points are null with a reason;
the old scalar empty/singleton zero conventions are separately recorded until
their consumers migrate. Preserve the stated zero convention in replication
point estimands and disclose eligibility rather than pretending a patient was
observed. No unqualified replacement interval method is introduced.

All-empty cohorts override sentinel-based uncertainty: with two valid positive
horizons but no arrivals/starts, wait/completion/censor points and uncertainty
remain null (`no_samples`); elapsed is null without completions. Their total
replications are two, eligible and included counts zero. Throughput zero remains
defined, with two eligible/included replications and defined zero variance.
For mixed eligible/empty cohorts the declared legacy equal-weight point convention
still includes both replications; eligible and included counts are distinct.
An undefined point must never carry a defined interval from zero sentinels.

Reject NaN/infinity and overflowing integer totals with a typed validation error
before Polars reductions. Integer/tick/key/null comparisons are exact. Finite
derived values use the existing bound `1e-12 + 1e-10 * abs(reference)`; no tolerance
equates null with zero or hides nonfinite values. Subtract tick limbs with the
pinned checked helper before analytical floating conversion; preserve borrow and
full-width values. A half-null pair or negative duration rejects explicitly.

## Key and provenance controls

ED facts are unique by `(run_id, replication_index, patient_id)`. Configuration
patients and resources have unique scenario-qualified dimension keys. Repeated
patient IDs across replications are valid. Validate keys, missing references and
cardinality before joins; no silent drop, multiplication or fallback resource.
Resource/location configuration is an association, not observed presence.

B7 cases use `(run_id, replication_index, case)`; assessments and callbacks add
their captured ordinal/sequence so legitimate equal-tick events survive. Preserve
maternal/infant episodes and nullable entities. Join captured policy by complete
jurisdiction/scheme/version/revision identity and canonical content; never replace
it with latest policy. Preserve null targets separately from valid zero targets.
Choose one authoritative callback representation and cross-check overlapping
embedded/top-level records instead of double counting. Do not invent outcomes.

Patient work capture has no movement observations. Its spatial status is
`not_captured`, with no manufactured transit/position rows and null movement
summary. Remove default15metre routes, waiting-as-transit and one-tick discharge.
Calibration fixture projections preserve explicit caller/route provenance;
absent start/end observations stay null rather than inferred observed endpoints.

## Independent oracles and phase boundaries

[Frozen vectors](../../../crates/careops-ed/tests/fixtures/r121_analytical.json)
include unequal weights (wait5, sample variance50, SE5, completion0.1), skew
(mean10, variance300, SE10), pooled durations (mean11 versus replication mean16),
empty/singleton/identical samples, nonfinite values, overflow, full-width limb
borrow, duplicate facts/dimensions, missing resources and valid cross-replication
identities. They are mathematical fixtures, not clinical observations.

R12.1 authors compiling baseline assertions and freezes these independent vectors.
Actual old-API failures are retained. R12.2 adds production native DataFrame API
calls and typed error assertions against the unchanged expected vectors. A missing
symbol compilation failure is not behavioral red evidence; a raw Polars primitive
probe is not proof of a parent wrapper. No ignored/skipped/xfail control substitutes
for the required native capability. Full green integration, numerical, recovery,
interchange, performance, security and final hosted gates remain mandatory at the
coherent implemented milestones. R12.1 authoring alone accepts none of them.
