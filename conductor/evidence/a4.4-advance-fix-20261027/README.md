# A4.4 advance-to atomicity fix

Status: implementation and focused gates pass; reviewer-agent confirmation pending. Packet: `conductor/execution/packets/A4.4.advance-fix.json`.

## Finding addressed

The final A4 review found that public `advance_to` assigned the new clock before expiry processing could return a revision-overflow error. `advance_to` now processes on a cloned ledger and commits only on success. This preserves state on direct call errors, and the `apply` transactional wrapper retains its existing behavior.

Regression `failed_expiry_revision_overflow_does_not_advance_clock` starts from a valid checkpoint, sets a live expiring operation's revision to `u64::MAX`, restores it, and confirms `RevisionOverflow` leaves the exact checkpoint bytes and clock unchanged. The earlier future-tick invalid-envelope and nested-compensation regressions remain in the same focused suite.

## Verification

Base: `18151304d8d484466d8d1b28122f7ec48d753b5d`. Pinned `libs/kairos` was initialized at `44261eb5cf8275f132f63a3396903d116d701dbe`; no Kairos source or pin was changed. Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`.

- `cargo test --locked -p careops-core --test transfer_contract --test ward_community_contract` — pass: 26 transfer tests and 10 ward/community tests.
- `cargo clippy --locked -p careops-core --all-targets -- -D warnings` — pass.
- `cargo fmt --all -- --check` — pass.
- `git diff --check` and `python3 tools/tasks.py packet-check conductor/execution/packets/A4.4.advance-fix.json` — pass.

Source SHA-256: `crates/careops-core/src/transfer.rs` = `6dd11dd2fd06ca23441d70f47b7e602ef4b4f12c7846fe9f3440d70f88e47175`; `crates/careops-core/tests/transfer_contract.rs` = `a8e4418ce50c0a18dec69cfd559feb45fa0a1b5cab6525433dd21b356c798432`.

No nested compensation, clinical validation, real-service acceptance, ED schema changes, or Kairos changes are claimed.
