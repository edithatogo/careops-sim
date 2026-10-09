# C2 current development toolchain policy packet

- Worktree: `/Users/doughnut/Documents/careops-sim-session-worktrees/kairos-c2-current-toolchain-policy-20261009`
- Branch: `codex/c2-current-toolchain-policy-20261009`
- Base HEAD: `adf5fed6a86670b5b07be5e7522f174f7823ef0c`
- Scope: Track 30 matrix, validator, test matrix, ADR-0015; this is an internal execution-profile policy only.

## Executed commands

1. Expected-red static validator (workflow is owned separately by Track 13):

   `pwsh -NoProfile -File conductor/tracks/30-toolchain-version-support-matrix/validate-toolchain-matrix.ps1`

   Exit 1 as expected. First mismatch: `Missing workflow lane matching matrix: channel: "1.99.0"`. Raw output: `static-validator.log`.

2. Local pinned toolchain version check:

   `PATH='/Users/doughnut/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:'"$PATH"; for tool in rustc cargo rustfmt; do version=$("$tool" --version) || exit 1; printf '%s\n' "$version"; case "$version" in *1.99*) ;; *) exit 2 ;; esac; done`

   Exit 0: rustc 1.99.0 (b940084d7 2026-09-28), cargo 1.99.0 (5f94df478 2026-08-27), rustfmt 1.10.0-stable (b940084d7e 2026-09-28). Raw output: `rust-installed.log`.

3. The Track 30 script's `-CheckInstalled` form was attempted with the pinned toolchain first in `PATH`, but static validation runs first and returned the same expected-red workflow mismatch before reaching the installed-version check. This is captured in the ADR; the local tool versions above were checked directly.

No Rust tests, builds, older toolchains, beta toolchains, network operations, or workflow edits were performed in this packet.

## Scope and claim boundary

The packet leaves Cargo `rust-version` declarations unchanged and labels them historical/unverified by this profile. It makes no claim of upstream discontinuation, changed minimum support, platform support, hosted CI acceptance, stable runtime contract, or C2 acceptance. Existing non-Rust matrix/validator rules are retained. Root coordinator must re-run the static validator after the Track 13 workflow integration.

## Verification correction

The first direct version probe attempted to find substring `1.99` in each tool's `--version` text. That criterion is invalid for rustfmt, whose component reports `rustfmt 1.10.0-stable`; the failed initial output is retained as `rust-installed.log`. The corrected command verifies that each executable resolves under the pinned Rustup `1.99.0` toolchain directory and checks exact `1.99.0` version text for `rustc` and Cargo. The corrected probe exited 0; output is `rust-installed-final.log`.

## Commit and remaining evidence

- Commit: `9a18656437c4e948596e0633f4b46588af516434` (`docs: qualify C2 on pinned Rust 1.99`)
- Tracked working tree is clean after commit. Branch remains local; no push was performed.
- `static-validator.log` SHA-256: `d242f5fe011844040bbfa96dc4d4e74c275da8a91c46f437fd98bf5f1dd150f8` (expected-red Rust workflow mismatch).
- `rust-installed-final.log` SHA-256: `d0bddeb72c62930e3d4979e4bcae58f8a3c818bb6bace6ea3a30b2597a8061da` (exit 0).
- `rust-installed.log` SHA-256: `c1e92110a4d9ad7c1f13b2b439274d8541419104328bf6d491aa065af10870cc` (preserved initial rustfmt version-text probe; not a valid pass).
- Complete source and log checksums are in `SHA256SUMS`.

Unverified: Track 13's workflow integration and post-integration static validator pass; hosted CI and native test execution under this packet; older Cargo-declared MSRV compatibility; Rust cross-platform support; public API/release gates; C2 phase acceptance. This policy packet claims none of these.

## ADR factual correction successor

- Commit: pending at receipt update.
- Corrected the Context to state that the existing native-owner workflow tests default calibration features and omit C2 Flow execution; local C2 receipts already exercise the flow profile, and Track 13 adds explicit hosted flow/demo coverage.
- Only `conductor/design/calibration/ADR-0015-c2-current-stable-native-ci.md` changed in this successor. Original expected-red and toolchain logs were preserved without rewriting.
- Validation: `git diff --check` passed; `agent_session.py check` passed. No tests/builds run.
- Successor commit: `2e6b12f792f774e9b8d2f8e03ee5cf9652b4266c` (`docs: clarify hosted C2 flow coverage boundary`).
- Corrected ADR SHA-256: `182f3c7310195c212bbaa3df994710390e40fcf8c1f1be97f2646366493cc6f5`.
