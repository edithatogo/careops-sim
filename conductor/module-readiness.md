# Kairos module readiness and ED delivery audit

Audit: 2026-09-25, Kairos `fae901558f07b7b717a676adbafbe2cdc78dea1c`.
The two original plans are necessary but insufficient for a completed ED library.
They omitted explicit ED pathway delivery, release qualification and several
cross-cutting development gates. Development-readiness (D) and generic-ED delivery
(E) now close that coverage gap. All later clinical domains remain roadmap-only.

## Completion levels

| Gate | Meaning | Required milestones |
| --- | --- | --- |
| G0: development ready | Reproducible Rust environment, contracts, capability baseline, evaluated agent harness and usable CI | D0–D2; Q0/C0/E0 reviewed |
| G1: native ED library complete | Generic ED model, headless API/runner, real telemetry, calibration, multicore replications, reproducible package with docs and release checks | Q5, C6, E4, D3/D4 |
| G2: later interactive product | G1 plus interactive scenario dashboard, exports and native/browser contract tests | E5, D4 release profile evidence |
| G3: accelerated/parallel profiles | Actual Metal, then within-run PDES, then distributed acceptance; measured correctness and performance | E6/E7/E8 separately; no all-or-nothing backend claim |

A completed native library does not imply every Kairos ecosystem module has been
implemented. Every module is classified below. Deferred capabilities must remain
explicitly unavailable; never pass an availability check through a CPU mock.

## Working MVP before full qualification

E1's runnable pathway and E2's integrated headless fixture provide the first useful
model increments. E2 MVP acceptance requires a documented run with configurable
beds/staff/locations, deterministic outputs, conserved patients/capacity and basic
wait/throughput/occupancy summaries. It is not G1 release/calibration completion.
E3 improves API/exports/recovery; C6/D4/E4 establish G1 qualification. Visualization,
CAD/capture, WebSocket UI service and accelerated backends are later work.

## Crate coverage and evidence required

Paths below are relative to `libs/kairos/crates/`. Source/manifest inspection
establishes current boundaries; only the five-package test slice noted below ran.

| Module(s) | Current inspected boundary | Required work / owner | Delivery gate |
| --- | --- | --- | --- |
| kairo-ecs-types | Tick time and generational IDs | Versioned handles/errors, precision/overflow rules; 01, Q0/C0/E0 | G0/G1 |
| kairo-ecs-core | Deterministic scheduler; raw schedule permits past events | Checked Flow admission, completion boundary tests, actual scheduler snapshot/restore; 01, Q1/Q4 | G1 |
| kairo-ecs-state | World and separate type-erased registry | Unified runtime, context codecs, despawn/lease invariants; 01/03, Q1/Q4 | G1 |
| kairo-ecs-rng | Explicit deterministic streams | Versioned task/purpose keys, distribution sampling, stable patient identities; 01, C2/E1 | G1 |
| kairo-ecs-des | Legacy FIFO resources and trajectories | Full queue/preemption/Flow track Q0–Q5; preserve legacy API; 03 | G1 |
| kairo-ecs-abm | Behavior loop with separate context | Shared DES/ABM runtime and minimal staff policy/transit; 03, Q4/C2 | G1 |
| kairo-ecs-arrow | Schema and smoke bytes, no Arrow deps | Real IPC/Parquet, versioned sidecars, bounded IO; 04, C1/C4 | G1 |
| kairo-ecs-cli | Basic manifests/replay surfaces | Real ED execution, studies, interruption recovery, cancel/progress/errors and worker runs; 22, C5/E3 | G1 |
| kairo-ecs-calibration (proposed) | Not present | C0 ADR then C1–C6; reusable Rust algorithms owned by 21 | G1 |
| kairo-ecs-bench | Existing benchmark/fixture integration | Representative ED/resource/Arrow/calibration workloads and thresholds; 12/18/31, Q5/C6/D3 | G1 |
| kairo-ecs-debug | Trace snapshots/deltas; not proof of full ECS serialization | Readable failure traces and replay integration; full interactive time travel optional; 40 with 01/22, Q4/E3 | G1 diagnostic subset |
| kairo-ecs-viz | Headless snapshot facade; native renderer unavailable | ED snapshot/delta adapter, backpressure and sampling contracts; 05, E5 | G2 |
| kairo-ecs-ffi | Existing core bridge, not new Flow API | Audit ownership/panic/tick widths and expose required ED surface; 02, E5 | G2 if used by selected Wasm route |
| kairo-ecs-wasm | Existing limited FFI-backed engine | ED API and u128/ID mapping, worker lifecycle and native parity; 09, E5 | G2 |
| kairo-ecs-uniffi | Bridge wrapper | Maintain regression compatibility; full ED expansion deferred to real consumer; 02 | Deferred expansion |
| kairo-ecs-diplomat | Bridge wrapper | Same protected ABI review; no redundant ED API rewrite; 02 | Deferred expansion |
| kairo-ecs-cs-bridge | FFI bridge | Preserve existing gates if shared ABI changes; no new C# ED work; 10 | Deferred expansion |
| kairo-ecs-gpu | Empty backend dependency features / explicit unavailable contracts | Real wgpu/WGSL Metal dispatch, CPU oracle and device benchmarks; 32, E6 | G3-Metal |
| kairo-ecs-webgpu | Browser contract/scaffold | Real browser device setup and parity through existing 33; no assumption that Pixi rendering is simulation compute | Deferred after native Metal |
| kairo-ecs-pdes | Conservative scheduler/in-memory reference scaffolding | Real core/Flow LP integration, resource ownership/lookahead/GVT, threaded evidence; 34, E7 | G3-PDES |
| kairo-ecs-mpi | Local protocol surface; no MPI runtime dependency | Real MPI transport, rank launch, migration and failure evidence; 35, E8 | G3-distributed |
| kairo-ecs-grpc | Local protocol surface; no tonic/prost runtime | Real services/auth/protocol, recovery and at-most-once migration; 35, E8 | G3-distributed |
| kairo-ecs-streaming | Optional Kafka/NATS/Flight feature names, no runtime deps | Batch Arrow sufficient initially; real live ingestion needs separate source/wall-clock contract; 36 | Deferred |
| kairo-ecs-ml | Optional backend feature names, no runtime deps | Burn remains existing direction; learned policies/surrogates need measured value and validation; 37 | Deferred |
| kairo-ecs-fmi | Optional co-simulation features, no runtime deps | Preserve extension contracts, implement only for concrete future model need; 38 | Deferred |

Python/R/Julia/Go/C# binding packages remain in existing owner tracks 06–11.
A Python binding crate is listed as excluded in the upstream workspace; do not
interpret that as a required implemented ED dependency. Run affected legacy
binding compatibility gates when shared interfaces change, without requiring new
full ED APIs in every language before G1. All reusable new computation stays Rust.

## Non-crate dependency closure

| Existing upstream tracks | ED obligation |
| --- | --- |
| 00, 19, 26 | Naming/licensing/citation and interoperability decisions; D0/E0; retain seed/schema provenance |
| 12, 18, 21, 31 | Analytic/differential/property tests, validation/uncertainty, representative performance and regression thresholds; Q/C/D3/E4 |
| 13, 20, 27, 30, 44 | CI, supply-chain/security, reproducible bootstrap, toolchain policy and real health gates; D0–D5 |
| 14, 17, 23, 24, 41, 45 | Executable generic-ED examples, API docs, Astro/Starlight integration where relevant, dashboard usability; E0/E4/E5 |
| 15, 16, 25, 28, 29, 42 | Compatibility, independent review, packaging and provenance; D4/E4; retain existing global publication holds |
| 39, 43 | Cloud/HPC deployment only after real distributed runtime; E8 handoff; not required for local G1/G2 |

Track 03/04 dependencies on 26 and all upstream dependency edges remain in force.
The scoped ED release evidence does not waive Kairos's broader package publication
requirements or mark global upstream tracks Done. D0 produces a machine-readable
capability/owner/gate inventory and resolves dependency/status inconsistencies.

## Concrete readiness gaps

- Parent repo had no AGENTS map, persistent context check, CI, Rust project or
  remote. Local map/check harness added now; buildable ED code and hosted CI await D/E.
- Upstream `rust-toolchain.toml` and mise use floating channels; MSRV=1.76 conflicts
  with current Arrow 60 MSRV=1.88. D1/C0 require a scoped compatibility decision.
- Upstream ci-core uses older pinned cargo tools, ci-policy installs others without
  versions; `cargo deny check advisories sources` does not run its license/bans
  policy. D2 aligns versions and runs intended policy categories.
- CodeQL matrix is JavaScript-only; assess and enable current Rust support in D3.
- Fuzz workflow installs stable and invokes cargo-fuzz; D3 selects a dated tested
  nightly lane for supported sanitizer/fuzz execution, with actual run evidence.
- Local developer validation currently requires unrelated polyglot runtimes.
  D1 adds a minimal native-ED profile while preserving broader upstream commands.
- Full ECS/scheduler/context checkpointing is an implementation dependency, not
  established by existing trace-debug snapshots. Q4 owns this with 01/22.
- GPU/PDES/network smoke labels do not prove device/threaded/multi-node execution.
  E6–E8 require actual hardware/runtime evidence and retain existing thresholds.

## Baseline verification

On 2026-09-25, Apple silicon macOS, rustc/cargo 1.98.1:

```sh
cd libs/kairos
cargo test --locked --offline -p kairo-ecs-core -p kairo-ecs-state -p kairo-ecs-rng -p kairo-ecs-des -p kairo-ecs-abm
```

Result: **60 tests passed**, no failures. This is existing baseline behavior,
not the new queue/calibration/ED acceptance suite. Submodule source and pin remain
unchanged. See [development evidence](evidence/development-audit.md).


## Parameter and input-evidence prerequisite (2026-09-27)

The [dedicated parameter track](tracks/ed_parameter_evidence_20260927/plan.md)
adds domain input completeness: DES/ABM taxonomy, provenance, ranges, distributions,
dependence, uncertainty and generic examples. P0 gates E0/C0, P3 gates C2, P4
gates E1 and P5 gates C6. G1 therefore includes actual model-to-catalogue coverage
and reproducible input examples. This does not introduce a later clinical domain.

## D0.2 pinned-source reconciliation (2026-09-27)

The D0.2 source audit is bound to parent commit `92ab172028bc9e3af1ede9a15cc254a12cddbc68`
and the exact Kairos submodule pin and checkout `fae901558f07b7b717a676adbafbe2cdc78dea1c`.
The local Kairos commit object is dated 2026-05-20 (+10:00), with parent
`f11e0be7dcf8cad705741a44b651ca953e269ae2`; report 10's alternative date and
parent do not match this object. The parent pin is the inspected commit itself,
not its Git parent.

The source-derived 24-manifest inventory, source-backed 25-row capability matrix, prerequisite/identity audit, and
research-claim reconciliations are retained in
[the manifest inventory](evidence/d0.2-kairos-manifest-inventory-20260927.json),
[the capability inventory](evidence/d0.2-kairos-capability-inventory-20260927.json),
[the prerequisite and identity audit](evidence/d0.2-prerequisite-identity-audit-20260927.json),
[the reports 10/11 audit](evidence/d0.2-research-audit-reports-10-11-20260927.md),
and [the reports 26/28 audit](evidence/d0.2-research-audit-reports-26-28-20260927.md).
The capability table has 25 planned/readiness rows while the checkout has 24
crate manifests; `kairo-ecs-calibration` is a proposed module without a manifest.
This is an intentional planning-vs-source distinction, not an inventory error.

Kairos source supports a deterministic serial scheduler and a limited FIFO
resource helper. It does not establish first-class priority/deadline/preemption
queues, integrated Flow/PDES, hardware acceleration, production Arrow IO, a
worker-launching agent harness, current hosted CI success, benchmark results, or
Luna qualification. Reports' commands remain unexecuted research material.
Report 26's wgpu MSRV claim also conflicts with the parent candidate snapshot and
remains for D1.2 source/registry verification. These source gaps and external
claims stay assigned to their existing D/Q/C/E gates.

The local license declarations conflict: Cargo metadata and README say
Apache-2.0, while `LICENSE` states Apache-2.0 and MIT. This audit records the
discrepancy without determining package-level legal intent. Repository URL,
public visibility, CI state, Pages publication, and release claims are local
source declarations only unless verified through their live authoritative
surfaces.

## Authoritative delivery sequence

[Functional MVP → hardened native v1 → extensions](delivery-contract.md) defines the
required features and acceptance recipes. E2 is the functional headless MVP; E4
is native v1. Visual/spatial import, live UI and advanced backends remain post-v1.

## ED-native support profile (D0.3)

[ADR-0001](decisions/ADR-0001-ed-native-support-profile.md) defines the staged
CareOps Sim release boundary. Its [machine-readable profile](evidence/d0.3-ed-native-profile-20260927.json)
classifies all 25 capabilities inventoried at the pinned Kairos revision. At the
module level: MVP uses `kairo-ecs-types`, `core`, `state`, `rng`, `des`, `abm`,
and `cli`; native v1 adds `arrow`, proposed `calibration`, `bench`, and the
diagnostic subset of `debug`; all remaining modules are post-v1. The JSON has the
complete exact module names, rationale, owners, source state and gate mapping.

The intended ED targets are Linux x86_64 and macOS aarch64, both currently
unqualified. D1.1/D1.2 must settle the exact toolchain and consumer MSRV; D2/D4
must establish platform CI and clean-consumer evidence before either can be
claimed as supported. This ED-scoped profile does not modify Kairos Track 30's
global support matrix or infer support from the local Mac. Track 25's root-specific
API/schema review applies to each affected public change; unused bindings and
FFI are impact-assessed as not exposed, not implicitly approved. Local simulation
outputs and later dry-runs do not lift Kairos global publication holds.

The profile was checked against pinned Tracks 25/30 and Kairos package/release
hold records. That source review is not a direct maintainer approval. The
[independent D0.3 review receipt](evidence/d0.3-execution-receipt-20260928.md)
records the precise status and open owner-alignment work; it must not be
represented as upstream approval.
