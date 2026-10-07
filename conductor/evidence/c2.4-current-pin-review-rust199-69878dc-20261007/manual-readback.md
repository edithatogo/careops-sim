# C2.4 elapsed-time coordinator readback at Kairos 69878dc

**Manual arithmetic: reconciled. C2.4 phase acceptance: withheld.** This readback is against committed synthetic Flow assertions at the exact current Kairos pin. It does not claim clinical validity or satisfy owner, portable-restart, hosted, release, or phase-acceptance gates. No Rust command was run for this manual review; runtime executions are the separately recorded Rust 1.99.0 Q4 receipt.

## Reviewed source and execution evidence

- Parent HEAD: `c7d8036f8235564d402ba374692206fc235d1bb7`
- Kairos source pin: `69878dcaf9d1eac7f1eba84ef19647642b77435d`
- `flow_bridge.rs` SHA-256: `898d96efb005e4e9da0fbc88ba997f7bbee63c58c0da419d1716a5ab485f2227`
- Current-pin Rust 1.99.0 test receipt: [`Q4 event-drain acceptance`](../c2.4-q4-event-drain-rust199-20261007/acceptance-rust199.json), SHA-256 `5ac4522e19a58f2d48aa36c24dfc1725a1511c6fc46ee0c8f15c004f105ac2e5`
- Runtime result in that receipt: 457 passed, 0 failed, 5 ignored across 43 result groups; targeted Suspend and Restart fixtures passed; strict all-target Clippy and format check passed.

## Base paired elapsed-time fixture

Source: `macro_and_micro_reconcile_queue_transit_and_service_elapsed_time` at `flow_bridge.rs:2106-2326`. The assertions give one 20-tick sampled Service duration, a Macro request submitted at tick 0 and granted at tick 5, and a routed Micro arrival at tick 2 followed by grant at tick 5.

| Arm | Non-overlapping intervals | Total |
| --- | --- | --- |
| Macro | Queue 5 + Service 20 | 25 ticks |
| Routed Micro | Transit 2 + queue 3 + Service 20 | 25 ticks |

Both arms span `[0,25)`. They are paired alternatives, not additive runs. Transit and queue are outside the Service interval.

## Urgent Suspend fixture

Source: `actual_flow_urgent_suspend_resumes_same_macro_and_routed_micro_service` at `flow_bridge.rs:2720-2761`, using helper assertions in `flow_bridge.rs:2470-2707`. Useful service runs from tick 5 to tick 8 (3 ticks), pauses while urgent work runs from tick 8 to tick 10, then resumes with 17 useful ticks remaining. Both arms complete at tick 27.

| Arm | Non-overlapping intervals | Total |
| --- | --- | --- |
| Macro | Queue 5 + useful Service 20 + pause 2 | 27 ticks |
| Routed Micro | Transit 2 + queue 3 + useful Service 20 + pause 2 | 27 ticks |

## Urgent Restart fixture

Source: `actual_flow_urgent_restart_reuses_sampled_service_for_macro_and_zero_transit_micro` at `flow_bridge.rs:2765-2821`, using the same helper. At tick 8 the first 3 ticks of useful work are discarded by Restart; the two-tick interruption occupies `[8,10)`, and a fresh 20-tick attempt completes at tick 30. Assertions confirm both Macro and explicit-zero Micro have zero transit elapsed and no transit carrier.

| Arm | Non-overlapping intervals | Total |
| --- | --- | --- |
| Macro | Queue 5 + discarded attempt 3 + pause 2 + restarted Service 20 | 30 ticks |
| Explicit-zero Micro | Queue 5 + discarded attempt 3 + pause 2 + restarted Service 20 | 30 ticks |

The helper asserts the interval sum equals completion time, terminal work/request completion, resource release, and zero pending Flow events. The paired fixtures use the same Service key/seed/draw position as asserted by their test oracles.

## Current disposition

The local elapsed-time review is current at `69878dcaf9d1eac7f1eba84ef19647642b77435d`. It does not resolve Track03/Track21 route-receipt, mixed-use pause, sensor-distance provenance, or public API/ADR questions; Track22 checkpoint/restart contract; hosted exact-head checks; release acceptance; clinical acceptance; or C2.4 Conductor phase acceptance. Keep the C2.4 plan checkbox open.
