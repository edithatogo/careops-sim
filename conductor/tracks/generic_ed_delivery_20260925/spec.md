# Specification: completed generic ED library and delivery profiles

Status: proposed. Scope is the generic ED and reusable infrastructure only; later
surgery/birthing/outpatient/waitlist/hospital/health tracks are not created.

## Product and architecture

Deliver a Rust-native ED model using Kairos's deterministic DES/ABM Flow runtime,
with a small public API, validated configuration, headless batch runner, metrics,
calibration and reproducible examples. Generic/public baseline first; later Cairns
ED is a separate site profile. The first working MVP is headless: a runnable generic ED with configurable
inputs and useful numeric outputs. The scenario dashboard and spatial visuals
follow later, without coupling core execution to UI.

Proposed parent workspace boundaries (freeze names/layout in E0): reusable domain
model crate, CLI/runner adapter and test/fixture crates; browser presentation is
separate. Kairos remains a source submodule with upstream-owned engine changes.
Avoid a generic plugin framework until an actual extension needs it. Future
simulation methods must share defined time/state/output ownership.

## Model requirements

- E-R1 Configurable arrivals (time-varying, stable case IDs), acuity and pathway
  mix; triage, assessment, diagnostics, treatment, discharge/admission/boarding.
- E-R2 Staff identity/skills/shift calendars/zone eligibility; beds/rooms/equipment
  capacity, cleaning and reservations; model queues and urgency through Q APIs.
  Explicit breaks/shift-end and in-flight-work policy; no negative capacity.
- E-R3 Minimal interpretable ABM dispatch/transit, Macro/Micro policy, no double
  counting; configurable time units, run horizon/warm-up and initialization.
- E-R4 Metrics with exact definitions and denominators: queue wait, time-to-care,
  length of stay, throughput/outcomes, boarding and utilization; report censoring,
  warm-up exclusions, unfinished work and stochastic uncertainty. Count each case
  and busy interval once, including abandonment/transfer if enabled by profile.
- E-R5 Typed configuration/API validation; resource/event/probe limits, cancellation,
  progress, checkpoint/restore, atomic artifact writes and reproducible CLI exports.
- E-R6 Public-data provenance and synthetic ground truth, deterministic fixtures,
  analytic limiting cases, sensitivity/identifiability and held-out validation.
- E-R7 Clean-consumer install, documented API/examples, supported feature/platform
  matrix, compatibility/release evidence and an explicit capability report.
- E-R8 Interactive scenario edit/run/compare/export with accessible controls and
  evidence of native/backend parity; rendering never affects simulation results.
- E-R9 CPU replication, Metal, PDES and distributed modes have independent real
  acceptance gates; no scaffold/fallback mislabelled as hardware execution.

Inputs: versioned scenario/model/site/data/seed manifests and optional routes.
Outputs: terminal outcomes, versioned Arrow/Parquet plus human-readable summary,
logs/provenance/checkpoints, public Rust API and optional dashboard artifacts.
Data distributions/policies must state assumptions; a generic profile is not
claimed to represent Cairns without local evidence.

## Ownership and dependencies

Parent owns ED logic/examples, orchestration adapter and presentation. Upstream
03/01/04/21/22 own engine capabilities through Q/C; 05/02/09 own snapshot/binding
extensions; 32/34/35 own accelerated execution. D supplies readiness and gates.
Read [module-readiness](../../module-readiness.md) for all required/deferred modules.

Costing/funding/VOI/process-analysis libraries discovered earlier remain optional
adapters, not blockers for the initial library. Re-audit their current API/data
licences when an actual output needs them; Rust-native runtime rules still apply.
No blanket porting of their Python implementations is implied.

Acceptance: E-R1–9 are qualified per [G0–G3](../../module-readiness.md), with task-
level tests in the plan. G1 is the completed native library; G2 adds the promised
later interactive product; backend profiles are separate later milestones. All
upstream contract and publication gates remain applicable to affected changes.

## Research integration requirements

E0 must review assisted transfer, supervision and contested equipment against single-resource scope before E2. E1 must separate latent state, agent knowledge, provisional disposition and realized outcomes. E2 must preserve assignment and interruption history, distinguish work/travel/wait, and document multitasking approximations without claiming simultaneous attention.

See [reports 5–8 incorporation](../../research/ed-research-incorporation-20260927.md).

## Reports 9–12 integration

The [integration decisions and candidate oracles](../../research/ed-research-incorporation-9-12-20260927.md)
apply to the tasks in this track. Proposed policies/versions remain review inputs;
no reported research check substitutes for locally executed acceptance.

## Reports 26–30 integration

The [research decisions and acceptance cases](../../research/ed-research-incorporation-26-30-20260927.md)
refine this track without completing implementation gates. External versions and
missing bundle contents remain unverified; existing ownership and DAG apply.

## Later spatial and live visualization requirement

The [spatial capability plan](../../spatial-visualization.md) records capture/CAD → a shared versioned
floor-plan package → PixiJS visualization and Kairos simulation, connected through
WebSocket state sync. Early native development uses synthetic graphs; full capture/
CAD adapters are separately qualified later. The backend remains authoritative.

## MVP-first delivery clarification

The working MVP covers configured beds/treatment spaces and staff capacity,
queues, patient flow, resource occupancy and basic outcome/resource summaries.
Use named locations/zones and a small route/distance graph where Micro transit is
needed; no drawn floor plan, CAD importer, PixiJS, WebSocket service, replay UI or
heatmap is needed. Counts and operational capacity are model inputs independent
of visual geometry. Generic values must be sourced or explicitly synthetic;
Cairns values are not assumed.

E1 produces the runnable pathway slice; E2's integrated fixture is the initial
headless MVP candidate. Require deterministic repeatability, patient/capacity
conservation and a documented executable scenario with basic tabular outcomes.
E3 improves the runner/API/exports and recovery, and C6/D4/E4 qualify calibration
and the hardened native library. Early usable examples do not claim those later
gates complete. Existing prerequisites remain; presentation cannot block native
acceptance. Implement the smallest supported model first and evolve it.

## Authoritative delivery sequence

[Functional MVP → hardened native v1 → extensions](../../delivery-contract.md) defines the
required features and acceptance recipes. E2 is the functional headless MVP; E4
is native v1. Visual/spatial import, live UI and advanced backends remain post-v1.
