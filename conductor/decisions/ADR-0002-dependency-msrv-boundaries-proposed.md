# ADR-0002: Dependency and MSRV boundaries for the ED-native profile

- Status: proposed; requires Kairos owner review before implementation
- Date: 2026-09-28
- Parent base reviewed: `9b06a425abfe86b68dc96e0b8388ef6f4e239f7f`
- Kairos pin reviewed: `fae901558f07b7b717a676adbafbe2cdc78dea1c`
- Owners to review: Kairos Tracks 13 (toolchain/CI), 25 (compatibility), 30
  (version support matrix), and 04 (Arrow schema/interoperability) as applicable
- Evidence: [D1.2 compatibility assessment](../evidence/d1.2-compatibility-assessment-20260928.md)

## Context

The pinned Kairos workspace declares Rust 1.76. The currently tested core and
consumer baseline passes at Rust 1.76.0 and 1.98.1, while latest candidate
Arrow 60 and Parquet 60 declare Rust 1.88, TOML 1.1 declares Rust 1.85, and
Rayon 1.12 declares Rust 1.80. These candidates are absent from Kairos manifests
today. The current benchmark dev graph already resolves Clap 4.6.1, which
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
   feature 24-member workspace tests (228 tests) on Rust 1.76.0 using the
   scratch lock, and on Rust 1.98.1 using the original lock, both from the
   pinned Kairos source. Apply it only after owner review. If rejected, owners
   must approve an explicit unpublished benchmark-tool floor and its support
   boundary.
5. Define two separate dependency lanes: a locked tested stable integration
   baseline and a dated advisory canary, each with exact Rust/dependency
   versions, target/features and captured results. Registry latest is never a
   required integration input until the candidate is installed, resolved and
   tested. Keep MLX's C++/Metal/Xcode/Clang lane separate from Rust wrapper
   validation.

## Approval and implementation gate

This is a decision proposal, not authorization to change the Kairos submodule.
Before implementation, record Track 13/25/30/04 owner review and resolve:

- whether Rust 1.76 covers benchmark/dev targets or only published runtime;
- whether to accept the tested lock-only Criterion/Clap graph resolution or
  declare a scoped unpublished tool-only floor;
- which Kairos package owns Arrow IPC/Parquet and the accepted MSRV;
- whether the tested Rust 1.98.1 pin replaces the stale 1.95 stable selector,
  and what date/criteria govern the canary lane; and
- what Track 25 version-drop notice, compatibility ADR and affected-owner
  review apply to each exact package root.

Once approved, make changes on a Kairos development branch, test the exact
minimum and current feature graphs and downstream consumers, update contracts,
CI, dependency policy and docs in the same reviewed change, then update the
parent submodule pin to that reviewed commit. Until then, preserve the current
pin and report the benchmark-only Rust 1.76 discrepancy as unresolved.
