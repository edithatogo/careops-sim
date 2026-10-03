# Local compiler-resolution evidence correction

On this host Homebrew Rust1.99.0 preceded rustup toolchains in PATH. Selecting
Cargo with rustup run did not always select its child rustc. Earlier Q1/Q2/C1
Cargo-only local MSRV/current logs remain preserved but do not prove the claimed
child compiler version. Native hosted exact-commit owner CI remains separate.

Superseding proof at Kairos5f5a8d9cf5a1ce3ece05312f41f489f8554a115a:
explicit Rust1.76 RUSTC/RUSTDOC/PATH, locked kairo-ecs-des and kairo-ecs-arrow
tests:59 passed. Actual rustc1.76.0(07dca489a2024-02-04) confirmed from
target/.rustc_info.json. Selected legacy DES/Arrow slice only, not whole workspace.
Log: active parent .artifacts/blocker-resolution/q2-c1-actual-msrv.log.
The optional seed-map package does not change engine crates' Rust1.76 floor;
its own Rust1.88 qualification is recorded separately when complete.

D3 E0's four regressions subsequently passed with actual Rust1.98.1 explicit
compiler/documentation paths. Final parser fuzz passed with actual dated nightly
1.101.0-nightly. Corrected receipts include executable paths, not only Cargo argv.
