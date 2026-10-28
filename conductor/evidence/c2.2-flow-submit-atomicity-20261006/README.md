# Flow timed-submit cumulative-cap preflight — 2026-10-06

## Result

Integrated Kairos source commit `da323b69315365eb8f0bee092d4a78828e96a223`
into parent base `c55c692cba2c8d7d38d9629b18753aa312b057f9`. Timed Flow submit
now preflights the scheduler's cumulative event counter for the acquire and
optional deadline before allocating a `ResourceRequest` or mutating
`WorkSpec.request`. The scheduler's existing defensive check remains in place.

The regression sets a test-only lowered capacity limit and exercises the real
Flow submit path for a two-event acquire+deadline operation. It confirms
`CounterOverflow` with no request, work association, scheduler counter, created
or scheduled counter, command, or world mutation. It avoids constructing
billions of events to reach the production cap.

## Local verification

Commands ran in `${PRIVATE_HOME}/Documents/careops-sim-session-worktrees/c2-implementation-20261006/libs/kairos`:

- `rustup run 1.76.0 cargo test --locked -p kairo-ecs-des` — exit 0; 249 passed, 0 failed, 1 existing ignored across 27 test targets, including the new regression. Full output: `native-test.log`.
- `rustup run 1.99.0 cargo clippy --locked -p kairo-ecs-des --all-targets -- -D warnings` — exit 0. Full output: `strict-clippy.log`.
- `rustup run 1.99.0 cargo fmt --all -- --check` — exit 0. Full output: `format.log`.
- `git diff --check` — exit 0.

No hosted exact-head evidence has been collected. This resolves one Flow
failure-atomicity prerequisite; it does not implement the calibration bridge or
accept C2.2/C2.1.

## Current stable Rust 1.99 verification

The historical 1.76 test receipt above is retained as provenance but is not
used for the current acceptance. At integrated Kairos `aecb28c7089b1861b72cbd96ce29fc2577ffe47f`, a fresh read-only verification used **only Rust 1.99.0**: all 253 DES tests passed, 1 existing test was ignored, strict all-target Clippy passed and workspace formatting passed. The capacity regression passed on the exact integrated source. Raw logs and hashes are recorded in [the acceptance receipt](acceptance-rust199.json); the prepared packet and execution result remain under the ignored `.artifacts` directories.

An independent source audit confirmed the event-capacity calculation runs before
request/entity creation, event scheduling or work association mutation. It noted
that the regression specifically exercises the two-event acquire-plus-deadline
failure case; an acquire-only cap failure is not separately tested.

The coordinator accepted the `C2.2.flow-submit-atomicity` instance locally at
this exact pin. C2.2 as a whole remains incomplete; C2.3, C2.1, hosted checks and
checkpoint/release gates remain open.
