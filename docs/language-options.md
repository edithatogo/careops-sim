# Language recommendation for CareOps Sim

Recommendation date: 2026-09-25. This informs the technology-stack discussion;
specific framework and dependency versions are not yet selected.

The user's requirement is Rust-native Kairos and reusable modules. Recommend Rust
for the CareOps Sim domain library, model runtime, adapters, and batch runner too:
this keeps engine development and ED model integration within the same language,
type system, build tooling, and concurrency boundary.

| Option | Recommended role | Reason and tradeoff |
| --- | --- | --- |
| Rust | Primary implementation | Directly consumes Kairos crates. Rayon is a candidate for independent replication/scenario parallelism; causality-preserving PDES still requires its own scheduler design. |
| Python | Optional notebooks, exploration, reference calculations, and thin bindings | Useful alongside the user's existing Python analytical APIs. PyO3 supports exposing native Rust functions to Python. A binding must call the actual Rust model; it does not establish native readiness of another library's Python-only methods. |
| Julia | Optional specialist numerical research | Its threading and distributed-computing facilities are relevant to scientific work. In this project, adding a second runtime and an engine bridge needs a specific analytical benefit beyond the agreed Rust implementation. |
| Mojo | Optional performance experiment | It supports CPU and GPU programming on specified platforms, including Apple silicon. Its capabilities warrant measurement on relevant kernels, but adding a Mojo computation module would need an explicit exception to the Rust-native requirement. |

This recommendation is an architectural judgment based on the existing Rust
engine and the user's requirements, not a measured claim that one language is
universally faster. A mixed-language frontend or notebook layer need not change
the implementation language of the reusable simulation library.

## Apple silicon priority

Following the user's request, prioritize Metal as the first GPU backend after
the CPU baseline. Kairos already plans wgpu/Metal and Rust-native Burn inference.
The [Apple silicon assessment](./apple-silicon-acceleration.md) recommends building
on those tracks. The user's alignment requirement makes direct CubeCL adoption
or an MLX comparison conditional on a demonstrated need or a substantial benefit
over the existing plan. MLX's Rust wrapper introduces a C++/C runtime
and therefore requires an explicit architectural decision under the Rust-native
requirement. No Python runtime is required to adopt a Rust-facing MLX binding.

Use the [Kairos alignment policy](./kairos-alignment.md) when selecting frameworks
and assigning implementation tasks. Existing upstream plans and contracts are
the baseline for implementation decisions.

## Primary sources reviewed

- [Kairos workspace](../libs/kairos/Cargo.toml) and
  [binding inventory](../libs/kairos/bindings/README.md), pinned at
  `fae901558f07b7b717a676adbafbe2cdc78dea1c`.
- [Rayon documentation](https://docs.rs/rayon/latest/rayon/): data parallelism,
  parallel iterators, scoped tasks, and configurable thread pools.
- [PyO3 guide](https://pyo3.rs/v0.29.2/): Rust extension modules for Python and
  associated build tooling. Compatibility with the selected Kairos toolchain
  must be checked before choosing a binding version.
- [Julia parallel computing manual](https://docs.julialang.org/en/v1/manual/parallel-computing/):
  threading, distributed computing, and GPU-related ecosystem interfaces.
- [Mojo platform requirements](https://mojolang.org/docs/requirements/): supported
  operating systems, processors, GPUs, and required toolchains. Support for a
  platform does not establish compatibility with Kairos or the ED model.
