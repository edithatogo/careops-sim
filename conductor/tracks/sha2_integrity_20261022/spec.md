# CareOps sha2 adoption

Planned only. Apply [value policy](../../dependency-value-policy.md) and [sequencing](../../dependency-sequencing.md).

## Inputs and dependencies

Accepted Kairos Track 93 Phase 2 receipt; local milestones none. Do not wait for Polars to finish when this track is its prerequisite.

## Outputs and contract

Replace sha256_simple (which is not SHA-256) only where retained contracts require SHA-256. Delete the obsolete pseudo-Parquet codec through Track 70 rather than repairing it into a parallel format. Qualify standard SHA-256 primitives and retained consumers before Track 70; do not require completed Polars outputs for this track to close. Version any changed digest/format contract; preserve historical artifacts and reject incompatible formats explicitly. Keep noncryptographic identity hashes and RNG stream assignments unchanged. CareOps already declares sha2 but still maintains handwritten SHA-256: package presence is not migration.

## Ownership

CareOps coordinator and current ED/core/CLI test or artifact owner own exact parent files. sha2 targets handwritten compute_sha256_hex; tempfile targets repeated TestTempDir and local staging; thiserror targets domain/CLI typed errors; proptest targets independent patient/resource/restart invariants. Only this track's dependency is in scope. No direct libs/kairos edits or assumed pin acceptance. Qualify exact consumer paths and shared manifest owners before dispatch.

## Acceptance

Independent standard empty/abc/multiblock/boundary-length vectors; Python hashlib or another independent implementation; single-bit corruption, truncated/invalid digest, full-byte coverage including high tick bits; old-format rejection/version tests. Do not treat the old custom hash as a SHA-256 oracle.

Preserve clinical/site semantics, recorded historical artifacts and public outputs. Runtime adoption requires independent failures/recovery and measured net benefit; no tests simply mirroring implementation.

## Stop criterion

Reconcile deleted targets after source/pin changes; no installation solely to close a track. Track acceptance covers bounded current consumers, not future Polars outputs: Polars keeps end-to-end format/durability acceptance.
