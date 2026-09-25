# Product context

CareOps Sim — Emergency Department supports ED clinicians, operations leads and
analysts comparing staffing, capacity and patient-flow scenarios. The approved
vision covers arrivals, triage, assessment, treatment, discharge and admission;
the initial configurable model includes acuity, diagnostics and admission
boarding. Results include waiting times, length of stay, throughput and resource
use, available through reproducible batch runs, exports and a future interactive
dashboard.

## Current planning increment

Enhance Kairos (KairoECS) with reusable, Rust-native ECS resource queues,
preemption, empirical trace ingestion, dual-fidelity execution, shadow replay and
calibration/validation metrics. Reduce model boilerplate while preserving
scheduler determinism. Keep ED-specific policies and data mappings in CareOps.

Prioritize existing Kairos designs and owners. Use recent source revisions with
explicit pins and compatibility evidence. Establish sequential correctness and
local multicore replication before accelerated or distributed execution. Apple
Metal validation follows the existing wgpu/WGSL plan; replacing that route needs
an evidence-backed architecture decision.

This increment produces specifications and implementation plans. It does not
claim to implement the simulation, calibrate real hospital data, validate an ED
model, or complete the wider interactive product-design process.

## General framework and expansion

Build a generalisable DES/ABM framework with extension points for future methods.
Keep reusable engines, model adapters and site profiles separate. Start with a
generic ED informed by public data/open examples, then adapt a distinct profile
to Cairns Emergency Department within CHHHS. Do not require private Cairns data to
build or run the generic model.

Then proceed in order through surgery, birthing suites, outpatient clinics,
waitlist management, whole of hospital and whole of health. Each starts generic/
public, then iterates to CHHHS. [The roadmap](roadmap.md) records sequencing;
later-domain detailed tracks are deferred.


## Development and delivery extension

The readiness audit adds a tested local context harness and explicit delivery
tracks for the generic ED library, dashboard and existing backend profiles.
[Module readiness](module-readiness.md) distinguishes these completion levels.
The engine/model implementations remain planned; current local verification
covers baseline Kairos tests and the context harness only.
