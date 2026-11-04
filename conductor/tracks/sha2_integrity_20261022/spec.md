# CareOps sha2 adoption

R50.1 prerequisite verification and consumer inventory are accepted; runtime adoption and the value decision remain planned. Apply [value policy](../../dependency-value-policy.md) and [sequencing](../../dependency-sequencing.md).

## Inputs and dependencies

Accepted Kairos Track 93 Phase 2 receipt; local milestones none. Do not block all Polars work on this track; qualify any required receipt before the named dependent migration task.

## Outputs and contract

The current parent replacement candidate is the handwritten standard FIPS SHA-256
`compute_sha256_hex` in `crates/careops-ed/src/api_runner.rs`. The nonstandard
`sha256_simple` is an upstream historical target and is absent from current
parent production source. Polars already removed the obsolete parent pseudo-
Parquet paths; do not reintroduce a parallel codec. Preserve the accepted upstream
historical receipts and distinguish them from current runtime qualification.

CareOps already declares and locks sha2 0.11.1. Core identity, birthing runtime and
build scripts already use it; ED currently has it only as a build dependency.
R50.2 must assess the normal ED runtime edge, exact features/minimal graph,
independent digest parity and net value before replacement. Existing correct
standard uses, canonical identity bytes and RNG assignments remain unchanged.
A byte-identical standard digest does not need an invented format migration;
actually changed digest/format contracts require explicit versioning. Current
historical receipt limitations and source-bound manifest guards are recorded in
[the R50.1 inventory](../../evidence/sha2-r501-inventory-20261104/README.md).


## Ownership

CareOps coordinator and current ED/core/CLI test or artifact owner own exact parent files. sha2 targets handwritten compute_sha256_hex; tempfile targets repeated TestTempDir and local staging; thiserror targets domain/CLI typed errors; proptest targets independent patient/resource/restart invariants. Only this track's dependency is in scope. No direct libs/kairos edits or assumed pin acceptance. Qualify exact consumer paths and shared manifest owners before dispatch.

## Acceptance

Independent standard empty/abc/multiblock/boundary-length vectors; Python hashlib or another independent implementation; single-bit corruption, truncated/invalid digest, full-byte coverage including high tick bits; old-format rejection/version tests. Do not treat the old custom hash as a SHA-256 oracle.

Preserve clinical/site semantics, recorded historical artifacts and public outputs. Runtime adoption requires independent failures/recovery and measured net benefit; no tests simply mirroring implementation.

## Stop criterion

Reconcile deleted targets after source/pin changes; no installation solely to close a track. Track acceptance covers bounded current consumers, not future Polars outputs: Polars keeps end-to-end format/durability acceptance.
