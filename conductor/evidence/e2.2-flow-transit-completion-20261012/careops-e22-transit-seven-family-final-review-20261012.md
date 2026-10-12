# Consolidated seven-family source/proof review

Read-only snapshot audit. No gates run, no source edits. These are completion gaps in the existing owner contract, not new scope. Production public API/builder/arrival tracking appears consistent in this bounded read; source-stage acceptance still requires frozen passing evidence.

## Minimum consolidated test correction

1. **Moving/unconsumed:** in `adapter_transit_actual_moving_and_service_unconsumed`, save the full service snapshot row before Start and compare it after actual Start, Progress and arrival. Current final Pending-only check does not prove unchanged original duration, useful/remaining progress and origin. Assert no request and empty resource list for the no-resource fixture. Do not compare after Horizon as a substitute for each transit cut.
2. **Retained AND consumed Pause:** current `adapter_transit_pause_retained_and_consumed_resume` has only the consumed path. Run two independently constructed identical adapters. In one, inspect Paused with old Domain retained and schedule Resume5 before consuming that old event; in the other, actually consume old Domain2, verify documented expected-event/ticket clearing, then schedule Resume5. Both must execute their complete suffix and arrive7. Compare final route/cursor/elapsed/phase and full unchanged service/sample state, and expected-event accounting at the distinct cuts. Both eventual suffixes must include the actual old event; merely renaming one branch is insufficient.
3. **Two arrival causes:** normal Domain arrival_dispatch must explicitly assert errorNone and Accepted(empty), as well as witness/event/cause. Resume frontier plus the private full V4 stale delta proof now cover the other required path; retain all full-image checks. Correct the known consumed timeline assertion only as root authorized.
4. **Admission/suppression:** after binding the same service, private test code must call BOTH public native observers: pending observer and dispatch observer using the saved actual arrival dispatch. Both return None. Keep actual Granted/Completed same WorkId/origin control. Compare adapter/native state before/after duplicate admission and duplicate begin_transit rejection; current duplicate admission checks only error, and duplicate transit is tested elsewhere without full adapter equality.
5. **Identity/error/unrelated:** existing foreign handle, bound service, route collision and private forged carrier checks are good. Extend bound fixture through actual completion, then reject begin_transit for terminal service with unchanged image. Private observer unit control may pass a genuine unrelated marker/Horizon dispatch and an error-marked clone of the actual arrival dispatch to `TransitAdapterState::observe`; assert no observed_event publication/no arrival observation. The error-marked clone is a receipt validation control, not proof an actual runtime error occurred. Preserve positive actual arrival baseline and do not expose a public forged-dispatch API.
6. **Horizon/zero route:** H-equality Active/observed0 and future LIVE progress are covered. Zero-route test currently stops at arrival; drain its bounded remainder through Horizon and assert no dispatch errors, no extra transit arrival, unchanged Pending service/sample and finite event count. No clinical zero-duration service should be introduced.
7. **Limits/callbacks:** exact M/L positive routes, C positive pause/resume, one-over controls and all four native limit dimensions are present. Combined entity/operation overflow negatives are present. Retain actual Accepted rows<=1 on dispatched transit callbacks and verify no service lifecycle/Notify before admission; make these assertions across the branch suffixes above rather than adding a new proof family.

## Existing API anchors

The integration test can compare `FoundationSnapshot.services` rows and real `AdapterDispatch.native`. Private unit tests in owned transit.rs can inspect adapter.flow, state.entries observed_event and native observers without adding public accessors. Existing `transit_codecs` now registers exact marker planner+named receipt and TransitV2 codec/domain; full native V4 comparisons remain capture-only because FrozenService decode is intentionally synthetic. No restore claim is required in this packet.

No additional fixture names, production APIs, budgets, sources or callback policies are needed. Consolidate these test changes once, freeze all three hashes, then root decides the explicitly reviewed remaining execution authority; preserve attempts1–8 and all failures.

## Reviewed snapshot hashes

crates/careops-ed/src/flow_adapter.rs: ab91943450144ade25d7fcd492ba7b05615bb4a78e8a148ad6e5e4483fb9e5b5
crates/careops-ed/src/flow_adapter/transit.rs: dffbee0c5d28508b68fa8565e5d6e7775f015a2f77875dacaacb5e215ce08afc
crates/careops-ed/tests/flow_adapter_transit_native.rs: 6744658dce913fac5a5670db50059a1ad0a7d14a66d35c24a35aea56e493118c
