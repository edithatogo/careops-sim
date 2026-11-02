# A6.1 ED compatibility and fixture parity

**Task:** `careops_core_domain_abstractions_20261020:A6.1`

**Base:** `1d16e2435d20c71b41da1347edc5809e3c8c1180`

**Status:** Accepted A6.1 task evidence; A6.2–A6.4 and A7 phase acceptance remain pending.

## Scope

This slice qualifies the existing ED compatibility facade against the retained
pre-extraction nominal run. The shared experiment lifecycle is already owned by
`careops-core::experiment` (A0.2); ED identity admission is already adapted to
`careops-core::identity` (A5.2). This change adds an exact golden regression at
the public ED run-document boundary. It does not change the fixture, domain
behavior, public API, or schema.

The regression test reads the historical output and current nominal input,
checks the input's SHA-256 against the baseline manifest, runs the current ED
adapter with the recorded seed, horizon, and input-file label, and compares the
complete serialized JSON bytes. The retained historical artifact is read-only.

## Schema evolution record

Schema evolution remains separately governed by
[`docs/support-matrix.md`](../../../docs/support-matrix.md#3-schema-versioning--forward-migration-policy).
No schema migration is introduced or implied by this fixture-parity check.

| Boundary | Current version | A6.1 disposition |
| --- | --- | --- |
| ED scenario input | 1 | Unchanged |
| Capacity/location profile | 3 | Unchanged |
| Run document and manifest | 1 | Unchanged |
| Arrow schema | 1.0 | Unchanged |

Any future schema change still requires a separately reviewed versioned
migration and migration tests. This evidence does not claim migration behavior.

## Verification

Working directory: `<source-worktree>/careops-sim`

Rust: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`

Nominal fixture SHA-256: `71215b7ff4a2b47b38ace339cadac15a68c1c49c9738856ff742e9400c2ff9ae`

Historical output SHA-256: `bb303ba2975da470d450c90d27eb54c1556695b06c45a6af66ed5536d3c3e425`

| Command | Result | Log SHA-256 |
| --- | --- | --- |
| `cargo test --locked -p careops-ed --test c63_compatibility_replay_performance` | Pass, 5 passed, 0 failed | `7d5279d381cf1fb67d9931f0c391c43e26c4ec77da389c9132b3d93d2a6b4865` |
| `cargo fmt --all -- --check` | Pass | Empty output (`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`) |
| `git diff --check` | Pass | Empty output (`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`) |

The added regression `historical_nominal_fixture_serializes_byte_identically`
passed, comparing the entire current serialized run document with the frozen
historical bytes.

## Independent review

The reviewer-agent examined implementation commit `e70a35d1c12547816b81c74ab985bb0b08d48ec2`.
It reported one low-severity finding: trailing-space Markdown hard breaks made
the committed implementation diff fail `git diff --check`. The formatting was
corrected in `e547b624bdb95ca6ce47e4cf6eb5fccb6733fe3c`. Re-review of the fixed
range found no remaining issues; the reviewer confirmed the focused byte-parity
test passes and `git diff e70a35d^ e547b62 --check` is clean. The coordinator
accepted A6.1 after this re-review.

## Limits

This closes only the ED compatibility slice and nominal byte-parity gate. It is
not the ED-to-ward-to-community composition in A6.2, the worker/recovery and
conservation proof in A6.3, hosted CI, clinical validation, or operational
acceptance.
