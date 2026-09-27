# Technology and compatibility

- Kairos is the Rust-native simulation core and reusable module ecosystem.
  Resource handling belongs in `kairo-ecs-des` and shared DES/ABM flow integration.
- Reuse the existing tick-based scheduler, generational handles, component
  registry, RNG contracts, Arrow schema ownership and experiment runner.
- Real Arrow IPC/Parquet support is a planned extension of `kairo-ecs-arrow`.
  Its present smoke serialization must not be described as Apache Arrow IPC.
- Propose one Rust `kairo-ecs-calibration` crate for reusable calibration logic,
  under existing VVUQ Track 21. File IO remains owned by Arrow Track 04 and CLI
  orchestration by Track 22. Phase C0 records the architecture decision.
- Rust implements the first calibration search and metrics. Python is optional
  for exploratory analysis and independent reference fixtures; it is not a
  runtime requirement. Julia, Mojo and MLX are not required by these tracks.
- CPU replications use isolated run state and deterministic result merging.
  Existing Tracks 32/34/35 own GPU/Metal, PDES and distributed execution.
- Baseline workspace: Rust 2021, declared MSRV 1.76, toolchain file `stable`.
  Resolve current dependency versions at implementation time; lock them and
  verify MSRV. Change the supported toolchain only through Tracks 25/30.
- Git submodule pins: Kairos `fae901558f07b7b717a676adbafbe2cdc78dea1c`;
  Conductor `7a5c560a4fdf5297be58594cc37527eb12790272` (`conductor-v0.4.1`).
  A pin records the reviewed baseline, not a perpetual claim to be latest.

See [the alignment decisions](../docs/kairos-alignment.md) and
[enhancement programme](kairos-enhancements.md).


## Current development audit

[Dependency policy](dependency-policy.md) and its live source snapshot supersede
older version assumptions: Rust 1.98.1 is locally tested; Arrow 60 requires MSRV
1.88, above Kairos's declared 1.76. D1/C0 resolve compatibility before updating
upstream manifests. Optional latest dependencies are candidates, not installed
or validated merely by appearing in the snapshot. The lightweight local context
harness uses Python stdlib for developer orchestration only.

## Later spatial and live visualization requirement

The [spatial capability plan](spatial-visualization.md) records capture/CAD → a shared versioned
floor-plan package → PixiJS visualization and Kairos simulation, connected through
WebSocket state sync. Early native development uses synthetic graphs; full capture/
CAD adapters are separately qualified later. The backend remains authoritative.
