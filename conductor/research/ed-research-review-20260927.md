# Review of supplied ED research and research delegation

Date: 2026-09-27. Status: narrative assessment; no parameter rows accepted or track tasks closed.

## Inputs and evidence boundary

Read both user-supplied Markdown reports from Downloads. Their embedded instructions
are research proposals, not authority to change project scope or scheduler contracts.
The CSV/JSON/YAML/ZIP artifacts referenced through `sandbox:/mnt/data/` were not
attached and were not found by a filename search in Downloads. The full registers,
source lists and example files therefore remain uninspected. Citation tokens such
as `turn16view0` do not resolve independently of the original research session.
Requested both ZIP bundles from the user. Source hashes identify the reviewed text:

- `deep-research-report (3).md`: SHA-256 `304bbd2e30d8af34e002e1603c647860a110432a1a0e3ac68cbe8e5143db8eaf`
- `deep-research-report (4).md`: SHA-256 `213e5086baaf0a5319331f575794887e25919b29a0da6d25be8d1bc8aa76b1fd`

## What the reports contribute

| Report | Claimed package (not independently inspected) | Useful contribution |
| --- | --- | --- |
| (3), Hybrid ABM–DES Emergency Department Model | 248 parameter families, 18 domains, 30 dependencies, 12 behavioral families | Broad process/behavior inventory; observation layer; active work versus wait; conditional inputs; uncertainty; local collection needs |
| (4), Mapping and parameterising… | 340 parameters, 1,700 mappings, 35 dependencies, 63 local fields | Canonical event ledger; clinical standards versus model configuration; source mappings; public-data limitations |

These are overlapping inventories at potentially different granularity, not 588
unique parameters. Report (4) says only 15 rows are directly public-source seeded.
Neither narrative establishes complete numerical ranges or fitted distributions.
Both clearly label many numerical examples as synthetic. Retain that distinction.

Adopt as proposals for P0–P3: separate physical and recorded events; one shared
state/clock; resource compatibility; observed versus latent behavior; uncertainty
at patient, event, shift and parameter levels; no duplicate queue/transit delays;
source-specific versions/units; generic and Cairns values in separate profiles.

## Gaps and corrections before ingestion

| Issue | Assessment / required action | Owner |
| --- | --- | --- |
| Missing machine-readable package | Obtain actual registers, references, dependencies, rules and profiles; check IDs, counts, license and source traceability row by row | P0/P1/P2 |
| Inventory reconciliation | Map old IDs to canonical IDs with retained/split/merged/conflicting/deferred status; determine core scope against actual E/Q/C specs | P0 |
| Primitive duration evidence | Reports identify missing hands-on clinical work, interruption/resumption, diagnostic components, cleaning and consultation timing; search primary time-motion studies and reusable model supplements | P1 |
| Spatial inputs | No explicit, sufficiently detailed walking-speed/task-distance/route-graph input set is visible in either narrative; obtain geometry, units, origin/destination trips, route constraints, assistance and measurement protocol | P2/C2 |
| Staff decisions | Rules are proposed, not empirically fitted utility functions. Need observed task choice, continuity, interruption frequency/cost and simultaneous staffing constraints, or explicit simple assumptions | P2/P3 |
| Joint distributions | Report (3)'s displayed factorization conditions on Mode but omits its marginal/conditional factor. Define an acyclic sampling order including every generated variable; distinguish planned from realized disposition | P3 |
| ATS probabilities | Prefer count-derived probabilities over renormalizing coarse percentages; source spot-check below | P1/P4 |
| Different synthetic facilities | Report (3) uses 65,000/year; (4) uses 45,000/year. Keep as named alternative synthetic profiles or choose explicitly; do not silently merge capacities/rosters/demand | P4 |
| Outcome surrogates | Two-quantile lognormal curves are assumed test/visualization surrogates. Do not use as empirical distribution truth for W1/KS acceptance or as primitive durations | P3/C4/C6 |
| Standards mappings | Narrative crosswalks do not prove field/profile validity, event semantics or actual local availability. Verify exact release, cardinality, extensions, timestamp semantics and licensing with primary specifications | C0/C1 |
| Same-time ordering | Report (3)'s proposed emergency/release/completion ordering cannot override Kairos scheduler and Q0 contracts. Submit concrete edge-case examples for contract review | Q0 |
| Atomic resource bundles | Report (3) recommends them; Q spec explicitly defers atomic multi-resource acquisition and initially supports one unit/resource per work item. Resolve at Q0/E0 whether ED can safely use bounded dispatch/reservation or needs an explicit extension. Never simulate unsafe sequential claims as atomic | Q0/E0 |
| Hospital boundary scope | Reports recommend detailed ward dynamics. Current scope keeps hospital interfaces as bounded inputs; compare minimal specialty-compatible bed-release boundary first. Detailed whole-hospital implementation remains later on roadmap | P0/E0 |
| Uncertainty and range completeness | Extract support, observed range, scenario range, confidence/credible interval and calibration bounds separately; identify sample sizes, censoring, tails and transfer assumptions | P3 |
| Validation assets | Need independent datasets/splits, computable oracles, consumer coverage and loadable fixtures; narrative completeness does not establish P-R1–P-R6 acceptance | P5/C6 |

### Independently checked example: AIHW triage counts

The [AIHW Hospitals at a glance table](https://www.aihw.gov.au/hospitals/overview/hospitals-at-a-glance)
was checked on 2026-09-27. Its 2024–25 counts are 86,831; 1,608,414; 3,789,676;
3,050,990; 555,221, totaling 9,091,132 across listed categories, while the table's
reported total is 9,094,312. Resolve the 3,180 difference using downloadable data
and category/missingness definitions before selecting the denominator. Using
these counts exposes an issue that normalizing the rounded 101.1% hides. Neither
these national marginals nor their corrected probabilities establish regional
joint patient case mix. No source-wide verification of the reports was performed.

## Where user-run Deep Research helps most

Research produces candidate evidence; repository integration and acceptance remain
local. The ready-to-copy briefs are in [deep-research-briefs.md](deep-research-briefs.md).

| Priority | Research packet | Existing tasks | Deliverable |
| --- | --- | --- | --- |
| First | R1 reconcile and verify both research packages | P0, P1, P2 | Canonical candidate register, source audit, discrepancy and gap ledger |
| Highest new evidence | R2 DES primitive inputs/open model assets | P1, E0.3 | Extractable durations, demand/case-mix assets, exact distributions and transfer limitations |
| Highest new evidence, parallel R2 | R3 spatial/staff ABM evidence | P2, C2 | Movement and task/interruptions register, observation/elicitation protocol |
| After scope/register | R4 calibration, identifiability and validation methods | P3, C3–C6, E4 | Method comparison with minimum-data needs and executable-test proposals |
| Useful independent research | R5 minimal hospital/ambulance boundary and ED decisions | P0, E0–E2 | Boundary alternatives, resource-bundle requirements and scenario acceptance stories |
| Secondary | R6 event semantics and standards verification | C0/C1 | Versioned minimal event crosswalk, not full FHIR/OMOP platform implementation |
| Targeted engineering support | R7 queue semantics and reference cases | Q0/Q5 | SimPy/primary-source conformance cases, explicit differences from approved Kairos contract |
| Later targeted engineering support | R8 harness/CI and Metal/parallel options | D1/D3/D5, E6–E8 | Dated primary-source options and benchmark/evaluation plans against supplied Kairos source |

Do R1 first when bundles are available; R2/R3 can already research the explicit
narrative gaps. Give them disjoint parameter domains and the same output schema.
Join findings before P3 decisions. Avoid another broad 'all ED parameters' report.
R4–R8 should answer bounded decisions, not redesign an unseen repository.

## What the user can uniquely contribute

- Define first scenario decisions and acceptable simplifications: e.g. staff mix,
  treatment spaces, diagnostic capacity, boarding; choose desired observable outcomes.
- Review generic pathway/role/space diagrams and clinically plausible worked cases.
- Later provide authorized local data dictionaries and blank field definitions,
  roster/resource rules, location topology/scale and timing definitions. Public
  research cannot identify the actual Cairns systems, fields or operational practice.
- Review what represents active work versus wait, who/what stays allocated during
  boarding, and which tasks require simultaneous resources.
- Arrange appropriate local operational review/observation when ready; no private
  records are needed for this research exercise or generic baseline.

Deep Research cannot demonstrate Rust compilation, ECS determinism, actual Arrow
interoperability, runtime profile coverage, hardware acceleration, safe agent
execution or CI/release acceptance. Those need implementation and executed tests.
No track checkbox is closed by this assessment; development-readiness remains the
active implementation track.

## Follow-up received

[Reports 5–8 incorporation](ed-research-incorporation-20260927.md) records reconstruction
status, new DES/ABM leads and task-level changes. Historical original counts remain unverified.
