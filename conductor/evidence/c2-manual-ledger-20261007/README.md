# C2 manual synthetic accounting readback — 2026-10-07

This readback addresses the plan's manual Macro/Micro elapsed-time accounting check using the existing, explicitly test-only P2.3 synthetic case. I read the structured fixture and its prior hand-worked acceptance explanation, then reran the fixture validator.

## Reconciliation

The fixture's Macro case has one elapsed window of 37.5 seconds. Its paired Micro representation decomposes that same window into:

- queue wait: 4.0 seconds
- active clinical work: 12.0 seconds
- transit: 18.5 seconds (18.5 m at the fixture's 1.0 m/s test speed)
- interruption pause: 3.0 seconds

The Micro components sum to 37.5 seconds. The Macro total and Micro decomposition describe the same elapsed window; do not add them together. The graph, values, and rates are synthetic representation fixtures, not ED observations, recommendations, or generic defaults.

## Evidence and limits

- Fixture: `model-inputs/ed/abm/p23-synthetic-accounting.json`, SHA-256 `0019e85efc859b7697a5c167e8e15e1f41d3661ead4720ef98c749036975f655`.
- Prior hand-worked explanation: `conductor/evidence/p2.3-coordinator-acceptance-20261002.md`, SHA-256 `91997b0853d0a34d71085bd018ca3fa597ac5aaa925528751b7301ac55c17a11`.
- Executed command from the repository root: `python3 tools/validate_abm_example.py` — exit 0; output: `synthetic graph and Macro/Micro accounting checks passed; no ED empirical value established`.
- Readback time: 2026-10-06 16:13:03 UTC.

This reuses the standalone synthetic accounting representation and documents the requested arithmetic. It is not an integrated Kairos Macro/Micro runtime run and does not establish runtime transit/wait interaction, portable checkpoint/restart, Track03 owner review, C2.4 phase acceptance, hosted checks, release, or clinical acceptance. Those gates remain open.
