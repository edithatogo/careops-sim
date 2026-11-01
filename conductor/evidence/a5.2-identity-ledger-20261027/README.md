# A5.2 deterministic identity ledger evidence

- Track: `careops_core_domain_abstractions_20261020`, phase A5
- Task: A5.2
- Reviewed packet: [A5.2 identity ledger](../../execution/packets/A5.2.identity-ledger.json)
- Contract: [ADR-0014](../../decisions/ADR-0014-person-episode-identity.md)
- Implementation review: independent reviewer-agent/subagent `/root/a52_checkpoint_rereview` reviewed exact commit `bc1f88b1aba78db9e5aa90c432a470dc0a756371`; no findings. Focused suites passed again (core 7/7, ED 5/5).
- Disposition: accepted locally after implementation, full workspace checks, and independent re-review

## Delivered behavior

`careops-core` now owns typed stable PersonId/EpisodeId and event IDs, deterministic SHA-256 derivation, canonical tick-batch commands, typed `BirthParentOf` and `AssociatedWith` relations, explicit episode entry/exit/death accounting, event replay/conflict handling, relationship-only removal, and checkpoint validation. Existing `careops_core::transfer::{PersonId, EpisodeId}` paths remain public re-exports with the same transparent u64 serialization. The ED adapter derives opaque synthetic source keys and admission keys from byte-exact UTF-8, keeps PersonId across readmission, and creates new EpisodeIds per admission. Its per-episode PathwayPatient keys keep maternal/infant progression independent.

Death rejects atomically while any service episode remains open; explicit exits are required first. Birth-parent self/transitive cycles reject; `AssociatedWith` cycles are allowed. Removing a parent relation leaves people and episodes intact. No ED scenario/output schema, Kairos source or pin changed.

Review fixes: core checkpoint restore reconstructs the final relationship set from relationship events in canonical tick/command order and rejects checkpoint state that adds, omits or retains an edge inconsistently. ED adapter restore now verifies each source key maps to its derived PersonId and matching core spawn event, and each admission key maps to its derived EpisodeId and matching core entry event. Regression tests tamper with both checkpoint types.

This proves deterministic local synthetic model behavior only. It is not clinical recovery validation, real-patient identity handling, portable Kairos checkpoint support or external delivery evidence.

## Exact source hashes

- `crates/careops-core/src/identity.rs`: `a40f6457a5d9ba64b0a42870fd8ecaa7be82fb8b8c55fc69a10ca62022570dc6`
- `crates/careops-core/tests/identity_contract.rs`: `291b380b3861bcf7b60be2f5ea5173f9a50c3c48d896d0ca29b7cf4f908d3647`
- `crates/careops-ed/src/identity.rs`: `86999d6c401bebc4aef85803fd703ec3b92977be4ff85dca77e445f02e996b63`
- `crates/careops-ed/tests/identity_relationship_contract.rs`: `d63f2c8534593e23093b5b686ec4b853b9f846b1e0c4ae0bf016526c5fe345f4`
- Kairos submodule remained at `44261eb5cf8275f132f63a3396903d116d701dbe`.

## Executed verification

Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; `cargo 1.99.1 (5f94df478 2026-08-27)`.

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked -p careops-core --test identity_contract` | Pass, 7 tests | Review tamper cases included; exit 0 |
| `cargo test --locked -p careops-ed --test identity_relationship_contract` | Pass, 5 tests | Review tamper cases included; exit 0 |
| `cargo test --locked --workspace` | Pass, 894 tests across 116 test targets; 0 failures; 1 ignored | `/tmp/careops-a52-reviewfix-workspace-test.log`; SHA-256 `e850632ffc2314920db20c6dcd6a570882eb884a6f5aa7a32131e90aa03e9aae`; exit 0 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | Pass | `/tmp/careops-a52-reviewfix-workspace-clippy.log`; SHA-256 `68b3323892ce017f1108b0058728c664bce5794c7535312fa2f665567779f2aa`; exit 0 |
| `cargo fmt --all -- --check` | Pass | No output; empty SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`; exit 0 |
| `git diff --check` | Pass | No whitespace errors; exit 0 |

Packet pre-review: `a52_identity_packet_review` reviewed the final A5.2 packet at `72b02a8920bf0ea40a356eb00e2c233c7be351cc`, confirmed all six SHA-256/u64 vectors and the death, cycle, ED mapping, and pathway boundaries, with no remaining packet blockers.
