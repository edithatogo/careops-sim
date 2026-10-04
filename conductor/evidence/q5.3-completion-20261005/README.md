# Q5.3 compatibility qualification — 5 October 2026

Reviewed source: `eae890b0a2a3524a543ec4ee4aca61346e273b52`, based on accepted
`34e1d776e0ab4e2c84eb07f7c45779220df7e9b1`, preserving Q5.2 and C1.4.
[Kairos development PR #218](https://github.com/edithatogo/kairos/pull/218) is stacked
on the existing development branch; acceptance of this pin does not merge or
release that earlier stack. Exact-head hosted acceptance passed: 54 successful checks, three expected skips,
no failures or pending checks. Both push and PR native-owner runs succeeded on
Linux x86_64 and macOS ARM. See [hosted acceptance](hosted-acceptance.json),
[all check states](child-checks.json) and [native-owner jobs](native-owner-runs.json).
The existing branch condition skips `Q5.2 canonical native regression (Ubuntu)` on Q5.3; the
existing Codecov OIDC upload was skipped. Neither is counted as a pass.

## Implementation and local verification

- Added one external consumer fixture with three meaningful public-surface/FIFO/trajectory tests.
- Added migration and backend boundary guides; updated example commands to Rust 1.99.0 and labeled Q4 records historical.
- Two requested gpt-6-luna source reviewers found no remaining blocker. Requested model is not served-model attestation. Worker tests/docs checks and review remain separate from coordinator execution.
- Canonical Rust 1.99 locked full DES default and no-default suites each passed 228 tests, with one existing benchmark test ignored. The crate has no feature section, so these configurations are equivalent today.
- Rust 1.76 locked no-default selected legacy, new public consumer and migration fixtures passed 11 tests. This is the retained compatibility floor, separate from canonical Rust 1.99 and optional Arrow 60 Rust 1.88.
- Formatting, strict all-target Clippy, the doctest command (zero doctests), normal Track 25 compatibility pack and Track 21–27 evidence-boundary validation passed. Formal release-gate validation was not invoked or waived.
- FIFO stdout matched all 3 exact expected lines; staff/bed/cleaning stdout matched all 17 source-derived expected rows. Expected files were prepared before execution, using existing fixture timelines and transition snapshot mutation order.
- Legacy Resource/DESContext/Trajectory source matched `fae901558f07b7b717a676adbafbe2cdc78dea1c` byte-for-byte after removal of exactly the additive Flow/preemption declarations/re-export. This is not dependency-graph identity or a general semver guarantee.
- All 15 registered protected surfaces retain their production source; only the new DES test fixture changes a registered root. No production code, scheduler, ABI/schema, dependencies or feature changes.

See [command receipts](receipts.json), [toolchain/input hashes](inputs.json),
[source comparison](source-baseline.json), [source reviews](review.json),
[example/doc checks](example-and-docs.json) and hashed raw logs. `run.py` is the
archived receipt generator used in the original Kairos worktree, not dispatch
authority or a runnable parent-repository recipe. Its receipts preserve original
working directory, head, start/end, exit status and output hashes. Worker-local
initial formatting failure remains retained; its corrected final source passed.

## Parent delivery gates

Context integrity, the 143-task DAG, the 80-parent/248-leaf MVP recipe integrity
check and the full 413-test local Python harness passed. See
[parent command receipts](parent-local-receipts.json). These are precommit local
checks with their recorded base and owned changes; hosted parent CI and merge
readback are separate. The committed pin checker is run after the gitlink commit.
Actual native checkout/host/compiler lines are retained in
[native source readback](native-source-readback.json).

## Ownership handoff and limits

Flow claims one unit of one resource. Staged claims are not atomic and can
deadlock; strict priority has no aging or starvation guarantee. Request
cancellation and scheduler-event cancellation are separate. Waiting deadlines
expire inclusively before grant and are cleared by first grant.

Track 22 owns a later portable checkpoint contract covering generations,
components, admission sequences, leases/revisions, pending scheduler/work and RNG.
Current pause/continue is the same live runtime only. For future Track 34/35
integration, keep a resource queue and its request/lease/work state within one
logical process until owners define cross-LP ownership, message identity, causal
timestamps and lookahead bounds. Local independent replications do not qualify
cross-LP shared queues or zero-lookahead cycles; those cycles retain deadlock
risk. Existing distributed scaffolds do not establish MPI/gRPC Flow execution.
Track 32 owns future device parity; Metal queue execution is unsupported.

The source-linked [migration guide](../../../libs/kairos/docs/flow/queue-migration.md)
and [backend handoff](../../../libs/kairos/docs/flow/queue-backend-boundaries.md)
connect these restrictions to existing owner records. Q5.4 phase closeout,
Track 25/security/release, clinical calibration, MVP and advanced-backend
acceptance remain separate.

Raw command logs are preserved byte-for-byte, including test runner blank lines
at EOF. The initial staged whitespace check flagged those four raw stdout logs
(exit2); source/metadata whitespace is checked with only archived `.log` files
excluded. No output bytes or receipt hashes were normalized to hide that result.
The first pin check ran before a gitlink commit and correctly rejected the old
committed pin (exit1); the postcommit check is a separate recorded gate.
