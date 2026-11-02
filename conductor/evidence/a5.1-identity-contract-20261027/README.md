# A5.1 identity and cardinality contract evidence

- Track: `careops_core_domain_abstractions_20261020`, phase A5
- Task: A5.1
- Base commit: `5e87fd1a83052fc54255e9eb7e15a0314d6170c5`
- Packet: [A5.1 identity contract](../../execution/packets/A5.1.identity-contract.json)
- Contract: [ADR-0014](../../decisions/ADR-0014-person-episode-identity.md)
- Cases: [machine-readable identity/cardinality matrix](identity-cardinality-cases.json)
- Reviewer: reviewer-agent/subagent `a51_identity_contract_review`
- Reviewed/final commit: `0c9f5a9ca06fed79c25c2a871000354962c9cdee`
- Disposition: accepted contract; no findings after two fixed review rounds

This is a contract task. The case matrix specifies test oracles for A5.2–A5.3; it does not execute or prove Rust runtime behavior. No Rust source, ED schema, Kairos source or parent pin is changed. The existing A4 transfer model and its single-active-transfer scope are preserved.

## Bound inputs

- Track plan SHA-256: `c52cba526a34ec270438a8a09ec943f1035023f09204e8a0c718899a2c7d9fda`
- Transfer lifecycle contract SHA-256: `47b75af12339c469343166a5a2eace02058170679a65d3b36856482b7ebff7dc`

The initial reviewer identified order-dependent arbitration when competing spawn events claim one `PersonId`. ADR-0014 now requires canonical logical-event ordering; case ID `ID-24-conflict-order-permutation` compares accepted/rejected events, full state, accounting and RNG across permutations.

The second review required a deterministic late-event boundary. ADR-0014 now rejects a newly submitted identity event atomically after its logical time is sealed; ID-25 verifies identity, edge, lifecycle, accounting, dedupe and RNG state are unchanged. Checkpoint replay is a separate explicit operation.

## Review disposition

The reviewer-agent reviewed commits `18f4a2a618494307a13cc50463b7cc4af52b969d`, `0fa1c4f4ca517b5b2a42869235414555a94fc75e`, and final `0c9f5a9ca06fed79c25c2a871000354962c9cdee`. It identified order-dependent spawn arbitration and an underspecified late-event rule. ADR-0014 and cases ID-24/ID-25 were amended; final review reported no findings.

## Executed checks

- `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/A5.1.identity-contract.json`: passed before final contract commit at its bound base `0fa1c4f4ca517b5b2a42869235414555a94fc75e`; the packet base is intentionally the parent revision of its own final commit.
- JSON case-matrix validation: passed; 25 unique case IDs with explicit expected outcomes.
- `git diff --check`: passed.
- Runtime tests: not applicable to this contract-only task; no runtime proof is claimed.
