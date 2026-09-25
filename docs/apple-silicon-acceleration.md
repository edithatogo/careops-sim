# Apple silicon acceleration recommendation

Reviewed on 2026-09-25 against Kairos commit
`fae901558f07b7b717a676adbafbe2cdc78dea1c`. The remote HEAD still matched this
revision during the review. This is an implementation recommendation for setup;
no hardware execution or performance benchmark has been performed.

The user's subsequent alignment direction takes precedence over exploratory
framework suggestions below: follow Kairos's existing wgpu/WGSL and relevant Burn
plans, implementing needed gaps. Adopt alternatives only when necessary or when
evidence establishes a substantial improvement. See
[Kairos alignment](./kairos-alignment.md) for task ownership and change criteria.

## Existing Kairos plans

| Existing surface | Relevant plan | Current implementation boundary |
| --- | --- | --- |
| [Track 32: GPU compute](../libs/kairos/conductor/tracks/32-gpu-compute-acceleration/plan.md) | `wgpu` for Metal/Vulkan/DX12, CUDA separately; shared kernel design, reproducible RNG, buffer ownership, CPU/GPU parity and benchmarks | `kairo-ecs-gpu` has a facade and reference fixtures. Its wgpu backend reports unavailable and has no real wgpu dependency. |
| [Track 33: browser WebGPU](../libs/kairos/conductor/tracks/33-webgpu-compute-browser/plan.md) | Share kernel designs with native GPU compute; integrate Wasm and browser device execution | Browser wiring remains unconfigured; API detection and CPU animation do not prove GPU execution. |
| [Track 37: ML inference](../libs/kairos/conductor/tracks/37-ml-ai-integration-inference/plan.md) | A native Burn backend and neural surrogate interface, alongside optional ONNX/TensorRT backends | Burn is a scaffold alias, with no real inference backend configured. |
| [Track 34: PDES](../libs/kairos/conductor/tracks/34-pdes-parallel-execution/plan.md) | Partitioned simulation and event exchange | Local protocol fixtures do not establish integrated parallel ED execution. |
| [Track 35: distributed execution](../libs/kairos/conductor/tracks/35-distributed-simulation-mpi-grpc/plan.md) | MPI and gRPC workers and transport | Real runtime transports remain to be developed. |

A source search for `mlx`, `mlx-rs`, and `mlx-c` found no matches in the checked
Kairos revision. Metal and Burn are existing planned directions; MLX would add a
new integration decision.

## Framework choice

Metal is Apple's GPU API. MLX is an array and machine-learning framework with
Apple GPU support. The recommendation is to prioritize actual Metal execution
through the existing Rust GPU abstraction and evaluate higher-level frameworks
for the workloads that benefit from them.

| Candidate | Proposed role | Fit and tradeoff |
| --- | --- | --- |
| [wgpu](https://github.com/gfx-rs/wgpu) | First native GPU backend, selecting Metal on Apple silicon | Native Rust API with Metal support; matches Kairos Track 32 and its existing WGSL shader direction. |
| [CubeCL](https://github.com/tracel-ai/cubecl) | Conditional alternative for Rust-authored compute kernels | Its Rust kernel language targets Metal, WebGPU and other backends. Direct adoption requires a demonstrated need or substantial benefit over Kairos's planned wgpu/WGSL route; it may also be supplied by the selected Burn backend. |
| [Burn](https://github.com/tracel-ai/burn) | Optional tensor operations and learned surrogates | Matches Track 37's Rust-native direction. Its accelerated backends use CubeCL, including Metal. Only introduce it when an agreed ED analytical or surrogate workload needs it. |
| [MLX](https://github.com/ml-explore/mlx) with [mlx-rs](https://github.com/oxiglade/mlx-rs) | Comparative experiment for tensor-heavy or surrogate workloads | MLX provides Python, C++, C and Swift APIs. The unofficial Rust wrapper uses MLX through its native binding layer; this preserves Rust application code but adds a C++/C runtime dependency. It is not a pure-Rust computation stack. |

These are supported directions in the frameworks' own documentation. They have
not been integrated or benchmarked with Kairos. A new framework should meet a
specific requirement without duplicating the existing backend abstraction.

## Proposed delivery order

1. Establish the serial ED reference and local multicore replication execution.
   Give random streams stable scenario/replication/entity identities so worker
   scheduling cannot change the defined experiment.
2. Make Apple silicon Metal the first real GPU implementation target. Extend
   Track 32's existing `GpuCompute` surface, choose a small representative
   parallel workload, and demonstrate device creation, dispatch and readback.
   Follow wgpu/WGSL first. Compare CubeCL only for an identified shortcoming or
   evidence-backed substantial improvement. CUDA is a subsequent target.
3. Develop within-simulation PDES and integrate it with the model and optional
   acceleration. Metal implementation need not wait for a general distributed
   scheduler; CPU reference semantics remain the comparison baseline for both.
4. Implement the planned Burn path for an agreed tensor/surrogate use case when
   needed. An MLX experiment is conditional on a demonstrated gap or substantial
   benefit and an accepted native dependency boundary. Model approximation error
   needs separate evaluation from exact engine parity.
5. Add real distributed MPI/gRPC runtime execution and extend hardware/browser
   coverage through the existing tracks. Share compatible kernel and model
   contracts without assuming that one target proves another.

This order expresses CareOps delivery priorities. Each step retains the
prerequisites, owned paths and gates in the existing Kairos tracks; it does not
reorder upstream tasks or add new framework work by itself.

## Compatibility and acceptance work

- Keep the engine and reusable domain modules in Rust, with optional GPU/ML
  features. Record foreign runtime dependencies explicitly if selected.
- Preserve Track 32's buffer ownership and RNG requirements. Resolve its current
  reference-event ordering against the actual core scheduler before claiming
  general ED event parity; its additive DES shader is not a complete ED scheduler.
- Prioritize independent numerical batches, agent updates, or analytical kernels
  selected by profiling. ED queue allocation and causally dependent events require
  an explicit ordering design before they can run concurrently on a GPU.
- Record the actual adapter, Metal backend, device limits, OS/toolchain, kernel
  revision, workload and effective execution path. A CPU fallback cannot satisfy
  a Metal acceptance check.
- Measure end-to-end costs including kernel compilation, dispatch, synchronization
  and readback. Report small and large workloads and the observed break-even point.
- Define exact parity for discrete state and event accounting, and justified
  numerical tolerances for floating-point calculations. Specify reduction order
  and supported data types for each backend.
- Check minimum Rust/compiler versions and backend dependency compatibility:
  Kairos currently declares Rust 1.76 in its workspace while current framework
  sources may require newer compilers. Record an intentional toolchain update or
  a compatible version choice; do not assume latest versions work together.
- Reference and extend Kairos's existing tracks when implementation begins.
  This discovery does not amend their status or mark upstream work complete.
