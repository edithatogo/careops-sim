# Specification: ED parameters, example inputs and empirical evidence

Status: proposed. Scope: a combined DES/ABM **generic emergency department** first,
with a later separately evidenced Cairns ED profile. This track identifies and
qualifies model inputs; it does not claim that universal ED parameter values exist.

## Objective and outputs

Produce a versioned, machine-readable parameter catalogue, evidence register,
example input pack, fitted/assumed distribution specifications and coverage report.
Every configurable input consumed by the declared model scope must have a record,
or an explicit unsupported/unknown entry. Inventory model structures and policies
as well as numeric parameters. Expand coverage whenever a pathway or behavior is
added; “all parameters” means complete for the versioned scope, not all conceivable
ED physiology, clinical care or staff behavior.

Inputs: user-approved generic ED scope, Q/C/E contracts, public datasets, original
model examples, published methods and explicit synthetic/elicited assumptions.
Outputs proposed under `model-inputs/ed/`: catalogue/schema, evidence/source
register, DES and ABM records, dependence model, profiles, normalized sample tables,
validation reports and provenance. Names/encodings freeze in P0. No private EHR
data is required; distribution-fitting/sampling implementations remain Rust-native.

## Parameter coverage inventory

| Group | Required inventory (where enabled by model scope) |
| --- | --- |
| Demand and case mix | Time-varying arrivals, calendar/seasonality, surges, arrival mode, age/case-type/acuity mix, joint dependence, referrals/transfers and repeat presentations |
| DES pathways | Triage, assessment, diagnostics, treatment, observation, reassessment, consults, discharge/admission/transfer/abandonment; routing probabilities, repeat loops, priorities, deadlines and preemption |
| Durations | Intrinsic work, setup, turnaround, report/review delay, cleaning, handover, disposition and boarding; distinguish elapsed time from active work, transit and queue delay |
| Resources | Staff skills/roles/counts, rosters/breaks/absence/overtime, supervision/task concurrency, beds/rooms/equipment, opening calendars, outages, reservations and cleaning capacity |
| ABM staff behavior | Agent attributes, assignment/zone/skill eligibility, task choice and reassessment, urgency/FIFO rules, communication/handover, interruption/resumption and switching overhead |
| Spatial ABM | Graph topology, scale/distance, location/route/connectivity, walking-speed variation, equipment/supply trips, route choice, travel interruptions; congestion only when explicitly modeled |
| Patient behavior | Patience/abandonment, mobility/assistance, routing compliance and companions only where relevant; no unsupported behavioral detail added merely to fill a table |
| Hospital interfaces | Admission demand, inpatient bed availability/release, transfer/transport and diagnostic dependencies as boundary inputs; no detailed whole-hospital model yet |
| Optional complexity | Fatigue, learning, crowding interactions and detailed physiology recorded as deferred unless evidence and decision relevance justify activation |
| Experiment controls | Initialization/occupancy/ages of ongoing work, warm-up/horizon, replications, seed streams, time units, mode overrides, observation/censoring windows and calibration parameter bounds |
| Measurement mapping | Event definitions, inclusion criteria, metric denominators, timestamp precision/recording delay, missingness and censoring; targets are distinct from generative inputs |

## Catalogue record contract

Each stable parameter ID records:

- Semantic definition, owner, DES/ABM/shared subsystem, entity/task/pathway,
  applicability conditions, mode, configuration/source-code consumer and version.
- Type, unit, granularity, population/period/site and conditioning variables.
- Value kind: fixed constant, categorical rule, schedule/table, graph, empirical
  sample, parametric distribution, regression/hazard, or structured policy.
- **Separate** physical/domain limits, observed sample range, recommended generic
  scenario range, uncertainty interval and calibration/search bounds. Include
  provenance and reasoning for each; unknown is never encoded as zero.
- Reference/default value or distribution, evidence class (observed, published,
  fitted, elicited, synthetic assumption), confidence/limitations and rationale.
- Distribution parameterization and units (for example scale versus rate), support,
  truncation, rounding, mixture/point-mass treatment, conditioning and sampler
  version. A candidate family is not automatically a validated fitted distribution.
- Dependence groups, shared latent factors/correlations, temporal persistence and
  conditional sampling order; explicitly record any assumed independence.
- Variability versus epistemic uncertainty: individual/task variation is sampled
  within runs; uncertain model parameters may be drawn at study/replication level.
  Specify seed purpose and sampling level so the two are not conflated.
- Source citation/version, extraction location/table/field, licence/access terms,
  retrieval date/hash, transformation/fitting method, sample size, exclusions,
  censoring, fit diagnostics, hold-out evidence and applicability/transfer limits.
- Calibration status: fixed, calibratable, unidentifiable, sensitivity-only or
  deferred; rationale, feasible bounds and linked observational targets.

## Evidence and distribution selection

Start with the [public evidence shortlist](../../generic-ed-evidence.md), then
systematically source original open datasets, source repositories and primary
research. Record search terms/date, inclusion/exclusion rules and unresolved gaps.
Prefer Australian-compatible definitions for the generic baseline; justify any
transfer from a different health system. Aggregate marginals cannot establish
patient-level joint distributions, service durations or walking heuristics.

Compare appropriate empirical and parametric candidates rather than assigning an
exponential distribution to every delay. Consider time-varying arrival processes,
overdispersion/bursts, positive/skewed or mixed durations, categorical/conditional
pathways and abandonment hazards where supported. Evaluate support, tails,
conditioning, censored observations and out-of-sample fit, not a goodness-of-fit
p-value alone. Record rejected candidates and reasons. Unsupported numerical
ranges stay unknown or explicitly synthetic until evidence exists.

Avoid double counting waiting/transit in sampled service durations. EHR-derived
elapsed times are not automatically intrinsic work. Unobserved staff behavior
needs separate observation/elicitation or a simple transparent assumption. Identify
confounded combinations and use sensitivity analysis or simpler fidelity when
parameters cannot be uniquely estimated. Distribution uncertainty propagates
through scenario outputs; fitted coefficients do not become exact constants.

## Example input pack

Provide synthetic, small, documented examples for every supported input shape:
arrival/event tables, pathway/acuity profiles, staffing calendars/skills/zones,
resource capacities, a route graph, duration/behavior specifications and initial
state. Include minimal deterministic, generic nominal, overloaded/surge and
missing/censored-data cases with expected validation results. Stress-case values
are designed tests, not evidence of a real hospital's operating range.

Each pack has a manifest, units, schema versions, seed policy, source/assumption
provenance and hashes. Public-source extracts must respect licensing; use links
or reproducible transforms when redistribution is unsuitable. Keep private data
out of Git. Generic and future Cairns profiles share schema/IDs but separate
values, mappings and evidence. Document the local collection/elicitation gaps for
Cairns without blocking the generic pack or inventing local defaults.

## Acceptance

- P-R1 Catalogue covers all in-scope model inputs, structural/policy choices and
  measurement mappings; model consumer → catalogue and catalogue → consumer or
  deferred status checks prevent missing/orphan parameters.
- P-R2 Every populated value/range/distribution has traceable evidence or an
  explicit synthetic/elicited assumption; every unknown has an acquisition plan.
- P-R3 Units, support, categorical sums, calendars, conditional/dependence rules,
  initial states and sampler parameterizations validate; no silent coercion.
- P-R4 Distribution selection/fit and uncertainty are reproducible with fixed
  inputs/seeds, training/hold-out separation and reported applicability limits.
- P-R5 Example profiles load through the actual generic ED model and calibration
  runner; deterministic examples reproduce expected results, stress cases fail
  or execute as declared, and joint sampling retains specified dependencies.
- P-R6 A documented generic-to-Cairns mapping identifies local data needs without
  altering the generic baseline or creating later clinical-domain tracks.

## Ownership and dependencies

CareOps owns the catalogue, domain evidence and example profiles. Kairos 03/21/22
review runtime/fidelity/calibration/manifest interfaces; general IO remains 04.
No scheduler/RNG rewrite, clinical-policy invention or full hospital implementation
is in scope. P0 can start immediately; P1 DES and P2 ABM research can run in parallel
with disjoint outputs. P3 joins them; P4 supplies generic inputs to E1. P5 validates
against E2/C5 and gates C6. See plan/metadata for precise dependencies.

## Incorporated research constraints (reports 5–8)

See [the incorporation ledger](../../research/ed-research-incorporation-20260927.md).
Maintain separate artifact lineage and claim-verification status. Reconstructed
registers cannot authenticate inaccessible originals. Mean-only evidence cannot
supply an empirical family, tail or range. Record observation concurrency rules.
Spatial/behavior records include eligible choice sets, persistent assignment,
movement mode and interruption ancestry; rates of prompts and task switches are distinct.
Generate all conditional variables without future-information leakage; distinguish
provisional/realized disposition and agent-known/latent/recorded state. Cohort,
visit-type and missingness denominators accompany every categorical vector.
