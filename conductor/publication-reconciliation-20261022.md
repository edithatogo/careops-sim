# Publication reconciliation — 22 October 2026

The publication branch is based on reviewed origin/main 5de9e4e. Only the planning changes from local 8d40080 through 4983aed are carried forward. Remote accepted R0/R1 evidence, active R2/R3 gates, source code, CI, dependency locks and Kairos pin 44261eb remain authoritative.

Local candidate runtime changes, pin c4e67e6 and archival claims remain recoverable on preserve/local-main-before-reconciliation-20261022 at 4983aed. They have not been silently accepted or deleted. Their eventual adoption requires the existing candidate/consumer gates. Polars R10 now explicitly waits for reconciliation R3 as well as accepted upstream Polars.

Prior planning reports describe the old local candidate baseline; their task/link counts and pin context are historical, not runtime evidence for this reconciled publication. No clinical/hardware/runtime acceptance follows from this planning PR.
