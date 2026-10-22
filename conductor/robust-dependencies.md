# Robustness-first dependency programme

Approved planning direction, 22 October 2026. No dependency installed and no runtime acceptance claimed. [Value policy](dependency-value-policy.md) is mandatory. The primary objective is robust existing behaviour and less bespoke code, not dependency count or visual novelty.

## Ownership and dependency register

One canonical track per dependency lives in its owning repository. Kairos owns generic Rust implementation; CareOps consumes accepted APIs under bounded parent packets and owns site/clinical/fleet/application semantics. No parallel implementation is created merely to mirror the upstream registry. Parent manual receipts are recorded in `conductor/dependency-handoffs.json` in CareOps. Upstream IDs below are local registry dependencies; external receipts remain separately verified.

| Dependency | Canonical owner record | Class | Prerequisites |
| --- | --- | --- | --- |
| serde | Kairos Track 74 (`74-serde-robustness`) | foundation | 70 |
| serde_json | Kairos Track 75 (`75-serde-json-robustness`) | foundation | 74 |
| toml | Kairos Track 76 (`76-toml-robustness`) | foundation | 75 |
| clap | Kairos Track 77 (`77-clap-robustness`) | foundation | 76 |
| rayon | Kairos Track 78 (`78-rayon-robustness`) | foundation | 72, 77 |
| zip | Kairos Track 79 (`79-zip-robustness`) | subsystem | 78 |
| quick-xml | Kairos Track 80 (`80-quick-xml-robustness`) | subsystem | 79 |
| prost | Kairos Track 81 (`81-prost-robustness`) | conditional | 78 |
| tokio | Kairos Track 82 (`82-tokio-robustness`) | conditional | 81 |
| tonic | Kairos Track 83 (`83-tonic-robustness`) | conditional | 82 |
| hecs | Kairos Track 84 (`84-hecs-robustness`) | evaluation | 78, 64 |
| clarabel | Kairos Track 85 (`85-clarabel-robustness`) | conditional | 78 |
| rstar | Kairos Track 86 (`86-rstar-robustness`) | evaluation | 87 |
| geo | Kairos Track 87 (`87-geo-robustness`) | conditional | 73 |
| mpi | Kairos Track 88 (`88-mpi-robustness`) | conditional | 78 |
| tract | Kairos Track 89 (`89-tract-robustness`) | conditional | 78 |
| argmin | Kairos Track 90 (`90-argmin-robustness`) | conditional | 78 |
| fmi | Kairos Track 91 (`91-fmi-robustness`) | evaluation | 78 |
| sobol-burley | Kairos Track 92 (`92-sobol-burley-robustness`) | evaluation | 72 |
| Valhalla | [Track](tracks/valhalla_transport_20261022/index.md) | Conditional, later | petgraph_spatial_routing_20261022:R31 |
| MapLibre GL JS | [Track](tracks/maplibre_regional_viewer_20261022/index.md) | Conditional, later | valhalla_transport_20261022:R42 + Three.js R35 |
| SUMO | [Track](tracks/sumo_traffic_qualification_20261022/index.md) | Conditional, later | valhalla_transport_20261022:R42 |
| sha2 | Kairos 93 | foundation | none; before 70 |
| tempfile | Kairos 94 | foundation | none; before 70 |
| thiserror | Kairos 95 | foundation | 77; before 71 |
| proptest | Kairos 96 | foundation expansion | none; before 70 |
| tracing | Kairos 97 | conditional | 78 |
| loom | Kairos 98 | evaluation | 78 |
| proptest-state-machine | Kairos 99 | evaluation | 96 |
| tracing-subscriber | Kairos 100 | conditional | 97 |

## Immediate serial priority

Kairos 93/94/96 foundations → both accepted Polars migrations → Kairos 74 Serde → 75 serde_json → 76 TOML → 77 Clap → Kairos 95 / CareOps thiserror → existing Kairos 71/CareOps rand → existing Kairos 72/CareOps rand_distr → Kairos 78 Rayon plus parent consumer verification → existing ECharts → petgraph → Three.js.

This is scheduling priority, not a claim every library technically depends on its predecessor. Polars already owns table processing; reconcile eliminated targets rather than rebuilding them. Track 74 entry checks CareOps R14; Track 78 entry checks CareOps R25; ECharts R26.1 checks Track 78. External gates cannot be proven by either local DAG alone. Polars now has bounded integrity/tempfile/property prerequisites; see the sequencing register.

## Bounded secondary work

FMI: assess Track 91 scope before executing 79/80; choose whole-adapter replacement or bounded ZIP/XML replacements without duplicated production infrastructure. Scope assessment is not automatic adoption of the smaller rust-fmi ecosystem. These do not block core robustness/chart sequence.

RPC: actual distributed consumer → 81 Prost → 82 Tokio → 83 Tonic. Keep PDES causality/authority in Kairos; socket names do not prove RPC/MPI interoperability. MPI 88, solver 85, inference 89, optimiser 90 and QMC 92 remain consumer/evidence-gated. hecs 84 and rstar 86 remain retain-versus-replace evaluations. No benchmarks or code savings are assumed.

Geography: accepted petgraph and concrete road-transfer need → 87 geo → CareOps Valhalla R40–R42. Regional viewer additionally waits for accepted Three.js → MapLibre R43–R45. SUMO R46–R48 waits for road-model qualification and an explicit traffic-interaction need; offline scenarios first, live coupling only after additional evidence. If existing travel-time matrices suffice, defer the road stack.

## Deferred and alternative choices

Rapier: deferred, no executable adoption track or dependency. Reconsider only for an accepted collision/vehicle-dynamics problem not answered by road routing/traffic simulation.

Bevy ECS, OSRM and ort are alternatives within hecs, Valhalla and Tract qualification, respectively; do not install both alternatives or create a backend framework. A/B Street is a research reference, not a selected dependency. DataFusion/WebGPU remain deferred. No new track for DES/resource replacement.

## Transport authority

CareOps owns hospital entrance/bay mapping, fleet/crews, mobilisation/loading/handover/turnaround, dispatch policy and calibrated site inputs. Kairos owns generic time/events/resources/geometry and reusable movement contracts only. In route mode Kairos owns progress/arrival; in live SUMO mode SUMO owns movement and reports authoritative arrival through an explicit step contract. Never have both schedule the same arrival. Viewer interpolation changes no outcomes. Road arrival, bay arrival and completed handover are distinct.

Pin route/map/traffic versions and hashes; source-qualified geographic data and calibrated delays are required. Freeze observed and simulated distinctions. Current transport prose contains illustrative speed/range values; do not promote them to validated defaults. MapLibre/Valhalla/SUMO are explicit non-Rust application/service exceptions, not dependencies of the core.

## Execution status

All new tracks are planned, with adoption_decision pending. Runtime adoption requires benefit, supported contracts and independent tests; defer/reject is a valid evaluation outcome and must not be marked implemented. Planning review/validation: [report](robust-dependencies-review.md). Current registrations do not satisfy prerequisites or release gates.

## Integrity and qualification refinement

[Dependency sequencing](dependency-sequencing.md) is the current complete ordering. New canonical Kairos tracks: 93 sha2; 94 tempfile; 95 thiserror; 96 proptest; conditional 97 tracing, 98 Loom, 99 proptest-state-machine and 100 tracing-subscriber. Parent direct-use tracks cover sha2/tempfile/thiserror/proptest. No new dependency installed.

## Schema validation, existing tooling and conditional capabilities

Refinement approved 22 October 2026; no runtime adoption.

| Work | Kind | Owner and order |
| --- | --- | --- |
| Criterion 101 | Consolidate existing tooling | Kairos benchmark owner; no core prerequisite, then CareOps R68–R70 for actual microbenchmark consumers |
| Rust jsonschema | New dependency qualification | CareOps R62–R64 after parent Polars R14 and upstream serde_json 75; shared schema contract before browser qualification |
| Ajv | New dependency qualification | CareOps R65–R67 after schema R64; gates ECharts implementation R27 and affected live input acceptance |
| ode_solvers 102 | Conditional scientific capability | Kairos after Rayon 78 and demonstrated continuous-dynamics need; independent of correcting existing RK4 claim |
| Tungstenite 103 | Conditional real transport | Kairos after Rayon 78 and accepted live consumer; parent schemas before affected input acceptance |
| tokio-tungstenite 104 | Conditional async adapter | Only after Tungstenite 103 and accepted Tokio 82; real async requirement, no automatic second transport |

Core sequence otherwise remains intact. Rust schema qualification can run once parent Polars and upstream JSON contracts are accepted, while TOML/Clap/error/RNG/sampling/Rayon work proceeds in its existing order. ECharts R26 can prepare; R27 waits for Ajv R67. The existing Python jsonschema validator remains an independent oracle. No new schema validator is added to Kairos without an actual generic consumer. Shared schemas cover structure; authoritative clinical and cross-record decisions remain native domain logic.

Benchmark consolidation is supporting work, not a blanket migration blocker. Each implementation track must still use an adequate baseline and its existing cost gates; do not replace specialist subprocess/GPU/MPI/RSS measurements with microbenchmarks.

Track 65 owns the open Euler/RK4 mismatch independently of optional ODE selection. Do not revive stale numerical capability claims from names or self-roundtrip tests. Track 103 replaces an in-memory placeholder only when real transport is required; application ordering/resync/idempotency/backpressure and cancellation remain project-owned. Any live parent packet must verify accepted ABI schemas and actual browser/native transport, not simply the ECharts completion status.

No ajv-cli, additional format plugin, transport framework or numerical backend is implicitly approved. Separate direct-dependency tracks are required if qualification demonstrates a need. SUMO remains conditional; Rapier remains deferred.
