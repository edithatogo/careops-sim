---
name: dependency-upgrade-review
description: Review Rust dependency and CI tool upgrades for CareOps Sim and Kairos against pinned versions, MSRV, platform support, security, licenses, and reproducibility. Use before proposing dependency or toolchain changes.
---

# Dependency upgrade review

Prepare evidence for a maintainer; do not install, publish, push, merge, or edit manifests/lockfiles unless a separate reviewed task explicitly owns those writes.

## Inputs

- Exact parent commit and Kairos pin/branch when applicable.
- `conductor/dependency-policy.md`, relevant manifests/lockfiles, support profile, and CI/toolchain configuration.
- A read-only registry/version snapshot and named advisory/licence sources.

If the pin, policy, or source snapshot is missing or stale, stop with `hold_for_evidence`.

## Review

1. Run `python3 tools/refresh_versions.py` only as a read-only candidate refresh; inspect the complete diff and retain its timestamp/source metadata. A candidate is not approval to upgrade.
2. Trace the affected dependency/tool graph, including target-specific and dev dependencies. Check current and declared MSRV, feature resolution, platform support, duplicate versions, licenses, advisories, and source integrity.
3. Preserve workspace compatibility commitments. Separate core runtime dependencies from optional data/config/dev tooling; do not introduce Python runtime requirements into Rust-native modules.
4. Specify the smallest version/lock change, exact tests across supported toolchains/features/platforms, rollback path, and unresolved hosted checks. Do not claim hosted/platform validation from local results.
5. Record exact commits, snapshot/input hashes, tool versions, commands, exit status, and artifact paths.

## Output

Return `candidate`, `no_change`, `revise`, or `hold_for_evidence`; include proposed version and source, dependency path, compatibility/security/licence evidence, exact verification plan/results, rollback, and unverified platforms. A green audit does not approve a version or release.
