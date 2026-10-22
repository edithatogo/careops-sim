# Test matrix

All runtime gates are future requirements, not implemented or executed tests. At entry, create exact executable targets, fixture hashes and baseline-derived budgets before replacement.

| Gate | Required oracle |
| --- | --- |
| sampling-independent-oracles | Analytical moments/CDF or quantile controls with fixed seeds, sample sizes and justified error bounds; tails, support, invalid/degenerate and bounded-integer cases. |
| sampling-hazard-parity | Independent exponential/piecewise survival and cause incidence, unchanged-covariate residual clocks, censoring and no hidden clamping; reviewed corrections separately identified. |
| sampling-resume-cost | Restart/worker equality on qualified targets, draw isolation and versioned manifests; actual production deletion and throughput/build cost. |

Planning: `python3 tools/tasks.py build`, `python3 tools/tasks.py check`, `python3 tools/context.py check`.

Runtime: exact named native/browser tests and independent oracles defined at entry; record command, cwd, source commit, toolchain, resolved versions/features, seed/input hashes, exit status and raw artifact path. No source-text assertions substitute for actual browser/runtime behaviour.
