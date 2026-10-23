# CareOps tempfile adoption

Planned only. Apply [value policy](../../dependency-value-policy.md) and [sequencing](../../dependency-sequencing.md).

## Inputs and dependencies

Accepted Kairos Track 94 Phase 2 receipt; local milestones none. Do not block all Polars work on this track; qualify any required receipt before the named dependent migration task.

## Outputs and contract

Replace naming/cleanup with TempDir and NamedTempFile where useful. Qualify staging and cleanup for retained consumers before the specific migration task that needs them; Track 70 retains end-to-end durable publication acceptance. Stage beside destination; handle overwrite/no-clobber explicitly. persist alone does not prove crash durability: file/directory sync where required, manifest-last ordering, disk-full and recovery remain owner responsibilities. No filesystem randomness enters simulation seeds or output identities.

## Ownership

CareOps coordinator and current ED/core/CLI test or artifact owner own exact parent files. sha2 targets handwritten compute_sha256_hex; tempfile targets repeated TestTempDir and local staging; thiserror targets domain/CLI typed errors; proptest targets independent patient/resource/restart invariants. Only this track's dependency is in scope. No direct libs/kairos edits or assumed pin acceptance. Qualify exact consumer paths and shared manifest owners before dispatch.

## Acceptance

Concurrent directory creation/isolation; cleanup on normal/error paths; intentional retained diagnostics; same-filesystem publication, failure before/after persist, no-clobber and old-output preservation. End-to-end process-kill/disk-full/manifest recovery remains mandatory in Polars and is not certified by this supporting track.

Preserve clinical/site semantics, recorded historical artifacts and public outputs. Runtime adoption requires independent failures/recovery and measured net benefit; no tests simply mirroring implementation.

## Stop criterion

Reconcile deleted targets after source/pin changes; no installation solely to close a track. Track acceptance covers bounded current consumers, not future Polars outputs: Polars keeps end-to-end format/durability acceptance.
