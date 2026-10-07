# C2 test-only regression integration at Kairos 6364a92

This receipt records three independently reviewed, test-only C2 gaps integrated at Kairos `6364a92b12a2b1c186ab1477814bc3dfb8a50aef`. The source changes add no production behavior, public APIs, dependencies, seed algorithms, or route semantics.

## Added regression coverage

- **C2.2 provider validation precedence**: overlapping invalid provider-version, stratum-ID, duration/weight/duplicate/sum-overflow inputs now assert the frozen validation order. Integrated source commit `98d4af2`; `work_duration.rs` SHA-256 `3c85675fe65a2c049fccc112f558017c66d3f9ec6e9f3a0f55ac3fdd1c06c697`. Worker result SHA-256 `a439b9ae706e07a7ffd899a554f81a54fc17171869943a9c65d064642fe12617`.
- **C2.5 continuation rejection preservation**: decision-mismatch rejection now begins with one active blocker and one queued waiter, and compares both request records, the full non-empty resource snapshot and bound work context afterward. Integrated source commit `07c9891`; source is in the final `flow_bridge.rs` below. Worker result SHA-256 `5b33d69ebce491504f25264b0fd6e3507996162dd27c72f7fd514bf776b94877`.
- **C2.1 mixed-subsystem runtime oracle**: one live Flow and adapter retain distinct Macro and routed Micro decisions, work/request/resource associations, no Macro transit scheduling, accepted-arrival-only Micro submission, and completion. Integrated source commit `6364a92`; source is in the final `flow_bridge.rs` below. Worker result SHA-256 `bca05b2b584f87816da6944963fa6737a4c72006e1ff11642ca2a22fa784463b`.

Final source SHA-256s: `crates/kairo-ecs-calibration/src/flow_bridge.rs` `18663b5407f4992d72bec07f6352b9a05ca6a59828b56299f841aa23a46df66c` and `crates/kairo-ecs-calibration/src/work_duration.rs` `3c85675fe65a2c049fccc112f558017c66d3f9ec6e9f3a0f55ac3fdd1c06c697`. The changes are tests only and do not reopen previously checked parent tasks.

## Exact-pin verification

At Kairos `6364a92`, stable Rust 1.99.0 only: the combined DES/ABM/calibration suite passed **457 tests, 0 failed, 5 ignored across 43 result groups**. Strict all-target Clippy passed with warnings denied, workspace formatting passed, and `git diff --check` passed. Raw command output is included as `suite.log` and `clippy.log`; their SHA-256 values are in `acceptance-rust199.json`.

The suite command used the locked workspace and `kairo-ecs-calibration/flow,kairo-ecs-abm/test-support`. Rust versions, exact argv, working directory, source/contract hashes, receipts and log hashes are recorded in the structured acceptance file. Parent `context.py check`, `tasks.py check` and `mvp.py check` also passed during integration.

## Acceptance boundary

This is local source and test evidence only. C2.4 Conductor phase review remains open. Track03 and Track22 owner dispositions, Track22 portable checkpoint/restart, hosted exact-head checks, release acceptance and clinical acceptance remain open. The owner review packets were refreshed but remain unsent and blank for owner disposition.
