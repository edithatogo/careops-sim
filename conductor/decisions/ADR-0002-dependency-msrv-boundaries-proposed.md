# ADR-0002: Dependency and MSRV boundaries for the ED-native profile

- Status: owner-approved direction; Track 13/30 local implementation review passed;
  D1.2 source commits and formal phase acceptance remain pending
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
   baseline at Rust 1.98.1 and a dated advisory beta snapshot (Rust
   1.99.0-beta.8 observed 2026-09-27), each with exact Rust/dependency versions,
   target/features and captured results. The beta selector is floating and its
   job is non-blocking; the recorded expected-version mismatch forces a visible
   refresh instead of silently redefining the snapshot. Registry latest is
   never a required integration input until the candidate is installed,
   resolved and tested. Keep MLX's C++/Metal/Xcode/Clang lane separate from Rust
   wrapper validation.

## Approval and implementation gate

The Kairos owner approved the direction in decisions 1-3 and 5 on 2026-09-28:
preserve the tested Rust 1.76 default-feature workspace floor; put future
Arrow/Parquet behind a separately reviewed package boundary at candidate floor
1.88; keep TOML parsing out of core and, if adopted, isolate it at candidate
floor 1.85; pin CI/developer stable to the tested 1.98.1 baseline and maintain
the dated advisory beta snapshot above. Decision 4's exact benchmark-only
six-package lock resolution was separately approved and committed earlier as
`45679c119732b56858e0f5a4c9d4788323f63122`.

The exact baseline, toolchain selectors, workflow checks and Track 30 validator
were updated together. Local Track 13/30 review found no blocking findings. The
final implementation run passed the 228-test, 64-suite default-feature
workspace on Rust 1.76.0 and Rust 1.98.1 and compiled
the workspace on Rust 1.99.0-beta.8. See the updated
[D1.2 compatibility assessment](../evidence/d1.2-compatibility-assessment-20260928.md)
for exact commands, log hashes and remaining integration gates. Owner approval
and local review do not imply D1.2 phase acceptance.

The following integration and acceptance gates remain before D1.2 can close:

- commit the reviewed Kairos source and parent evidence, then update the parent
  pin to the exact reviewed commit;
- verify the root's Track 25 compatibility contract is unchanged and the
  version-drop policy is not triggered because no existing MSRV is raised;
- retain Arrow/Parquet schema ownership and independent interoperability as a
  prerequisite for any future package implementation; and
- rerun parent D1.2 and integration tests from the final recorded Kairos pin.

No Arrow, Parquet, TOML, or Rayon dependency is added by this decision. A separate
Cargo.lock-only security update pins patched `crossbeam-epoch 0.9.20` after the
local Track 13 audit found RUSTSEC-2026-0204 in the benchmark dev graph; its MSRV
is 1.61 and both cargo-deny and cargo-audit pass. Existing
package promises remain unchanged. Any future compatibility change must update
contracts, CI, dependency policy and docs together. The parent pin should
is now pinned to `339af4e7365e70ad7e67fe3e934e4fb215fbaf8b`; local integration passed. Formal phase acceptance remains at D1.6.
