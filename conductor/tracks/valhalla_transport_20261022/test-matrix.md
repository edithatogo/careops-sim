# Test matrix

Future tests; no runtime proof from registration.

Required behaviour: Independent route/matrix cases, closure/disconnected entrances, coordinate/units, monotonic progress, sampled-duration provenance and repeatable route-cache replay; service failure is explicit.

Value/stop: No adoption if existing validated travel-time matrices answer the question; C++ service operations must justify extra maintenance. OSRM is an alternative to reassess only if Valhalla fails, not a second installed engine.

Planning checks: `python3 tools/tasks.py build`, `python3 tools/tasks.py check`, `python3 tools/context.py check`. Phase entry resolves concrete native/browser/service commands and numerical/performance budgets. Record command/cwd/commit/toolchain/version/input/seed hashes, exit status and raw artifacts. Actual cross-process/browser operation, not source assertions, is required.
