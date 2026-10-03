# Q4 persistent same-tick budget — qualified development pin

Prepared 4 October 2026 (Australia/Brisbane). This records a bounded experimental
budget capability. Full Q4 remains incomplete; every Q4 task checkbox stays open.

## Source, contract and ownership

Qualified Kairos development commit:
`358bb156c638e3bc5a67129e306ee0ae65d6f61d`, published on
`codex/careops-q4-same-tick-budget`.
It descends from the prior preview pin
`132fb4d525c2bd7b110fc11548c8d1b5fdf9a648`. Exactly three paths differ from that
pin: the Q4 runtime contract, the new public budget fixture and `flow.rs`.

| Binding | SHA256 |
|---|---|
| `conductor/design/flow/q4-runtime-contract-v1.md` | `59aabd644161d479ea7507106f782ce7fe181ec44a2ccf5926b3ac843279dca5` |
| `crates/kairo-ecs-des/src/flow.rs` | `3d7660de32d0b1e36ccac3f35b65cdc9a741e2ef51a8c0cb7ccd3b7ae075fbd8` |
| `crates/kairo-ecs-des/tests/flow_same_tick_budget_v1.rs` | `7013e0210f18fc08f46f916a111b4809231fa8c99b95236e1d40e12bd9bfc8be` |

The coordinator independently accepted this bounded internal Track 01/03/25
contract and concrete source. This is not an external maintainer signature,
full Q4 phase acceptance or a universal Rust source-compatibility claim.
The new experimental public error variants and configuration/inspection structs
retain the existing experimental compatibility and release holds.
Core preview, scheduler ordering, types, state, RNG, ABM, Arrow, calibration,
Q3 fixtures and upstream governance are unchanged from the qualified foundation.
No engine duration redraw or competing DES/ABM world is introduced.

The runtime previews the exact scheduler head and plans at its effective timestamp.
It preflights aggregate lifecycle rows and actually deliverable notifications,
checked counters, time/revisions, typed factory/context descriptors and distinct
live cleanup entities before consuming the event. Accepted plans then consume
that exact preview and commit. Factories and callbacks run only after admission;
no fallible Flow result follows callback mutation. User callback/factory panics
are not a rollback or recovery promise.

The positive default limit is 100000 transitions per tick, a safety cap rather
than clinical or performance tuning. A constructor override is fixed before the
run. Exceeding the limit permanently halts that run, retains pending work and
leaves time, dispatch statistics and the last legal tick ledger unchanged.
Repeated calls, including `run_for(0)`, cannot bypass it. There is no event-time
bump, consumed-event side channel, cancellation escape or reconfiguration recovery.
Normal legal progress to a later tick resets the ledger; raw/stale events cost zero.

## Test-first lineage and retained failures

Contract amendment `dd5eff5d` preceded public fixture commit `daae8be9`.
Actual Rust 1.88.0 and 1.98.1 fixture compilation each exited 101 with exactly
25 diagnostics restricted to missing frozen budget APIs. No assertion ran.
Those missing APIs masked an integer-versus-`EventKind` assertion mismatch.
The separately reviewed fixture amendment `4e6c5dde` wraps the four expected
kind assertions in `EventKind::custom`; numbers, priorities, timelines and
expected costs are unchanged. Both compiler lanes again produced only the
25 expected missing API diagnostics before the implementation commit.

Original fixture receipt:
`/private/tmp/kairos-q4-budget-fixtures-v1/.artifacts/q4-budget-fixtures/result.json`,
SHA256 `2a7a97e2b7cdfb2939501bb1173566bd243aa8dbc5d664c90facefc28b66926c`.

Mechanical amendment receipt:
`/private/tmp/kairos-q4-budget-fixture-kind-amendment/.artifacts/q4-budget-fixture-kind-amendment/result.json`,
SHA256 `5f4517aacb36a30bc264207770da74956c121c4c9c20df37ded0dcac2d0b0982`.

The first runtime compile failed on the owned private `EventKind` assertion;
its original source/log were preserved. The next suite ran 38 unit tests
successfully and failed the new private zero-duration oracle. Existing Flow
already completes duration-zero work inline: admission emits Queued, Granted
and Completed in one three-row plan. The oracle was corrected to preserve that
behavior, rather than changing production semantics. Its original real
completion token remains an identified stale, zero-cost dispatch at the same
tick; there is no second completion or invented time advance.

## Actual local execution and bounded oracles

Final source passed scoped rustfmt. Actual Rust 1.88.0 and 1.98.1 each ran
`cargo test --locked -p kairo-ecs-des`: **88 tests passed, zero failed**
(40 unit and 48 integration), with zero doctests and no compiler warnings.
Commands, compiler binary hashes, environments, absolute owned temporary/cache
paths, working directory, UTC start/end times and raw exit codes are retained.

The nine public fixture families cover defaults, effective dispatch timestamp,
first admission fail-stop, atomic release/replacement rejection, exact-limit
normal reset and event-count limits, delivered notifications, blocked callbacks,
known stale completion and eight independent due-boundary/cancel timelines.
All expected costs and insertion/priority orders were frozen before execution.

Seven added private tests cover:

| Private oracle | Bound |
|---|---|
| `arithmetic_injections_preserve_real_head_and_all_admission_state` | Five admission/lease/event/execution/tick overflow injections; unchanged head, statistics, commands, world, membership and context. |
| `cleanup_and_budget_add_overflow_preserve_head` | Despawn reservation and checked ledger-add overflow before consumption. |
| `zero_duration_admission_is_one_three_row_plan_or_retains_pending_command` | Limit 2 rejects the whole cost-3 admission; limit 3 commits exactly three rows and identifies the stale due-now token. |
| `private_release_due_boundary_budget_is_atomic_in_both_dispositions` | Within-budget invalid release preserves due completion/replacement; over-budget retains the complete pending transaction. |
| `restart_factory_is_not_called_for_budget_rejected_resume` | Rejected resume leaves the initial factory count at one. |
| `missing_owner_context_or_handler_notification_is_zero_cost_at_limit` | Three invalidated deliveries consume zero and invoke no callback. |
| `cleanup_missing_typed_context_or_dead_work_is_rejected_before_consume` | Two cleanup descriptor/world injections retain pending work and authoritative state. |

Retained Q3 unit oracles additionally cover attempt/execution/busy overflow,
aggregate second-victim failure, accepted restart factory invocation, callback
once-only behavior, notification event reservations and checked uint32 ordinals.
Earlier inline preflight expectations now require errors before consumption,
including retention of a failed release reservation. Frozen public Q3 fixtures
were not weakened. The exact mapping and source proof are retained below.

Every production scheduler insertion belongs to Flow ingress or the planned
internal token list. Existing Flow lifetime counters and the actual scheduler
scheduled/dispatched counters are checked against `u32::MAX` before consumption.
This bounds post-consumption scheduling arithmetic. Cleanup preflight checks
live entities, typed context, owner/spec/progress/template membership and distinct
entity IDs before descriptor invocation or despawn. These are source-backed
arithmetic/invariant proofs; billions of allocations were not executed.

The unchanged generator v1 model test runs 576 bounded cases with six explicit
seeds; the existing membership grid runs 32 seeds × 100 operations. These are
bounded property/model checks, not exhaustive state-space or performance proof.
The Rust 1.76 Cargo development-dependency gap remains separate; this integration
makes no whole-suite MSRV waiver or stable release claim.

Retained local directory:
`/private/tmp/kairos-q4-budget-runtime-kind-successor/.artifacts/q4-budget-runtime/`.

- `result.json`: `dd3d37ff55b47fd18e20846a630ef13b43da4897ddb23364b78a0811a1984351`.
- `des-1.88.0-qualified.log`: `96576c2f2bbcfbcdb72c9edb0303d9fec7977c57312371f6d9228b2cb5d06972`.
- `des-1.98.1-qualified.log`: `5776cbf72927bcb2fce38e1f5ea4be07ccfae111e010334671d1a322956a073c`.
- `private-gates-and-proof.md`: `98b07fcb34e1175d958872428e8f6ea27c43a2247ff167350b0429ad5c1e8a48`.

## Exact-head hosted owner proof

[CareOps native owner run 37138038107](https://github.com/edithatogo/kairos/actions/runs/37138038107)
succeeded at exact development head `358bb156c638e3bc5a67129e306ee0ae65d6f61d`.

| Actual host | Job ID | Observed execution |
|---|---|---|
| ubuntu-24.04, x86_64-unknown-linux-gnu | 111246485884 | Eight native owner packages and optional calibration floor passed; all nine public and seven private budget tests, plus the existing Q3 seeded model, explicitly ran and passed. |
| macos-15, aarch64-apple-darwin | 111246486016 | The same owner packages, floor and named budget/Q3 tests explicitly ran and passed. |

The unchanged workflow verifies the actual source/host/compiler, uses Rust 1.98.1
for reusable native packages and Rust 1.88.0 for the optional calibration floor.
Qualification is supported by actual test names in both logs, not overall green alone.

Retained hosted directory:
`/private/tmp/careops-q4-interface-proposal/.artifacts/q4-budget-publication/`.

- `hosted-result.json`: `281309f8bad9e3c10f5fb5b964328fad1fb90d8d9e0506bc62b59f1ba52e9153`.
- `ubuntu-24.04.log`: `f324ff8a58ec424e9be1b61f4baaa5a9b92c6d39a8701c32dcf50767a198e589`.
- `macos-15.log`: `4fe26b6a3c209cd209d9cfe6a1dcd3a3bf4b7e4d95a1d07847874584e9b9d67d`.
- `run.json`: `581e59edbafc6735477d7f0dcc6c9fb15163c245b9873e627898386ff12694ee`.

## Focused parent integration and holds

Parent source base is `7be98f2bb800754a2b62f3cfc0e62417e49ec668`.
Only the Kairos gitlink, current-state pin/narrow Q4 routing, D2 qualification
contract and this evidence change. Of 23 prior child bindings, 21 remain
byte-identical; only `flow.rs` and the Q4 runtime contract hashes change.
The budget fixture adds the 24th binding. Every binding is rehashed against the
actual checked-out child. The Conductor extension pin remains unchanged.

Parent integration and acceptance are governed by exact-head hosted checks and
native PR merge readback. Local context/diff checks and the post-commit HEAD pin
validation are separate receipts; child owner CI alone does not prove parent
integration. No unchanged native suite is repeated locally for this metadata step.

Q4 remains incomplete: general domain hooks, callback command batches, shared
DES/ABM integration fixtures, migration example, Track04 lifecycle-sidecar export
and the synthetic staff/bed/cleaning workflow retain separate joins.
No Q4 checkbox, Q5, C1/C2/C4, unrelated D3/D4, portable checkpoint or release gate
is closed by this focused pin. No child source, parent CI or unrelated plan changes
are included. Existing experimental compatibility and release holds remain.
