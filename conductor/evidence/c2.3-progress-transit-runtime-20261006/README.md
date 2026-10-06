# C2.3 transit runtime progress — 2026-10-06

## Status reconciliation — 2026-10-07

This file records intermediate progress before the final transit-bridge leaf review. The later [C2.3.transit-bridge acceptance](../c2.3-transit-bridge-rust199-20261006/README.md) at Kairos `7672a67988134438228014e7b3e0f56855cc65e8` locally accepts the calibration `BoundIntrinsicWork` rejected-dispatch observation, exact one-shot retry, replay refusal, accepted arrival, and original timed-request reconciliation described below as still open. Treat the earlier “not accepted” and “remains open” statements as historical snapshots. The C2.3 parent/phase, Track03 owner review, hosted, release, and clinical gates remain open.

## Result

Kairos receipt commit `1637d80ddc3a0da17c84afd6ea6eba0f7abe84e6` contains
source commit `34de9c0fb90b4472db2ba2b6f164eccfee2f7250`. It connects a
`TransitContext` to actual DES planned domain dispatch over the existing route
cursor. The implementation handles future start, timed segment progress,
Pause/Resume, stale events, one actual timed service Acquire at arrival, and
accepted/rejected batch state atomicity. Accepted command tickets are reconciled
to actual EventIds through a DES accepted-batch receipt callback.

This is local runtime progress, not accepted C2.3 dispatch. Two reviewed
follow-ups now bind the initial start EventId atomically into TransitContext,
require matching EventId and tick before Ready/Paused start handling, and reject
duplicate start scheduling while an EventId remains pending. Independent review
found no remaining mismatch in these exact-ID paths. Adapter validation of the
carrier/actor/task/resource association and rejected-dispatch retry reconciliation
remain follow-on work.

## Verification

Executed in `/tmp/kairos-c23-transit-runtime` on Rust 1.76.0 / 1.99.0:

- `CARGO_TARGET_DIR=/tmp/careops-c23-transit-runtime-target rustup run 1.76.0 cargo test --locked -p kairo-ecs-des -p kairo-ecs-abm` — exit 0; all DES/ABM unit, integration and doc tests passed; log `/tmp/c23-transit-runtime-tests.log`, SHA-256 `18b17e982d8510e6db4eb5634d1e37106a8e16699d540a8cb60bfba31a4efaa5`.
- `CARGO_TARGET_DIR=/tmp/careops-c23-transit-runtime-target rustup run 1.99.0 cargo clippy --locked -p kairo-ecs-des -p kairo-ecs-abm --all-targets -- -D warnings` — exit 0; log `/tmp/c23-transit-runtime-clippy.log`, SHA-256 `7a7c5f339ce35cb7811e73ee64d146487d7e72eb2090f8175330980b4fb58593`.
- `rustup run 1.99.0 cargo fmt --all -- --check` — exit 0; log `/tmp/c23-transit-runtime-fmt.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `git diff --check` — exit 0.

Packet receipt: `libs/kairos/.artifacts/mvp/C2.3.progress.flow-transit-runtime/result.json`.

## Remaining C2 work

C2.3 actual calibration-bound dispatch, identity/retry reconciliation, C2.2
callback and seed runtime joins, checkpoint/resume integration, and C2.1 paired
Macro/Micro acceptance remain open. Hosted checks, release readiness, portable
checkpoint, and clinical claims remain out of scope.

## Exact start identity follow-ups

- Source commit `606a27841a00c82589bc32fb5c1542088706d134` adds
  `FlowRuntime::schedule_domain_and_bind<C>` and exact initial start EventId/tick
  checks; receipt commit `2d40b3c3bc1d21e5e55dcb33ee7a44b5404a4bf0` records the
  result. An independent reviewer confirmed same-tick alien events cannot begin
  movement or clear a paused start.
- Source commit `6d8e56341aabc54ea4e94455afb214b47d8a438a` makes duplicate initial
  scheduling fail before allocation and proves the original EventId remains
  bound; receipt commit `c72f3a7ac540b01174fc62398ca2ad283f422cc9` records it.
- Both follow-ups passed all DES/ABM tests, strict Clippy and formatting using
  Homebrew Rust 1.99. Combined current-pin test record: 282 passed, 0 failed,
  1 ignored across 32 result groups. Logs are under `/tmp/c23-exact-start-*` and
  `/tmp/c23-single-pending-start-*`, with hashes recorded in the Kairos packet
  receipts.
- The user specified current stable Rust 1.99 as the only required C2
  verification toolchain. Both follow-ups passed DES/ABM tests, strict Clippy and
  formatting on Rust 1.99. An earlier packet's Rust 1.76 command was unavailable
and is not required under the current scope. Hosted checks remain unrun.

## Calibration-bound transit bridge candidate — 2026-10-06

The development pin now includes Kairos commits `3ec35e51cd46216b0982ce13058008ca9258c829`
and `a514590e485bfb5aa4df675d13940d98f62b0b7b`. The first adds the
calibration `BoundIntrinsicWork` route bridge, actual carrier/task/resource
validation, receipt and event lineage, accepted-arrival request reconciliation,
and the explicit one-shot ABM retry seam. The second adds real runtime-generated
rejected-start and rejected-progress retry regressions. Both worker receipts are
copied alongside this README and remain `ready_for_review`.

The final exact-head checks at Kairos `a514590e` used only the user-specified
Rust 1.99.0 toolchain. `cargo test --locked -p kairo-ecs-calibration -p
kairo-ecs-abm --features kairo-ecs-calibration/flow --lib` passed 17 ABM and 78
calibration tests (2 ignored); strict Clippy and workspace formatting also
passed. Full command strings, log paths and SHA-256 values are in the two result
receipts.

This candidate is not accepted as C2.3 dispatch. Independent review confirmed
that the tests use real `flow.step()` rejected receipts, retain exact carrier,
event, due, progress and priority, schedule one replacement, and reject replay.
The injected checked-time overflow remains true at the retry tick, so the
replacement is rejected again. A follow-up ABM-only test now uses a test
planner that rejects one start and one progress callback, then accepts their
explicit same-tick retries; it verifies movement, arrival and one actual timed
acquire. This exercises the ABM retry seam, not calibration's
`BoundIntrinsicWork` observation/retry/reconciliation path. C2.3 remains open
pending that integrated recovery proof, Track03 owner review and hosted gates.
The parent task does not mark C2.3, C2.2, C2.1, or C2.4 complete.

## Parent-pin integration recheck

After updating the parent pin to `a514590e485bfb5aa4df675d13940d98f62b0b7b`,
the coordinator reran all three gates from
`/Users/doughnut/Documents/careops-sim-session-worktrees/c2-implementation-20261006/libs/kairos`
using `rustc 1.99.0 (b940084d7 2026-09-28)` and `cargo 1.99.0
(5f94df478 2026-08-27)`. The locked ABM/calibration library tests exited 0
(17 ABM, 78 calibration, 2 ignored), strict Clippy exited 0, and workspace
`cargo fmt --all -- --check` exited 0. Logs and hashes:

- `/tmp/c23-parent-pin-tests.log` — SHA-256
  `15db8b3f8a443e254c0559c11802eb13400837279e93d5dbce160565b803057f`
- `/tmp/c23-parent-pin-clippy.log` — SHA-256
  `d7dae192715495fa89ccbe279af3efb9792c1dae1768228e369035e68a1df673`
- `/tmp/c23-parent-pin-fmt.log` — SHA-256
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`

## ABM successful-recovery seam supplement

Kairos source commit `aecb28c7089b1861b72cbd96ce29fc2577ffe47f` adds a
test-only planner in `kairo-ecs-abm` that rejects the actual start and first
progress callback once each, then delegates to `TransitContext::plan`. The
test invokes the explicit retry helper with the actual rejected dispatch,
checks the accepted replacements, retained progress, replay refusal, movement,
one arrival and one accepted timed acquire linked to the original service work.
It does not exercise calibration `BoundIntrinsicWork`, so it is supplemental
runtime evidence and does not accept C2.3.

The independent source reviewer found no remaining correctness or parallel
test isolation issue. The final integrated checks used Rust 1.99.0 at Kairos
`aecb28c`:

- `cargo test --locked -p kairo-ecs-calibration -p kairo-ecs-abm --features kairo-ecs-calibration/flow --lib` — exit 0; 18 ABM tests passed, 78 calibration tests passed and 2 were ignored. Log `/tmp/c23-recovery-oracle-final-tests.log`, SHA-256 `01249c0164daab6a4d572692c56676c00c3a70eb5706a1626f7acb5e65acb2b4`.
- `cargo clippy --locked -p kairo-ecs-calibration -p kairo-ecs-abm --features kairo-ecs-calibration/flow --all-targets -- -D warnings` — exit 0. Log `/tmp/c23-recovery-oracle-final-clippy.log`, SHA-256 `5fa56c356ca0f304456646462ee6f2c026d6742d2cf7581fc3105c738923398d`.
- `cargo fmt --all -- --check` and `git diff --check` — exit 0.

## C2.3.progress leaf acceptance at Rust 1.99 exact pin

The coordinator locally accepted the narrow `C2.3.progress` leaf at Kairos
`aecb28c7089b1861b72cbd96ce29fc2577ffe47f`. An independent read-only source
review confirmed that `TransitProgressState` checkpoint/restore retains the
immutable route, current edge, elapsed ticks and remaining duration. The actual
Flow pause/resume test also preserves in-flight segment progress across pause,
a stale progress event and resume, then reaches one arrival and one timed acquire.

Fresh verification at that exact pin used only the user-specified Rust 1.99.0:

- `rustup run 1.99.0 cargo test --locked -p kairo-ecs-calibration -p kairo-ecs-abm --features kairo-ecs-calibration/flow` — exit 0; 186 passed, 0 failed, 4 ignored. Log `/tmp/c23-progress-rust199-tests.log`, SHA-256 `242ee4c46370b3afb2c99ce6c7b6e78be417ee81ea43a2562d7654518dd3c55a`.
- `rustup run 1.99.0 cargo clippy --locked -p kairo-ecs-calibration -p kairo-ecs-abm --features kairo-ecs-calibration/flow --all-targets -- -D warnings` — exit 0. Log `/tmp/c23-progress-rust199-clippy.log`, SHA-256 `d7c4f31c9c81c01ecd74870967152f06ad4a97787d6a9f9f77ee447443658019`.
- `rustup run 1.99.0 cargo fmt --all -- --check` — exit 0. Log `/tmp/c23-progress-rust199-fmt.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

This accepts only the local progress leaf. Its checkpoint is an in-memory cursor,
not a portable or full Flow runtime checkpoint. The calibration-bound rejected
event recovery/arrival join, Track03 owner review, C2.3 parent, C2.2/C2.1 joins,
hosted exact-head checks and release gates remain open. Machine-readable receipt:
`acceptance-progress-rust199.json`.
