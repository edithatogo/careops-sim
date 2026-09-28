# Current dependency and toolchain policy

Live registry observations are recorded in
[dependency-candidates.json](evidence/dependency-candidates.json), including source
URLs, fetch time, release date and declared MSRV. Refresh with
`python3 tools/refresh_versions.py`; it never installs or updates dependencies.
The snapshot is a candidate inventory, not a tested lockfile or promise that all
packages should be adopted. Unknown MSRV means unknown, not compatible.

## Live current candidates, refreshed 2026-09-28

| Area | Registry candidate | Adoption decision |
| --- | --- | --- |
| Rust | 1.98.1 | Owner-approved exact development/integration pin; tested with the full default-feature workspace |
| Arrow array/schema/IPC, Parquet | 60.0.0; MSRV 1.88 | Owner-approved future package boundary; defer dependency adoption until C1 schema/interoperability review |
| serde / serde_json | 1.0.229 / 1.0.151 | Evaluate compatible lockfile update in owning change |
| toml | 1.1.6+spec-1.1.0; MSRV 1.85 | Owner-approved only as a future isolated config/tool package if needed; keep out of core |
| rayon | 1.12.0; MSRV 1.80 | No mandatory within-run scheduler dependency; retain deterministic event order |
| crossbeam-epoch | Patched `0.9.20` lock pin (MSRV 1.61); registry latest observed `0.9.21` | Lock-only repair for RUSTSEC-2026-0204 in the benchmark-only Criterion/Rayon graph; cargo-deny advisory/source gate passes. |
| clap / clap_builder | 4.6.7; MSRV 1.85 | The original Kairos benchmark lock resolved Clap 4.6.1, which Cargo 1.76 cannot parse. The Kairos owner approved a six-package lock-only resolution, committed on `codex/d12-bench-lock-msrv` and tested across the default-feature workspace at Rust 1.76.0 and 1.98.1; the parent pin/evidence records the exact commit. |
| rand | 0.10.3; MSRV 1.85 | Optional sampling candidate; do not replace deterministic RNG algorithm silently |
| wgpu | 30.0.1; MSRV 1.87 | E6 real Metal candidate, gate shader/API/device compatibility |
| Burn | 0.21.0; MSRV 1.92 | Deferred existing ML track; no dependency added now |
| nextest / deny / audit | 0.9.146 / 0.20.2 / 0.22.2 | D1 pin dev tools; installed audit observed 0.22.1, upgrade remains pending |
| semver-checks / llvm-cov / mutants | 0.50.0 / 0.9.1 / 27.1.0 | D3 targeted quality tools with overhead measured |
| proptest / criterion / insta | 1.11.0 / 0.8.2 / 1.48.0 | Targeted invariant/benchmark/snapshot tooling; no unnecessary runtime dependencies |
| cargo-fuzz / zizmor | 0.13.2 / 1.30.1 | D3 fuzz/nightly; D2 workflow analysis |
| checkout / upload-artifact / download-artifact | v7.0.1 / v7.0.1 / v8.0.1 | Resolve releases to full verified commit SHAs and check runner compatibility in D2 |
| CodeQL action | v4.38.2 | Use action release, not the separate codeql-bundle tag |
| Renovate / actionlint / gitleaks | 44.115.12 / v1.7.12 / v8.30.1 | Pin tested automation/tool versions in D1/D2 |
| Codex CLI | rust-v0.157.1 | Observed upstream candidate; not a claim about this desktop app's version |

Sources are the linked crates.io/GitHub/Rust distribution endpoints in the JSON.
No production dependency, host toolchain or application model was upgraded by
this audit. The upstream lock contains the owner-approved benchmark-only Clap-family
compatibility resolution plus a separate advisory fix for crossbeam-epoch; it does not change Kairos manifests, runtime dependencies or MSRV.
Conductor v0.4.1 is still its latest observed release; default-branch HEAD is now
`6e8f9a860bcdd6a2c423473c12e745200688c633`. Evaluate that commit in an isolated
canary before deciding whether an unreleased change improves this workflow.

## Bleeding-edge with reproducibility

Maintain two lanes: an exact tested integration baseline and an advisory current/
preview canary. Adopt the newest useful compatible stable release promptly;
prerelease/nightly/HEAD experiments use exact commits/dates, bounded benchmarks,
rollback evidence and explicit capability labels. Do not leave required builds
on floating `latest`, `stable`, action tags or unbounded `cargo install` commands.

Record exact dependency resolutions in Cargo.lock and tool/action pins in the
owning repository. CI uses `--locked`; publishable Rust libraries still need
appropriate dependency ranges and minimum/current resolution tests. Pinning only
a library's lockfile does not constrain downstream consumer resolution.

D1's policy direction is owner-approved: preserve the existing default-feature
workspace floor at Rust 1.76; pin the developer/integration stable lane to Rust
1.98.1; record Rust 1.99.0-beta.8 observed 2026-09-27 as a non-blocking advisory
canary. Keep a raised floor for any existing package/feature behind Tracks 25/30's
transition policy. Future Arrow/Parquet belongs behind a separately reviewed
package boundary at candidate floor 1.88; optional TOML parsing belongs outside
core at candidate floor 1.85. Do not advertise Rust 1.76 for a feature graph that
includes Arrow 60. Do not add these candidate dependencies until source contracts,
manifests, CI, interoperability checks and docs are aligned. See the D1.2
compatibility assessment for test commands, receipts and pending Track 13/30
review; owner approval does not itself close the D1 phase.

Updates: weekly grouped low-risk patch/minor review, immediate advisory triage,
separate major/API/RNG/schema changes. New versions are reverified immediately
before adoption. Run conformance, resume, feature/MSRV and affected performance
checks. Automatic dependency merges only after required checks and stable policy;
never auto-merge RNG/time/schema, security-permission or public-release changes.
Automation is planned here; no recurring task has been enabled by this audit.
