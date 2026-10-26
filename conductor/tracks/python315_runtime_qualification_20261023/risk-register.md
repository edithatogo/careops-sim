# Risks

| Risk | Control |
|---|---|
| Reusing stale candidate evidence | Compare source/lock/input hashes and rerun affected checks |
| Python version declared but ambient subprocess differs | Explicit repo-local path and actual version output at each entry boundary |
| Unsupported native wheel/container/target | Execute the required target or retain a concrete blocker |
| Older CI lane or generated template survives | Complete active-use inventory and negative runtime checks |
| Live environment overwritten | Terminal-process confirmation and safe handover receipt |
| Concurrent Polars/FMI or shared-workflow edits | Exact claims and current-owner handoffs, no pin changes inferred |
| Historical proof rewritten or overstated | Preserve dated evidence; distinguish local, hosted and main acceptance |

## Closeout — 2026-10-24

The listed controls were qualified at parent main `43ea690`: actual final3.15.1/local-prefix checks, explicit failure reporting, hash-locked wheels/auditor, real environment preservation and POSIX shim regression, protected PR/main readback and unchanged Kairos pin. Prior evidence and the failed fresh environment are retained. Future dependency/platform upgrades require new affected qualification; completion is not broader domain acceptance.
