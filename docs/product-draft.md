# CareOps Sim — Emergency Department

Status: product definition draft for user review. This becomes
`conductor/product.md` after approval.

## Initial concept

Build an emergency department simulation using Kairos and other relevant
libraries from the user's GitHub portfolio. Develop the required Kairos parallel
execution components using a bleeding-edge approach. Kairos and reusable
computation modules must remain Rust-native.

## Vision and users

CareOps Sim helps ED clinicians, operations leads, and analysts compare demand,
staffing, treatment-space capacity, and patient-flow scenarios. It combines a
configurable ED model with an interactive scenario dashboard, reproducible batch
runs, and exportable results.

## Model scope

The initial model covers patient arrivals, triage and acuity, assessment,
diagnostics, treatment, discharge, and admission boarding. Configuration exposes
arrival patterns, patient mix, service-time assumptions, staffing by role and
shift, treatment spaces, diagnostic capacity, routing, and queue priorities.

Inpatient availability initially enters through configurable admission and
boarding constraints. Broader hospital and district models remain extension
opportunities. Specific site parameters and targets are to be supplied or agreed;
an explicitly synthetic example scenario is the proposed starting point.

## User workflows and outputs

Users can edit and save scenarios, run a baseline and alternatives, and compare
waiting times, length of stay, throughput, boarding, queue lengths, and resource
use. Results include distributions and uncertainty across replications, with
breakdowns by acuity and pathway where relevant.

The dashboard and batch interface share the same simulation model and scenario
definitions. Exported results include scenario inputs, units, assumptions,
replication identifiers, seeds, source revisions, and the execution backend so
analysts can reproduce and explain a result.

## Native architecture and library reuse

Kairos provides the simulation engine. ED behaviour belongs in CareOps Sim;
general engine, scheduler, random-stream, and parallel-runtime improvements
belong in the appropriate Kairos crates. Keep library changes independently
reviewable and record their exact submodule commits in this repository.

Follow Kairos's existing specifications, plans, dependency graph, ownership and
acceptance criteria as closely as possible. Reuse its scenario/experiment runner,
telemetry and validation contracts. Implement required missing features in the
owning tracks. Adapt an existing approach only for a demonstrated requirement or
a substantially better alternative, documenting the evidence, compatibility
impact and smallest necessary change. CareOps priorities select work within
these constraints.

The recommended architecture is a Rust-native CareOps Sim library and batch
runner, with Rust-native reusable computation and integration modules. Optional
Python notebooks or thin bindings can support analysis without becoming a
required simulation runtime. Julia and Mojo remain candidates for separately
justified research experiments. Dashboard framework selection belongs to the
technology-stack step.

Candidate portfolio integrations are workforce microcosting, `mchs` for ED
activity funding, `voiage` for decision uncertainty, and `careops-process` for
event analysis. Each needs an explicit input/output contract and compatible
native support. Python-only functionality is a reference or optional external
analysis path until the needed Rust capability is implemented and validated.
Shared methods should be developed in their owning library. Costing, funding,
and decision-value outputs retain their distinct definitions and assumptions.

## Parallel execution development

1. Establish a deterministic serial ED model and deliver local multicore
   scenario/replication execution with the dashboard and exports.
2. Prioritize Apple silicon Metal as the first GPU target through Kairos's
   existing wgpu/WGSL plan and demonstrate actual device execution against the
   CPU reference. Metal work can proceed once its upstream prerequisites are met,
   without waiting for the general distributed runtime. CubeCL is an alternative
   to assess only if an unmet requirement or substantial benefit justifies it.
3. Develop and integrate parallel execution within an individual simulation
   through Kairos PDES, covering event ordering, shared resources, causality,
   random streams, and worker coordination.
4. Implement the planned Rust-native Burn path if an agreed tensor or
   surrogate-model workload needs it, following Kairos's existing ML contracts
   and relevant prerequisite tasks. MLX remains a possible alternative requiring
   a demonstrated need or substantial improvement; its C++/C runtime boundary
   must also be explicitly accepted before adoption.
5. Develop actual distributed MPI and gRPC execution, including worker launch,
   event transport, failures, and multi-process or multi-node evidence.

GPU and distributed support are explicit development phases. Their current
Kairos scaffolds do not establish working runtimes. CUDA and browser WebGPU are
subsequent targets to assess during stack and hardware selection. Extend Kairos's
existing GPU, WebGPU, PDES, distributed, and ML tracks with compatible changes.

Use recent source revisions and evolving capabilities while recording exact
dependency and toolchain versions. Every dependency update must preserve or
explicitly migrate the model contracts and reproducibility guarantees.

## Success criteria

- The same scenario and recorded execution configuration reproduce results.
- Resource ownership, patient accounting, and event timing remain valid across
  the complete patient pathway, including the simulation end condition.
- Dashboard and batch execution agree for identical inputs.
- Multicore scheduling preserves each replication's defined stochastic outcome;
  within-simulation parallelism preserves final observable state parity with the
  serial model under Kairos's PDES contract. Interleaving across logical processes
  may differ.
- GPU and distributed phases demonstrate real backend execution and documented
  parity criteria. Any floating-point tolerances are specified and justified.
- Performance comparisons record workload, hardware, backend, worker counts,
  and elapsed time; speedup is measured rather than assumed.
- Optional adapters pass contract and reference checks for their stated support
  scope. Model calibration and operational validity have their own evidence.

Quantitative runtime, scale, and calibration targets will be agreed during
implementation planning using representative workloads.
