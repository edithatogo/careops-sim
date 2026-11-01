# A4.2 Transfer Core Evidence

## Scope

Implemented the ADR-0012 transfer lifecycle as a versioned, deterministic,
single-scheduler ledger. The implementation adds typed identities and envelopes,
logical message deduplication and revision-gap buffering, explicit owner/location
and resource-claim transitions, deadline processing, linked compensation,
checkpoint serialization and fail-closed restore validation.

Network delivery is not exactly once. This work does not add clinical eligibility
policy, ED schema, multi-resource allocation, Kairos source or pin changes, or a
new dependency.

## Bound inputs

- Base commit: `e7cb6f4a83ba511e22eb6e82321a6c0484eff60e`
- Packet: `conductor/execution/packets/A4.2.transfer-core.json`
- ADR-0012 SHA-256: `47b75af12339c469343166a5a2eace02058170679a65d3b36856482b7ebff7dc`
- A4 plan SHA-256: `7f7583b8dc56fd67d6837b94ed159ad51990023dc0b9eb6f9e647684acf2acfc`

## Implementation and checks

- `crates/careops-core/src/transfer.rs` SHA-256: `238258d595861c2f6722a5e931ab52a0c63b1ecb85d4f000b628fff26f26799f`
- `crates/careops-core/tests/transfer_contract.rs` SHA-256: `5eb1a240c98e5101e1ea788e497679c9759a0ac0d3e7fc8297ff7cae3d125b55`
- `crates/careops-core/src/lib.rs` SHA-256: `10793970cd7673c1e02102b3df724b15f7f0e18ff78c6807bc26170262842778`
- Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`.
- `cargo test --locked -p careops-core --test transfer_contract` — exit 0; 24 passed, 0 failed.
- `cargo clippy --locked -p careops-core --all-targets -- -D warnings` — exit 0.
- `cargo fmt --all -- --check` — exit 0.
- `tools/tasks.py packet-check conductor/execution/packets/A4.2.transfer-core.json` — exit 0 at the bound base; no packet errors.
- `git diff --check` — exit 0.

The captured check output is recorded here, in this receipt. Working directory:
`<source-worktree>/careops-sim`.

```text
$ cargo test --locked -p careops-core --test transfer_contract
running 24 tests
test result: ok. 24 passed; 0 failed; 0 ignored

$ cargo clippy --locked -p careops-core --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s

$ cargo fmt --all -- --check
exit 0; no output
```

Tests cover valid and invalid transitions, conservation and claim conversion,
rejection from Requested/Offered/Reserved, cancellation from Reserved/Departing,
deadline ties, deduplication, reordered/conflicting revisions, post-arrival
compensation, horizon censoring, checkpoint/restart and malformed checkpoint
rejection, including owner, location, episode, person and claim consistency.

## Agent review

Independent agent review checked the implementation, tests and restore invariants
against ADR-0012 and the A4.2 packet. Initial findings on transit rejection,
revision-slot collisions, terminal buffering, incomplete restore consistency,
missing rejection/cancellation coverage and terminal-person conservation were
addressed with code changes and regression tests. Final review result: no
remaining actionable findings. Reviewer: `/root/a42_transfer_review`; no files
were edited by the reviewer.

## Limits

Evidence is local Rust contract testing and agent code review only. It does not
establish runtime integration with ED consumers, Kairos integration, clinical
acceptance, network delivery semantics or operational readiness. A4.3 and the
integrated A4.4 review remain outstanding.
