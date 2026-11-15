# Contributing to careops-sim

CareOps Sim is a Rust-native healthcare operations simulation workspace built around Kairos/KairoECS. Contributions should advance bounded healthcare composition, reproducible scenarios, deterministic behaviour, calibration contracts or documentation.

## Before you start

Read [AGENTS.md](AGENTS.md), [the Conductor index](conductor/index.md), the assigned task and [single-maintainer harness](conductor/harness/single-maintainer.md). Reconcile active sessions and path ownership before writing; use the existing isolated-worktree, advisory-claim and scope-check protocol. Keep simulation time/state and deterministic seeds authoritative. Kairos engine changes and parent dependency-pin updates have separate ownership.

For bugs, provide the affected version or commit, environment, minimal reproduction, expected result, observed result and sanitised diagnostic logs. For a substantial proposal, describe the use case and compatibility impact in an issue before expanding implementation scope. Use existing issue templates when offered.

## Development and validation

Use the repository-local Python runtime specified in [AGENTS.md](AGENTS.md), verify it before running commands, and preserve environments used by active jobs. Start with the documented context checks:

```sh
.venv/bin/python tools/python_runtime.py
.venv/bin/python tools/context.py resume
.venv/bin/python tools/context.py check
.venv/bin/python tools/tasks.py check
```

Run the affected Python test file or native Rust tests from the selected task. The full Python integration command is `.venv/bin/python -m unittest discover -s tests -v`. Native checks, pinned toolchains and schema requirements remain defined by the active plans and [CI](.github/workflows/ci.yml); context validation does not replace runtime tests.

Use generic public or explicitly synthetic scenarios. Record toolchain, seed, input hashes and observed results. Preserve unknown capabilities and empirical release blockers; passing a constructed fixture does not establish clinical, hardware or site-specific acceptance.

## Pull requests

Create a focused branch from the current `main` branch and submit a pull request to this repository. Link the relevant issue or existing Conductor task, explain the resulting behaviour, and update affected examples, contracts and documentation. Register added or removed tracked paths in [.careops-security/inventory.json](.careops-security/inventory.json) and follow the [privacy gate](.careops-security/README.md); an inventory entry does not approve sensitive content. Keep unrelated changes in separate pull requests.

Report executed commands, outcomes and the commit they validate. Identify unavailable checks and remaining limitations explicitly. Follow the repository's existing review and automated-check requirements; retain required gates and current ownership. Address findings and verify the final head before integration. Repository changes do not themselves authorise a release or external operational action.

## Security, rights and conduct

Follow the [security policy](https://github.com/edithatogo/.github/blob/main/SECURITY.md). Use private vulnerability reporting if available; otherwise request a private contact without publishing vulnerability details. Keep credentials, personal information and restricted source material out of issues, fixtures, logs and artifacts.

Read the [existing licence](LICENSE) and preserve attribution and third-party terms. Follow the [code of conduct](https://github.com/edithatogo/.github/blob/main/CODE_OF_CONDUCT.md) and keep technical discussions respectful.

## Basis for this guide

This guide follows [GitHub's contribution-guideline guidance](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/setting-guidelines-for-repository-contributors) and the contribution-process and testing principles in [OpenSSF Best Practices](https://www.bestpractices.dev/en/criteria). It does not assert certification.
