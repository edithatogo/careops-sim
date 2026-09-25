# Alignment with the existing Kairos plan

User direction recorded on 2026-09-25: align as closely as possible with Kairos's
existing plan, implement the needed aspects, and adapt it only when necessary or
when a proposed approach is demonstrably substantially better.

Reviewed baseline: `fae901558f07b7b717a676adbafbe2cdc78dea1c`.

## Implementation rule

Reuse existing APIs, crates, schemas, track tasks and acceptance criteria first.
Implement missing capabilities in their owning Kairos track. Keep ED-specific
pathways, resource policies, scenarios and dashboard behaviour in CareOps Sim.
Map each implementation task to its upstream track and the specific unmet need.

CareOps delivery priorities select dependency-ready portions of the Kairos plan;
they do not replace its dependency graph or reopen unrelated completed work.
Work through the prerequisites in the registry, specification, plan and handoff.
Resolve conflicts between those sources before relying on the affected contract.

## Work mapping

| CareOps requirement | Existing Kairos owner | Alignment requirement |
| --- | --- | --- |
| Scheduler, state and reproducible randomness | [Track 01](../libs/kairos/conductor/tracks/01-heart-kairo-ecs-core-state/plan.md) | Preserve tick time, identifiers, ordering and RNG contracts; keep ED-specific logic outside the core. |
| Patient trajectories and resource queues | [Track 03](../libs/kairos/conductor/tracks/03-flow-des-trajectory-abm-behavior/plan.md) | Use existing DES/ABM surfaces. Add general queue features here only when required; configure ED triage policy in CareOps. |
| Events, results and exports | [Track 04](../libs/kairos/conductor/tracks/04-analyst-kairo-ecs-arrow/plan.md) | Reuse versioned telemetry contracts and extend compatible schemas as needed. |
| Batch runs, scenarios and replications | [Track 22](../libs/kairos/conductor/tracks/22-experiment-runner-scenario-management/plan.md) | Extend its runner and manifest work for real execution; avoid creating competing seed, replay or scenario formats. |
| Replay, uncertainty and validation | [Track 21](../libs/kairos/conductor/tracks/21-verification-validation-uncertainty/plan.md) and [Track 12](../libs/kairos/conductor/tracks/12-conformance-testing-benchmarks/plan.md) | Use existing fixture and benchmark ownership; add ED fixtures and domain-validation evidence. |
| Apple silicon acceleration | [Track 32](../libs/kairos/conductor/tracks/32-gpu-compute-acceleration/plan.md) | Implement the planned wgpu/Metal backend and WGSL route through `GpuCompute`, respecting core and bridge contracts. Prioritize Metal hardware evidence within this scope. |
| Browser acceleration, when needed | [Track 33](../libs/kairos/conductor/tracks/33-webgpu-compute-browser/plan.md) and [Track 09](../libs/kairos/conductor/tracks/09-typescript-wasm-binding/plan.md) | Preserve the dependency on native GPU work and Wasm bindings; reuse compatible kernel designs. |
| Parallelism within one ED run | [Track 34](../libs/kairos/conductor/tracks/34-pdes-parallel-execution/plan.md) | Implement the planned conservative PDES, LogicalProcess, channel, lookahead and GVT work; retain the sequential scheduler as oracle. |
| MPI and gRPC | [Track 35](../libs/kairos/conductor/tracks/35-distributed-simulation-mpi-grpc/plan.md) | Build real transports on Track 34's LP model; preserve its scheduler and Track 04's telemetry contracts. |
| Learned surrogates, when justified | [Track 37](../libs/kairos/conductor/tracks/37-ml-ai-integration-inference/plan.md) | Use its inference contracts and planned Burn backend; satisfy the relevant preceding contract and model-validation work. |
| Deployment runners, when needed | [Track 39](../libs/kairos/conductor/tracks/39-cloud-hpc-batch-runners/plan.md) | Keep deployment orchestration in this track, distinct from Track 35 transport implementation. |
| Public APIs and toolchains | [Track 25](../libs/kairos/conductor/tracks/25-api-design-review-compatibility-governance/plan.md) and [Track 30](../libs/kairos/conductor/tracks/30-toolchain-version-support-matrix/plan.md) | Follow protected-surface review, supported-version policy and migration requirements. |

## Specific decisions

- **Metal priority:** implement the existing wgpu/WGSL route first. Selecting
  Apple hardware for initial device validation fits Track 32's documented scope.
- **CubeCL:** a possible alternative or a dependency of the chosen Burn version,
  not a new mandatory workstream. A standalone kernel-framework change requires
  evidence that the existing route cannot meet a requirement or that the benefit
  substantially exceeds integration and maintenance costs.
- **MLX:** an optional future proposal requiring the same evidence, plus a clear
  decision about its C++/C runtime boundary under the Rust-native requirement.
- **PDES correctness:** preserve final observable state parity with the serial
  oracle. Track 34 permits different per-tick interleaving across LPs; do not
  silently strengthen it into an identical global event-trace promise.
- **Performance:** preserve upstream acceptance targets and representative
  workloads. Add ED-specific measurements; any change to an upstream target
  needs documented justification rather than silently lowering the threshold.
- **Scope of upstream work:** implement the missing portions needed for agreed
  ED, parallel and acceleration milestones. A scaffold or a `Done` metadata label
  is insufficient evidence of the runtime capability needed by CareOps.

## Evidence required for an adaptation

Record the existing track/task, the concrete unmet requirement, why the current
approach is inadequate, and the smallest compatible change. For a proposed
improvement, compare correctness, representative performance, portability,
maintenance and dependency costs. State the compatibility and migration impact,
tests and benchmarks, and affected owning tracks. Apply Kairos's existing
architecture-decision and protected-surface rules where applicable.

Update the owning plan, handoff and relevant registry/closeout records with the
implementation evidence. Keep Kairos changes in separate commits within its
submodule, then record the compatible commit in CareOps Sim. Preserve existing
required gates and distinguish local proof from hardware or deployment evidence.

## Source inconsistencies to resolve during implementation

- Track 34 references `docs/core-contract.md`, which is absent at this revision.
  The existing contract is
  [conductor/contracts/core-contract.md](../libs/kairos/conductor/contracts/core-contract.md),
  consistent with the
  [PDES sequential baseline](../libs/kairos/docs/pdes/sequential-determinism-baseline.md):
  order by time, priority and insertion sequence.
- Track 32's additive GPU reference uses a different fixture ordering. Preserve
  the core ordering contract when developing actual ED event dispatch; the
  fixture does not authorize a different general scheduling rule.
- Track 22 references `conductor/contracts/vvuq-contract.md`, which is absent at
  this revision. Review Track 21's actual replay/seed documents and fixtures,
  then repair the missing cross-track reference or contract in its owning work.
- The readiness narrative, track registry and handoffs describe different levels
  of completion. Use the registry for declared dependencies/status and actual
  code, handoffs and validation evidence to establish capability readiness.

These are recorded discovery findings. This setup work does not change upstream
plans or implementation and does not mark upstream tasks complete.
