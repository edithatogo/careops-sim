# A4.4 transfer review fixes

Status: fixes implemented and local gates pass; integrated exact-commit phase review remains pending. Packet: `conductor/execution/packets/A4.4.transfer-fix.json`.

## Review findings addressed

- `TransferManager::apply` now applies clock advancement, expiry processing and command handling to a cloned ledger and commits that clone only after a successful result. A future-tick invalid transition therefore leaves the complete checkpoint unchanged, including virtual time and unrelated expiries. Valid commands still process expiry before same-tick command effects.
- V1 now rejects `DestinationRejected` on a compensation operation. This deliberately bounds compensation to one child level until chain completion and ancestor claim-release semantics have a versioned contract. The rejection is atomic; the existing one-level diversion still completes and releases the root claims.

## Verification

Working tree base: `bc1dbcb58ef8fe099f23a9393f3b193ddccb98d0`; code edits were run in a clean isolated worktree at that base. Submodule `libs/kairos` was initialized at pinned `44261eb5cf8275f132f63a3396903d116d701dbe`; no pin or Kairos source changes.

Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`, `aarch64-apple-darwin`.

| Command | Result | Captured stdout+stderr SHA-256 |
| --- | --- | --- |
| `cargo test --locked -p careops-core --test transfer_contract --test ward_community_contract` | pass: 25 transfer + 10 ward/community tests | focused run passed; output captured in tool session |
| `cargo clippy --locked -p careops-core --all-targets -- -D warnings` | pass | output captured in tool session |
| `cargo test --locked --workspace` | exit 0; all workspace tests and doc-tests passed | `d62a836cd913b2c6bd49708c1c089bc3a26ddad4398da6bdfdf60870dff081ae` (98,850 bytes) |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | exit 0 | `4767d75f9409e57cb822869d4012024794df038e7c695785197d047f3f4cab3a` (1,902 bytes) |
| `cargo fmt --all -- --check` | exit 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` (empty output) |
| `git diff --check` | exit 0 | no diagnostics |

The two focused regressions are `invalid_future_tick_transition_does_not_commit_clock_or_unrelated_expiry` and the nested-diversion assertion in `post_departure_rejection_creates_compensation_without_duplicating_person`.

## Agent review and boundaries

Reviewer agent `/root/a44_transfer_fix_review` reviewed the code diff against ADR-0012 and reported no findings. It confirmed atomic rollback and retained one-level completion; it also identified nested compensation as explicitly unsupported. The integrated reviewer `/root/a44_integrated_review` initially found the two issues addressed above. A fresh exact-commit integrated review remains required before A4.4 acceptance.

SHA-256: `crates/careops-core/src/transfer.rs` = `37ce4b034f182c12a67ceca54fd55e92a78c90c869ff50bd71b848d36e4ca9c8`; `crates/careops-core/tests/transfer_contract.rs` = `6c57463658dd2d280315a0050ea1eb3775edc0972a83dd87bf6ff8923ffe5632`.

No clinical validation, real-service acceptance, nested compensation support, ED schema change, Kairos source/pin change, or hosted CI result is claimed.
