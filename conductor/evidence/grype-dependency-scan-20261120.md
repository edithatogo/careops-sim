# Native candidate dependency scan — 20 November 2026

This bounded D2 release/CI maintenance adds a separate CycloneDX dependency
inventory from the `dist/resolved-workspace.Cargo.lock` copied after the locked
build. Grype enforces critical findings before the main/manual candidate can be
retained and attested; PR vulnerability findings are advisory. Operational scan
errors remain failures. The original one-package SPDX byte inventory, unpacked
consumer test and exact-byte/signature checks are preserved.

The source advisory workflow inventories all lockfile packages at its recorded
checkout. It does not infer linked binary content or qualify Kairos behavior.
The isolated writer used an advisory path claim and bounded context; local checks
and pending hosted/native gates are recorded in the scoped PR. No active core,
Polars or domain track is claimed, reopened or completed.
