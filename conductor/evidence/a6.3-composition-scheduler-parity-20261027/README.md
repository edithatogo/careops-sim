# A6.3 — Integrated composition scheduler parity

Status: accepted locally after independent reviewer-agent review and scope correction. This is a
deterministic synthetic conformance fixture, not a clinical or production
scheduler validation.

## Scope and oracle

The test composes the retained nominal synthetic ED fixture with the ED identity
adapter, core transfer manager, and minimal ward/community boundary sequentially
inside one generic batch replication callback. ED is run to completion first,
transfer transitions are then scripted, and the ward/community boundary is
evaluated by its pure function. This proves deterministic sequential composition
in one process. It does not prove one unified event scheduler across these
subsystems. Hospital boundaries are logical domain boundaries and do not imply
PDES partitions.

Each successful result records replication index/seed, linked person and episode
IDs, ED completion count, ward admission and actual departure ticks, community
arrival and handover ticks, final owner, and the single occupied community-place
claim. Tests require serialized output equality at 1, 2, and 4 workers, and
interruption/checkpoint/resume equality with an uninterrupted baseline. A
separate assertion locks the three pathway stream namespace values and proves
100 service and routing draws do not perturb ten progression draws. Existing
core tests repeat the stream noninterference and generic runner recovery oracles.

## Inputs

- Base commit: `950e0bfb739ee35a8e55d1a8b5701f96015dc067`
- Rust: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; Cargo locked workspace dependencies.
- Nominal ED fixture SHA-256: `71215b7ff4a2b47b38ace339cadac15a68c1c49c9738856ff742e9400c2ff9ae`
- Generic experiment runner SHA-256 at base: `584c74d85ac8161ec10d4a3b7c7fa6e0992749d234b242ca7267e06a107b9e23`
- Pathway stream module SHA-256 at base: `04c886c431388beb4899973ff745ce66a5e0a3b6742317b4a7cf1abf34ca231e`
- Existing composition test SHA-256 at base: `34c9ff97ec08221619bae63c60820cc09045531dcefb9a46412c68a54a933012`
- Kairos submodule remained at `44261eb5cf8275f132f63a3396903d116d701dbe`.
- Packet `conductor/execution/packets/A6.3-composition-scheduler-parity.json` passed `tools/tasks.py packet-check` before implementation.

## Executed verification

All commands ran in `<source-worktree>/careops-sim`.

| Command | Outcome | Captured output SHA-256 |
| --- | --- | --- |
| `cargo test --locked -p careops-ed --test ed_ward_community_composition` | pass; 7 tests | `d4e8112a56958dc274d8400a7de6b8f492d88dbc63963f2591f80413054546ac` |
| `cargo test --locked -p careops-core --test pathway_failing_cases_tests --test logistics_consumer` | pass; 17 tests | `4590d90f3cc2871418d15823e98273b58bccbfe51dd35f312d5ee6dddf8e4aaf` |
| `cargo fmt --all -- --check` | pass | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `git diff --check` | pass | no output |

Captured logs are in `/tmp/careops-a63-ed-test.log`,
`/tmp/careops-a63-core-tests.log`, and `/tmp/careops-a63-fmt.log` on the
implementation host. The integration test exercised 8 replications at 1/2/4
workers and six replications in the checkpoint/recovery comparison.

## Limits and review

The ED input and its service/event times are fixed by the retained nominal
fixture; this evidence does not assert stochastic ED variation, clinical
validity, operational throughput, production scheduling, or distributed
execution. This receipt does not change the upstream Kairos pin. Acceptance is
local to the stated synthetic conformance boundary.

## Independent review

Reviewer-agent examined implementation commit
`b814bfb7df14826cd35de86fd4c998e9b3430b1d`. It reported one medium finding:
the earlier evidence and packet overstated shared-scheduler integration. This
revision narrows the claim to sequential composition in one process and makes
clear that the subsystems do not share a demonstrated event loop. The reviewer
reported no other actionable findings after re-reviewing the correction at
`deba8a2ce824c9f64ea2a411826b374950d2c625`. Its focused tests passed 7/7 for the
ED composition target and 17/17 for the core consumer/stream targets.
