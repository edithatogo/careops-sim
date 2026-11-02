# A3 phase review — team bundles and safe interruption

- Track: `careops_core_domain_abstractions_20261020`
- Task: A3.4
- Reviewed integrated HEAD: `3bce4b2e1d76e37037c5a85d3804e973a6180610`
- Source range: `ee5f870..1a5b620` (A3.1–A3.3 implementation and fixes)
- Reviewer: independent reviewer-agent/subagent `review_a3_phase`
- Disposition: accepted; A4.1 is next
- Packet: [A3.4 phase review](../../execution/packets/A3.4.phase-review.json)
- Contracts: [ADR-0011](../../decisions/ADR-0011-staged-resource-claims-and-interruption.md)

## Integrated review

The reviewer examined the A3 plan, specification, test matrix, agent contract,
ADR-0011, source and tests, A3.1–A3.3 packets and receipts, and the exact Kairos
pin. The reviewer reported no actionable findings and recommended acceptance.
The implementation and receipts accurately limit claims to staged single-resource
coordination and neutral staffing accounting. Atomic multi-resource grant,
portable checkpoint restore, clinical eligibility and downstream starvation
guarantees remain unclaimed. The Kairos dependency is test-only; ED schema/runtime
and the parent Kairos pin were not changed.

## Executed verification

Toolchain: Rust 1.99.1 (`58cf8f9c3`), Cargo 1.99.1 (`5f94df478`).
Kairos pin: `44261eb5cf8275f132f63a3396903d116d701dbe`.
Integrated parent HEAD reviewed: `3bce4b2e1d76e37037c5a85d3804e973a6180610`.

| Command | Outcome | Captured output SHA-256 |
| --- | --- | --- |
| `cargo test --locked --workspace` | Passed: 112 test groups, 846 passed, 0 failed, 1 ignored; all doc-tests passed. | `e5b14fc08be5ba75b5174ba0af0a9dcfb2c0bd1909402a8ec5c2e0e8b363c412` |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | Passed. | `2bc683fb4e4f4f21ee9ee39e2b8ecce2efe20d3b1be57bda5316aa519ab3d30a` |
| `cargo fmt --all -- --check` | Passed. | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `.venv/bin/python tools/tasks.py check` | Passed: 401 task records and DAG valid. | `ab8b9f02b3213a06d5ea7abe5c5c139318dcecab9d957cb97570eaa7024af7d3` |
| `.venv/bin/python tools/context.py check` | Passed: 25 tracks, 109 milestones, 1944 links, 0 errors. | `0f1ed15739a2166cd4c8de3a450c8584d09032158a50a968c8019aa1a1034112` |
| `.venv/bin/python tools/mvp.py check` | Passed: 82 parents, 258 leaves and complete joins. | `3c7b9a3062f62964126c8db247833e935b8557a4baeffb7a30525cbe5f410966` |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/A3.4.phase-review.json` | Passed; no packet errors. | `2abaac2832351140c25f21f6fe23ba23c80e9d02f9b5043cee4c5c0d13d2ebe5` |
| `git diff --check ee5f870..1a5b620` | Passed; no whitespace errors. | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Captured logs are retained under ignored `.artifacts/careops-a3.4/` in this
worktree. Empty-output checks have the SHA-256 of the empty byte stream.

## Reviewed source hashes

| Path | SHA-256 |
| --- | --- |
| `crates/careops-core/src/allocation.rs` | `a2d4005dec1f6b879d488a82a9b2187bb0378789929da06f09adc73d8bc5a733` |
| `crates/careops-core/src/staffing.rs` | `68b564adbc095ca7a31288c2aac3081445ce9d32be0fcd46ced450d1b8f63e9b` |
| `crates/careops-core/tests/allocation_contract.rs` | `114be35cf4253e1fdfd5524b5c7a9e5ab4956ba6433fb05e65eb299708467722` |
| `crates/careops-core/tests/allocation_kairos_conformance.rs` | `afa24b8cf053af6679865799ec4fa01259cc2d4eb61f2fa8ffbe1b26c1ac8b77` |
| `crates/careops-core/tests/staffing_contract.rs` | `713f7f99578d4da7fcfb3b5c7188229099a2dd60de872636523f61a6fa3b9ea4` |
| `conductor/execution/packets/A3.4.phase-review.json` | `429b3fe86600a5835e2c40685522009ed623f4dd80b51a247a3d68f4faf6b54e` |

No empirical/clinical validity or consuming ED adapter acceptance follows from
this phase review. A4.1 contract work is next.
