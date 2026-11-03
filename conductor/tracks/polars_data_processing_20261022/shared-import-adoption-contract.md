# R14 shared trusted-import adoption contract — 4 November 2026

This reviewed packet owns the exact parent data-source and default-off FFI
acceptance change. The coordinator retains the existing parent lane; the stopped
upstream owner used its separate standard Git worktree and committed473ff2c.
Runtime0741026 and Polars9643968 remain fixed. No main/submodule runtime pin,
other worktree, environment, registry publication or clinical/site scope changes.

## Prerequisite and dependency review

Upstream473ff2c0e640a7e457f500593f6ea1f06602ab35 has independent source/security
review,2 direct importer tests,42 existing native FFI tests,29 default controls,
strict Clippy/docs,11 negative controls,5 normal graphs and all4 policy groups.
[Exact hosted qualification](https://github.com/edithatogo/kairos/actions/runs/41667625899)
passes Linux, macOS and Windows, including committed source archive checks.
The remote Git consumer fetched that exact commit, passed2 tests and the unchanged
parent policy, and added/upgraded no registry package. Its native graph excludes
engine/core/state/RNG; default importer graph is empty. PR367 remains draft and
conflicts with later main; dispatch qualified this exact source without changing
that merge gate or adopting unrelated main work.

The canonical trusted import implementation moved byte-for-byte into the shared
crate. Arrow's unsafe prohibition, old engine/default/no-default contracts,
private owned wrappers, raw foreign-import obligations, callback ownership and
C ABI/schema remain unchanged. Upstream ADR9 records the opaque Rust defining
module relocation; old FFI source paths reexport the same types/functions.
The33-entry live registry snapshot is candidate information only. No registry,
CI tool, Python or Rust toolchain upgrade is part of this packet.

## Parent interface and compatibility

All data Git edges use exactly473ff2c; the new root native-ffi-acceptance feature
is default-off, activates only six optional edges and has an explicit required-
feature test target. The shared importer replaces the rejected engine-FFI graph;
no policy exemption or second runtime is added. Core/root/browser/default graphs
must retain their existing isolation. Native CI selects the root feature once;
metadata and deny check the activated graph.

Writers emit the new data revision only. ED reads exactly the new revision and
originalf347505, with unchanged runtime/Polars/schema/types/metadata/hash/limits
checks. Conversion preserves original data/model/input provenance and binds both
manifest hashes; mixed-source comparisons remain incompatible. Independent
readers select a closed expected profile, default new; historical conversion
selects f347 explicitly. Old files and all five earlier receipts remain immutable.
B7 recovery retains strict current model/build/input checks; ED historical reads
do not authorize old B7 checkpoint reuse. No schema-DDL/JSON table adapter returns.

## Acceptance and rollback

Require actual populated ED/calibration/file-backed B7 C Data and Stream tests,
owned source-drop/empty/multibatch/earlydrop/null/schema negatives, original capture/
recovery and numerical suites, real old-fixture conversion and independent reads,
new-release26-case/stdout budgets, strict format/Clippy, all4 activated policy
groups and final package/coverage/mutation/hosted gates. No planned check is a pass.

Use two source commits: first the actual adopted source and actual26-case receipt;
then a guard anchored to that immutable first snapshot. Historical receipts are
validated against their original commits, never rebound to the new lock or pin.
The first commit is a qualification snapshot, not final-head CI acceptance; no
intermediate head is pushed as accepted. The15 critical bindings and unknown-
source/feature/profile negatives close the follow-up guard. Final R14 remains open.

Rollback retains parent85419da/currentf347 source, old release0c2e8487 and all
original fixture/benchmark receipts. Restore a coherent old source/profile in an
isolated owned lane, use the matching build/capture contract, and preserve failed
attempts; never relabel metadata, overwrite native outputs or resume across builds.
