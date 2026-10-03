# Q3 runtime: qualified development pin

Parent base: `226ffd9163e0d8d9b1599e05c597bdec7d16c8a8`.
Old Kairos pin: `dffd6f6889353d4a13bb39b7981983a53df7d888`.
Qualified development pin: `1455f76226db61a3dc930aced57849d2a66cbdb1`, branch `codex/careops-q3-qualified-timed-runtime`.

## Scope and evidence

The additive runtime implements timed allocation/completion, strict eligible preemption with Suspend/Abort/Restart, checked effort/revisions, typed owned factory/handlers and deferred notifications. Internal stale completion tokens are empty no-ops before resource boundary processing; explicit user commands retain independent due-boundary transitions. Aggregate arithmetic/token/despawn preflight precedes ECS changes and factory operations.

Factories/callbacks/destructors are trusted; arbitrary application panic/side-effect rollback is not promised. Q4 callback ingress/persistent same-tick budgets and Q5 staged-clone performance remain deferred.

Independent root review accepted the bounded source candidate locally. Actual selected Rust1.98.1 and1.76.0 runs each passed71 DES tests without warnings, including all frozen Q3 fixtures and private transaction/stale-event cases. The repeated-interruption grid contains24 cases (three strategies, one through eight interruptions), not exhaustive verification.

[Exact-head owner run37118457361](https://github.com/edithatogo/kairos/actions/runs/37118457361) succeeded on x86_64-unknown-linux-gnu and aarch64-apple-darwin. Both jobs passed reusable native package tests and optional calibration Rust1.88 floor. Root independently read the exact run/head/job/step results. Local retained job logs contain actual1.98.1/1.88.0 compiler and host readbacks.

The calibrated seed-map foundation and reviewed Arrow signed-span correction are retained. Only existing DES lib.rs/flow.rs contract hashes change; the now-production private selector hash is added. Every other preexisting contract source hash matches.

## Root manual interval and context review

Root independently inspected frozen fixture assertions, runtime source and actual logs without repeating the suite.

| Oracle | Low busy intervals | Urgent interval | Low busy / waiting / useful at completion |
| --- | --- | --- | --- |
| Primary Suspend | [0,3), [5,12) | [3,5) | 10 / 2 / 10 |
| Primary Restart | [0,3), [5,15) | [3,5) | 13 / 2 / 10; wasted3 |
| Primary Abort | [0,3); terminates3 | [3,5) | busy3, remaining7 |
| Secondary Suspend | [0,4), [7,13) | [4,7) | 10 / 3 / 10 |
| Secondary Restart | [0,4), [7,17) | [4,7) | 14 / 3 / 10; wasted4 |
| Secondary Abort | [0,4); terminates4 | [4,7) | busy4, remaining6 |

Primary owned context generation0 is preserved under Suspend/Abort. Restart uses owned initial template generation0 and factory generation1. The callback is deferred and once-only, captures busy3 and useful0/remaining10 for Restart versus useful3/remaining7 for Suspend/Abort. Attempt increments only for Restart; completed resumed/restarted work has execution2. Original sampled duration stays stored; factory accepts only the initial template, with no new duration-draw path introduced.

## Retained proof hashes

Local source/hosted evidence remains under the qualified Kairos checkout `.artifacts/q3-qualified-verification`; these paths are retained local artifacts, not public CI artifact delivery claims.

- Final Rust1.98.1 log: `540eaa227f8e1edfd3fa4820380bfa61ccd115d384d6b1189f04e338fe25d914`.
- Final Rust1.76.0 log: `0681d564ee32f7ffe88edebad80969faf4b886c5ae9e564e1b84cec6788ab2e7`.
- Hosted owner receipt: `67d53eb797021bfb5c337aeb3b2b45608d65773aa9addf93aae56fd95f166df8`.
- Hosted readback: `8b8e47a9095f4fc0fb5db2d319bd9c138483773fc977d3f5c650beb976a0a7d4`.
- Linux owner log: `e98d081e62b9428b0e634a6024bb5a861f5307a43dc1b5270c3fe90e4a15242c`.
- ARM owner log: `ac60f1bc141e84872ea961e202888e9caa8f22721935856670eb98a0a5fefaec`.

## Acceptance boundary

This qualifies the development pin only. All parent Q3 checkboxes and active phase remain unchanged/pending. Full Conductor Q3.4 review requires a separate owner/phase disposition packet; no hosted pass alone closes it. Full C1/C2 and remaining D3/D4 retain independent gates. No D3 workflow, configuration or installer files are changed by this pin update.

Approved parent packet SHA: `c0364ef851a18f84ce7175085375b97364e282fd91ca0a7082423279ece3c34f`.
Parent execution repeats only context, diff and postcommit exact-pin checks; native suites are not repeated locally.
