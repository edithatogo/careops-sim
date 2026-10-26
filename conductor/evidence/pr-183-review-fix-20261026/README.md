# PR #183 integration review fix — 2026-10-26

## Finding

An independent acceptance reviewer found that generic experiment execution accepted callback results without checking that each result retained its assigned replication index and seed. Resume also trusted checkpoint identity and index bookkeeping, allowing malformed or duplicated replications to be represented as a complete batch.

## Resolution

Commit `88135e4` validates callback result index and seed before storing results in both new and resumed runs. Resume now validates schema version, non-zero total count, completed result identities, bounds and uniqueness, completed/pending disjointness, and full coverage before dispatch (including the no-pending fast path). Invalid checkpoints fail with `MalformedConfig`; callback identity mismatches fail with `ExecutionFailed`.

The acceptance reviewer re-reviewed the fix and reported the P1 finding fixed with no remaining issue in scope.

## Verification

- `rustup run 1.99.1 cargo fmt --all --check` — passed.
- `rustup run 1.99.1 cargo test --locked -p careops-core --test experiment_runner_contract` — passed, 2 tests.
- `rustup run 1.99.1 cargo test --locked -p careops-core` — passed, all core tests.
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` with explicit Rust 1.99.1 tool paths — passed.

PR Actions on the updated branch are still required before merge.
