# Valhalla road-route qualification

Planned conditional evaluation only. Existing-system Rust robustness comes first; this is a later application capability.

## Inputs and dependencies

Accepted petgraph_spatial_routing_20261022:R31; [{"repository": "kairos", "track": "87", "phase": "2", "status": "unresolved", "gate_task": "R40.1", "required_receipt": "Accepted geographic geometry API and exact upstream commit plus parent consumer validation"}]. Use only synthetic/permitted geospatial inputs with source/version/licence/attribution recorded. Illustrative speeds/ranges in earlier transport prose are not calibrated defaults.

## Outputs and value

Real road paths/matrices only when hospital transfers need geographic routes beyond validated travel-time matrices. Produce adopt/defer/reject evidence first; install only after the [value policy](../../dependency-value-policy.md) passes.

## Contract

Use one pinned regional road dataset and routing configuration; cache route responses with hashes and units, access/turn rules and provenance. No uncontrolled live requests inside reproducible experiments. Kairos supplies generic movement/events; CareOps owns service invocation, hospital entrances, vehicle/crew profiles and dispatch/handover policy. Ordinary driving estimates are not emergency-driving evidence. Default movement is route-segment progress with calibrated delays, not microscopic traffic. Treat hospital road arrival, bay arrival and handover completion separately.

## Owned and blocked paths

CareOps application transport adapter, scenario profiles and tests; resolve exact existing consumer files before dispatch. Shared manifests/locks owned by parent coordinator. No direct libs/kairos writes; no clinical policy invention, blanket consent/data access, production deployment or runtime claims.

## Acceptance and tests

Independent route/matrix cases, closure/disconnected entrances, coordinate/units, monotonic progress, sampled-duration provenance and repeatable route-cache replay; service failure is explicit.

## Stop criterion

No adoption if existing validated travel-time matrices answer the question; C++ service operations must justify extra maintenance. OSRM is an alternative to reassess only if Valhalla fails, not a second installed engine.

## Quality and release

Independent behaviour/negative/recovery tests plus measured memory/latency/build/operating burden; source/seed/version and accepted pin receipts. Planning checks do not prove runtime or operational acceptance. New capability does not justify an unbounded framework or another parallel backend.

## Reference

https://valhalla.github.io/valhalla/
