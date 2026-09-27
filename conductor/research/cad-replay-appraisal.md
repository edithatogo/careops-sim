# Appraisal: supplied CAD, replay and hybrid-model notes

Reviewed 2026-09-27. Supplied recommendations are design inputs, not authorization
to replace Kairos, install tools or change clinical policy. The headless MVP remains
first. No facility CAD file has been supplied or validated in this review.

## Adopt with qualification

- Prefer an available, suitably licensed, current architectural/as-built drawing
  over new mobile capture as the starting geometry. Check revision, model-space
  units, block/reference transforms and known dimensions. CAD does not eliminate
  uncertainty or prove current operational configuration.
- Preserve the source file; create a simplified derived view/export with a layer
  manifest. Retain walls, openings, bays, stations and labels needed for semantic
  interpretation. Hide unnecessary MEP/annotation layers rather than irreversibly
  deleting information from the source. Relevant services may inform bay capability.
- Maintain a separate dated as-operated register: physical bays, staffed/open beds,
  monitoring/equipment capabilities, closures, repurposed rooms, surge/overflow
  spaces, zones and calendars. Link each record to stable location IDs and geometry
  revision; do not infer functional capacity from the number of drawn bed symbols.
- Review this register with appropriate local operational/facilities knowledge.
  A short NUM/clinical-lead walkthrough is a useful first pass, not a guaranteed
  complete 15-minute validation or replacement for unresolved checks. Record
  reviewer/date, disagreements and unknowns. No meeting or data collection is
  authorized by this planning note.
- Derive a small traversable route graph and validate connectivity, access and
  lengths. Shortest paths are exact only for the declared graph/weights, not proof
  of actual human routes. Euclidean/Manhattan distances require justified geometry;
  do not route through walls or treat all mobility modes as identical.
- First later visualization should replay completed runs in 2D. Reuse versioned
  telemetry and spatial projections with play/pause/seek and comparison. WebSocket
  live monitoring/intervention remains a subsequent supported profile; it is useful
  beyond human interventions but need not block initial playback or headless work.

## Do not adopt as stated

| Supplied claim | Disposition |
| --- | --- |
| CAD removes all governance/privacy barriers | It can avoid camera capture; access, redistribution and site-sensitive information still need review. No local approval is presumed. |
| CAD is inherently millimetre accurate and immediately executable | Verify units, revision, references and geometry; annotate semantic/topological meaning. No universal conversion-time estimate. |
| Python/SimPy is the optimal project backend | Retain Rust-native Kairos and reusable modules. Optional offline conversion/reference tools require a separate evaluated decision; no Python runtime requirement. |
| PriorityResource implements preemption | Priority and preemption are different; SimPy documents PreemptiveResource separately. Existing Q contract governs consent, victim selection, Suspend/Abort/Restart and safe task boundaries. |
| High acuity automatically evicts any bed/patient | Clinical allocation/interruption policy must be explicit. Staff-task preemption is not automatic physical patient eviction. |
| Walking speed 1.2–1.4 m/s is a universal default | Treat as an unsupported candidate until sourced for actor, mobility, task and context; use labelled synthetic values if needed. |
| Non-homogeneous Poisson arrivals are required | Candidate model only; P3 checks demand structure, overdispersion and fit. |
| Hybrid DES/ABM is intrinsically superior; DES cannot model active agents | Retain minimal justified agent rules and shared state. Choose detail from decision relevance and validation, not the framework label. |
| EHR timestamps uniquely calibrate DES or isolated micro heuristics | Existing C track retains censoring, confounding/identifiability and unclamped holdout gates. |
| Browser Wasm gives zero-copy/no serialization or guaranteed speed | Qualify actual bindings, transfer and runtime benchmarks. No frame-by-frame mutable ECS exposure. |
| A 24-hour run takes 2 seconds; all imports finish in minutes | No timing promise without measured model/input/hardware evidence. |

JaamSim, FlexSim, AnyLogic, SimPy, Ciw and HSMA/vidigi examples can inform reference
fixtures or usability comparisons. They are not additional required engines or
new workstreams. Their full feature/licence comparison was not audited here.
Python ezdxf/Shapely/NetworkX remain optional offline/reference candidates only;
new reusable simulation/spatial modules remain Rust-native. Select a converter
against actual sample files, supported formats and licensing when required.

## Evidence checked

- [ezdxf drawing units](https://ezdxf.readthedocs.io/en/stable/concepts/units.html):
  model-space units and insertion context need explicit interpretation.
- [AnyLogic CAD import](https://www.anylogic.help/anylogic/presentation/cad.html)
  and [pedestrian markup](https://anylogic.help/markup/pedestrian-markup.html):
  drawing import and wall/space markup are separate modeling operations; they do
  not prove a usable Kairos navigation graph can be imported automatically.
- [SimPy shared resources](https://simpy.readthedocs.io/en/4.1.1/topical_guides/resources.html):
  priority queueing and preemption are separate resource behaviors. This pinned
  reference is not a claim about the latest available release.

## Task ownership

P1/P2/P4 own capacity evidence, spatial provenance and the as-operated input shape.
E2 consumes availability/capability policies; C2 owns deterministic transit hooks.
E3 records sufficient versioned state/route data for later playback. E5 starts
with replay and adds CAD import/live transport in bounded later packets. Q/C
already cover queue primitives and calibration; do not create duplicate tracks.
