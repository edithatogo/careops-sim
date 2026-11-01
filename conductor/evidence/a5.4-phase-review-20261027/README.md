# A5 integrated identity and birth phase review

Status: **accepted locally** at source commit `79570f5e30ccfd53dd588d4cf44026febb5bd8e6`.

## Scope and independent review

Reviewed A5.1–A5.3 against ADR-0014 and ADR-0015: deterministic person/episode
identities; explicit entry/exit/death lifecycle; typed relationship add/remove;
ED source/admission mapping; idempotent multiple-birth messages; infant ordinal
provenance; and separate maternal/infant destinations. Fetal-to-post-birth
mapping remains unmodeled because no fetal identity entity or pre-birth API
exists in current CareOps source. Destination intents remain per recipient and
do not imply atomic multi-recipient transfer.

Independent reviewer-agent `/root/a54_identity_phase_review` examined exact
integrated HEAD `79570f5e30ccfd53dd588d4cf44026febb5bd8e6` and found no remaining
issues. The review first identified three checkpoint gaps: unlogged/duplicate
life-cycle transitions, omitted ED mappings, and a valid later-tick duplicate
retry watermark. Fixes were applied in `e5a7a8e` and `79570f5`, with focused
regressions and reviewer re-review. The final review reran core identity 9/9,
birth-message 5/5, and ED identity 6/6 focused tests.

## Integrated verification

Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`,
`.venv/bin/python 3.15.1`; macOS ARM. Kairos remained at
`44261eb5cf8275f132f63a3396903d116d701dbe`.

| Command | Result | Captured stdout+stderr SHA-256 |
| --- | --- | --- |
| `cargo test --locked --workspace` | exit 0; 902 passed, 0 failed, 1 ignored across 117 test targets | `1b277758721708b208e9b4efb7f2f6bfd1babf4de954d25620f4136bd7e47aaa` |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | exit 0 | `0d8070ce7d3c371ccd6e55db81167b2d17553f002f51688fb5296d11467c2a7f` |
| `cargo fmt --all -- --check` | exit 0; empty output | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `.venv/bin/python tools/tasks.py check` | exit 0; 401 task records and valid prerequisites/DAG | `ab8b9f02b3213a06d5ea7abe5c5c139318dcecab9d957cb97570eaa7024af7d3` |
| `.venv/bin/python tools/context.py check` | exit 0; 25 tracks, 109 milestones, 1,961 local links | `19ae56c1fec9270ec81991b206e956c9bbc5cfe6873cb6090a9e677576614d10` |
| `.venv/bin/python tools/mvp.py check` | exit 0; 82 parents, 258 bounded leaves, joins/hashes valid | `3c7b9a3062f62964126c8db247833e935b8557a4baeffb7a30525cbe5f410966` |
| `git diff --check` | exit 0; empty output | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

## Final source and contract hashes

- `crates/careops-core/src/identity.rs`: `588fcea5fc750c07ef14b2e6e921fc8a1d28b3e3ae658d59a2c2a7991a3213cd`
- `crates/careops-core/src/birth.rs`: `f671627369da67166591d1278bd29da701dc7b2b532496748d5e2ea94fa1bdca`
- `crates/careops-core/src/transfer.rs`: `3a6272de2ecbb7c2fea58720a703174660eac2f69e60e4f4c34e67107d92a601`
- `crates/careops-core/tests/identity_contract.rs`: `81180ac52b334d34f15c6c98ded2c666a9ebd1e93386590a5d566f0f44b466bb`
- `crates/careops-core/tests/birth_message_contract.rs`: `c60f62caf3a249d457a47fe5636bb81406d3ddf4c376f9d9df9a42c3843e38e8`
- `crates/careops-ed/src/identity.rs`: `8dd4dc89a6fcb169b8fba78279fedbaf6271c4c04911ac4cc819ae910de1d01b`
- `crates/careops-ed/tests/identity_relationship_contract.rs`: `df45802c9b37d7d2354cc431f9f9acd8ea4588dd8c148f21252b7598b1ac7846`
- `ADR-0014`: `914ddd05cad88a02c1a96def0da765ae38b059b0b0b855e3209f5fe301423c34`
- `ADR-0015`: `b5f04ff11051bcdd67d3341cfba5acfec66adfdf72a4e4feabf8b09a63064cc7`

This is local synthetic contract/conformance evidence. It does not establish
clinical validity, real-patient identity handling, fetal mapping, hosted CI,
release readiness or operational acceptance. A6.1 is the next incomplete task.
