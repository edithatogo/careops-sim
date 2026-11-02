# PR191 context recovery and PR194 integration — 2 November 2026

PR194 was marked ready and merged at
`672d3e8f78f5e502a760b9d41200fdbaf4d9d144`, from exact head
`2a5d601e4ccac3feef9310a8dfe829ed8371a4c8`: 16 successful hosted checks,
zero failures, conditional main-only attestation skip, and required aggregate
passed. Original owner checkouts and private bindings remain unchanged.

PR191 base: `96e8cc810bcbca50389983f42f04d7c4e6dec223`. Its original context
run `41309901039` exhausted the 30-minute orchestration budget. Coverage/setup
used approximately 12 minutes; the retained mutation artifact accounts for only
47 of 117 mutants (46 caught, one unviable, zero timeouts). The run was cancelled
during native compilation/linking. This is incomplete evidence, not a mutation
pass. Original failed/cancelled receipts remain retained.

## Bounded repair

- Merge accepted main into the existing PR lineage in this chat's isolated
  worktree. Resolve only Cargo and mutation-binding conflicts. Preserve both
  native capture and core identity modules; do not rewrite either implementation.
- Keep the PR's qualified data pin `4134a3733ba7e833a8c520621ec2af5a4bd4b31a`
  and runtime Git revision `07410266bf54b6f73c59db2f8cbfbc5697c53734`. Bind
  the staged DES conformance dependency to the same runtime revision, keeping
  its request/time types identical to the simulation graph. No upstream source
  changes or new revision promotion. The lock adds only that retained-revision
  DES/state requirement; no registry package version upgrade is requested.
- Bind the combined source to its freshly generated 117-mutant catalogue and
  one-line-shifted identities. Preserve the PR191 source-bound two equivalences,
  four exact unviable identities, successful-baseline/command checks and zero
  timeouts. This does not convert the incomplete old run into acceptance.
- Give the finite context job a 75-minute orchestration budget, based on the
  observed incomplete native-data run, and cap each Cargo build at one compiler
  job to reduce concurrent native-link memory pressure. Retain two mutation
  workers, the 180-second per-mutant limit, all 117 mutants, the 80% coverage
  floor, all native tests, policy/security checks and required aggregate. No
  ignored tests, retries, advisory suppression or feature filtering is added.

## Verification and acceptance contract

Verified repo-local Python 3.15.1 and Rust/Cargo 1.99.1. Local focused evidence:
23 workflow tests, 20 mutation-validator tests, one real staged-runtime
conformance test, pin contract, context/task/MVP integrity, formatting, and both
workspace/fuzz dependency-policy checks pass. The new budget/scope regression
test failed before the workflow repair and passed afterward.

Full combined-source native, Python, coverage, mutation and policy results must
come from the exact PR head's terminal hosted checks. Their live authority is
[PR191 checks](https://github.com/edithatogo/careops-sim/pull/191/checks), with
private raw command/source/fixture/log hashes under `<PR191_RECOVERY_EVIDENCE>`.
A green earlier revision or a partial mutation catalogue cannot satisfy this gate.
Independent integrated-commit review remains required.

R11.1 retains its scoped test-authoring acceptance. R11.2 is not accepted by
this CI repair; R11.3 crash durability, R11.4 review, R12–R14 and core A7/B7
joins remain pending. The earlier dependency blocker in the original handoff is
historical: the newer PR191 graph passes current advisory, duplicate, licence
and source policy locally; exact combined-source hosted evidence still controls.
No clinical, operational, release, shared-event-loop or new hardware acceptance.
