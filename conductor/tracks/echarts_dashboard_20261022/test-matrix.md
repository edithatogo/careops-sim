# Test matrix

All runtime gates are future requirements, not implemented or executed tests. At entry, create exact executable targets, fixture hashes and baseline-derived budgets before replacement.

| Gate | Required oracle |
| --- | --- |
| charts-analytical-values | Chart series, bins, intervals, units and denominators match independent analytical outputs; empty/singleton/null/nonfinite and large scenarios. |
| charts-browser-accessibility | Actual browser rendering, keyboard selection, readable summaries/tables, resize and export content/provenance; malicious labels cannot execute. |
| charts-cost-cleanup | Pinned bundle cost, representative interaction latency, repeated mount/dispose heap behaviour and deletion of replaced chart code. |

Planning: `python3 tools/tasks.py build`, `python3 tools/tasks.py check`, `python3 tools/context.py check`.

Runtime: exact named native/browser tests and independent oracles defined at entry; record command, cwd, source commit, toolchain, resolved versions/features, seed/input hashes, exit status and raw artifact path. No source-text assertions substitute for actual browser/runtime behaviour.
