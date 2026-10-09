# Scheduler checkpoint invariant correction, attempt 2

- Base HEAD: `1d1de4d3fda05bf165481de4830a3ca6ba40368f`
- Working tree: `/Users/doughnut/Documents/careops-sim-session-worktrees/kairos-c2-scheduler-checkpoint-20261009`
- Final source commit: `f079f57404194518cd2fb827b607c208c1a60b59` (tree `1bd7fa3469e326b38999a631e8b46581eabb3ff9`).
- Toolchain: Rust 1.99.0 (`rustc 1.99.0 (b940084d7 2026-09-28)`), Cargo 1.99.0 (`5f94df478 2026-08-27`).
- Environment: PATH and RUSTC/RUSTDOC point to `/Users/doughnut/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin`; `CARGO_HOME=/tmp/c2-route-receipt-cargo-home`; `CARGO_TARGET_DIR=.artifacts/c2-scheduler-checkpoint/target`.
- Input hashes: ADR-0016 `28f16f0b97d3b76942c71361aa27f6ffd8183c6d4aba41a9462c17e023eefb7d`; Track01 contract `a4c3ca5966d314d994f49fc47fa995fdd168acc0d53f1fe87dd5f511eaa6fbf3`.
- Attempt-1 logs remain unchanged in the parent directory. Their hashes were rechecked before recording this attempt.

## Verification

- TDD red: `cargo test -p kairo-ecs-core --test scheduler_checkpoint_v1` failed only because import accepted nonzero logical time with zero dispatched events. `red.log`, SHA-256 `d773b484b566f0cbc916d9edeedeead25ef5e0fa6c4d155b6703c009a812f6fb`.
- A first red invocation had an incorrectly constructed synthetic near-wrap fixture; corrected the fixture, retaining that raw diagnostic separately as `red-invalid-fixture.log`, SHA-256 `8da0222e700fce602617107c5ae879c98b783e9b70009feebeefa695032f2a1b`.
- Focused scheduler checkpoint tests: 5 passed, 0 failed. `focused.log`, SHA-256 `f3c31dfe7b8e63e530568a7a7ffb2fd9e316c074366a8929f98b51363bab9e2c`.
- Full `cargo test -p kairo-ecs-core`: 44 passed, 0 failed, 0 ignored across 6 result groups. `core-tests.log`, SHA-256 `55c542ff6adde26977971337920f7a3fc505825c4a8bf0584674cbaede9d67b5`.
- `cargo clippy -p kairo-ecs-core --all-targets -- -D warnings`: exit 0. `clippy.log`, SHA-256 `6f94ac50a0dd36638507c244b8202055a269bd88af0a263cb18ce955b1687f85`.
- `cargo fmt --all --check`: exit 0. `fmt.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `git diff --check`: exit 0.

Full checkpoint acceptance remains open; this is scheduler component evidence only.
