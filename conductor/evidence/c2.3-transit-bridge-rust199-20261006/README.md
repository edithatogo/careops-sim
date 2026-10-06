# C2.3.transit-bridge local acceptance — Rust 1.99

The coordinator locally accepted one exact-pin `C2.3.transit-bridge` instance at Kairos commit `7672a67988134438228014e7b3e0f56855cc65e8` on `codex/c2.3-transit-bridge-rust199`. The reviewed packet and worker result are copied beside this receipt. An independent packet review checked its prerequisite evidence, hashes, limits and commands; a separate read-only source review found no correctness or scope issue.

The implementation adds a non-default `kairo-ecs-abm/test-support` feature and hidden helper. The test-only helper rejects the first actual transit planner callback on its thread; later calls delegate to the existing planner, and the real accepted-receipt callback remains active. The calibration test drives `BoundIntrinsicWork` through the actual scheduler-produced rejected receipt, exact retry, replay refusal, accepted retry, arrival, and actual timed request. It checks the original work/resource/owner association and the exact owned/consumed event IDs, including the distinct Acquire event. It then verifies `finish_transit` returns that work and request.

The coordinator reran each packet command from `libs/kairos` using only Rust 1.99.0 (`rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0 (5f94df478 2026-08-27)`):

- `rustup run 1.99.0 cargo test --locked -p kairo-ecs-calibration -p kairo-ecs-abm --features kairo-ecs-calibration/flow,kairo-ecs-abm/test-support` — exit 0; ABM 30 passed; calibration 157 passed; 4 ignored. Log `/tmp/c23-transit-bridge-rust199-coordinator-test.log`, SHA-256 `8d9f712afeae622293881dbcb82394c17e4b17de36db66339f8269fc6892ec05`.
- Strict all-target Clippy with the same packages/features and `-D warnings` — exit 0. Log `/tmp/c23-transit-bridge-rust199-coordinator-clippy.log`, SHA-256 `beed191cf5cd39a57477a6d66e9acb53f3b4efe7566a30e0ada805357a0fe6ff`.
- `rustup run 1.99.0 cargo fmt --all -- --check` — exit 0. Log `/tmp/c23-transit-bridge-rust199-coordinator-fmt.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

This accepts only the local transit-bridge leaf instance. Track03 owner review is still open and required for C2.3 phase acceptance. C2.3, C2.2 and C2.1 parent joins remain open; hosted exact-head checks, portable/full-runtime checkpointing, release and clinical acceptance are not established. See `acceptance-rust199.json` for source hashes, exact commands and scope limits.
