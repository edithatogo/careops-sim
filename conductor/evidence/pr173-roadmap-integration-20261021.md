# PR173 roadmap integration — 21 October 2026

Planning base: `576f50bf871c12194db62c5b2e05dab8e5d87a0b`.
Reviewed local PR173 addendum at `942f78b` (after `5456965`), including C-R0–C-R7
and further owner/acceptance refinements. Input SHA-256: `1241f4991ea56260584c404e9778b3c9e3035ffb32ff477061b05b070768f69b`.
No remote PR head, CI or merge status is inferred. The three PR173-owned files
and PR266 were not edited. Commit history identifies this integration revision.

## Task mapping

| Addendum leaf | Existing task IDs | Incorporated obligation |
| --- | --- | --- |
| C-R0 | R0.1–R0.3, R1.3 | Reconcile copies and exact symbols/features/schemas/dependencies/receipts; preserve baselines |
| C-R1 | R2.1–R2.2, R3.1–R3.3 | Capability-preserving candidate, Q/C/physical IO/ED/checkpoint parity and rollback |
| C-R2 | R0.2–R0.3, R2.2, A3.1 | Named enums, ctypes version probes, CPU fallback and unsupported allocation are not exercised capability |
| C-R3 | R0.3, R2.3 | E6/E8 contract-only classification; separate real Mac Metal, CUDA PC and physical Mac/NAS gates |
| C-R4 | A0.2, A6.1/A6.3 | Small runner extraction first; facade/seed/worker parity, progress/cancel, output transaction and recovery |
| C-R5 | A3.1–A3.3 | Allocation/work continuation and dispatch hooks after runner qualification; cancellation/rollback/stale completion |
| C-R6 | A0.2–A0.3, A5.1–A5.2, A6.1, A7.1/B7 | ED/logistics neutral proof, separate domain adapter before broad adoption, clinical semantics remain downstream |
| C-R7 | R3.3 and every phase exit | Exact command/cwd/source/pin/features/toolchain/seed/input/host/backend/exit/raw-output identity |

The existing dependency chain places A0.2 before A3; no new track or duplicate
runner is introduced. R3 remains the baseline/candidate-decision gate. A7 still
joins A6 with B7. B0 evidence intake stays independent. Each broad task is split
into reviewed bounded leaves before dispatch; a plan is not a runtime packet.

## Current source verification and unresolved integration

Rechecked local pin `44261eb5cf8275f132f63a3396903d116d701dbe` against audited
main `f0d30e28c3299dcdb6ca138d95319ecaaf29a54a`: merge base is
`384e8546d69f9cbf2746fcb2ab646263256e6dec`, distinct from either endpoint.
The arrow-io and calibration source paths present at the pin are absent from that
main tree. This verifies divergence/path differences, not equivalence or current
remote state; R0/R2 must qualify replacements and preserve the working baseline.

Fresh readback of the original checkout at `fc4fdd6` still finds uncommitted
R2.1 advancing automatically to f0d30e2 and A3.2 treating AllOf/AnyOf as atomic
allocation. Those files were preserved. The corrected branch is the reviewed
integration candidate; all-chat alignment and integration into main are not
claimed. The original coordination note does not override this readback.

Read e62 buffer methods and metadata fixture: they change local state/counters
and construct device descriptions. Read e83: it uses threads/mpsc and computes
efficiency as 1/node_count. These remain useful contract checks, not proof of
GPU dispatch, distinct hosts or measured scaleout. No hardware was accessed here.
Historical Mac/NAS identities and failed CUDA access are audit context only;
R2.3 requires fresh device/access readback before execution.

A0.1 now requires exact-file ownership for 67 core/state/ABM and 68/69 DES overlap,
with existing owners and only consumed 64/66 capabilities as prerequisites.
A1/A3/A4/A5 incorporate age/residual clock, resource conservation, logical effect
idempotency and independent relationship/lifetime failure oracles.

## Validation

Validation executed in `/Users/doughnut/.codex/worktrees/careops-roadmap-contracts/careops-sim` at planning base `576f50b` with `3.15.0` Python. Seeds are not applicable to structural planning validation.
All new implementation tasks remain unchecked; no source pin/runtime code changes.

| Executed command | Exit | stdout + stderr SHA-256 |
| --- | --- | --- |
| `python3 tools/context.py check` | 0 | `5c808dcead7552939348f676a0e49891e31c75840288045aea81fcdf4a4e54bf` |
| `python3 tools/tasks.py check` | 0 | `e716c64f63ce4d00b7675e976f3629d11d5c2e59d13a90ea4b79002ccf9d5c2c` |
| `python3 tools/mvp.py check` | 0 | `3c7b9a3062f62964126c8db247833e935b8557a4baeffb7a30525cbe5f410966` |
| `git diff --check` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Raw structural logs retained locally as `/tmp/pr173-integration-check-0.log`
through `-3.log`; hashes above identify their exact output. Context: 8 tracks,
54 milestones, 1,452 local links and no errors. Catalog: 225 records with complete
coverage and valid DAG. MVP: 82 parents / 258 leaves. Targeted harness command
`python3 -m unittest discover -s tests -p test_tasks.py -v` separately passed all
32 tests (0.612 seconds); its output was read directly in this chat.

Compared old/new task records: all accepted flags and dependency edges unchanged.
Checked transitive A0.2-before-A3.1 ordering and independent B0.1; both passed.
Historical acceptance remains 145 checked / 80 pending. No runtime tests, device
workloads, hosted CI, merges or remote publishing were performed in this slice.
