# CareOps thiserror adoption

Planned only. Apply [value policy](../../dependency-value-policy.md) and [sequencing](../../dependency-sequencing.md).

## Inputs and dependencies

Accepted Kairos Track 95 Phase 2 receipt; local milestones ['polars_data_processing_20261022:R14']. Do not wait for Polars to finish when this track is its prerequisite.

## Outputs and contract

Derive Error/Display incrementally while retaining public variants, source chains, messages where contractual, stable codes and FFI mapping. Do not replace typed library errors with anyhow or redesign APIs just to use macros. Apply after parsing/CLI foundation and before RNG work; future modules adopt the convention within their own tracks.

## Ownership

CareOps coordinator and current ED/core/CLI test or artifact owner own exact parent files. sha2 targets handwritten compute_sha256_hex; tempfile targets repeated TestTempDir and local staging; thiserror targets domain/CLI typed errors; proptest targets independent patient/resource/restart invariants. Only this track's dependency is in scope. No direct libs/kairos edits or assumed pin acceptance. Qualify exact consumer paths and shared manifest owners before dispatch.

## Acceptance

Error variant/code/message parity and source-chain tests; FFI mappings unchanged; no sensitive fields exposed. Count net deleted boilerplate and compile-time impact; leave trivial cases if conversion adds no value.

Preserve clinical/site semantics, recorded historical artifacts and public outputs. Runtime adoption requires independent failures/recovery and measured net benefit; no tests simply mirroring implementation.

## Stop criterion

Reconcile deleted targets after source/pin changes; no installation solely to close a track. Track acceptance covers bounded current consumers, not future Polars outputs: Polars keeps end-to-end format/durability acceptance.
