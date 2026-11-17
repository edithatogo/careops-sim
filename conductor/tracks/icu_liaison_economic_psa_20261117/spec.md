# ICU liaison ANZICS economic simulation — specification

Track: icu_liaison_economic_psa_20261117. Mode: synthetic/public evidence,
not CHHHS clinical validation, operating service, deployment or publication.
Source: user-authorised ICU liaison business-case work, 17 November 2026.

## Scope and acceptance

Provide public ANZICS benchmark provenance; reconcile conversations into a
non-verbatim public evidence/decisions note; produce an editable assumptions
file (no constants embedded in source) and deterministic Rust probabilistic
cost-consequence results. Distinguish CCR versus APD denominators, observed
versus hypothesised inputs, gross resource change versus budget/capacity.
Default missing CHHHS inputs are illustrative, not silently filled as facts.

Use existing Rust workspace, serde/serde_json, sha2 and pinned Kairos RNG.
Do not add a competing simulator or new third-party dependency. This is a
decision-analytic PSA only; temporal ICU/ward bed flows remain a later Kairos
DES gated on local timestamp and staffing-demand evidence.

Owned source paths: crates/careops-core/src/icu_liaison.rs,
crates/careops-core/examples/icu_liaison.rs, adjacent single-line lib export,
model-inputs/icu-liaison/, docs/icu-liaison/, this track, GEMINI.md,
tools/install_gemini_conductor.sh and append-only conductor/tracks.md.
No other module, parent CareOps suite, CI/security, current-state, dependency
pins, root manifest or lockfile changes. Other active owners retain priority.

Inputs: annual eligible discharges; ≥12h delay rate; addressable fraction;
effectiveness in addressable patients; released hours per improved case;
72h unplanned return risk/effect; incremental return cost; net avoidable
index ICU day value; full incremental service cost. Every input carries source,
provenance class and triangular uncertainty bounds. Validate units/ranges,
negative relative effect, unknown fields, and reject raw patient records.

Outputs: stable JSON with input SHA256, seed, quantile summaries of released
bed-days, returns avoided, gross/net resource effects, and non-negative-net
probability; explanatory caveats. No QALYs, mortality savings, occupancy claim
or budget approval from synthetic data. Clinical rescue can increase returns.

Independent gates: same-seed replay, no-effect accounting, identity oracle,
invalid source/range rejection, negative-effect scenario; native cargo tests,
Clippy, formatter, integration checks and independent review before merge.
Conductor plan checkboxes must reflect executed evidence, not intention.
