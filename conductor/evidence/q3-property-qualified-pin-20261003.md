# Q3 seeded timed-cycle property qualification — 2026-10-03

Qualified development pin: Kairos `42896037a7fac793f63d2f8867584f4faeb0b78e`, branch `codex/careops-q3-seeded-cycle-property`. No Kairos main PR or development-stack merge. Parent Q3 phase closeout remains pending.

## Source and local proof

The child adds only `crates/kairo-ecs-des/tests/flow_preemption_property_v1.rs`, SHA256 `d900dc3a9256794ea99822b2fd4f2362fb1bac9ac0ef802fca2ccaf7db71f904`. All15 existing D2 source hashes were rehashed unchanged; the contract adds only this test hash.

Test-local SplitMix64 v1 uses six explicit seeds,32 cases per seed and three strategies:576 cases per run. Independent interval/accounting predictions apply after EVERY dispatch, including empty dispatches advancing time. Exact preemption counts, all terminal reasons and uniqueness, causal ordinals, capacity/membership, context, callbacks and initial-template factory counts are asserted.

Scoped format and targeted property test passed on actual Rust1.98.1 and1.76.0. One named test executes576 cases; no unchanged local71-test rerun is claimed. Root independently reviewed the strengthened source and receipts.

Local receipt: `/private/tmp/kairos-q3-seeded-cycle-property/.artifacts/mvp/Q3.3.cycles.seeded-model/result.json`, SHA256 `cfeff46576f39b53089b159060def379fc204712b599eec2224a71e9d891c7b2`. Final1.98 log SHA256 `0658a3a1e212790aa4106cf6018b900286f8ca879592dbd70a751fd2192ee0e8`;1.76 `a94055f957c7878710048e33724b1db8a39bbd3bf694e97a3c30ae1549903262`. Earlier first-pass logs remain retained.

## Actual hosted execution

[Owner run37121692873](https://github.com/edithatogo/kairos/actions/runs/37121692873) succeeded at exact4289603 on Ubuntu24.04 job111198891261 and macOS15 job111198891384. Both logs identify the property binary and named `seeded_timed_cycles_match_independent_interval_model_v1 ... ok`. Reusable native packages and optional calibration-floor steps also passed. Root independently verified both host executions.

Hosted receipt: `/private/tmp/kairos-q3-seeded-cycle-publication/.artifacts/q3-cycle-owner/receipt.json`, SHA256 `7d3ae2a408cb19f89c55b3c19872c067a824f37061d4cbe03bbb6e6b8168f9ea`. Linux full-log SHA256 `c9b8c946310d89c05b6ac96f7f50a60379596a4e8eecdcdba4c1cd3eac017d4e`; macOS `2e23c9b43d95cf5bc814daab6f5e47e6c8587fc58919072ab209c0b0a1bb9e60`.

## Acceptance limits

Coverage is bounded, not exhaustive; no shrinking or worker-count-independence claim. Public FlowDispatch hides event kind and completion-token revision/lease. Black-box accounting applies every dispatch with exact transition/terminal counts; existing private identity-injection test1751 separately covers strict stale-token identity.

Existing primary/secondary manual interval/context and Track01 state/RNG reviews remain unchanged. Track25 experimental-breaking development-only migration and release hold remain in force. Q3.3/Q3.4 checkboxes and active phase are unchanged. Q4/Q5, C1/C2 and D3/D4 retain separate acceptance.
