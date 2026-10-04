# C1.3 ingestion development acceptance — 5 October 2026

Accepted source: Kairos `c052c66532faef4f57f405eeb9162e4a1870ac15`,
[PR #212](https://github.com/edithatogo/kairos/pull/212), stacked on C1.2 PR #211.

- Immutable source mapping and checked timestamp normalization feed bounded
  chunks, multilevel external sorting and declared chronology/occupancy policy.
- Exact source bytes are read directly and hashed during consumption; duplicate
  JSON fields fail before conversion. Complete-case quarantine, exclusions,
  outcomes and privacy-safe count/hash manifests fail closed on missing evidence.
- Bounded identity/case/run state and finalization work reject overflow/oversize.
  Canonical population hashes match under reversed rows, chunks and spill limits.
- Rust 1.88 and 1.99 calibration suites pass 57 tests each. Actual PyArrow IPC
  file/stream and Parquet layouts (batch/group 1/1 and reversed 2/3) produce 54
  Rust outputs with independent full schema/metadata/population verification.
  Both explicitly invoked transported-source pipeline checks pass.
- Independent C0 logical schema/population validation reconciles 7 source
  candidates to 6 events, 1 exclusion and 2 outcomes.

Executed command/cwd/source/tool/input/output hashes, actual synthetic archives,
checksums and failed attempts are retained at the pinned child path
`conductor/evidence/c1.3-ingestion-20261004/`. Its acceptance receipt distinguishes
the exact qualified Rust source from the later fixture compression-only edit.
Hosted native-owner run [37207473299](https://github.com/edithatogo/kairos/actions/runs/37207473299)
passed all eleven jobs at this exact pin; API readback is retained in
`hosted-native.json`.

C1.3 is accepted for development only. C1.4 remains open, as do real-feed and
clinical validation, stable API and release readiness. PR #212 is not merged
while broader inherited CI, documentation and package gates remain unresolved.
The existing Q5.2 development-source hold and benchmark results are preserved.
