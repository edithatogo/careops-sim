# Test matrix

| Requirement / phase | Automated evidence | Manual verification |
| --- | --- | --- |
| D-R1 / D1 | Clean bootstrap, wrong-version/missing-tool tests, locked builds and MSRV/consumer resolution | Fresh macOS ARM/Linux checkout |
| D-R2 / D0/D1 | Context links/pins/metadata/milestone DAG; kill/restart and stale/fake evidence tests | Resume without repeating discovery |
| D-R3 / D1 | Held-out skill evals: missed bug, hidden clamp, fake pass, unauthorized action, seed/MSRV regression | Audit skill provenance and permissions |
| D-R4 / D2 | Required-check negative case, path skip and incompatible submodule test | Read back repository rules and exact-commit CI |
| D-R5 / D3 | Property/fuzz/soak, coverage/mutation for critical invariants, feature/API/doctest/benchmark gates | Review minimal counterexamples and threshold evidence |
| D-R6 / D4 | Malformed/oversized input and trust-boundary tests, package/attestation consumer verify, rollback | Independent release rehearsal |
| Maintenance / D5 | Idempotency, no-change silence, budget/timeout/retry/pause tests | Stop/resume without duplicate edits |

Current local harness regression cases are in `tests/test_context.py`. Broader
skill/CI/release tests are planned; a passing link check does not satisfy them.

## Additional research-derived cases

Include the task-specific cases in [reports 9–12 integration](../../research/ed-research-incorporation-9-12-20260927.md)
when preparing executable packets. These are proposed oracles, not recorded passes.
