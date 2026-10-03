# Q3: Track 01/Track 25 review and experimental migration note

Date: 2026-10-03. Protected root: `crates/kairo-ecs-des`.
Reviewed Kairos development commit: `1455f76226db61a3dc930aced57849d2a66cbdb1`.
Parent baseline: `81f5d7d43b2ec233b8cfe6f87f73f15040ced9ed`.

## Decision and reviewer authority

The parent coordinator independently reviewed the implementation in the
Track 25 compatibility role and accepts **experimental-breaking** classification
for the concrete Q3 Rust symbols. The same coordinator independently reviewed
interruption/state/RNG behavior in the Track 01 role using source hashes and
manual interval/context audits. These are qualified internal review roles,
not external maintainer signatures. Implementation writer `q1_contract_review`
records their disposition and does not self-approve release.

This supplements [ADR0003](../design/queue/ADR-0003-flow-runtime-contract-proposed.md).
It preserves that accepted architecture and the existing protected experimental
root/policy. Development acceptance does not authorize alpha/beta/RC/1.0
publication or close the full Track 25 programme. **Release remains held.**

## Exact Rust changes and compatibility risk

| Surface | Concrete change | Consumer effect |
| --- | --- | --- |
| `RequestState` | Adds `Suspended`, `Completed`, `Aborted` | Existing exhaustive Rust matches can fail to compile; terminal detection must include Completed/Aborted |
| `ResourceRequest` | Adds public `timed`, `can_preempt`, `preemptible` fields | Existing public struct literals and DTO adapters may require updates |
| `LifecycleRecord` | Adds public `transition: LifecycleTransition` | Struct literals/adapters may require updates; Active state alone cannot distinguish Granted/Resumed/Restarted |
| New types | `PreemptionStrategy`, `WorkState`, `WorkProgress`, `WorkHandlers<C>`, `LifecycleTransition` | Experimental typed interruption/progress/callback surface |
| Builder | `timed_work(WorkId)`, `can_preempt(bool)`, `preemptible(PreemptionStrategy)` | Explicit opt-in to timed completion and independent incoming/holder interruption flags |
| Runtime methods | `create_restartable_work`, `register_work_handlers`, `work_progress` | Owned initial template/factory, optional typed handlers registered before work, checked read-only progress |

These existing-enum/struct changes are not universally nonbreaking Rust semver
additions. No new `non_exhaustive` policy, adapter or source-compatibility promise
is invented by this review.

The frozen [Q3 experimental contract](../../libs/kairos/conductor/tracks/03-flow-des-trajectory-abm-behavior/q3-experimental-api-contract.md)
has SHA-256 `a593f30ac40127412cdb4a6b499ca7442c460dc5c3df7978e3906544cc7e2366`.
Its accepted timed, interruption and notification semantics are unchanged.

## Migration note

First affected stage: **Q3 experimental development revisions**, concretely
the reviewed commit above. No first permitted stable/beta release is declared.

Consumers of experimental Flow should:

1. Pin the exact reviewed development revision rather than infer compatibility
   across future experimental changes.
2. Update exhaustive `RequestState` matches for Suspended/Completed/Aborted.
   Include Completed and Aborted in terminal-state handling alongside Released,
   Cancelled and TimedOut.
3. Update `ResourceRequest`/`LifecycleRecord` literals and DTO adapters, or use
   runtime-returned values. Request builders remain the checked command ingress.
4. Read `LifecycleRecord.transition` for lifecycle distinctions; do not derive
   Granted/Resumed/Restarted solely from Active request state.
5. Use `timed_work` for automatic work completion. Existing `for_work` and
   `submit_work` remain untimed context association.
6. For Restart, provide an owned initial template and trusted factory producing
   logically fresh context. Original sampled duration is reused. Optional typed
   handlers must register before work creation; delivery is later, with captured
   progress and live owned context.

Legacy `DESContext`/FIFO `Resource` users and existing
`ABMContext`/`BehaviorSimulation` users require no migration into Flow.
An optional Q4 migration example remains deferred.

## Track 25 twelve-question disposition

| Review question | Disposition |
| --- | --- |
| Rust API | Exact types/methods and experimental-breaking enum/struct risks above |
| C ABI | Not affected; Flow is not exported through FFI |
| Python | Not affected; no Flow binding added |
| R | Not affected; no Flow binding added |
| Julia | Not affected; no Flow binding added |
| TypeScript/Wasm | Not affected; no Flow binding added |
| C# | Not affected; no Flow binding added |
| Go | Not affected; no Flow binding added |
| Arrow schemas | Existing event-log schema unchanged; DES typed lifecycle records do not claim Q4 Arrow encoding completion |
| Conformance | Existing cross-language fixtures unchanged; new versioned Rust Flow tests provide bounded Q3 evidence |
| Batch-friendly | Multiple buffered commands supported; no vectorized batch or performance guarantee |
| Deterministic replay | Integer effort/time, canonical queue/victim ordering and deferred transition snapshots reviewed; no portable continuation/replay serialization claim |

Package roots are not renamed, split, merged or removed. Availability and
existing experimental DES/ABM inventory/matrix classification remain unchanged.
No package catalog, compatibility policy or protected inventory edits are needed
for this governance disposition.

## Track 01 independent state/RNG disposition

**Accepted for this bounded development implementation by the parent
coordinator in the Track 01 review role.** This decision derives from independent
source and manual audit, not from CI status.

Shared source bytes match the earlier qualified `dffd6f6` foundation:

| Source | SHA-256 |
| --- | --- |
| `crates/kairo-ecs-core/src/lib.rs` | `8c7d09d7f55837a5d1557f49b97cbfc7014ba6a51dca5fb93ddd1695f80da7e3` |
| `crates/kairo-ecs-types/src/lib.rs` | `9613c19c885183844ab48c157c34e9df718fc3c01363e982a847fb3e8240542b` |
| `crates/kairo-ecs-state/src/lib.rs` | `88cf420d5cb2b083bd76e8882a83fcc049001e905e197bba2785f72c1ec5aead` |
| `crates/kairo-ecs-rng/src/lib.rs` | `68397e53959221c17a9705da1dd6e23a8221df9825e879222cdd9b154029a991` |
| `crates/kairo-ecs-abm/src/lib.rs` | `41d1e7f20860d260c6c02e8afb98aab94ac14cf8e1689acc11357216c32df7f9` |


The runtime composes existing private Scheduler/World/ComponentRegistry APIs.
The review covered safe typed-store context erasure and template/context/progress
cleanup; checked lease/attempt/execution identity; strict stale-completion
validation before any neighboring due boundary; whole-transaction arithmetic/
token/despawn preflight; and preservation of independent due transitions when
an explicit command is semantically rejected.

The coordinator manually checked primary and secondary busy/waiting intervals,
owned context generation and callback capture against the frozen fixtures and
source. Primary: Suspend ends 12, busy 10/wait 2; Restart ends 15, busy 13/wasted 3;
Abort at 3, busy 3/remaining 7. Secondary: Suspend ends 13, busy 10/wait 3; Restart
ends 17, busy 14/wasted 4; Abort at 4, busy 4/remaining 6. Full interval tables are in
the [qualified pin evidence](q3-qualified-runtime-pin-20261003.md).

Suspend/Abort preserve context generation 0. Restart's owned initial template
generation 0 produces factory context generation 1; original sampled duration
remains stored and no new duration-draw path is introduced. The engine RNG
algorithm, seed/stream derivation and shared state/type representations remain
unchanged. This is interruption/state review, not statistical or clinical
calibration acceptance.

Checks are separate supporting evidence: 71 selected DES tests passed on each
actual Rust1.98.1 and1.76.0; exact owner run
[37118457361](https://github.com/edithatogo/kairos/actions/runs/37118457361)
passed both hosts. Private faults cover aggregate rollback, semantic boundaries,
factory/notification/cleanup preflight and adversarial stale tokens. The cycle
grid contains 24 bounded cases, not exhaustive state-space verification.

## Release hold and remaining scope

Release hold: **yes**. Development-only concrete API/state review is accepted;
release reviewer decisions and publication gates remain open. No alpha
publication is authorized. Before beta/RC/1.0, incorporate this exact-root
migration and compatibility disposition in the owning Kairos release documents,
complete required maintainer/API/release reviews and run the existing
`docs/design/validate-compatibility-pack.ps1 -ReleaseGate` gate. This parent note
does not replace those publishing-repository obligations.

The inspected Kairos validator checks inventory/policy/matrix/release-note
alignment; it does not require a new local concrete ADR/review artifact. No
Kairos policy, inventory or release compatibility file is edited here, so its
validator is not required for this parent-only note. No validator pass is
claimed, and its future release requirement is unchanged.

Trusted factories/callbacks/destructors are not isolated against arbitrary
panics or application side effects. Facade preflight does not strengthen global
core overflow or serialization guarantees. Q4 general handler ingress and
persistent same-tick budgets, and Q5 staged-clone performance, remain deferred.

All parent Q3 checkboxes and active phase are unchanged. Full Conductor Q3.4
closeout requires a separate accepted status packet; this note does not
automatically close it or advance other delivery phases.

Approved governance packet SHA-256:
`7330c0571fbba4441ee2f312d26bc8eaa6607e65297449277d5013a9e5bc2db4`.
