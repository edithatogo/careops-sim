# Q5.4 independent manual replay

Read-only replay at Kairos `eae890b0a2a3524a543ec4ee4aca61346e273b52`, clean worktree `/private/tmp/kairos-q54-reader`, with Rust 1.99.0 (`rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0 (5f94df478 2026-08-27)`). The targeted locked test command and both example commands, exact argv, cwd, source/manifest hashes, exit codes, output hashes and raw log paths are in `receipts.json`.

The targeted test run passed 11 tests across the four requested binaries: migration 6, deterministic conformance 1, primary preemption 1, and Q5.3 legacy public surface 3. All 12 emitted conformance batch lines reported `aggregate_fnv1a64=47cfb7dca4211252` across repeated, ordered, reversed and permuted assignments with 1/2/4 workers. The source fixture uses fixed test-local seeds; this does not qualify engine RNG, PDES, or portable checkpoints.

The primary preemption fixture confirms Suspend completes at tick 12; Restart completes at tick 15; Abort transitions to Aborted at tick 3 and has no completion event. The separate staff/bed/cleaning example's complete stdout matched its saved oracle byte-for-byte. It is a different trace, with separate staff, bed and cleaning requests and `normal_staff` completing at tick 10. FIFO example stdout also matched its saved oracle byte-for-byte.

The migration/backend documents describe Flow as a single-world CPU facade with same-runtime live continuation. They reserve portable checkpointing for Track 22 and do not claim cross-LP shared-resource equivalence, zero-lookahead safety, MPI/gRPC Flow transport, Metal queue execution, or general release/API acceptance. This replay is targeted local evidence only; it does not rerun the full suite, hosted checks, or separate release/security/clinical gates.

Public-safe artifacts in this directory are `README.md`, `manual-review.json`, `receipts.json`, and the six `*.stdout.log` / `*.stderr.log` raw command logs. Private harness context, lease metadata and the chmod-0600 token are stored outside the repository under `/private/tmp`; none is included in the public-safe artifact set.
