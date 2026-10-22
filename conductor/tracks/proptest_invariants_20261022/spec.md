# CareOps proptest adoption

Planned only. Apply [value policy](../../dependency-value-policy.md) and [sequencing](../../dependency-sequencing.md).

## Inputs and dependencies

Accepted Kairos Track 96 Phase 2 receipt; local milestones none. Do not wait for Polars to finish when this track is its prerequisite.

## Outputs and contract

Reuse existing proptest rather than introduce a second property framework. Establish independent reference models, reproducible seeds, bounded cases/shrinking and saved minimal regressions before replacing internals. Initial acceptance covers existing behaviour; future dependency tracks add their own properties, so acceptance does not wait for their implementations. Never make the implementation its own oracle or remove conformance/fuzz tests.

## Ownership

CareOps coordinator and current ED/core/CLI test or artifact owner own exact parent files. sha2 targets handwritten compute_sha256_hex; tempfile targets repeated TestTempDir and local staging; thiserror targets domain/CLI typed errors; proptest targets independent patient/resource/restart invariants. Only this track's dependency is in scope. No direct libs/kairos edits or assumed pin acceptance. Qualify exact consumer paths and shared manifest owners before dispatch.

## Acceptance

Mutation/negative controls show properties detect broken conservation, ordering and invalid-input rollback; seed replay and shrinking work; generated IDs include full ranges; test budgets enforced. New-format resume properties remain Track 70 acceptance responsibility.

Preserve clinical/site semantics, recorded historical artifacts and public outputs. Runtime adoption requires independent failures/recovery and measured net benefit; no tests simply mirroring implementation.

## Stop criterion

Reconcile deleted targets after source/pin changes; no installation solely to close a track. Track acceptance covers bounded current consumers, not future Polars outputs: Polars keeps end-to-end format/durability acceptance.
