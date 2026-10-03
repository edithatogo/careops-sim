# Q4 scheduler preview qualification — 2026-10-03

## Accepted development capability

The qualified Kairos development source is
`132fb4d525c2bd7b110fc11548c8d1b5fdf9a648` on
`codex/careops-q4-scheduler-preview`. It descends from the previously qualified
Q3 pin `237c2c08d04cf038ed324c272ffbdb63dae6f57b`.
Exactly three paths differ: the Q4 experimental runtime contract, seven preview
fixtures, and 28 added lines in core lib.rs.

The copied six-field `ScheduledEventPreview` and `Scheduler::peek_next`
project the next live event after existing cancellation pruning. Preview does
not dispatch, advance time, change counters or remove live pending membership.
Existing step, schedule, cancel, facade, types, state and RNG are unchanged.
The coordinator independently reviewed the concrete source and accepted this
bounded internal Track 01B/03/25 experimental disposition. It is not an external
maintainer signature, an ABI/binding change or full Q4 acceptance.

Contract SHA256:
`1224c42a1c83270a2bfe31d64f40fcedb22a5065474958016853856a24c63580`.
Core source SHA256:
`5f3798c0b8a3f3ac328eae91abdb4fa780a3e103958f01215fff8e3ee7404e3d`.
Fixture SHA256:
`a588b487bca7ee95787432383dc453c8cb83fd0c88dcee55829024257cd2d1db`.

## Test-first lineage and local execution

The contract was committed as `7edfb184`, then the test-only source as
`257e15a4`, before core implementation commit `132fb4d5`.

Actual Rust 1.98.1 and Rust 1.88.0 Cargo fixture compilation each exited 101
with exactly four missing preview type/method errors. No assertion ran during
these compile-red checks. The original Cargo Rust 1.76.0 attempt stopped before
fixture compilation because locked proptest 1.11.0 requires Rust 1.85 or newer.
That attempt is a retained development dependency qualification gap, not
accepted red or green evidence.

A separately reviewed Rust 1.76.0 lane built core/types libraries, selected
exact rlib paths from Cargo JSON, verified library target kinds and compiled
the fixture with explicit extern paths. Its genuine missing-API compile red
was retained before implementation. The first artifact wrapper attempt failed
because Cargo target names were hyphenated; both that attempt and the corrected
JSON parser receipts remain available. No lock or source workaround was used.

After implementation, scoped rustfmt passed. Actual Rust 1.88.0
`cargo test --locked -p kairo-ecs-core` passed 39 tests: 17 unit, 4 conformance,
8 integration, 7 preview and 3 property tests, with zero failures. There were
zero doctests. Actual Rust 1.76.0 library build, direct fixture compile and
seven-test binary each exited zero. This direct lane is not qualification of
the full Cargo Rust 1.76 development dependency suite or a release waiver.

The seven fixtures check all DTO fields, complete SchedulerStats, repeat
preview, exact next dispatch identity, time/priority/FIFO ordering including
u128::MAX and priority extremes, cancelled heads/future tails, and changes
after cancellation or scheduling an earlier event.

Retained local receipt:
`/private/tmp/kairos-q4-preview-implementation/.artifacts/q4-preview-qualified/implementation-result.json`,
SHA256 `dff0ff8de5837deb7915d97624e9a27bf6448b9d60931267362ebda74746c468`.

Retained compile-red follow-up:
`/private/tmp/kairos-q4-preview-qualified/.artifacts/q4-preview-qualified/followup-result.json`,
SHA256 `edb2f46963963a22a20de87111f9dd1e680b842ed8e23a6ee88c057af0ed0274`.

Original Cargo Rust 1.76 gap:
`/private/tmp/kairos-q4-preview-fixtures/.artifacts/q4-preview-fixtures/compile-1.76.0.log`,
SHA256 `986bca36e81d83b36de27ffbc0a445f9ccc9825eafaa19a2b10a93483ff5bf2c`.

## Exact-head hosted owner evidence

[CareOps native owner run 37127023617](https://github.com/edithatogo/kairos/actions/runs/37127023617)
completed successfully at exact head
`132fb4d525c2bd7b110fc11548c8d1b5fdf9a648`.

| Host | Job ID | Qualification |
|---|---|---|
| macos-15, aarch64-apple-darwin | 111214275524 | Native owner packages and optional calibration floor passed; all seven preview tests ran and passed. |
| ubuntu-24.04, x86_64-unknown-linux-gnu | 111214275699 | Native owner packages and optional calibration floor passed; all seven preview tests ran and passed. |

The unchanged push workflow uses the reviewed actual Rust 1.98.1 compiler for
reusable native packages, including core, and verifies each actual host. It
uses Rust 1.88.0 for the optional calibration floor. Both job logs explicitly
contain the scheduler_preview_v1 test binary and all seven named tests marked
ok; qualification is not inferred from overall workflow green alone.

Retained directory:
`/private/tmp/kairos-q4-preview-publication/.artifacts/q4-preview-publication/`.

- hosted-result.json SHA256:
  `cfc41e207f8d0376cb38a10c2837df35c2a1a3e5168f38d766e6538a52c35c95`.
- macos-15.log SHA256:
  `d6f4c8bfcc73c778be9db93f8cd29d75f5109eb0901fc41f3ef10d5aa8dc7d94`.
- ubuntu-24.04.log SHA256:
  `c1a64982b20bcb8cf7bbcd34e088dd0bea897c5781cd270a81cb714ebe948c5e`.
- run.json SHA256:
  `3551d168bc6cda60d554d1fba95c24ae88fe9582ac3d8918237c435947dbb726`.

## Parent integration and remaining holds

Parent source base is `8725b00c`. The integration changes only the Kairos
gitlink, current-state submodule pin, D2 qualification contract and this evidence.
Twenty existing contract hashes remain unchanged; only the core source hash
changes, and the new contract/fixture hashes extend the list from 21 to 23.

Parent integration and acceptance are governed by exact-head hosted checks and
PR merge readback. Local context/diff checks and post-commit HEAD pin validation
are retained separately; this document does not claim a parent merge by itself.

Q4 remains incomplete and unaccepted as a phase. This preview is a prerequisite
for Flow budget planning; no Flow budget runtime implementation is accepted.
Persistent default 100000/fail-stop semantics are frozen for later work, while
concrete inspection API and fixtures require their separate reviewed packet.
Callback batch types, general domain hooks, shared DES/ABM adapters and Track04
lifecycle export remain separate joins. Q5, C1/C2/C4, D3/D4, portable checkpoints
and full release readiness are not completed by this integration. Existing
experimental source-compatibility and release holds remain.
