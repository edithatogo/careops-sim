# A2 phase review — effective capacity and calendars

- Track: `careops_core_domain_abstractions_20261020`, phase A2
- Reviewed source head: `9f0327cad8e5fc3d85d8e242b959d4a66343428c`
- Reviewer: independent reviewer-agent/subagent
- Disposition: accepted for A2; A3–A7 remain incomplete
- Contract: [ADR-0010](../../decisions/ADR-0010-effective-capacity-and-calendar-boundaries.md)
- A2.1 contract evidence: [receipt](../a2.1-capacity-contract-20261026/README.md)
- A2.2 implementation evidence: [receipt](../a2.2-capacity-core-20261027/README.md)

## Review and remediation

The reviewer examined the integrated A2 source and tests file by file against
CC-05, the phase plan, product boundaries and the ED compatibility surface. The
first review found three issues:

1. Known-zero capacity could be reported unknown, and active dispositions could
   exceed known physical inventory when equipped capacity was unknown. Commit
   `9f0327c` constrains dispositions by the minimum known physical/equipped
   inventory and lets any known zero effective ceiling establish zero assignable
   capacity. Tests cover physical/equipped/open/staffed zero, unknown ceilings,
   and demand outcomes.
2. Underflow/overflow rejection coverage omitted reserved and maintenance
   transitions. Tests now assert reservation/unavailable underflow and
   maintenance overflow leave state unchanged.
3. The project and track summaries still said A2.1 was next. This closeout
   updates `conductor/index.md`, `tracks.md`, the track index and current state
   to show A2 accepted and A3.1 next.

The reviewer rechecked the code/test fixes on `9f0327c` and found no remaining
source findings. The closeout aligns ADR-0010 with the corrected known-zero and
physical-inventory semantics. ED schema/runtime source and Kairos pins were not
changed. Workspace compatibility is verified; a consuming ED adapter and
byte-parity proof remain future work.

## Executed verification

Toolchain: Rust 1.99.1 (`58cf8f9c3`), Cargo 1.99.1 (`5f94df478`); Kairos pin
`44261eb5cf8275f132f63a3396903d116d701dbe`; Conductor pin
`7a5c560a4fdf5297be58594cc37527eb12790272`.

| Command | Outcome |
| --- | --- |
| `cargo test --locked -p careops-core --test capacity_contract` | 25 passed, 0 failed on the reviewed source after fixes. |
| `cargo clippy --locked -p careops-core --all-targets -- -D warnings` | Passed. |
| `cargo fmt --all -- --check` | Passed. |
| `cargo test --locked --workspace` | Exit 0 at reviewed source head, including CareOps ED/CLI and Kairos submodule crates. |
| `.venv/bin/python tools/tasks.py check` | Passed: 401 task records and DAG valid. |
| `.venv/bin/python tools/context.py check` | Passed before closeout: 25 tracks, 109 milestones, 1919 local links, 0 errors. |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/A2.phase-review-fixes.json` | Passed at its bound base. |
| `git diff --cached --check` | Passed for contract/registry closeout changes. |

Changed Rust source SHA-256:
`d3500bf715079f7e2f65d06b769d80f3a08851d8a64aaf6c697b78707a3e981d`.
Changed test SHA-256:
`680855c1390981a8d975d1d24cb573de60fec0b0eae62b5bce1aa14493c1c91b`.

No empirical or clinical validity, ED adapter behavior, Kairos allocation
capability, or operational acceptance is claimed by this phase.
