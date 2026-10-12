# C2.4 integrated Macro/Micro elapsed-time manual readback

**Disposition:** manual source/readback check completed for the bounded synthetic fixtures. This records the C2 plan's elapsed-time reconciliation; it does not accept C2.4 or C2.

## Base runtime-ledger fixture

Read the paired actual-Flow test `macro_and_micro_reconcile_queue_transit_and_service_elapsed_time` in `libs/kairos/crates/kairo-ecs-calibration/src/flow_bridge.rs` (Kairos source pin `6053d8dd2f8057c423decc929c90a8bcd10837f9`, file SHA-256 `e2c3b5eb5381b85a083f879a017d7be5cd573cdabd23b82eda486bab27e41959`). The test and its accepted receipt observe Flow submit/grant/completion times, a two-tick Micro route and accepted arrival, and the same Service duration/key/derived seed in both arms.

| Arm | Disjoint intervals | Elapsed window |
| --- | --- | --- |
| Macro | Queue `[0,5)` = 5 ticks; Service `[5,25)` = 20 ticks | `5 + 20 = 25` ticks |
| Micro | Transit `[0,2)` = 2 ticks; queue `[2,5)` = 3 ticks; Service `[5,25)` = 20 ticks | `2 + 3 + 20 = 25` ticks |

Each arm spans the same `[0,25)` window. The Macro and Micro totals are paired alternatives and must not be added together. The service interval starts at the resource grant, so neither the transit nor the wait is double-counted as work.

The associated [`runtime-ledger receipt`](../c2.4-runtime-ledger-rust199-20261007/acceptance-rust199.json) is SHA-256 `b4e250c598d1cd540f32bde27889961365d8709909201b433927616f8a7e3189`; it records the earlier leaf and exact-parent verification. The later Q4 receipt below covers the current `flow_bridge.rs` file hash.

## Separate urgent-interruption corroboration

The distinct test `actual_flow_urgent_suspend_resumes_same_macro_and_routed_micro_service` uses the same nominal service setup with an urgent interruption. Its actual callbacks and completion assertions produce:

| Arm | Disjoint intervals | Elapsed window |
| --- | --- | --- |
| Macro | Queue 5; useful Service 20; interruption pause `[8,10)` = 2 | `5 + 20 + 2 = 27` ticks |
| Micro | Transit 2; queue 3; useful Service 20; interruption pause 2 | `2 + 3 + 20 + 2 = 27` ticks |

For each arm, useful service is 3 ticks before suspension plus 17 ticks after resume. The pause is elapsed time but is excluded from useful service. This is a separate paired fixture run from the 25-tick base ledger; do not combine the two rows or double-count the service interval. The [`Q4 acceptance receipt`](../c2.4-q4-interruption-rust199-20261007/acceptance-rust199.json) is SHA-256 `88109c5dc3a35eada33d0f047ac33d5271f7a5050f138393ff4a435a265a97c1` and was tested at Kairos pin `6053d8d` using only Rust 1.99.0.

## Verification boundary

This is a manual calculation against committed test assertions and their receipts; no test or Rust command was run for this readback. The current Kairos pin `6303a2552257f1fcbd2aaa002d38352a2a467592` differs from the Q4-tested pin only by the later `#[doc(hidden)]` attribute on `TransitContext::schedule_transit_retry`. The runtime suite has not been rerun at `6303a25`, so the cited executions remain historical at their recorded pins.

An independent read-only fact-check confirmed the source hashes, receipt hashes, and interval formulas above. That reviewer also ran no tests or Rust commands.

These are test-only synthetic cases, not clinical validation, observed ED performance, portable restart, hosted exact-head evidence, release acceptance, or a C2.4 phase decision. Track03 owner disposition, Track22 portable checkpoint/restart, hosted, release, clinical, and C2.4 Conductor phase gates remain open.
