# ADR-0002: Dependency and MSRV boundaries for the ED-native profile

- Status: partially approved; the exact benchmark-only lock resolution below is
  approved and implemented; remaining compatibility-boundary decisions require
  the relevant Track 13/25/30/04 owner reviews
- Date: 2026-09-28
- Parent base reviewed: `9b06a425abfe86b68dc96e0b8388ef6f4e239f7f`
- Kairos source reviewed: `fae901558f07b7b717a676adbafbe2cdc78dea1c`
- Kairos implementation commit: `45679c119732b56858e0f5a4c9d4788323f63122`
- Owners to review: Kairos Tracks 13 (toolchain/CI), 25 (compatibility), 30
  (version support matrix), and 04 (Arrow schema/interoperability) as applicable
- Evidence: [D1.2 compatibility assessment](../evidence/d1.2-compatibility-assessment-20260928.md)

## Context

The initially reviewed Kairos workspace declares Rust 1.76. The tested core and
consumer baseline passes at Rust 1.76.0 and 1.98.1, while latest candidate
Arrow 60 and Parquet 60 declare Rust 1.88, TOML 1.1 declares Rust 1.85, and
Rayon 1.12 declares Rust 1.80. These candidates are absent from Kairos manifests
today. At the reviewed source pin, the benchmark dev graph resolved Clap 4.6.1, which
requires Rust/Cargo 1.85/Edition 2024 and fails before compilation under Cargo
1.76. Its locked Rust 1.98.1 test passes.

Kairos Track 25 treats compatibility promises as protected surfaces. Track 30
requires a two-release-cycle/six-month deprecation process for version drops;
Track 13 owns Rust toolchain and CI changes. Track 30's matrix and validator
currently name Rust 1.95 as stable, predating this September 2026 evidence.

## Proposed decisions

1. Preserve Rust 1.76 for the existing published core/consumer baseline. Do not
   change the workspace floor until the exact affected roots, consumer impact,
   deprecation transition and owner signoffs are approved.
2. Do not add Arrow 60/Parquet 60 to a package or feature graph advertised as
   Rust 1.76. Put future real Arrow IPC/Parquet support behind its own package
   boundary and explicitly declare/test its floor (candidate 1.88), after Track
   04 agrees the schema and independent interoperability contract.
3. Keep TOML configuration parsing out of existing core. If the new TOML
   candidate is adopted, isolate it in the relevant tool/config package and
   declare/test the minimum consumer floor (candidate 1.85).
4. Do not make Rayon a required DES/ABM runtime dependency or use it to reorder
   within-run events. Retain deterministic scheduling semantics. The preferred
   benchmark-only fix is the tested lock-only compatibility resolution
   (`clap`/`clap_builder` 4.5.58, `clap_derive` 4.5.55, `clap_lex` 1.0.1,
   `anstream` 0.6.21, `anstyle-parse` 0.2.7), which passed the full default-
   feature 24-member workspace tests (228 tests) on Rust 1.76.0 and Rust 1.98.1.
   The Kairos owner approved this exact six-package lock-only change on
   2026-09-28; it is committed as `45679c1` on `codex/d12-bench-lock-msrv`. If
   this resolution is later rejected, owners must approve an explicit
   unpublished benchmark-tool floor and its support boundary.
5. Define two separate dependency lanes: a locked tested stable integration
   baseline and a dated advisory canary, each with exact Rust/dependency
   versions, target/features and captured results. Registry latest is never a
   required integration input until the candidate is installed, resolved and
   tested. Keep MLX's C++/Metal/Xcode/Clang lane separate from Rust wrapper
   validation.

## Approval and implementation gate

The Kairos owner (user-confirmed in this conversation) approved only the exact
benchmark-only six-package Cargo.lock resolution listed in decision 4 on
2026-09-28. This approval does not approve manifest/MSRV, API, CI, Arrow/TOML,
stable-selector or other compatibility changes. The approved lock change is
committed in Kairos as `45679c119732b56858e0f5a4c9d4788323f63122`; the full
workspace passed at Rust 1.76.0 and 1.98.1 on macOS ARM. See the updated
[D1.2 compatibility assessment](../evidence/d1.2-compatibility-assessment-20260928.md)
for commands and log hashes.

The following decisions remain open and require the relevant Track 13/25/30/04
owner review before changing compatibility promises or adding dependencies:

- whether Rust 1.76 covers benchmark/dev targets or only published runtime;
- whether Rust 1.76 covers benchmark/dev targets beyond the repaired Criterion
  graph or a scoped unpublished tool-only floor is required;
- which Kairos package owns Arrow IPC/Parquet and the accepted MSRV;
- whether the tested Rust 1.98.1 pin replaces the stale 1.95 stable selector,
  and what date/criteria govern the canary lane; and
- what Track 25 version-drop notice, compatibility ADR and affected-owner
  review apply to each exact package root.

For those remaining decisions, preserve existing package promises until
reviewed. Any approved compatibility change must update contracts, CI,
dependency policy and docs together. The parent pin may advance to the lock-only
commit above after integration evidence is recorded; this does not imply the
remaining decisions are accepted.
