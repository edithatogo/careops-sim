# C2.1 paired explicit-zero Restart qualification

Coordinator acceptance of the reopened C2.1 paired Restart oracle at exact Kairos
pin `51b5e5dcd09bf747e272dbfee71f924c2ccbf017`, parent base
`74c047e5b52b1f4cf6ff8b9f93995ed8f4d381b1`.

The test-only fixture pairs actual Flow Macro with explicit-zero Micro under
`PreemptionStrategy::Restart`. It asserts identical Service stream key, seed,
sampled duration and draw position; zero transit and no transit carrier in both
runs; identical terminal work progress; and completion at tick 30 after the
three-tick discarded attempt, two-tick interruption and full 20-tick restarted
service. The existing routed Suspend case remains in the shared fixture and was
run separately and in the full library suite.

The first coordinator package run exposed a regression in the generalized test
helper: `BoundIntrinsicWork::carrier` is unset before route start, so the routed
Micro Suspend case was mistaken for zero transit. The helper now classifies the
case from the bound `TransitRequest` before starting transit. The initial failed
run is retained in `integration-first-attempt-failure.log`; the corrected
Suspend test and complete library suite pass.

## Verification

All Rust commands used stable Rust/Cargo 1.99.0. The exact command arguments,
working directory, exit status, summary, and log hashes are in
`acceptance-rust199.json`.

- Corrected explicit-zero Restart worker test: 1 passed.
- Existing routed Suspend integration test: 1 passed.
- Full `kairo-ecs-calibration` library suite: 87 passed, 0 failed, 2 ignored.
- Strict Clippy with warnings denied: passed.
- Workspace formatting check: passed.

An independent read-only review confirmed the zero-transit paired oracle and
preserved Suspend assertions. This is local exact-pin evidence only. Hosted
checks, portable restart, release, clinical acceptance, and Track03 owner/phase
review remain open. It does not qualify a Transit RNG because `TransitContext`
has no RNG field.
