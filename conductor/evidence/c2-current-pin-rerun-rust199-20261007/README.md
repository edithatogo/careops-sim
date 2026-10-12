# C2 combined current-pin verification — Rust 1.99.0

Parent commit: `0528d7fdab323aebd0d86daa9c24f7d3c04b8f39`. Exact Kairos pin: `3926d0f8f401b605ea62b15cde7b5ef1ffba89c1`. This refreshes the local DES/ABM/calibration package suite after the C2.1 and C2.5 test-only additions.

The three package suites completed **449 passed, 0 failed, 5 ignored across 43 test groups**. Strict all-target Clippy (`-D warnings`) and workspace formatting passed. The captured test log includes `actual_flow_urgent_suspend_resumes_same_macro_and_routed_micro_service`, the paired C2.1 Restart/Suspend oracles, and the C2.5 decision-mismatch test. The five ignored tests have explicit physical transport evidence or separately commissioned timing prerequisites; no ignored case is counted as pass.

The current exact-pin run also exercises the C2.2 provider, fidelity policy, seed, and Flow APIs alongside C2.3 route/resource behavior. It refreshes integrated compatibility evidence but does not replace each task’s independent leaf receipt or establish its semantic acceptance.

## Commands

All commands ran from `libs/kairos` with `rustc 1.99.0 (b940084d7 2026-09-28)` and Cargo 1.99.0. The packet and binding are retained in this directory.

- `rustup run 1.99.0 cargo test --locked -p kairo-ecs-des -p kairo-ecs-abm -p kairo-ecs-calibration --features kairo-ecs-calibration/flow,kairo-ecs-abm/test-support --target-dir .artifacts/mvp/C2.4.q4-urgent-interruption-ledger.q4-rerun-3926d0f/target` — [test log](test.log), SHA-256 `de9380cce092550836f55bd157bc2b34201b338c842341c9004409ba0f98e414`.
- `rustup run 1.99.0 cargo clippy --locked -p kairo-ecs-des -p kairo-ecs-abm -p kairo-ecs-calibration --features kairo-ecs-calibration/flow,kairo-ecs-abm/test-support --all-targets --target-dir .artifacts/mvp/C2.4.q4-urgent-interruption-ledger.q4-rerun-3926d0f/target -- -D warnings` — [Clippy log](clippy.log), SHA-256 `1dc5ceff6278beb31dd35e86250ed1b02b0bcad00ac3fd2865e43a4d05af493c`.
- `rustup run 1.99.0 cargo fmt --all -- --check` — [format log](fmt-check.log), SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

A wrapper that assigned the shell’s read-only zsh variable `status` returned exit 1 after the test process had finished; the variable was corrected to `test_rc` and the exact test command rerun successfully with its full log retained. No source changes occurred.

## Disposition

This is local current-pin verification only. C2.4 phase acceptance remains withheld. Track03/Track22 owner packets remain prepared but unsent; hosted checks, portable process restart, release, and clinical acceptance remain open. No owner semantics were inferred and no phase checkbox was changed.
