# Foundation independent review (read-only)

Reviewed parent HEAD `a6ddf1c89c420bfea3f3114cc24097247e9e92b3`; at review time the checkout had the expected three uncommitted foundation paths. Source hashes at final read:

- `crates/careops-ed/src/flow_adapter.rs`: `1143273f980565b4d927a14d4e63695c52db2007af9c1bda32a73dcc8a8f2fc5`
- `crates/careops-ed/tests/flow_adapter_native.rs`: `4a8ae3910a2ec0adb29b90d9a2255b294073cf46646419b547afa7750e25dbd6`
- `crates/careops-ed/src/lib.rs`: `627063077f14c5c97d7c9226d490b43fe2f32eb307b441b8e67361cc9e12db5f`

## Review

The nested capacity-two test binds the two preemptions, grants, resumptions, and completions to request IDs and expected ticks; it checks the non-preemptible holder stays active and checks suspended remainders. The horizon tests cover completion equality and a real positive-duration grant at H with its later completion still pending.

Two remaining proof gaps:

1. `foundation_native_positive_service_and_lineage` only exercises `SampleOrigin::Fixed` (the shared helper also constructs Fixed). Add a sampled origin and assert exact pass-through at completion. A minimal case sets useful-before=2, remaining=2, original=4, and `Sampled { root_seed: 0x51a7, stream_slot: 17, draws_before: 41, draws_after: 43 }`; assert the real completed observation has the same request/work/identity, exact origin, useful elapsed 2, total useful 4 and Completed state. This proves lineage preservation; do not claim an RNG counter check because this API has no sampler.
2. There is no `ClosedPhase` test. At t=1, dispatch a Decision marker, snapshot, then attempt CalendarExpiry at t=1 with operation ID `reuse`; require `ClosedPhase` and unchanged snapshot. Positive control: schedule `reuse` at t=1 with Ready, require success, one marker/event increment, and `peek_next` at priority 40. This covers rejection atomicity and confirms a rejected ID was not consumed.

The earlier Horizon-marker concern is downgraded to a semantics clarification, not a defect. `step` returns `None` only when the queue is empty or its next time is greater than H (`flow_adapter.rs:1003-1009`); Horizon dispatch advances the phase cursor and emits an observation (`1101-1110`), while `schedule_marker` intentionally allows same-priority events (`910-953`). The implementation drains such counted downstream H events before `None`, consistent with the contract's equal-priority allowance. No additional Horizon change is requested from this review.

No tests/build commands were run. Findings and exact test oracles were sent directly to the worker and root.
