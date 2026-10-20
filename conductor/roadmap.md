# Product Expansion Roadmap & Milestone Masterplan

This document establishes the master delivery progression for the CareOps Sim whole-of-health simulation system, guiding multi-scale modeling from individual clinical units up to a multi-hospital regional network.

## Master Milestone Sequence (M0 to M6)

| Milestone | Scope | CareOps Sim Package / Track | Upstream Kairos Engine Enabler |
|---|---|---|---|
| **M0** | **Engine Reconciliation & Baseline** | `repo_reconciliation_kairos_upgrade_20261020` | Waves 9 & 10 (Apple Silicon MetalBackend, Apache Arrow zero-copy FFI, SimPy parity) |
| **M1** | **Multi-Domain Core Abstractions** | `careops_core_domain_abstractions_20261020` (`careops-core`) | Track 67 (Linked Entities), Track 68 (Boundary Channels), Track 69 (Competing Risks) |
| **M2** | **Acute Episodic Units** | `careops-ed` (G1 baseline) & `birthing_suite_delivery_20261020` (`careops-birthing`) | `ConditionWait` team co-allocation & `PreemptiveResource` |
| **M3** | **Surgical Suite & Inpatient Ward Flow** | `careops-surgery` (Theatres, PACU, Day Surgery) & `careops-wards` (Acute, ICU, Postnatal, Subacute) | `Store<T>` equipment packs, surgical tray turnover, bed-blocking backpressure |
| **M4** | **Ambulatory Services, Procedural Spaces & Registries** | `careops-clinics` (Specialist Outpatients), `careops-procedural` (Endoscopy, Cath Lab), and `careops-waitlist` | Asynchronous stock-and-flow registry waitlists, Cat 1/2/3 breach scoring |
| **M5** | **Whole-of-Hospital Campus Orchestration** | `careops-hospital` (Campus coordination, central bed management, imaging/pathology transversal services) | Multi-unit composite state, causal provenance, CoW snapshots |
| **M6** | **Regional Health Network & Policy Integration** | `careops-network` (Multi-hospital CHHHS network, road/air retrieval streams, interstate/external flows, `nhra_gt` ABF funding) | `kairo-ecs-pdes` (LP per hospital), `kairo-ecs-grpc`/`mpi`, `kairo-ecs-game-theory` |

---

## Universal Triage, Transport & Boundary Architecture

Every milestone builds upon shared architectural foundations:
1. **Universal Triage & Priority**: Coherent mapping across emergency (ATS 1–5), acute emergency surgery (Cat 1–4), elective surgery (Cat 1–3), specialist outpatient clinics (CPC Cat 1–3), and retrieval urgency (Priority 1–3). Details in [Universal Triage Framework](design/universal-triage-priority-framework.md).
2. **Multi-Modal Transport & Retrieval**: Modeling road ambulances, non-acute transport vans, rotary rescue helicopters, fixed-wing air ambulances (RFDS), taxis/rideshares, and private vehicles, staffed by EMTs, ACPs, CCPs, or medical retrieval teams. Details in [Transport & Retrieval Design](design/transport-retrieval-interfacility-flow.md).
3. **Cross-Border Inflows/Outflows**: District boundary flows including intra-district rural escalations/repatriations, neighbouring HHS referrals (Torres and Cape HHS), interstate quaternary transfers, and international/tourist presentations.
4. **Activity-Based Funding & Game-Theoretic Policy**: Operational decision modeling (e.g. elective cancellation vs emergency backlog clearing) connected to the National Health Reform Agreement (NHRA) funding caps and NEP/NWAU pricing via `nhra_gt` and Kairos game theory solvers.

---

## Data Progression Strategy

For each stage, establish a reproducible generic model using public data and open
examples where available, with clearly labelled synthetic assumptions where
necessary. Then refine through a separate CHHHS example profile supported by
available local data and validation. The ED adaptation specifically targets
Cairns Emergency Department. Public-data availability and local access are not
assumed to provide the detailed timestamps needed for micro-level calibration.

Keep generic model logic, data mappings, parameter sets and site-specific policies
separate, with provenance for each. CHHHS/Cairns adaptation must not overwrite the
generic baseline or turn a local assumption into a universal engine default.
Compare the generic and local versions explicitly as evidence improves.

The framework combines DES and ABM with extension points for later methods. Core
contracts expose time, entities/components, events, model adapters and versioned
outputs; clinical pathways, site rules and presentation remain separate. Future
methods must integrate with deterministic scheduling, seed/state ownership and
verification contracts rather than creating a competing clock or state store.
Do not add speculative solver/plugin machinery before a concrete method needs it.

The Kairos queue/calibration enhancement tracks support reuse through neutral resource/
task identities, configurable policies and empirical mappings. Acceptance remains
general engine primitives plus generic ED synthetic fixtures. Whole-system
composition and other clinical domain models are not implementation work here.

Create detailed tracks for later stages only when requested. No detailed tracks
for stages 2–7 have been created.

## Conditional Kairos event-kind registry

After the headless MVP and native v1, evaluate a global event-kind registry or
typed namespaces if actual cross-runtime collisions or validation requirements
arise. This is an exploratory candidate, not an MVP or v1 dependency. Track 01
and Track 25 should assess migration of `EventKind::Custom(u32)` and its Arrow,
FFI and CLI representations before committing to a global contract. Until then,
FlowRuntime uses the locally reserved 4000–4003 range in the Kairos core contract.

## Later spatial and live visualization requirement

The [spatial capability plan](spatial-visualization.md) records capture/CAD → a shared versioned
floor-plan package → PixiJS visualization and Kairos simulation, connected through
WebSocket state sync. Early native development uses synthetic graphs; full capture/
CAD adapters are separately qualified later. The backend remains authoritative.

## Post-MVP reuse review

After functional MVP E2 acceptance, [review delivered capabilities for reuse in Kairos](design/post-mvp-kairos-abstraction-review.md). Keep ED policies and site profiles separate; extract only capabilities supported by neutral contracts and concrete cross-domain evidence. This review does not add an MVP prerequisite or authorize speculative framework work.
