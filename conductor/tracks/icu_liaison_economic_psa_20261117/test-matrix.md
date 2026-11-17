# Acceptance and independent oracles

Run from repository root with the pinned dependency submodules and Rust
toolchain (1.99.1 for project CI). Tests in
`crates/careops-core/src/icu_liaison.rs` cover:

| Oracle | Expected result |
| --- | --- |
| Same scenario and seed repeated | Identical serialised output |
| Zero effectiveness, positive staffing cost | Zero gross benefit; net cost equals negative staffing cost |
| Negative relative effect | Increased ICU returns (negative avoided returns) |
| All inputs fixed at modes | Exact independently computed accounting identity |
| Missing public-source URL, invalid probability | Fail closed |
| Non-synthetic import, unsupported schema | Fail closed |

Required checks: `cargo test --locked -p careops-core --lib icu_liaison`,
`cargo test --locked -p careops-core --examples`, `cargo fmt --all --check`,
`cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`,
`python tools/context.py check`, `python tools/tasks.py check`, the
repository's full CI and privacy/security workflows. Use required local
Python 3.15.1 for context. Do not mark an unrun tool check as passing.
Independently inspect source-versus-mode parameter agreement and that
all reported monetary amounts are AUD hypothetical inputs, not ANZICS costs.
