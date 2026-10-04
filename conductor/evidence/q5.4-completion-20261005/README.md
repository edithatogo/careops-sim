# Q5.4 queue development closeout — 5 October 2026

Status: accepted experimental queue development qualification; parent delivery checks and merge readback are recorded separately.

## Source and acceptance scope

Runtime source `eae890b0a2a3524a543ec4ee4aca61346e273b52` preserves accepted C1.4 and Q5.2. This task reconciles queue development evidence; it adds no Rust production code, dependencies, schema or scheduling changes. Child governance successor acceptance is recorded separately after its exact-head checks.

## Independent manual replay

A bounded Luna reader used a clean isolated worktree and a 21,590-byte hashed context. [Executed commands](receipts.json) passed 11 selected tests and both examples. [Readback](manual-review.json) records all twelve batch hashes as `47cfb7dca4211252`, exact FIFO and staff output equality, and primary Suspend/Restart completion at 12/15 with no Abort completion. The staff example has different work durations. The coordinator independently rechecked all recorded input/log hashes and exact stdout. Private lease metadata is excluded.

## Q-01–Q-07 evidence

The [source and test matrix](../../tracks/des_queue_preemption_20260925/test-matrix.md) maps each invariant to actual tests. [Q5.1](../q5.1-conformance-20261004.json) covers synthetic canonical traces and independent-runtime worker invariance, not engine RNG or shared parallel execution. [Q5.3](../q5.3-completion-20261005/README.md) records 228 passing tests/one ignored in each equivalent suite, selected Rust 1.76 compatibility, fifteen protected surfaces and legacy source equality; its 60 source hashes and twenty log hashes were independently rechecked here.

[Q5.2](../q5.2-completion-20261005/acceptance.json) records 39 scenarios × five repeats (195 processes), all thirteen 100,000-request cases, and two isolated Ubuntu canonical comparisons under unchanged thresholds. Earlier four-failure and two-failure local comparisons remain retained. Five-sample p95 is the maximum observed sample; process RSS is not per-claim memory; the legacy comparator is a FIFO primitive, not an equivalent Flow runtime. No speedup or clinical-load claim follows.

## Residual disposition and release notes

[Risk dispositions](../../tracks/des_queue_preemption_20260925/risk-register.md) retain strict-priority starvation, staged multi-resource deadlock, caller-owned Restart side effects, experimental API and release/security holds. Same-runtime continuation is qualified; Track 22 portable checkpoint, Track 32 Metal and Track 34/35 cross-LP queues remain unsupported/unverified. Clinical calibration and the ED MVP are separate. Upstream Unreleased notes and Track 03/12/25 handoffs are reconciled without changing historical overall track status.

The ledger permits one entry per track. Track 03's latest bounded review replaces its current phase entry while preserving the previous Q4 source/disposition in an explicit historical record; no new upstream ID is invented.

## Executed gates and delivery boundary

[Local governance receipts](governance-receipts.json), [independent review](review.json), [unchanged runtime source boundary](source-boundary.json), and [hosted acceptance](hosted-acceptance.json) retain actual results. Child governance head `8cd03c8f791ae58b33e5cc61b244071937a839ac` passed exact-head Linux/macOS native owner and all reported non-skipped checks. Skipped checks remain skipped; no new Q5.2 canonical performance run is claimed. Child [PR219](https://github.com/edithatogo/kairos/pull/219) stays stacked/draft. Parent local receipts, pin-contract check, hosted CI and merge readback are delivery evidence recorded separately; they must pass before acceptance is reported to the user.

Next preparation candidate is D3.3, subject to current parallel ownership and a reviewed leaf packet. Generic ED, calibration, hardened-v1 and formal release tasks remain in their existing plans; no further broad planning is required to complete this queue track.

Parent local gates: [actual receipts](parent-local-receipts.json) record context integrity,143-task DAG,MVP map80parents/248leaves and413 harness tests passing. Input metadata hashes are retained; tools/tests remain unchanged at the recorded source base. The committed-gitlink check and hosted delivery/merge are executed after publication. Raw log bytes and SHA-256 are preserved, including trailing blank lines; whitespace checks exclude only retained `.log` artifacts.
