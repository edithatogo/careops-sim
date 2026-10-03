# Q4 builder and FIFO migration qualification

## Bounded development slice

Kairos development commit `60d5f4e9d6755f6b2731c8a87540ec1a14b1b85b` on `codex/careops-q4-builder-migration` retains the accepted typed continuation runtime `d742df7e390317a6e7a2c22d797927c6c2ee0c18`. Its exact three-path delta is a new public builder/migration test, a runnable DES example and the Flow README update. No production, manifest, lockfile, core/state/RNG or CI source changed.

The builder already existed. The new independent fixture verifies its current default and explicit owner/time/work/priority/deadline/preemption policy translation; missing-owner, wrong-owner, manual preemptibility, duplicate association and past-time rejection; scheduler admission priority distinct from resource queue priority; exact deadline/completion boundary in both token insertion orders; equal-priority FIFO migration and actual buffered lease release; and same-runtime event-boundary pause versus uninterrupted causal records with owned context.

The Suspend interval oracle is low work [0,3) and [5,12), urgent work [3,5): low useful/busy effort 10 ticks, remaining zero, and a two-tick waiting interval. No component attachment, context cloning or portable restore is used. Failed admission compares work association and the public full scheduler/budget snapshot. Flow does not expose its heap head publicly, so this fixture does not claim heap-identity inspection.

The FIFO example keeps synthetic legacy entity IDs separate from actual Flow actors and compares label order only. Legacy Resource request/release is synchronous; Flow returns pending requests and requires dispatch to commit grants and actual lease releases. The example asserts a released lease cannot be reused. It does not assert identical IDs, return shapes or event counts between the APIs.

## Executed local evidence

Commit `b986c5011f2c41e774147bf2b80109979a93de30` adds only `crates/kairo-ecs-des/tests/flow_builder_migration_v1.rs`, SHA-256 `e14865e211794e1700b7ca351dc6eefb9e5a1974e40b4feb600941c9816d3f09`. Actual explicit Rust 1.88.0 and 1.98.1 each executed `cargo test --locked --offline -p kairo-ecs-des --test flow_builder_migration_v1`: six tests passed on each compiler, no failures. The frozen draft was copied byte-for-byte. Production and all other immutable inputs were rehashed unchanged before and after.

The local receipt is `/private/tmp/kairos-q4-builder-fixtures-v1/.artifacts/q4-builder-fixtures-v1/result.json`, SHA-256 `f19a8cb00af550e1359cd7b87e5d0649c6cc0a985916cbbb805d6b9672d498d8`. Raw Rust 1.88 log SHA is `bbb6a3d21c7dd13a332e6751444e1af7e7ced8ed941c98044c3ec54d69d198ac`; Rust 1.98 log SHA is `eb4ef0643a23acd177c986164893bc6fcb050ed0e9b0340fb9d719dd47623477`. A receipt helper initially expected exit zero from `git diff --no-index --check` for a new file. That command returned one with no whitespace findings because the files differ; the expectation was corrected and the actual staged whitespace check passed. No test or source correction followed that receipt issue.

Commit `60d5f4e9d6755f6b2731c8a87540ec1a14b1b85b` adds only the example and README. The example SHA is `3479886e4109f2d15246931096197668a2abd6968374a8d06360523c70b1668f`; README SHA is `c16e8bb53dc02ef9375fc4c0023a1da85842c7c8b703c216c89f6b4bacbca241`. Actual Rust 1.88 and 1.98 executed `cargo run --locked --offline -p kairo-ecs-des --example flow_fifo_migration`: both exited zero with exactly these three stdout lines:

```text
Legacy FIFO: first -> second -> third
Flow FIFO: first -> second -> third
Flow commands commit at dispatch; released leases cannot be reused.
```

The local example receipt is `/private/tmp/kairos-q4-fifo-example-v1/.artifacts/q4-fifo-example-v1/result.json`, SHA-256 `bfcd73476197e9131e7176c4822818eefea273830129165cca71489b21e9f9fa`. Exact executable hashes, selected RUSTC/RUSTDOC/PATH, working directories, owned absolute TMPDIR/cache/targets, UTC times, exits and separate stdout/stderr hashes are retained in both local receipts. Scoped formatting passed. This is targeted qualification, not a newly executed full DES suite on both local toolchains.

## Exact-head hosted owner evidence

[CareOps native owner run 37148199275](https://github.com/edithatogo/kairos/actions/runs/37148199275) succeeded at exact child `60d5f4e9d6755f6b2731c8a87540ec1a14b1b85b`. macOS 15 job `111276373189` verified actual Rust 1.98.1 on `aarch64-apple-darwin`; Ubuntu 24.04 job `111276373407` verified `x86_64-unknown-linux-gnu`. Both passed the native reusable package lane and optional calibration floor. Each actually passed 112 DES tests: 50 unit plus 62 integration tests. All six new builder test names are present as passed. The default Cargo test lane compiles the example; it does not claim hosted example execution. Actual example execution is the separately retained local proof above.

Retained hosted directory: `/private/tmp/kairos-q4-builder-owner-proof-20261004/.artifacts/q4-builder-owner-proof/`.

- `result.json`: `3dbf08c8f08a12bc9a9e2005efae42f11d430713a48b2880e96bf8573369c81d`.
- `DES-counts.json`: `ff70de4b234fd0951dd72472be1f1a3fb7efaed7d7d9cbb966598cca8ce919de`.
- `111276373189.log`: `998e7132ab4799e0b43226885a4fc3b53faf68527b6eab7e3cdc8d3e477380d7`.
- `111276373407.log`: `33219814b0ed248437c0db0f46ee50539a0aa409781e17627d6ac8e98fcb6351`.

Root independently reviewed the source/oracles, immutable hashes and actual raw qualification evidence. That is bounded internal development acceptance, not an external maintainer signature.

## Parent acceptance and remaining barriers

All 26 prior source bindings are unchanged; the fixture, example and README add three bindings, for 29 total. The parent retains the same Q4 active phase and pending checkboxes. The PR36 recipe alignment is preserved untouched. Exact-head required hosted checks, review gates and native PR merge readback govern parent integration acceptance. A committed-head pin check belongs after the reviewed source commit, not at the old HEAD. No extra parent native suite is repeated locally for this metadata-only pin integration.

Full Q4 remains incomplete. The shared ABM adapter needs a concrete borrowed world/time, RNG and entity contract: the existing ABMContext owns a separate scheduler/world/registry, and its constructor seed does not establish the needed stream contract. Lifecycle encoding needs Track 04 review of captured per-transition resource/work snapshots and final schema; current LifecycleRecord fields and final-state lookups do not satisfy that snapshot requirement. The staged staff/bed/cleaning synthetic workflow and phase review remain separate joins.

Portable checkpoint/resume remains deferred to Track 22. Experimental public API source-compatibility and release holds remain; there is no stable compatibility waiver. Q5, full C1/C2 and remaining D3/D4 and security qualifications stay separate. The default-branch push advisory summary of two high vulnerabilities is not runtime security closure.
