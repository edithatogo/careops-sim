# Test matrix

All runtime gates are future requirements, not implemented or executed tests. At entry, create exact executable targets, fixture hashes and baseline-derived budgets before replacement.

| Gate | Required oracle |
| --- | --- |
| viewer-browser-parity | Actual browser orthographic selection/labels/overlays/floors/replay and accessible controls/tables; full-range IDs/ticks and malformed/revision-mismatched data. |
| viewer-multifloor-live | Perspective and multi-floor occlusion/picking, units/axes, replay versus live, lost/reordered deltas and reconnect snapshot recovery. |
| viewer-performance-independence | Rendering off/30/60FPS yields identical authoritative traces; named-device latency/memory/bundle budgets, context loss, repeated mount/dispose and old-renderer retirement evidence. |

Planning: `python3 tools/tasks.py build`, `python3 tools/tasks.py check`, `python3 tools/context.py check`.

Runtime: exact named native/browser tests and independent oracles defined at entry; record command, cwd, source commit, toolchain, resolved versions/features, seed/input hashes, exit status and raw artifact path. No source-text assertions substitute for actual browser/runtime behaviour.
