# Plan: ED parameter and example-input evidence

Status: research intake complete; catalogue qualification and implementation pending. Use [the execution protocol](../../execution-model.md).
Workers handle one parameter family/source at a time; the coordinator reviews
population transfer, distribution selection and statistical assumptions.


## Research already completed and remaining acceptance

Completed: requested research responses received, duplicates reconciled, embedded
material indexed and relevant findings incorporated into specifications/plans.
See [research handoff](../../research/research-handoff.md) and its evidence links.
This completed intake is distinct from the numbered implementation/qualification
tasks below. No broad repeat search or separate-file recovery is a prerequisite.

Remaining: reuse supplied tables, payloads and narrative; verify relevant primary
sources/current source code, document citations and transformations, reconcile
conflicts, and fill only demonstrated gaps. Mark locally authored structured
records as transcribed or derived; never claim recovery of an unseen original.
A missing source stays unverified or an explicit synthetic assumption with a
validity limit. Acceptance requires this track's actual outputs and tests.

## P0 — Define scope, taxonomy and input schemas

- [x] P0.1 Reconcile the supplied report narrative and embedded tables/payloads
  against every in-scope ED pathway, resource, agent decision, spatial feature and
  experiment/measurement control. Create a canonical coverage/gap map with report
  hash/section/line provenance; distinguish retained, merged, conflicting, deferred
  and genuinely absent content. Reuse the completed embedded-content index.
  Historical headline counts are not acceptance targets. F3's incomplete historical
  crosswalk does not block a new scope-complete catalogue from supplied evidence.
- [x] P0.2 Define stable parameter IDs, catalogue/evidence/profile schemas, units,
  separate range classes and consumer mappings. Write invalid/unknown/provenance
  fixtures before the schema validator; agree interfaces with E0/C0 owners.
  Define bed/treatment-space counts, room/zone IDs, staff/equipment capacity and optional route distances as ordinary model inputs. Keep units and stable IDs extensible; detailed geometry/import/visual schemas wait for E5. A tiny named-location fixture is sufficient.
- [x] P0.3 Define source-verification/extraction protocol and split DES/ABM work
  into bounded packets. Each starts with supplied content, traces claims to primary
  sources, and searches only a recorded gap; specify outputs and review rules.
- [x] P0.4 Conductor — review and verify phase (workflow.md).

Exit: P-R1 scope and schema contract accepted. Manual check: trace a patient and
staff task from arrival to disposition; every consumed input has a proposed ID.

### P0.2 accepted design interface

The [P0.2 acceptance record](../../evidence/p0.2-interface-design-acceptance-20260930.md)
and [resolved 101-row matrix](../../../model-inputs/ed/schema/parameter-usage-matrix.json)
supersede the proposed mapping for design-interface use. This accepts no
empirical values, Rust runtime, E2 profile execution or full MVP behavior;
each applicable `open_gate` remains an implementation requirement.

### P0.2 review artifact (historical proposal)

The coordinator prepared a 101-ID consumer/unit/profile mapping proposal at
[`parameter-usage-matrix-proposal.json`](../../../model-inputs/ed/schema/parameter-usage-matrix-proposal.json).
Its coverage check and review boundary are recorded in
[`p0.2-usage-matrix-proposal-20260929.md`](../../evidence/p0.2-usage-matrix-proposal-20260929.md).
This is draft input to E0/C0 review; it does not close P0.2 or freeze keys,
units, empirical values, or profiles. The bounded owner review and blank 101-row
response template are in [the E0/C0 review brief](../../evidence/p0.2-e0-c0-owner-review-brief-20260930.md).

## P1 — Source DES demand, pathway, resource and duration evidence

Entry: P0. May run alongside P2 and engine development.

- [x] P1.1 Transcribe and verify supplied public arrival/case-mix, pathway and hospital-boundary
  inputs, source definitions/licences and open model examples; search only
  missing or conflicting evidence. Save
  citations, extraction locations and reproducible transforms, not just links.
  Catalogue specialty/time/calendar-conditioned ED-eligible bed-offer inputs and transfer delays separately from raw hospital discharge counts or sampled boarding times. Record offer persistence/withdrawal and hidden competing-demand assumptions; no universal default follows from report 29.
- [x] P1.2 Verify supplied capacities/calendars, triage/clinical-work/diagnostic/cleaning/
  boarding durations, route probabilities and patience evidence. Separate active
  work from elapsed waits/transit; record population and observation limitations.
  Retrieve the Gerdtz/Bucknall triage table and IHACPA clinician-time report; distinguish
  mean-only evidence, incomplete fits, synthetic defaults and principal-activity sampling.
  Separate physical bay count from staffed/open capacity and monitoring/equipment capability, closures and surge calendars. Keep a dated as-operated register; drawing symbols cannot establish usable beds.
- [x] P1.3 Populate DES records with evidenced ranges, candidate distributions,
  dependencies and explicit assumptions/gaps. Test units, provenance, probability
  sums and impossible combinations; independently review each source extraction.
  For each empirical claim, record primary URL/DOI, edition/revision, page/table/field, population/period, units, licence, access date and transformation; link the report excerpt separately. Citation tokens alone are unresolved. Record verification outcome and reviewer.
- [x] P1.4 Conductor — review and verify phase (workflow.md).

Exit: P-R2 DES evidence pack, with unknowns explicit. Manual check: reproduce a
source-derived record from its original table and transformation steps.

## P2 — Source ABM behavior, staffing and spatial evidence

Entry: P0; independent of P1 until the P3 join.

- [x] P2.1 Reconcile supplied staff/agent attributes, task priorities, zone/skill eligibility,
  interruption/switching/handover and shift/break behavior; identify public evidence
  or simple explicit rules, and where direct observation/elicitation is required.
  Specify eligible-action snapshots, persistent assignments, interruption ancestry,
  first subsequent task and eventual resumption; separate incoming prompts from switches.
- [x] P2.2 Verify supplied spatial examples and fill gaps in graphs/layouts, scale/connectivity, trips, speeds,
  mobility/assistance and routing rules. Record per-agent/task variability and
  contextual effects; classify fatigue/congestion complexity as deferred or justified.
  Include movement modes, O/D purposes, graph revision/access restrictions, sensor
  smoothing and walk/wait/work labels; keep unknown speeds and co-working needs explicit.
  Document geometry uncertainty, manual annotations and georeferencing separately from routing/clinical assumptions; ordinary indoor metre coordinates are not RFC 7946 GeoJSON.
  Prefer verified existing CAD over new capture when available; preserve source/revision and derived layer manifest, confirm units/transforms and known distances, and record operational review/unknowns. No real CAD is required for the generic MVP.
- [x] P2.3 Populate ABM records and assumptions with ranges/distribution candidates,
  dependence and identification gaps. Test graph/unit consistency and rules against
  small hand-worked examples; flag overlap with DES work/wait durations for P3.
  Use the same primary-source/provenance contract as P1.3; distinguish observed behavior, literature interpretation and chosen heuristic. Maintain a gap record with owner, acquisition route and impact; unsupported values cannot become empirical defaults.
- [x] P2.4 Conductor — review and verify phase (workflow.md).

Exit: P-R2 ABM evidence pack. Manual check: explain every interval of one staff
trip/interrupted task without silently introducing an unobserved behavioral rule.

## P3 — Select distributions, dependence and uncertainty models

Entry: P1 and P2; statistics/method decisions require coordinator review.

- [ ] P3.1 Write synthetic known-distribution/dependence, censored, sparse-stratum
  and confounded-data fixtures. Specify parameterization/support/tail checks,
  training/hold-out partitions and expected validation failures before fitting.
  Include explicit censor/event ambiguity, time-of-knowledge, observation
  window boundaries, patient/day dependence and joint-versus-marginal fixtures.
- [ ] P3.2 Compare empirical/parametric/conditional candidates with reproducible
  fitting diagnostics and held-out checks; record chosen/rejected families,
  sample limitations and uncertainty. Use explicit assumptions where data cannot fit.
- [ ] P3.3 Reconcile DES/ABM boundaries; define conditional sampling order, shared
  factors and seed purposes. Separate variability from uncertain parameters;
  define hard limits, scenario ranges and search bounds with their own rationales.
  Test arrival-mode generation, future-diagnosis/disposition leakage, cohort-specific
  ATS denominators and observation missingness separately from clinical priority.
- [ ] P3.4 Conductor — review and verify phase (workflow.md).

Exit: P-R3/4 reviewed sampling contract for C2 and model configuration. Manual
check: reproduce one fit and one conditional draw; demonstrate that transit and
service are not both calibrated from the same undifferentiated elapsed interval.

## P4 — Build and validate the generic example input pack

Entry: P3. This phase produces configuration data, not the E1 model implementation.

- [ ] P4.1 Create schema-valid minimal deterministic, generic nominal and surge/
  overload profiles with linked sources or clearly labelled synthetic assumptions;
  include arrival tables, calendars, graph, distributions and initial state.
- [ ] P4.2 Add malformed/missing/censored examples and expected diagnostics; check
  probability/conditional tables, clock units, support bounds, resource feasibility,
  initialization and provenance. Store manifests/hashes and licensing notes.
  Include synthetic physical-versus-open capacity, repurposed/closed bay and surge-calendar cases; unknown operational status cannot silently mean available.
- [ ] P4.3 Document parameter meanings, example loading, limitations and overrides;
  define a separate future Cairns mapping and required local evidence/elicitation.
  Hand off the frozen generic pack to E1 and calibration fixtures to C5.
  Use report 30 temporal crosswalk as a candidate mapping with unknown local availability; require Cairns location boundary, source profiles, correction rules and ETL lineage before treating timestamps as observed.
- [ ] P4.4 Conductor — review and verify phase (workflow.md).

Exit: P-R3/6 and complete synthetic/public-backed input shapes; schema checks pass.
Manual check: a reader can distinguish observed values, assumptions, ranges and
uncertainty without inspecting implementation code. Actual loading is tested at P5.

## P5 — Verify catalogue coverage against the actual hybrid ED model

Entry: P4, E2 and C5. Feeds C6 validation; must not depend on C6 itself.

- [ ] P5.1 Implement consumer/catalogue coverage checks against the actual model
  configuration/API: reject missing IDs, unexplained defaults, unused active
  parameters, unit drift and unsupported profile/schema versions.
- [ ] P5.2 Load every example through model and calibration runner; test fixed-seed
  repeatability, Macro/Micro consistency, conditional/dependence preservation,
  intended diagnostics and sensitivity to selected influential parameters.
- [ ] P5.3 Publish a coverage/evidence/uncertainty report, unresolved acquisition
  backlog and versioned handoff. Block validated-profile claims on unsupported
  assumptions; allow explicitly experimental generic scenarios with visible limits.
- [ ] P5.4 Conductor — review and verify phase (workflow.md).

Exit: P-R1–6 evidence for the declared scope; C6 may run final held-out model
validation. Manual check: add a temporary unregistered config field and verify
coverage fails, then remove it and reproduce the documented generic example.

## MVP gpt-6-luna workpack

Every task in this track that is an ancestor of E2.4 is decomposed in the
[MVP leaf recipes](../../execution/mvp/README.md) and
[readable work breakdown](../../execution/mvp/work-breakdown.md). These recipes
are mandatory preparation inputs: freeze/bind interfaces, source slices, paths,
commands and reviewer acceptance before dispatch. Parent tasks close only after
all leaf instances and the original phase acceptance pass. Post-MVP tasks are
outside this workpack. No Luna execution or qualification is implied by coverage.
## P2.2 evidence closeout

See [the P2.2 coordinator acceptance](../../evidence/p2.2-coordinator-acceptance-20261002.md) and linked family records. P2.2 is accepted for supplied-source verification and explicit gap inventory only; event-level staff trip intervals, operational access rules and route choice remain unidentified.

## P2.3 catalogue and synthetic-check closeout

See [the P2.3 coordinator acceptance](../../evidence/p2.3-coordinator-acceptance-20261002.md) and [ABM catalogue README](../../../model-inputs/ed/abm/README.md). P2.3 is accepted for exact-ID schema-v1 catalogue coverage, provenance/gap tracking and synthetic representation checks only. Seventeen active inputs remain unknown; spatial congestion and four optional-complexity rows remain deferred. No empirical generic ED parameter, policy, dependence structure or calibration is accepted. P2.4 independent P2 phase review remains open.
