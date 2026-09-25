# Risks

| Risk | Mitigation / owner |
| --- | --- |
| Generic ED treated as Cairns | Separate profiles, provenance, later local validation; E0/E4 |
| Staff+bed acquisition deadlocks | Staged claims and explicit resource-order policy; E2/Q |
| Shift changes or cleaning leak capacity | In-flight policy, state invariants and boundary fixtures; E2 |
| Run horizon/warm-up biases metrics | Explicit estimands, censored counts and sensitivity; E1/E4 |
| Recovering a run duplicates work/artifacts | Stable IDs, atomic writes, checkpoint tests; E3/C5/Q4 |
| GUI silently becomes simulation authority | Worker/snapshot protocol and render-rate invariance; E5 |
| Backend scaffolds counted as complete | Device/threaded/multi-node runtime evidence for each profile; E6–E8 |
| Every optional ecosystem module delays usable ED | Required/deferred capability matrix and scoped release gates |
| Accelerated math changes selected calibration | CPU oracle, tolerances, tie/ranking checks and fallback; E6 |

Performance budgets, supported profiles and validity boundaries must be recorded
before release; unspecified thresholds cannot be claimed satisfied.
