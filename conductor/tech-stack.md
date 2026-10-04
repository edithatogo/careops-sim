# Technology and compatibility

- Kairos is the Rust-native simulation core and reusable module ecosystem.
  Resource handling belongs in `kairo-ecs-des` and shared DES/ABM flow integration.
- Reuse the existing tick-based scheduler, generational handles, component
  registry, RNG contracts, Arrow schema ownership and experiment runner.
- Real Arrow IPC/Parquet support is implemented in optional `kairo-ecs-arrow-io`
  on the qualified development branch. The optional lifecycle sidecar uses that
  IPC path; legacy smoke serialization is not Apache Arrow IPC. Full calibration
  and release acceptance remain separate.
- Propose one Rust `kairo-ecs-calibration` crate for reusable calibration logic,
  under existing VVUQ Track 21. File IO remains owned by Arrow Track 04 and CLI
  orchestration by Track 22. Phase C0 records the architecture decision.
- Rust implements the first calibration search and metrics. Python is optional
  for exploratory analysis and independent reference fixtures; it is not a
  runtime requirement. Julia, Mojo and MLX are not required by these tracks.
- CPU replications use isolated run state and deterministic result merging.
  Existing Tracks 32/34/35 own GPU/Metal, PDES and distributed execution.
- Baseline workspace: Rust 2021, declared MSRV 1.76 for default features, exact
  canonical developer/integration toolchain 1.99.0 (current stable, owner-selected 2026-10-04). Track 13 CI separately exercises
  the Rust 1.76 default-feature workspace. The dated Rust 1.99.0-beta.8 snapshot
  (observed 2026-09-27) is a non-blocking canary. Change this policy only through
  the reviewed Track 25/30 contracts.
- Historical D1.2 submodule audit baseline: Kairos development branch `codex/d12-bench-lock-msrv`
  at reviewed D1.2 commit `339af4e7365e70ad7e67fe3e934e4fb215fbaf8b`;
  Conductor `7a5c560a4fdf5297be58594cc37527eb12790272` (`conductor-v0.4.1`).
  Current qualified pins are recorded in `conductor/current-state.json` and the exact-head owner contract. A pin records the reviewed baseline.

See [the alignment decisions](../docs/kairos-alignment.md) and
[enhancement programme](kairos-enhancements.md).


## Current development audit

[Dependency policy](dependency-policy.md) and its live source snapshot supersede
older version assumptions: Rust 1.99.0 current stable is pinned and the 1.76 default-feature
workspace floor is tested. Arrow 60 requires MSRV 1.88, above Kairos's declared
1.76, and is isolated behind optional package/features and its explicit compatibility lane.
Optional latest dependencies remain candidates, not approved additions. The
lightweight local context harness uses Python stdlib for developer orchestration
only.

## Later spatial and live visualization requirement

The [spatial capability plan](spatial-visualization.md) records capture/CAD → a shared versioned
floor-plan package → PixiJS visualization and Kairos simulation, connected through
WebSocket state sync. Early native development uses synthetic graphs; full capture/
CAD adapters are separately qualified later. The backend remains authoritative.
