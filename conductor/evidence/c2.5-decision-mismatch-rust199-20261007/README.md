# C2.5 decision-mismatch preservation — Rust 1.99.0

The test-only oracle isolates the `valid_decision` rejection branch: it retains the valid same-runtime identity, changes only the bound Macro decision to Micro, and asks capture to reject. It checks `InvalidWork`, Flow identity/clock/progress/budget, the adapter’s original Macro decision and pending-policy boundary behavior, every `BoundIntrinsicWork` component, and next Service-stream draw continuity against a deterministic twin. No production or public API changed.

Kairos source commit: `3926d0f8f401b605ea62b15cde7b5ef1ffba89c1`. The receipt in `libs/kairos/.artifacts/mvp/C2.5.continuation.decision-mismatch/result.json` records the focused worker test, packet hash, source hash and correction history. The calibration-crate test boundary cannot clone/compare private adapter internals; adapter preservation is therefore proved through its available observable decision and pending-policy behavior, consistent with the bound packet. An independent read-only review confirmed branch isolation and bound-value coverage, and recommended additional Flow context/resource checks.

## Integration verification

Executed from `libs/kairos` at source commit `3926d0f8f401b605ea62b15cde7b5ef1ffba89c1` with `rustc 1.99.0 (b940084d7 2026-09-28)` and Cargo 1.99.0:

- `rustup run 1.99.0 cargo test --locked -p kairo-ecs-calibration --features flow,kairo-ecs-abm/test-support --lib` — 88 passed, 0 failed, 2 ignored. Includes C2.1 paired Restart and routed Suspend regressions; the two ignores require actual transport input evidence. Full output: [tests.log](tests.log).
- `rustup run 1.99.0 cargo clippy --locked -p kairo-ecs-calibration --features flow,kairo-ecs-abm/test-support --all-targets -- -D warnings` — passed.
- `rustup run 1.99.0 cargo fmt --all -- --check` — passed.

An initial test invocation from the parent repository root failed because Cargo features cannot be selected for a package outside its workspace. It changed no files; the exact command passed from `libs/kairos`. The post-dispatch `python3 tools/tasks.py packet-check .artifacts/packets/C2.5.continuation.decision-mismatch.json` returned input/source hash drift and base-commit drift, as expected after the dispatched source commit and parent integration advanced; it is not a new dispatch authorization. `python3 tools/tasks.py check`, `python3 tools/mvp.py check` and `python3 tools/context.py check` all pass after metadata synchronization.

The source SHA-256 is `6cb8218ff09ac51848cb449053256501e2f3c9b7ac39c0799367b5c424ec2ea4`; the bound packet SHA-256 is `ca44e183682e74c2791625c84e3e0443b4e05f5492cada217b2e698c8bd069b0`.

This is local continuation coverage only. Track03/Track22 owner decisions, hosted checks, portable checkpoint/restart, C2.4 phase review, release and clinical acceptance remain open.
