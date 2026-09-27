# Test matrix

| Requirement | Automated checks / phase | Manual verification |
| --- | --- | --- |
| P-R1 | Taxonomy/schema P0; bidirectional actual consumer coverage P5; unregistered/default/orphan negative cases | Patient/staff path input walkthrough |
| P-R2 | Required citation/assumption fields, source hashes, range-kind distinctions, unknown ≠ zero P1/P2 | Independent extraction from original table/code |
| P-R3 | Units, support, sums, schedules, graph connectivity, dependencies, invalid parameterization P3/P4 | Hand-worked conditional draw and resource case |
| P-R4 | Known synthetic fit, seed repeatability, held-out split, censoring/confounding failures P3 | Reproduce fit and explain rejected candidates |
| P-R5 | Load examples in E2/C5, mode/seed/conditional invariants, sensitivity and expected invalid inputs P5 | End-to-end generic example from documented inputs |
| P-R6 | Generic/site profile separation, no private fixtures, provenance/version checks P4/P5 | Cairns acquisition/mapping gap review |

Do not assert real data follow a family because a synthetic sampler test passed.
Acceptance thresholds and exact validator/fit commands are frozen in P0/P3 worker
packets; this planning change does not claim parameter estimates or fit results.
