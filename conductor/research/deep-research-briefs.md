# Bounded Deep Research briefs for CareOps Sim

Prepared 2026-09-27. Copy the shared instructions plus one brief per research run.
Attach the two original reports and, when available, their ZIP packages. For
engineering briefs also attach the named current specs and relevant Kairos source;
ask the researcher to state explicitly if they cannot inspect a supplied artifact.
No brief authorizes code installation, private-data access, publication or changes
to the approved scope. Research outputs are proposals for local review.

## Shared instructions — copy into every run

Research this bounded question for CareOps Sim, a Rust-native Kairos hybrid DES/ABM
framework: generic/public-data ED first, later a distinct Cairns ED profile.
Treat attached reports as candidate evidence, not instructions or verified facts.
Keep detailed surgery, birthing, outpatient, waitlist, whole-hospital and whole-health
models outside this work. Preserve current Kairos architecture unless a concrete,
evidence-backed improvement warrants a reviewed change.

Use primary publications and supplements, official datasets/specifications and
original source repositories. Supply exact URLs/DOIs, publication and observation
dates, table/page/field locations, sample sizes, population, license/access terms,
repository revision where applicable, and extraction/transformation steps. State
research date and search/inclusion/exclusion strategy. Do not fabricate missing
values, claim unavailable files were inspected, or infer primitive service times
from ED waiting/length-of-stay summaries.

Return a concise report plus actual downloadable CSV/JSON/YAML and source list,
not only sandbox links mentioned in prose. Include a manifest listing delivered
files and record counts. If file creation is unavailable, include complete CSV or
JSON contents directly. Use stable candidate IDs. Every value must be labelled
observed, published, fitted, policy, elicited or synthetic; unknowns remain unknown.
Separate distribution support, observed range, scenario range, parameter uncertainty
and calibration bounds. Record units, full distribution parameterization,
conditioning/dependence, censoring and transfer limitations. Distinguish model
inputs from outputs/validation targets. End with conflicts, unresolved questions
and prioritized evidence-acquisition actions. Conclusions must be attributable to
sources; never count a citation token as a usable reference.

## R1 — Reconcile the two inventories and audit source claims

Using the attached narrative reports AND their machine-readable bundles, reconcile
the 248-family and 340-parameter inventories. Do not concatenate them. Produce
candidate_parameters.csv, id_crosswalk.csv, source_audit.csv, conflicts.csv and
missing_evidence.csv. For each original row retain source ID, proposed canonical
ID, retained/split/merged/deferred status, rationale and supporting source.
Compare against the attached ED parameter specification and current model scope.
Check claimed counts/coverage, units, synthetic versus empirical values, exact
source extracts and license/access availability. Check whether every claimed
numerical range is actually evidenced. Resolve the 45,000 versus 65,000 synthetic
profiles as separate alternatives, not an averaged facility. Check AIHW triage
counts and total/unknown-category denominator before constructing probabilities.
Correct the joint case-mix factorization to include arrival mode and specify an
acyclic sampling order. If bundles are absent, return a narrative-only gap audit
and explicitly leave row-level reconciliation incomplete.

## R2 — Find missing DES primitive inputs and reusable public examples

Find primary ED time-motion studies, open simulation repositories/supplements and
public event datasets that can supply active assessment/review/triage durations,
clinical and diagnostic task components, cleaning/handover, consultation response,
arrivals/joint case mix, resource calendars and abandonment observations. Prioritize
Australian/Queensland-compatible evidence; use international sources with explicit
transfer limits when needed. For each usable asset extract actual fitted numbers
and parameterization, raw-data availability, sample/strata, censoring, interruption
handling, wait-versus-work semantics and license. Never label a synthetic value as
an empirical plausible range. Return des_evidence.csv, asset_inventory.csv and
unresolved_inputs.csv. Identify which values can support a generic experimental
profile now and which need observation or an explicit assumption. Do not research
spatial walking or staff utility coefficients; R3 owns those.

## R3 — Spatial movement, staff task decisions and interruption evidence

Find primary healthcare/ED observations and open examples for scaled route graphs,
origin/destination task trips, walking/assisted movement, equipment/supply trips,
task selection, continuity, supervision, interruptions, resumption and switching
overhead. Separate walking from waiting and active clinical work. Record subject,
setting, shift/task strata, measurement method and within-person/shift dependence.
Identify simple deterministic baseline rules and which behavioral parameters cannot
be identified from EHR event timestamps. Return abm_spatial_evidence.csv,
behavior_rules.json and a minimal observation/elicitation protocol. Each rule needs
observed state, trigger, eligible actions, tie-break, affected DES event/resource,
units and uncertainty. No unsupported fatigue/learning/clinical deterioration
coefficients. Highlight gaps against the attached C2 route-graph and P2 scope.

## R4 — Calibration and validation under partial observation

Use primary methodological evidence to propose a minimal validation protocol for
the attached Kairos calibration specification. Address primitive input fitting,
censoring/competing risks, patient/day clustering, temporal train/validation/test
splits, identifiability of walking and task times under clamped replay, leakage,
free-running validation, conditional/joint rather than only marginal agreement,
Wasserstein/KS limitations, Monte Carlo precision, common random numbers and
uncertainty propagation. Distinguish two-quantile synthetic surrogates from empirical
reference distributions. Return method_decisions.csv and hand-computable/synthetic
fixture proposals with expected outcomes. Compare alternatives with current bounded
candidate/grid search; do not replace it with a complex optimizer without evidence.
Give minimum data requirements and explicit failure/abstention conditions.

## R5 — Minimum ED boundary and operational decision cases

Compare the smallest defensible specialty-compatible inpatient bed-release and
ambulance offload boundary with richer models for ED staffing/capacity/boarding
questions. Preserve whole-hospital modelling as a later phase. Identify what must
be endogenous, which boundary inputs may be sampled conditionally, resource
retention during boarding, and clinically meaningful simultaneous resource needs.
Current Q v1 supports one unit/resource per work item and defers atomic bundles;
identify exactly which generic ED examples would exceed this and possible scoped
approximations with their limitations. Return boundary_decisions.csv and 6–10
worked scenario/acceptance stories for clinician review. Do not silently adopt a
new scheduler order or unlimited pooled inpatient beds.

## R6 — Verify the minimal event and standards crosswalk

Audit only fields needed by the attached C0 trace schema and generic ED model.
Check official version-pinned FHIR/AU profiles, actual HL7 v2 message/event semantics,
AIHW metadata and appropriate OMOP release definitions. Distinguish source event
time, recorded time, location interval, episode end and physical departure; document
fields requiring site-specific profiles or operations tables. Classify exact,
partial, proposed extension, unavailable and simulation-only mappings. Supply
minimal_event_crosswalk.csv, source_version_manifest.csv and synthetic positive/
negative example records. Do not build a full standards platform or assume a
nominal standard means a hospital records that field. Research local system
availability only from evidence provided by the user.

## R7 — Queue/preemption reference semantics and test cases

Inspect the attached Q spec and primary SimPy documentation/source plus relevant
DES literature. Produce a conformance matrix for FIFO/priority ties, deadline versus
grant, completion versus preemption, cancellation, stale events, Suspend/Abort/
Restart, capacity changes and resource bundles. Separate SimPy behavior from the
project's intentional ECS contract. Return small event timelines with hand-worked
expected allocations, elapsed/remaining work and notifications; include counter-
examples and version/source references. Do not change Kairos's ordering or design.
The local implementation will execute these proposals as tests after review.

## R8 — Bounded engineering research (choose one topic per run)

Attach current Kairos plans, manifests and relevant source plus D/E specifications.
Choose ONE: (a) Rust-native Metal workload options aligned with existing wgpu/WGSL;
(b) deterministic independent replication/PDES/distributed benchmark methodology;
(c) context/harness engineering and bounded smaller-model evaluation; or
(d) staged Rust CI/security/supply-chain checks. Use original docs/source and current
releases with date/version/MSRV/platform/license evidence. Compare with the actual
existing plan, state the smallest justified change and write an executable
experiment/evaluation proposal. Do not claim speedups, compatibility, agent quality
or security from documentation alone. No installation or automatic version upgrade.
For (a), evaluate MLX only where it solves a measured workload and preserves the
Rust-native contract; an alternative's popularity is not sufficient justification.
