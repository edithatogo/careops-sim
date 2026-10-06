# C2.3 transit runtime progress — 2026-10-06

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
- Rust 1.76 is unverified for the exact-ID packet: `/Users/doughnut/.rustup`
  points into the absent `/Volumes/PortableSSD/toolchains` mount. No 1.76 pass is
  claimed. Hosted checks remain unrun.
