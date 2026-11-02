# A6.2 ED to ward to community composition

**Task:** `careops_core_domain_abstractions_20261020:A6.2`

**Base:** `892efc0d7759c03bcb1c0564537ca001f798a6b0`

**Status:** Accepted A6.2 task evidence; phase acceptance remains pending A6.3–A6.4.

## Scope

Added an ED integration test that runs the existing nominal synthetic scenario,
binds its patient through `EdIdentityAdapter`, and composes two successive core
transfer lifecycles: ED to ward, then ward to community. The ward/community
boundary determines departure from both community-place and transport
availability, explicit transport duration, handover delay, and ED boarding until
the ward bed becomes available.

Negative paths cover an explicitly rejected offer, an expired offer, and
cancellation after destination reservation. The horizon case keeps the patient
in transit with the ward origin claim held and no observed community arrival.
The tests assert identity continuity, transfer state, ownership, location and
resource claims. Inputs and outcomes are synthetic tick-based fixtures.

This is a deterministic composition conformance fixture using existing modules;
it does not add a production event scheduler or a complete ward/community model.
It does not claim clinical validity, measured discharge behavior, or PDES
partitioning.

## Verification

Working directory:
`<source-worktree>/careops-sim`

Rust: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`

| Command | Result | Log SHA-256 |
| --- | --- | --- |
| `cargo test --locked -p careops-ed --test ed_ward_community_composition` | Pass, 4 passed, 0 failed | `7a3fd39572ce83d6fbcc4e9739926b80ed3c849490500964f6da6d1c7a4a12a2` |
| `cargo test --locked -p careops-core --test transfer_contract --test ward_community_contract` | Pass, 36 passed, 0 failed | `fec05e41dcf38b19aa0a164243c90f1b2d934894ba02a206198a769bbb462cd5` |
| `cargo clippy --locked -p careops-ed -p careops-core --all-targets -- -D warnings` | Pass | `fecede7c2157efe2f16c8c8b9dce94b0fb4dd384dca8207046c8bf7636eb0ffe` |
| `cargo fmt --all -- --check` | Pass | Empty output (`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`) |
| `git diff --check` | Pass | Empty output (`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`) |

Input hashes:

- Nominal ED scenario: `71215b7ff4a2b47b38ace339cadac15a68c1c49c9738856ff742e9400c2ff9ae`
- Core transfer implementation: `3a6272de2ecbb7c2fea58720a703174660eac2f69e60e4f4c34e67107d92a601`
- Core ward/community implementation: `9c8bdea4615b4245a191c8b2e76f6f35d15cbcdbecc57e234fdd2638098dc846`
- Added composition test source: `34c9ff97ec08221619bae63c60820cc09045531dcefb9a46412c68a54a933012`

## Limits

The ward/community model remains exogenous and minimal by contract. This test
proves composition of ED execution, identity, transfer ownership and the
existing boundary oracle; it does not assert empirical or operational
acceptance. A6.3 still owns full conservation, stream namespace, noninterference,
worker invariance, and interruption/recovery evidence.

## Independent review

Reviewer-agent examined integrated commit `24f55b1ea0b13ee15757790e71b83c61912b2836`.
It found no actionable issues and confirmed the four composition tests pass.
The coordinator accepted A6.2 with the scope limits above.
