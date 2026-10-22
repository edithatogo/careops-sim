# Test matrix

All runtime gates are future requirements, not implemented or executed tests. At entry, create exact executable targets, fixture hashes and baseline-derived budgets before replacement.

| Gate | Required oracle |
| --- | --- |
| routing-deterministic-paths | Independent small-graph optimality, cost/hops/edge-ID ties, shuffled insertions, zero-cost cycles, directionality and overflow. |
| routing-multifloor-access | Synthetic multi-floor connectivity, inaccessible lifts/stairs/doors, disconnected destination, topology revision/cache invalidation and unit/identity rejection. |
| routing-des-boundary | Lift queues/resource waits remain authoritative DES state; route rendering does not schedule events; code/build/query-memory audit without fictitious deletion claims. |

Planning: `python3 tools/tasks.py build`, `python3 tools/tasks.py check`, `python3 tools/context.py check`.

Runtime: exact named native/browser tests and independent oracles defined at entry; record command, cwd, source commit, toolchain, resolved versions/features, seed/input hashes, exit status and raw artifact path. No source-text assertions substitute for actual browser/runtime behaviour.
