# Q4 typed continuation and domain batch qualification

## Bounded development acceptance

The internally reviewed Track 03/25 runtime slice is qualified at Kairos development commit `d742df7e390317a6e7a2c22d797927c6c2ee0c18` on `codex/careops-q4-typed-continuations-runtime`. It is a descendant of the existing qualified same-tick budget pin `358bb156c638e3bc5a67129e306ee0ae65d6f61d`. The source, frozen fixture and contract were reviewed before execution. This evidence accepts this slice, not the whole Q4 phase or a stable release.

The concrete contract is `libs/kairos/conductor/design/flow/q4-typed-continuations-v1.md`. The implementation adds safe typed continuation/domain registration, live-context delivery, legacy-first notification order, opaque per-batch tickets and bounded whole-batch command admission. It retains one authoritative scheduler/world/registry, checked preview admission and persistent same-tick fail-stop behavior. No portable codec, binding, core ordering or RNG algorithm change is included.

A callback executes once after admission. Its owned context effects remain even when its command batch is rejected. Commands are validated together before entity IDs, work associations, release reservations or scheduler tokens are committed. The callback cap defaults to 1024 and is independently configured without changing existing `FlowConfig` literals. Internal arithmetic exhaustion is checked before consumption or partial mutation. User panics and allocation failure are not transactional rollback guarantees.

## Executed local qualification

Exact source: `crates/kairo-ecs-des/src/flow.rs`, SHA-256 `97fdaf2b281b572864c55fadbd3b654e722f566fc8d626d08bec373557642856`.

Working directory: `/private/tmp/kairos-q4-continuation-runtime-v1`, parent source commit `7230485f610f298be90260799473d4f68252c8ae`. Actual selected compiler binaries were hashed and called explicitly; `RUSTC`, `RUSTDOC`, `PATH`, owned absolute `TMPDIR`, cache and targets are in the retained command receipts. Executed `cargo test --locked --offline -p kairo-ecs-des` once successfully with Rust 1.88.0 and once with Rust 1.98.1. Each passed 106 tests: 50 unit tests and 56 integration tests, with zero failures and zero doc tests. These include all ten private fault families and all eight frozen public continuation tests. Scoped formatting and whitespace checks passed.

The first Rust 1.88 attempt compiled, then passed 49 of 50 unit tests and stopped before integration tests. A newly authored private test inferred `Cell<i32>` while its registered callback required `Cell<u32>`, causing `InvalidWork` during setup. The approved correction explicitly typed that private cell; production and frozen fixtures were unchanged. The failed attempt is retained, not represented as a pass.

The private cases cover batch identity exhaustion, cumulative entity and scheduling admission, deadline reservations, past/foreign/forward/non-Acquire references, duplicate release reservations, aggregate legacy/new token preflight, ignored cap poison, stale context/work delivery, defensive missing registration, captured completion/restart context and actual primary/deadline receipt IDs. The scheduler count exhaustion case injects a pure planning-helper argument; it does not mutate core scheduler counters. Generated queue/preemption checks remain bounded generators, not exhaustive verification.

Retained local receipts and hashes:

- `/private/tmp/kairos-q4-continuation-runtime-v1/.artifacts/q4-continuation-runtime-v1/final-result.json`: `b67cd3a458be4f13b268b2e3d8ee5160c8aa91bfc4b977f51aafcf2a2c5cd4e4`.
- `des-full-1.88.0-corrected.log`: `84ca7b76506b0d732fde4ab9da333ed21ed2a086acbe257348bba01ebed2aaf0`.
- `des-full-1.98.1-corrected.log`: `782a635aab7eb4daef192bc7cac28b5327e7fa35d2527be61bc418c8bae702a7`.
- `attempt1-des-full-1.88.0.log`: `e9ebcfdc44c3644c21e076329292f013968f099b42f5ecdffd83b90cfcae9303`.

## Exact-head hosted owner qualification

[CareOps native owner run 37145949301](https://github.com/edithatogo/kairos/actions/runs/37145949301) succeeded at the exact child commit. Ubuntu 24.04 job `111269798837` verified the actual `x86_64-unknown-linux-gnu` compiler/host. macOS 15 job `111269799041` verified `aarch64-apple-darwin`. Both used actual Rust 1.98.1 and passed the reusable core/types/state/RNG/DES/ABM/Arrow/calibration package lane, including all 106 DES tests and every named new private/public continuation test. Both also passed the optional calibration Rust 1.88 floor lane. These are executed jobs, not skipped native-success claims.

Retained hosted artifacts are under `/private/tmp/kairos-q4-owner-ci-receipt-20261004/.artifacts/q4-owner-ci-proof/`:

- `hosted-result.json`: `3fe3cff852b8a876ba74edb9b96ada11e18551233760f566bd6d29956cd91c56`.
- `run.json`: `ddc4adfed3b7e1fd65f91a8f0d81b7b78e1532c5990471d47519ea20be9a14f1`.
- `111269798837.log`: `78243ccd496f47215560edae6b1328613768995c6f3e51784a2eeaf6b99dd2f7`.
- `111269799041.log`: `9c08d56691f316edbccf4e718005cf5c8202e9d2e1629f31c3b9bb6eadaf3f4f`.

Root independently reviewed the source, immutable inputs, exact receipts and raw logs. That is internal development acceptance, not an external maintainer signature.

## Parent integration and remaining gates

The parent contract retains all 23 unchanged existing source bindings, updates only `flow.rs`, and adds the frozen continuation contract and fixture: 26 bindings total. Parent integration acceptance is governed by exact-head hosted required checks, review gates and PR merge readback. The existing recursive-checkout context lane verifies the committed pin and contract; a pin check before the source commit is not a valid committed-head result. No extra local parent native suite was repeated for this metadata integration.

All Q4 checkboxes stay pending. Remaining joins include declarative builder/migration coverage, shared DES/ABM adapters, `resource_lifecycle.v1` telemetry under Track 04 while preserving `event_log.v1`, and the synthetic staff/bed/cleaning workflow. Q5, full C1/C2 and remaining D3/D4 retain separate evidence and acceptance. Portable checkpoint work stays with Track 22.

Public error variants and dispatch receipt fields retain the experimental source-compatibility and release hold. There is no universal Rust semver compatibility claim, stable API waiver or full Rust 1.76 Cargo dev-suite qualification. The push reported two high default-branch vulnerability alerts; successful runtime CI does not close or dismiss them.
