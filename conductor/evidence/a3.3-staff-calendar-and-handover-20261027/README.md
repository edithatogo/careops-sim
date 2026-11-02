# A3.3 staff calendars and handover evidence

- Track: `careops_core_domain_abstractions_20261020`, phase A3
- Task: A3.3
- Bound base: `3a1a7b37b4cf629caca3809d88f08a997cbeb26f`
- Contract: [ADR-0010 UTC calendar boundaries](../../decisions/ADR-0010-effective-capacity-and-calendar-boundaries.md)
- Interruption boundary: [ADR-0011](../../decisions/ADR-0011-staged-resource-claims-and-interruption.md)
- Packet: [A3.3 staff calendar and handover](../../execution/packets/A3.3-staff-calendar-and-handover.json)

## Delivered behavior

`careops-core::staffing` uses the accepted UTC half-open `Calendar` and
`UtcInterval` types. Staff assignments require roster coverage, exclude break
intervals and reject overlapping assignments for the same staff identity.
Handover reserves incoming overlap plus the following duty as one validated
operation; distinct outgoing and incoming staff can overlap only for that same
booking. Escort continuation retains the same staff and escort chain across
adjacent intervals. On-call readiness is at least the configured call-in delay,
with checked overflow. A non-interruptible procedure assignment cannot be
released or handed over early. Roles and clinical/site eligibility remain
outside this neutral accounting module.

## Independent reviewer-agent review

Reviewer-agent/subagent `review_a33_staffing` read the implementation, packet,
tests, ADR-0010 and ADR-0011. It reported no actionable findings. The review
confirmed UTC half-open coverage, break handling, no double booking, atomic
handover validation, escort identity continuity, checked call-in latency, and
non-interruptible procedure claims. No clinical policy or eligibility claim is
made.

## Executed verification

Toolchain: Rust 1.99.1 (`58cf8f9c3`), Cargo 1.99.1 (`5f94df478`).

| Command | Outcome |
| --- | --- |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/A3.3-staff-calendar-and-handover.json` | Passed at bound base `3a1a7b3`; no packet errors. |
| `cargo test --locked -p careops-core --test staffing_contract` | Passed: 7 tests, 0 failures. |
| `cargo clippy --locked -p careops-core --all-targets -- -D warnings` | Passed. |
| `cargo fmt --all -- --check` | Passed. |
| `git diff --check` | Passed. |

SHA-256:

- `crates/careops-core/src/staffing.rs`:
  `68b564adbc095ca7a31288c2aac3081445ce9d32be0fcd46ced450d1b8f63e9b`
- `crates/careops-core/tests/staffing_contract.rs`:
  `713f7f99578d4da7fcfb3b5c7188229099a2dd60de872636523f61a6fa3b9ea4`
- `conductor/execution/packets/A3.3-staff-calendar-and-handover.json`:
  `8c1d911f8c6a8d767d6aad9c8e9c4a792af275d24bb8e85548e340fe6e10a3e0`
