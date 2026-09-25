# Library discovery for CareOps Sim

Discovery date: 2026-09-25.

## Initial concept

The user wants an emergency department simulation using their Kairos library and
other relevant libraries from their GitHub account. They also authorized source
submodules where these enable library development.

## Repository preparation

- Conductor tooling: `extensions/conductor`, release `conductor-v0.4.1`, commit
  `7a5c560a4fdf5297be58594cc37527eb12790272`.
- Kairos source: `libs/kairos`, commit
  `fae901558f07b7b717a676adbafbe2cdc78dea1c` (the observed default-branch HEAD;
  no tags were returned by the remote tag query).
- The project-level `conductor/` path is reserved for this project's context.
- Neither a simulation implementation nor library compatibility has been tested.

Submodule checkout instructions:

```sh
git submodule update --init --recursive
```

The recorded commits fix the source versions. For development within Kairos,
create a branch inside `libs/kairos`, commit upstream changes there, and record the
new submodule commit in the parent repository. Merely editing the submodule does
not record those changes in the parent repository.

## Candidate reuse

| Repository | Observed capability | Proposed role and remaining work |
| --- | --- | --- |
| [kairos](https://github.com/edithatogo/kairos/blob/fae901558f07b7b717a676adbafbe2cdc78dea1c/README.md) | Rust scheduler, world state, random streams, DES resources and ABM APIs; active prerelease with preview bindings | Required simulation engine. Check the selected interfaces and reproducibility before implementation. |
| [microcosting_healthworkforce](https://github.com/edithatogo/microcosting_healthworkforce/blob/main/README.md) | Python labour-cost calculations, productive-time adjustments, rate conversion, projections and sensitivity analysis | Candidate staffing-cost adapter. Assess reusable functions and required spreadsheet inputs; parameter provenance and rates remain to be supplied. |
| [mchs](https://github.com/edithatogo/mchs/blob/master/README.md) | Python NWAU calculator including ED activity; support and parity depend on calculator and year | Candidate funding-output adapter. Choose the applicable year and inputs and check its evidence before use; funding and resource costs are distinct outputs. |
| [voiage](https://github.com/edithatogo/voiage/blob/main/README.md) | Value-of-information analysis with Python orchestration and selected Rust kernels | Candidate decision-analysis adapter once scenarios, uncertainty draws and a decision-value model are defined. |
| [careops-process](https://github.com/edithatogo/careops-process/blob/main/README.md) | OCEL 2.0 contracts and process discovery, delay and conformance analysis | Candidate event-export/analysis adapter. Its documented domain is administrative and governance workflows; an ED domain mapping would need to be designed. |
| [careops-costing](https://github.com/edithatogo/careops-costing/blob/main/README.md) and [hwcc](https://github.com/edithatogo/hwcc) | Workbook migration context and Power Platform artefacts | Possible requirements references; no reusable runtime library established in this review. |

Except for the pinned Kairos code described below, this is repository-documentation
discovery, not an API audit or a successful integration claim. Additional libraries
have not been added as dependencies.

## Kairos implementation observations

At the pinned revision, [`kairo-ecs-des/src/lib.rs`](../libs/kairos/crates/kairo-ecs-des/src/lib.rs)
uses a FIFO `VecDeque` for resource waiting queues. It provides scheduler event
priorities, but this does not implement acuity-based resource allocation for an ED.
The ED model will need an explicit allocation policy.

`DESContext::new(_seed)` does not use its seed argument. Reproducible stochastic
arrivals and service durations therefore require explicit wiring to the random
stream APIs and verification of the resulting model. A constructor seed alone is
not evidence of reproducible stochastic behaviour.

## Existing simulation repositories

The complete default-branch file trees returned by GitHub contained only
`README.md` and `LICENSE` for each of:

- [sim_hospital_emergencydepartment](https://github.com/edithatogo/sim_hospital_emergencydepartment)
- [sim_hospital_wholeofhospital](https://github.com/edithatogo/sim_hospital_wholeofhospital)
- [sim_hospital_wholeofdistrict](https://github.com/edithatogo/sim_hospital_wholeofdistrict)

No reusable implementation was found in those default branches. Other branches
were not examined.

## Approved vision

**CareOps Sim — Emergency Department:** Build a reproducible Kairos-based
simulation of patient arrivals, triage, assessment, treatment, and discharge or
admission. Compare staffing, treatment-space capacity, demand, and patient-flow
scenarios using waiting times, length of stay, throughput, and resource use.
Costing, funding, process analysis, and uncertainty libraries are candidate
optional integrations.

The user approved this vision on 2026-09-25 and selected interactive product
definition. They additionally require a bleeding-edge approach for planned Kairos
components and development of compatible parallel execution components.

The user subsequently selected:

- ED clinicians, operations leads, and analysts comparing capacity and staffing;
- a configurable ED with acuity, staffing, treatment spaces, diagnostics, and
  admission boarding;
- local multicore CPU first, then GPU and distributed backends in explicit phases;
- an interactive scenario dashboard with reproducible batch runs and exports.

They also require Kairos and other reusable modules to remain Rust-native and
requested advice on the implementation language for CareOps Sim. The recommendation
is recorded in [language options](./language-options.md). Python-only capabilities
in the candidate libraries therefore require native development before inclusion
in the reusable runtime, or remain optional external analysis tools.

The [product draft](./product-draft.md) incorporates these choices. Site-specific
parameters, quantitative success targets, the technology stack, and the first
implementation track have not been approved.

The user subsequently requested Apple silicon/MLX/Metal prioritization and reuse
of Kairos's existing plans. The [Apple silicon assessment](./apple-silicon-acceleration.md)
identifies the existing wgpu/Metal and Burn tracks, records the absence of MLX
references at the pinned revision, and updates the product draft to prioritize
Metal as the first GPU target following the CPU baseline.

The user further requires close alignment with Kairos's existing plan, adding
needed implementation and adapting the plan only where necessary or demonstrably
substantially better. The [alignment policy and track map](./kairos-alignment.md)
record this direction. Existing wgpu/WGSL, PDES, runner and relevant Burn plans are
the implementation baseline; alternative frameworks are conditional proposals.

## Parallel execution discovery

Kairos currently contains these components as Rust workspace crates within the
single source submodule. They are not separate Git submodules. No repository split
is required to develop them in place.

| Component | Observed boundary at the pinned revision | Development and compatibility work to plan |
| --- | --- | --- |
| `kairo-ecs-pdes` | Local protocol and deterministic reference fixtures; its validation documentation explicitly excludes core-scheduler integration and actual speedup | Integrate actual parallel execution with the ED event model; check causality, deterministic random streams, queue/resource ownership, and parity across worker counts. |
| `kairo-ecs-mpi` | Dependency-free placeholder transport and protocol contracts; no MPI runtime dependency | Implement real rank/worker transport, launch, event exchange and failure behaviour; execute multi-process tests on a supported runtime. |
| `kairo-ecs-grpc` | Placeholder transport and local protocol proofs; no `tonic` or generated protobuf runtime dependency | Implement real coordinator/worker services, serialization, networking and failure handling; verify multi-node execution. |
| `kairo-ecs-gpu` | Explicit unavailable contracts for wgpu and CUDA; a separate CPU fallback exists for local validation | Implement actual device backends and suitable kernels; verify device execution, numerical behaviour and measured performance. |
| `kairo-ecs-webgpu` | Feature-gated crate and shader/contract surfaces | Audit browser runtime integration and establish real browser/device evidence before treating it as supported. |

Primary local sources:

- [PDES validation boundary](../libs/kairos/docs/pdes/validation-evidence.md)
- [Distributed runtime blockers](../libs/kairos/docs/distributed/deployment-guide.md)
- [GPU backend availability](../libs/kairos/docs/gpu-compute/backend-selection.md)
- [PDES manifest](../libs/kairos/crates/kairo-ecs-pdes/Cargo.toml)
- [MPI manifest](../libs/kairos/crates/kairo-ecs-mpi/Cargo.toml)
- [gRPC manifest](../libs/kairos/crates/kairo-ecs-grpc/Cargo.toml)
- [GPU manifest](../libs/kairos/crates/kairo-ecs-gpu/Cargo.toml)
- [WebGPU manifest](../libs/kairos/crates/kairo-ecs-webgpu/Cargo.toml)

Proposed interpretation of bleeding-edge development: work against recent,
explicitly recorded source revisions and evaluate new runtime capabilities through
named development phases. Record revisions, toolchains, features and execution
backends with results so experiments remain reproducible. Refreshing source pins
must include compatibility evidence for the ED model and selected adapters.

Independent scenario/replication parallelism and parallel execution within a
single simulated ED are different capabilities. Both need explicit requirements;
neither proves the other. A backend must report what actually executed, and
measured hardware/runtime evidence must support any performance claim.

Runtime tests and performance measurements have not been run as part of this
discovery. These are implementation requirements to resolve in the agreed plan.
