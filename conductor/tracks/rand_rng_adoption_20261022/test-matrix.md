# Test matrix

All runtime gates are future requirements, not implemented or executed tests. At entry, create exact executable targets, fixture hashes and baseline-derived budgets before replacement.

| Gate | Required oracle |
| --- | --- |
| rng-versioned-replay | Pinned generator known-answer vectors, full entity generations and purpose namespaces, intended CRN coupling, unrelated-draw isolation and 1/2/4-worker exact traces. |
| rng-checkpoint-compatibility | Roundtrip before/after partial-buffer draws and nonzero stream positions; uninterrupted versus resumed exact draws/events; malformed, old and unknown versions reject. |
| rng-dependency-cost | Reviewed MSRV/target matrix including Wasm, no accidental entropy source; build/binary and per-draw/whole-run cost; bespoke-generator deletion audit. |

Planning: `python3 tools/tasks.py build`, `python3 tools/tasks.py check`, `python3 tools/context.py check`.

Runtime: exact named native/browser tests and independent oracles defined at entry; record command, cwd, source commit, toolchain, resolved versions/features, seed/input hashes, exit status and raw artifact path. No source-text assertions substitute for actual browser/runtime behaviour.
