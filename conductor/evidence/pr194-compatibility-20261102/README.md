# PR194 exact-pin compatibility repair — 2 November 2026

Base: `03207671e632a3ea90f06b785407de3abdadbb92`. Kairos remains
`07410266bf54b6f73c59db2f8cbfbc5697c53734`; no upstream source or pin changes.

The conformance adapter now uses the pin's purpose-specific
`StagedResourceRuntime`, `StagedRequestId`, `StagedResourceId` and
`StagedRequestState`, its checked constructor, direct submission and capacity
snapshot. The capacity-one contention, observed lease revision, queued grant,
release readback and CareOps acknowledgement assertions are retained. This
qualifies the bounded single-resource contract only; it does not restore the
removed general-purpose Flow API or qualify atomic multi-resource allocation.

The only ED lib source change in the integration packet adds `pub mod identity;`.
It shifts existing mutation locations by one line. The validator binds the new
source/catalog hashes and shifted identities; all 116 rows, the same two exact
equivalent misses, three unviables, command identity, baseline and zero-timeout
requirements remain enforced. The historical source/catalog rejection controls
remain active. A live mutation experiment is required before acceptance.

The fuzz lock adds the eight transitive packages required by the core lane's
sha2 dependency. Existing locked package versions remain unchanged. Parent
`cargo update --workspace` leaves its lock unchanged.

## Verification and acceptance

Executed in the isolated repair worktree with Rust/Cargo 1.99.1 and repo-local
Python 3.15.1. Logs are private under `<PR194_EVIDENCE_ROOT>`.

- Allocation conformance: 1 passed (the original target failed on missing imports).
- Mutation validator: 20 passed (originally 3 failures and 3 errors).
- Locked workspace tests: 554 passed across 77 targets.
- All-target/all-feature Clippy with warnings denied: passed.
- Parent and fuzz cargo-deny: advisories, bans, licences and sources passed.
- Full hash-locked Python suite, source-bound mutation experiment, independent
  review and exact-head hosted CI: pending.

An initial local workspace run exhausted the system disk; build output moved to
a dedicated external directory. An interrupted mutation run shared a build
directory with another validation process; its successor's baseline detected
contaminated compiled output and failed. That result is rejected. The replacement
experiment uses cargo-mutants' isolated build directories. An initial Python
suite lacked the locked schema dependencies; it is not acceptance evidence.
Historical receipts are retained as historical evidence, not fresh qualification.
A7.1 still requires birthing B7.4; clinical/operational and hardware gates remain.
