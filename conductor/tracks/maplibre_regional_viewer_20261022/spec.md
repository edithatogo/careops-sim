# MapLibre regional transport viewer

Planned conditional evaluation only. Existing-system Rust robustness comes first; this is a later application capability.

## Inputs and dependencies

Accepted valhalla_transport_20261022:R42. MapLibre additionally requires accepted Three.js R35. Use only synthetic/permitted geospatial inputs with source/version/licence/attribution recorded. Illustrative speeds/ranges in earlier transport prose are not calibrated defaults.

## Outputs and value

Geographic tile/camera/coordinate handling for real regional movement, beyond the existing building-only viewer. Produce adopt/defer/reject evidence first; install only after the [value policy](../../dependency-value-policy.md) passes.

## Contract

CareOps dashboard owns MapLibre and integration with accepted Three.js, not Kairos. Share authoritative entity IDs/clock/routes, use correct regional-to-building transforms and entrances. Keep map presentation separate from movement outcomes; preserve accessible tables, tile/data attribution, offline/retry behaviour and bounded GPU resources. MapLibre is JavaScript, an explicit UI exception to the Rust preference.

## Owned and blocked paths

CareOps apps/dashboard, regional-view contracts and browser tests; resolve exact existing consumer files before dispatch. Shared manifests/locks owned by parent coordinator. No direct libs/kairos writes; no clinical policy invention, blanket consent/data access, production deployment or runtime claims.

## Acceptance and tests

Actual browser map/route/model rendering, coordinate transforms and entrance transition, lossless IDs, accessible controls, no-tile/no-GPU fallback, repeated lifecycle cleanup and named-device cost budgets.

## Stop criterion

No adoption for a building-only scene or an unneeded geographic dashboard; no permanent duplicate vehicle viewer or movement authority.

## Quality and release

Independent behaviour/negative/recovery tests plus measured memory/latency/build/operating burden; source/seed/version and accepted pin receipts. Planning checks do not prove runtime or operational acceptance. New capability does not justify an unbounded framework or another parallel backend.

## Reference

https://maplibre.org/maplibre-gl-js/docs/examples/adding-3d-models-using-threejs-on-terrain/
