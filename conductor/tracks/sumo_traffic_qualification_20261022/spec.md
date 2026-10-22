# Conditional SUMO traffic co-simulation

Planned conditional evaluation only. Existing-system Rust robustness comes first; this is a later application capability.

## Inputs and dependencies

Accepted valhalla_transport_20261022:R42. Use only synthetic/permitted geospatial inputs with source/version/licence/attribution recorded. Illustrative speeds/ranges in earlier transport prose are not calibrated defaults.

## Outputs and value

Only an accepted question where congestion, intersections, road interactions or emergency yielding materially changes decisions beyond calibrated route travel times. Produce adopt/defer/reject evidence first; install only after the [value policy](../../dependency-value-policy.md) passes.

## Contract

Evaluate offline SUMO scenarios first. Live coupling only if offline distributions cannot answer the question and explicit value review accepts operating cost. Pin simulator/network/demand/seed and validate jurisdiction-specific emergency behaviour, left-hand traffic where applicable. For live coupling, one authority owns route progress/arrival: SUMO when enabled, Kairos when using its route model. Specify tick-to-step conversion, timestamp boundaries, event ordering, cancellation, restart and failure before TraCI/libsumo selection. Never schedule a second independently sampled arrival. CareOps owns traffic scenario/service adapter; Kairos changes only reusable explicit event/time interfaces through owner handoff.

## Owned and blocked paths

CareOps application transport adapter, scenario profiles and tests; resolve exact existing consumer files before dispatch. Shared manifests/locks owned by parent coordinator. No direct libs/kairos writes; no clinical policy invention, blanket consent/data access, production deployment or runtime claims.

## Acceptance and tests

Independent traffic controls and calibrated arrival distributions; offline replay first; live mode additionally tests deterministic stepping, reconnect/kill/recovery, no duplicate/lost arrivals, vehicle/crew conservation and renderer-off trace equality.

## Stop criterion

Keep dormant unless a concrete traffic-interaction need and qualified demand data exist. Prefer no SUMO to an unvalidated complex traffic model. No dependency installation or live co-simulation commitment from registration.

## Quality and release

Independent behaviour/negative/recovery tests plus measured memory/latency/build/operating burden; source/seed/version and accepted pin receipts. Planning checks do not prove runtime or operational acceptance. New capability does not justify an unbounded framework or another parallel backend.

## Reference

https://sumo.dlr.de/docs/TraCI/
