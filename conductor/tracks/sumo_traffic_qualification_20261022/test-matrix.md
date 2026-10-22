# Test matrix

Future tests; no runtime proof from registration.

Required behaviour: Independent traffic controls and calibrated arrival distributions; offline replay first; live mode additionally tests deterministic stepping, reconnect/kill/recovery, no duplicate/lost arrivals, vehicle/crew conservation and renderer-off trace equality.

Value/stop: Keep dormant unless a concrete traffic-interaction need and qualified demand data exist. Prefer no SUMO to an unvalidated complex traffic model. No dependency installation or live co-simulation commitment from registration.

Planning checks: `python3 tools/tasks.py build`, `python3 tools/tasks.py check`, `python3 tools/context.py check`. Phase entry resolves concrete native/browser/service commands and numerical/performance budgets. Record command/cwd/commit/toolchain/version/input/seed hashes, exit status and raw artifacts. Actual cross-process/browser operation, not source assertions, is required.
