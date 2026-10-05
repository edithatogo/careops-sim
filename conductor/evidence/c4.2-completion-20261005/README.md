# C4.2 runtime completion — 5 October 2026

Private reusable calibration runtime is qualified at Kairos development pin `21e48b257b5c89dd9756b647d9ee17f698f9496b` ([draft PR221](https://github.com/edithatogo/kairos/pull/221)). Executed runtime source is `59d7dbb0c004654e3da90951d42e2b983ef23993`; the evidence successor is distinct.

- Deterministic sorted-CDF W1/KS D; exact signed u128 paired residuals; compensated descriptive bias/MAE/RMSE; fixed source-clock windows/groups and retained diagnostic counts/statuses.
- All 42 actual frozen cases conform in Rust on both toolchains and the independent exact Fraction Python comparator. 119 calibration tests pass on each of Rust 1.99/1.88, with four named default ignores; candidate and timing tests were explicitly executed.
- Strict all-target calibration Clippy, workspace formatting and existing governance checks pass. False commit/toolchain and mutated actual metrics are rejected.
- [Exact-head native owner CI](https://github.com/edithatogo/kairos/actions/runs/37245222736) passes on Linux/macOS; [child checks](child-pr-checks.json): 42 success/2 conditional skips.
- [142-member qualification archive](qualification.tar.gz) and [two-member hosted log archive](hosted-native-logs.tar.gz) have [checksums](SHA256SUMS), inventories, receipts and full member readback. [Independent archive readback](archive-independent-readback.json) verifies every qualification member.

[Acceptance](acceptance.json), [integrated review](integrated-review.json), [actual report](actual-runtime-report.json), [independent readback](independent-actual-readback.json), [current pin contract update](current-contract-update.json), and [coordination limitations](coordination-deviations.json) retain scope. Checked u128 intermediates can return Invalid on overflow. Individual residuals stay exact; compensated binary64 summaries flag magnitude conversion loss only. Diagnostic counts overlap independently. One debug 100000-point/side check took 2256ms; this is not release/speedup evidence.

C4.3 Arrow sidecars, C4.4/C-04, public API review, clinical calibration and release remain open. Upstream phase/registry/ledger is unchanged; the child remains draft on the development stack. Existing development-readiness state and historical C1/Q5/C4.1 qualifications are preserved. Parent local and hosted publication gates are recorded separately after this pin update; no broader acceptance follows.
