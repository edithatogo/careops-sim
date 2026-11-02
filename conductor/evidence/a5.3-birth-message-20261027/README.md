# A5.3 idempotent birth-message and destination evidence

- Track: `careops_core_domain_abstractions_20261020`, phase A5
- Task: A5.3
- Decision: [ADR-0015](../../decisions/ADR-0015-birth-message-and-destination-contract.md)
- Packet: [A5.3 birth message](../../execution/packets/A5.3.birth-message.json)
- Implementation commits: `e033bf3` and paired-ledger review fix `887e967`
- Disposition: local synthetic contract only; no clinical or operational validation

## Behavior

CareOps core now registers a complete multiple birth against the A5.2 identity
ledger. `BirthMessageId` is distinct from `BirthEventId`; exact replay returns
the same result and changed content under that message ID rejects. Empty infant
sets, duplicate ordinals, duplicate event keys, reused birth events, invalid
destinations and unknown mothers reject without partial identity state. Infants
receive distinct `PersonId`s with birth-event/ordinal provenance and typed
mother links. Maternal and infant destination intents are separate per-person
records. The existing transfer contract remains one person/episode per
operation, so these intents do not claim coordinated or atomic multi-recipient
transfer behavior.

The paired birth-message checkpoint validates infant identities and parent-link
add events against the identity event history. A later explicit relationship
removal remains valid and does not erase birth provenance.

Fetal-to-post-birth mapping is **not modeled** in the current source. Source
inspection found no fetal identity entity, fetal ID or pre-birth state API in
`careops-core` or `careops-ed`; the track's conditional “where modeled” clause
therefore has no mapping to implement. The birth identity begins at the recorded
birth event. A future fetal model needs its own explicit mapping and provenance
contract before any transition can be claimed.

## Source hashes

- `crates/careops-core/src/birth.rs`: `f671627369da67166591d1278bd29da701dc7b2b532496748d5e2ea94fa1bdca`
- `crates/careops-core/src/identity.rs`: `77092a98f960795da7b5b09798478edf7765b390db43ee7738d0f7783b32b027`
- `crates/careops-core/src/lib.rs`: `d958e56ac819a22293de8dd24dd112b84ab365640dd13f22e8d7c0afbdc6d887`
- `crates/careops-core/tests/birth_message_contract.rs`: `c60f62caf3a249d457a47fe5636bb81406d3ddf4c376f9d9df9a42c3843e38e8`
- Kairos submodule unchanged at `44261eb5cf8275f132f63a3396903d116d701dbe`.

## Executed verification

Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; `cargo 1.99.1 (5f94df478 2026-08-27)`.

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked -p careops-core --test birth_message_contract` | Pass, 5 tests | Includes unpaired-ledger replay rejection; exit 0 |
| `cargo clippy --locked -p careops-core --all-targets -- -D warnings` | Pass | `/tmp/careops-a53-core-reviewfix-clippy.log`; SHA-256 `9f778bf3d076c8cf272ffdf4924a2a4bd44ae1fa73fd0218b05c4dc3a3069b55`; exit 0 |
| `cargo fmt --all -- --check` | Pass | No output; exit 0 |
| `git diff --check` | Pass | No whitespace errors; exit 0 |
| Source inventory for fetal entity/mapping API | None found in `crates/careops-core` or `crates/careops-ed` | Read-only search and source inspection; this is a current-source boundary, not a claim about external consumers |

Independent reviewer-agent `/root/a53_birth_implementation_review` reviewed exact final commit `887e9678486698b1d4be854bee2c7b8fc4c66383`; the initial Medium paired-ledger finding was fixed and re-review found no remaining issues. The reviewer reran focused tests 5/5 and Clippy. The implementation was accepted locally. These results establish deterministic local synthetic
behavior only, not clinical validity, real-patient identity handling, fetal
mapping, or external delivery.
