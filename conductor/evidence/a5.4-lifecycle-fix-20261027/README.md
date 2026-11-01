# A5.4 identity lifecycle checkpoint review fixes

- Trigger: A5.4 independent phase review at `2191d6a8d893167d14abc99ad2d08b31367f3981`
- Findings: High core lifecycle history gap; Medium incomplete ED adapter-map coverage
- Fix commits: lifecycle/map reconstruction `e5a7a8e`; watermark compatibility `79570f5` (independently reviewed, no remaining findings)
- Packet: [bounded A5.4 lifecycle fix](../../execution/packets/A5.4.lifecycle-fix.json)

## Changes

Core restore now replays all identity events through a temporary ledger in
canonical tick/command order and compares the resulting people, episodes,
relationships, and birth ordinals to stored checkpoint state. It requires the stored seal
watermark to cover event history and preserves later duplicate-retry watermark
advancement. This rejects unlogged death/exit state, repeated exits or deaths, invalid
chronology and event accounting that cannot be reproduced.

ED adapter restore now requires the source-person and admission maps to cover
exactly the people and episodes in its private core ledger, in addition to the
existing per-key derivation and event checks.

The phase re-review also identified that an exact duplicate retried at a later
tick advances `sealed_through` without creating a second event record. Restore
now reconstructs the state represented by events, requires the stored watermark
to cover all recorded events, and preserves a later watermark from that valid
duplicate-only seal. A regression proves restore retains tick 5 and still
rejects a new event at that sealed tick.

## Source hashes

- `crates/careops-core/src/identity.rs`: `588fcea5fc750c07ef14b2e6e921fc8a1d28b3e3ae658d59a2c2a7991a3213cd`
- `crates/careops-core/tests/identity_contract.rs`: `81180ac52b334d34f15c6c98ded2c666a9ebd1e93386590a5d566f0f44b466bb`
- `crates/careops-ed/src/identity.rs`: `8dd4dc89a6fcb169b8fba78279fedbaf6271c4c04911ac4cc819ae910de1d01b`
- `crates/careops-ed/tests/identity_relationship_contract.rs`: `df45802c9b37d7d2354cc431f9f9acd8ea4588dd8c148f21252b7598b1ac7846`
- Kairos submodule remained pinned at `44261eb5cf8275f132f63a3396903d116d701dbe`.

## Verification

| Command | Result | Evidence SHA-256 |
| --- | --- | --- |
| `cargo test --locked -p careops-core --test identity_contract` | Pass, 8 tests | `c52126b83c1b73467617f8ec252df467cd99ae6d750d1b09c88313f1f7ec6130` |
| `cargo test --locked -p careops-ed --test identity_relationship_contract` | Pass, 6 tests | `bbc561476debe4ef454a1b13a3c2e29fd4c7a295a30dcc4b2ddc64370bca70f2` |
| `cargo test --locked -p careops-core --test identity_contract` after watermark fix | Pass, 9 tests | `60ac34b47091ad007e8b1e5de03d76c806611bef95b132d4f7958bf9077b80eb` |
| `cargo fmt --all -- --check` | Pass, empty output | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `git diff --check` | Pass, empty output | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

The High and Medium lifecycle/map findings were fixed at `e5a7a8e`; the later watermark finding was fixed at `79570f5`. Independent phase re-review of `79570f5` found no remaining issues and reran core identity 9/9, birth-message 5/5, and ED identity 6/6. A5.4 full workspace and Conductor gates passed at the same source commit. This is local checkpoint/conformance evidence only.
