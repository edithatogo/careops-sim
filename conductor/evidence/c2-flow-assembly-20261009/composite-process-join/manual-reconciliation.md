# C2.4 manual deterministic runtime reconciliation

Source `58827f30fde5bc4330bc38eff1875470592b74e0`. Internal Fidelity reviewer
read actual retained child traces and current executed paired-test assertions.
Root cross-checked the named tests in the locked five-package log. This is
synthetic implementation accounting; no empirical or clinical inference follows.

## Actual fresh-process dispatch readback

Trace frames contain u16 little-endian label lengths and u64 payload lengths.
`flow-dispatch` frames are actual restored suffix dispatch debug values; image
frames are exact composite bytes. Input/suffix hashes and child exits reconcile
in `independent-artifact-readback.json` and `process-invocations.txt`.

| Compiled case | Observed suffix ticks | Accounting |
| --- | --- | --- |
| Macro Zero | Grant0; complete20 | Intrinsic20; transit0; admission wait0 |
| Micro Zero | Grant0; complete13 | Intrinsic13; transit0; admission wait0 |
| Transit Ready | Route0→5000→10000; grant10000; complete10019 | Route elapsed10000; intrinsic19; post-arrival wait0. The intermediate waypoint is not added again. |
| Suspend active cut | Preempt8; resume10 with useful3/remaining17; complete27 | Intrinsic20=3+17; interruption2; source setup admission wait5 |
| Restart active cut | Preempt8; restart10; complete30; attempt1; useful20/busy23 | Restarted intrinsic20; prior discarded busy3; interruption2; admission wait5 |
| Actual suspended/restart pending cuts | Saved at8; restored resumes/restarts10; completes27/30 | Saved state contains pre-cut busy3. Count it once, rather than adding it again as suffix work. |

Macro/Micro Zero above are separate case keys and samples. Their20/13 values are
not paired CRN evidence. The following current executed tests provide that proof.

## Current executed paired assertions

All four named tests show `ok` in current `tests.log` (lines92,93,108,111).
Values below are executed source assertions, not retained raw per-row exports.

| Current native oracle | Same-stream assertion and elapsed accounting |
| --- | --- |
| `macro_and_micro_reconcile_queue_transit_and_service_elapsed_time` | Shared intrinsic20, asserted derived seed `0x4e65_a0fe_ae85_3a9b`. Macro queue5+service20=25. Routed Micro transit2+queue3+service20=25; arrival2 and grant5 define the queue interval. |
| `macro_and_zero_micro_share_service_draws_and_submit_actual_timed_work` | Same Service key, seed, sample, draw interval0..1, request and terminal Flow state. Actual timed work has admission/completion events; zero Micro has no transit events. No fixed duration constant is inferred from this oracle. |
| `actual_flow_urgent_suspend_resumes_same_macro_and_routed_micro_service` | Same key/seed, one draw, sample20. Macro5+20+interruption2=27; routed Micro2+3+20+2=27. At8 useful3/rem17; resume10; terminal useful20/rem0. |
| `actual_flow_urgent_restart_reuses_sampled_service_for_macro_and_zero_transit_micro` | Same key/seed, one draw, sample20, no transit/carrier. Queue5+discarded attempt3+interruption2+restarted attempt20=30. Terminal useful20, attempt1, cumulative busy23. |

The older failed paired compilation log is historical and is not used as a pass
or raw readback. These current source assertions and actual process dispatches
support no double-counting of deterministic work/transit/wait intervals. Broader
operational, C5, stable API, release and clinical acceptance stay separate.
New-head hosted source/artifact readback and coordinator phase disposition remain
required before marking C2.4 complete.
