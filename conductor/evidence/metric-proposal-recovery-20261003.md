# Metric design recovery — 2026-10-03

Recovered the six historical E0.3 definitions (queue wait, time to care, length of stay, throughput, boarding, resource utilization) as an explicit E1 proposal. Source: `0bfe013157d462663a6d2c612630629c71c79fe2`, `conductor/design/ed/e0.3-metrics-contract.json`, exact UTF-8 SHA-256 `9a6d6ee2987ce698fa11a9c5fc7582f454f8d147d95028bbfb542ee202018eb1`. Baseline: `5b8851c7f04be46bbe5d10385806e7ba82e3b617`.

The historical support inventory described a one-arrival runner and is replaced by the current synthetic multi-work-item FIFO inventory. Historical review notes are preserved externally with the retired worktrees; they are not current acceptance proof. No old schemas, test scaffolds, or completion records were promoted.

The proposal uses half-open observation windows, whereas accepted E0 includes completions at the horizon and has no warm-up exclusion. E1 must select/version its endpoint and cohort contract and verify events exactly at the cutoff/horizon before adoption. Clinical endpoint events, capacity calendars, censoring, units, and replication assumptions still need their existing implementation/evidence gates. No empirical values or numeric defaults are added.

Verification is document/context integrity only. Command receipts and outputs are retained locally in `/tmp/metric-proposal-checks.json` and `/tmp/metric-proposal-check-*.log`; hosted checks are evaluated on the PR head before merge. This recovery does not accept E1 or any metric implementation.

The exact historical bytes are now versioned at `conductor/design/ed/proposals/sources/e0.3-metrics-contract-0bfe013.json`, SHA-256 as above. A fresh checkout can verify the hash without the unpublished historical commit. The artifact is source material with obsolete runner claims, not current authority or acceptance.
