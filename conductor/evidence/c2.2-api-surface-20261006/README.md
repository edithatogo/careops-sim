# C2.2 experimental fidelity API surface — 2026-10-06

## Result

Integrated Kairos commit `40da7d8c726a5ef4122daeeeb6bce622fd523a5c` into
parent base `b9eb1b4fbd848738c587466c085b4eeb68609095`. The change exposes
`kairo_ecs_des::fidelity` as an experimental public Rust module, keeps the
existing signatures and behavior, records exact signatures/compatibility and
failure limits, and adds downstream public-import tests.

Independent read-only API review found no blocking design objection to this
narrow additive surface. The review's seven objections and dispositions are
recorded in the API review document in the pinned Kairos commit. This is local
experimental API qualification only; it does not claim stable release,
checkpoint, FFI/bindings, Flow bridge, full C2.2/C2.1, hosted checks, or clinical
acceptance.

## Exact local evidence

Commands ran in `${PRIVATE_HOME}/Documents/careops-sim-session-worktrees/c2-implementation-20261006/libs/kairos`:

- `rustup run 1.76.0 cargo test --locked -p kairo-ecs-des` — exit 0; downstream `fidelity_public_api` tests 2/2 passed. Full output: `native-test.log`.
- `rustup run 1.99.0 cargo clippy --locked -p kairo-ecs-des --all-targets -- -D warnings` — exit 0. Full output: `strict-clippy.log`.
- `rustup run 1.99.0 cargo fmt --all -- --check` — exit 0.
- `git diff --check` — exit 0.

The test and Clippy logs were captured under `/tmp/careops-c22-fidelity-api-target`.
No hosted exact-head evidence has been collected.

## Next join

The next implementation candidate is C2.2 Macro/Zero Flow bridging. Route transit
remains gated by C2.3 dispatch. C2.2 seeds/checkpoint/resume and C2.1 paired
runtime acceptance remain open.
