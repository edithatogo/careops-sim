# Risks

| Risk | Mitigation / owner |
| --- | --- |
| Newest dependency breaks old MSRV/API | Exact candidate audit, 25/30 ADR and feature/consumer tests; D1 |
| Agent evaluates its own invented criteria | Prespecified held-out evals and independent counterexamples; D1 |
| Harness becomes a second complex platform | Small read-only map/check first, incremental measured additions; D0/D1 |
| GitHub created prematurely or too late | Buildable-fixture/value gate before sustained implementation; D2 |
| Single-maintainer rules block all merges | Operable required-check policy, no impossible second-human rule; D2 |
| Green workflow is a smoke-only false pass | Negative gate tests and exact-commit evidence; D2/D3 |
| Delayed security allows secret/data exposure | Cheap controls early, threat review before sensitive/network profiles; D1/D4 |
| Automation loops cause noisy/conflicting changes | Budgets, leases, idempotency, retries and pause controls; D5 |

Only the audited bootstrap is currently implemented; all broader mitigations need
measured evidence before their risks can close.
