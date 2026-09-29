# Product expansion roadmap

User direction recorded on 2026-09-25. This is sequencing and scope guidance,
not a set of implementation tracks or a delivery commitment.

| Order | Model scope | Data progression |
| --- | --- | --- |
| 1 | Emergency department | Generic/public baseline → Cairns Emergency Department, within CHHHS |
| 2 | Surgery | Generic/public baseline → CHHHS example |
| 3 | Birthing suites | Generic/public baseline → CHHHS example |
| 4 | Outpatient clinics | Generic/public baseline → CHHHS example |
| 5 | Waitlist management | Generic/public baseline → CHHHS example |
| 6 | Whole of hospital | Generic/public baseline → CHHHS example |
| 7 | Whole of health | Generic/public baseline → CHHHS example |

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
