# C2.2 provider sampling tests — Rust 1.99.0

This receipt records additional unit coverage for the existing private intrinsic
work-duration provider. It does not change provider behavior or public APIs.

## Scope and source

- Kairos commit: `678453838f391f3dd869b79fd502077f63f3a067`
- Parent pin commit: `7b9e5bcb4e382959f08ee80a473400548d0d6a9e`
- Parent baseline during execution: `9896dd9ddd5bef614aabbd0e1522d9cbc2e2d2b5`
- Source: `libs/kairos/crates/kairo-ecs-calibration/src/work_duration.rs`
- Source SHA-256: `4c140e2b73334f6d6a0355ec53a3fe9138985029ebfaeb86da3ada9e9e3d15ef`
- Contract: `libs/kairos/conductor/design/calibration/c20-work-provider-v1.md`
- Contract SHA-256: `51ac6dc6783c280368dfd4630e4c33c5d585bf4c9657484e5e6c4ccb5c541333`
- Rust/Cargo: `1.99.0` only (`rustc 1.99.0 (b940084d7 2026-09-28)`;
  `cargo 1.99.0 (5f94df478 2026-08-27)`)

The tests cover the canonical weighted sequence and draw positions, ordered
support and the cumulative bucket boundary, fixed `u128::MAX` with no draws,
typed constructor/lookup errors with unchanged stream continuation, and real
stream rejection accounting. Independent review found no expected-value or
continuation mismatch. The contract's rejection followed by draw-position
overflow rollback remains covered by the existing seed-map test.

## Executed checks

Working directory: `/Users/doughnut/Documents/careops-sim-session-worktrees/c2-implementation-20261006`

| Command | Result | Log |
|---|---:|---|
| `rustup run 1.99.0 cargo test --manifest-path libs/kairos/Cargo.toml -p kairo-ecs-calibration work_duration::tests --locked` | exit 0; 7 passed | [focused tests](logs/work-duration-tests.log) |
| `rustup run 1.99.0 cargo test --manifest-path libs/kairos/Cargo.toml -p kairo-ecs-calibration --locked` | exit 0; 152 passed, 5 ignored | [package tests](logs/package-tests.log) |
| `rustup run 1.99.0 cargo clippy --manifest-path libs/kairos/Cargo.toml -p kairo-ecs-calibration --all-targets --locked -- -D warnings` | exit 0 | [Clippy](logs/clippy.log) |
| `rustup run 1.99.0 cargo fmt --manifest-path libs/kairos/Cargo.toml -p kairo-ecs-calibration -- --check` | exit 0 | [format check](logs/fmt-check.log) |
| `git -C libs/kairos diff --check` | exit 0 | passed before the Kairos commit |

The machine-readable command, artifact and hash record is [acceptance-rust199.json](acceptance-rust199.json).

## Boundary

This is additional C2.2 provider test evidence only. It does not establish
C2.1 paired runtime behavior, full C2 completion, portable checkpoint/restart,
or C2.4 phase acceptance. Track03/Track22 decisions and the hosted, release and
clinical gates remain open.
