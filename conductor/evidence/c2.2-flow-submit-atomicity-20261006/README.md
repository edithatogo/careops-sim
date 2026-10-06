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

Commands ran in `/Users/doughnut/Documents/careops-sim-session-worktrees/c2-implementation-20261006/libs/kairos`:

- `rustup run 1.76.0 cargo test --locked -p kairo-ecs-des` — exit 0; 249 passed, 0 failed, 1 existing ignored across 27 test targets, including the new regression. Full output: `native-test.log`.
- `rustup run 1.99.0 cargo clippy --locked -p kairo-ecs-des --all-targets -- -D warnings` — exit 0. Full output: `strict-clippy.log`.
- `rustup run 1.99.0 cargo fmt --all -- --check` — exit 0. Full output: `format.log`.
- `git diff --check` — exit 0.

No hosted exact-head evidence has been collected. This resolves one Flow
failure-atomicity prerequisite; it does not implement the calibration bridge or
accept C2.2/C2.1.
