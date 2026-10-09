# Flow checkpoint correction evidence

- Worktree: `/Users/doughnut/Documents/careops-sim-session-worktrees/kairos-c2-full-flow-image-20261009`
- Base: `9015ce09f52d7cb5e61f092fd532ab65db54889e`
- Rust toolchain: `1.99.0-aarch64-apple-darwin` (`rustc` and `rustdoc` explicitly selected)
- Cargo home: `/tmp/c2-route-receipt-cargo-home`
- Isolated target: `/tmp/c2-full-flow-corrections-target`

## Verification

Full five-package joined feature suite: 574 passed, 0 failed, 5 ignored across 58 test-result rows. Command:

`cargo test --locked -p kairo-ecs-des -p kairo-ecs-core -p kairo-ecs-state -p kairo-ecs-abm -p kairo-ecs-calibration --features kairo-ecs-calibration/flow,kairo-ecs-abm/test-support`

Log: `full-join-test.log`.

Strict joined all-target Clippy, exit 0:

`cargo clippy --locked -p kairo-ecs-des -p kairo-ecs-core -p kairo-ecs-state -p kairo-ecs-abm -p kairo-ecs-calibration --all-targets --features kairo-ecs-calibration/flow,kairo-ecs-abm/test-support -- -D warnings`

Log: `full-join-clippy.log`.

Workspace format check `cargo fmt --all -- --check` and `git diff --check` both exit 0.

Focused tests exercise first-dispatch and cross-tick budget halts produced by Flow itself, exact budget snapshot roundtrip, rejection of a non-head live scheduler event as the halt preview, and restart manifest key-byte caps during capture and restore. Actor pending-despawn membership now uses an already bounded set.

This is native Flow checkpoint evidence, not full C2 acceptance.
