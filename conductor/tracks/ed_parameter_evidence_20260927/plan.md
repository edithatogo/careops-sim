# Plan: ED parameter and example-input evidence

Status: proposed. Use [the execution protocol](../../execution-model.md).
Workers handle one parameter family/source at a time; the coordinator reviews
population transfer, distribution selection and statistical assumptions.

## P0 — Define scope, taxonomy and input schemas

- [ ] P0.1 Inventory every proposed ED pathway, resource, agent decision, spatial
  feature and experiment/measurement control against E/Q/C specs; record enabled,
  optional and deferred features and a gap list. Reconcile reports 5–8 by semantic
  ID, preserving reconstructed versus original coverage and unresolved artifact access.
- [ ] P0.2 Define stable parameter IDs, catalogue/evidence/profile schemas, units,
  separate range classes and consumer mappings. Write invalid/unknown/provenance
  fixtures before the schema validator; agree interfaces with E0/C0 owners.
- [ ] P0.3 Define reproducible source-search/extraction protocol and split DES/ABM
  research into disjoint bounded packets with explicit outputs and review rules.
- [ ] P0.4 Conductor — review and verify phase (workflow.md).

Exit: P-R1 scope and schema contract accepted. Manual check: trace a patient and
staff task from arrival to disposition; every consumed input has a proposed ID.

## P1 — Source DES demand, pathway, resource and duration evidence

Entry: P0. May run alongside P2 and engine development.

- [ ] P1.1 Search and extract public arrival/case-mix, pathway and hospital-boundary
  inputs, source definitions/licences and suitable open model examples. Save
  citations, extraction locations and reproducible transforms, not just links.
- [ ] P1.2 Identify capacities/calendars, triage/clinical-work/diagnostic/cleaning/
  boarding durations, route probabilities and patience evidence. Separate active
  work from elapsed waits/transit; record population and observation limitations.
  Retrieve the Gerdtz/Bucknall triage table and IHACPA clinician-time report; distinguish
  mean-only evidence, incomplete fits, synthetic defaults and principal-activity sampling.
- [ ] P1.3 Populate DES records with evidenced ranges, candidate distributions,
  dependencies and explicit assumptions/gaps. Test units, provenance, probability
  sums and impossible combinations; independently review each source extraction.
- [ ] P1.4 Conductor — review and verify phase (workflow.md).

Exit: P-R2 DES evidence pack, with unknowns explicit. Manual check: reproduce a
source-derived record from its original table and transformation steps.

## P2 — Source ABM behavior, staffing and spatial evidence

Entry: P0; independent of P1 until the P3 join.

- [ ] P2.1 Inventory staff/agent attributes, task priorities, zone/skill eligibility,
  interruption/switching/handover and shift/break behavior; identify public evidence
  or simple explicit rules, and where direct observation/elicitation is required.
  Specify eligible-action snapshots, persistent assignments, interruption ancestry,
  first subsequent task and eventual resumption; separate incoming prompts from switches.
- [ ] P2.2 Source example spatial graphs/layouts, scale/connectivity, trips, speeds,
  mobility/assistance and routing rules. Record per-agent/task variability and
  contextual effects; classify fatigue/congestion complexity as deferred or justified.
  Include movement modes, O/D purposes, graph revision/access restrictions, sensor
  smoothing and walk/wait/work labels; keep unknown speeds and co-working needs explicit.
- [ ] P2.3 Populate ABM records and assumptions with ranges/distribution candidates,
  dependence and identification gaps. Test graph/unit consistency and rules against
  small hand-worked examples; flag overlap with DES work/wait durations for P3.
- [ ] P2.4 Conductor — review and verify phase (workflow.md).

Exit: P-R2 ABM evidence pack. Manual check: explain every interval of one staff
trip/interrupted task without silently introducing an unobserved behavioral rule.

## P3 — Select distributions, dependence and uncertainty models

Entry: P1 and P2; statistics/method decisions require coordinator review.

- [ ] P3.1 Write synthetic known-distribution/dependence, censored, sparse-stratum
  and confounded-data fixtures. Specify parameterization/support/tail checks,
  training/hold-out partitions and expected validation failures before fitting.
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
- [ ] P4.3 Document parameter meanings, example loading, limitations and overrides;
  define a separate future Cairns mapping and required local evidence/elicitation.
  Hand off the frozen generic pack to E1 and calibration fixtures to C5.
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
