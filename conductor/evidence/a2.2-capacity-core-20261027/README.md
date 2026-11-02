# A2.2 neutral capacity core

- Task: `careops_core_domain_abstractions_20261020` / A2.2
- Contract: [ADR-0010](../../decisions/ADR-0010-effective-capacity-and-calendar-boundaries.md)
- Implementation packet: [A2.2.capacity-core.json](../../execution/packets/A2.2.capacity-core.json)
- Reviewer-fix packet: [A2.2.pool-aggregate-review-fix.json](../../execution/packets/A2.2.pool-aggregate-review-fix.json)
- Accepted integration head: `df0612abfe5ea0ab23dc1d5d971fa40218c496bc`
- Runtime: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; `cargo 1.99.1 (5f94df478 2026-08-27)`
- Source SHA-256: `eed0c7651daeba43b7e112ce59212b1d9b37d885e8ad83e2f7b8e597b5208713`
- Test SHA-256: `6d7e6bea2754cb91faf972c9acefad993bc43d2d81cdd1b236376d21396ffac5`

## Behavior

The core models physical/equipped/open/staffed ceilings separately from
reserved/occupied/cleaning/unavailable dispositions; preserves unknown versus
known zero; rejects invalid conservation; deduplicates pool aliases; rejects
conflicting aliases and unlike-unit/time aggregates; matches opaque capability
sets; and uses half-open UTC intervals with caller-supplied offset resolution.
It does not implement Kairos allocation grants, local zone databases or
UTC-to-simulation-tick conversion.

## Executed verification

| Command | Outcome |
| --- | --- |
| `cargo test --locked -p careops-core --test capacity_contract` | 18 passed, 0 failed. |
| `cargo test --locked -p careops-core` | 71 passed across unit, integration and contract tests; 0 failed; 0 doc tests. |
| `cargo clippy --locked -p careops-core --all-targets -- -D warnings` | Passed with warnings denied. |
| `cargo fmt --all -- --check` | Passed. |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/A2.2.capacity-core.json` | Passed at its bound base; packet validation had no errors. |
| `.venv/bin/python tools/tasks.py check` | 401 task records valid. |
| `.venv/bin/python tools/context.py check` | 25 tracks, 109 milestones, 1915 local links, 0 errors. |

## Reviewer-agent result

The independent code reviewer found that aggregation could combine distinct
pools across unlike units or snapshot times. Commit `df0612a` now rejects those
totals and includes negative tests. The reviewer rechecked the exact fix and
reported no remaining issue. This is an engineering contract only, not clinical
or operational acceptance.
