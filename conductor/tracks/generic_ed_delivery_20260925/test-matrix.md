# Test matrix

| Requirement / phase | Automated evidence | Manual verification |
| --- | --- | --- |
| E-R1 / E0/E1 | Schema, one-patient, branching, no arrivals, overload, zero/long service, terminal count conservation | Reconcile one case per pathway |
| E-R2 / E2 | Shift/break/handover, zone/skill, cleaning/reservation, boarding/diagnostic contention | Inspect resource ownership over shift change |
| E-R3 / E2 | Macro/Micro paired draws, route interruption/restore, no double-counted transit | Explain task work+wait+transit intervals |
| E-R4/5 / E3 | Formula/denominator oracle, censoring/warm-up, cancellation/disk failure, atomic writes, 1/2/N workers/resume | Cancel/resume and compare full artifacts |
| E-R6/7 / E4 | Analytic/differential/invariant, sensitivity/held-out tests, package consumer and supported-platform gates | Reproduce examples independently |
| E-R8 / E5 | Native/Wasm IDs/time/API parity; off/30/60 render-rate invariance, slow consumer/memory and accessibility checks | Run/edit/compare/export through dashboard |
| E-R9 / E6 | Actual Metal dispatch + CPU oracle/tolerance/ranking/device failure/benchmark | Prove device execution, inspect timings |
| E-R9 / E7 | Real threaded PDES, serial observable parity, causality/GVT/deadlock/lookahead tests | Inspect cross-LP claim/message lifecycle |
| E-R9 / E8 | Actual MPI/gRPC nodes, retries/duplicates/migration/failure/shutdown and telemetry | Recover a worker/network fault |

Acceptance uses the exact scoped G1/G2/G3 profile. No global Kairos completion,
Cairns validity or performance claim follows from synthetic/native evidence alone.

## Additional research-derived cases

Include the task-specific cases in [reports 9–12 integration](../../research/ed-research-incorporation-9-12-20260927.md)
when preparing executable packets. These are proposed oracles, not recorded passes.
