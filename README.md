# CareOps Sim

[![CI](https://github.com/edithatogo/careops-sim/actions/workflows/ci.yml/badge.svg)](https://github.com/edithatogo/careops-sim/actions/workflows/ci.yml)
[![Session Guard](https://github.com/edithatogo/careops-sim/actions/workflows/agent-session.yml/badge.svg)](https://github.com/edithatogo/careops-sim/actions/workflows/agent-session.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE-APACHE)
[![Rust: 1.99.0](https://img.shields.io/badge/rust-1.99.0-orange.svg)](rust-toolchain.toml)
[![Platforms: Linux x86_64 | macOS ARM64](https://img.shields.io/badge/platform-Linux%20x86__64%20%7C%20macOS%20ARM64-lightgrey.svg)](conductor/dependency-policy.md#cross-platform-reproducibility-policy)
[![Conductor Progress: 100%](https://img.shields.io/badge/progress-100%25%20(145%2F145%20tasks)-brightgreen.svg)](conductor/tracks.md)

A planned Rust-native generic emergency-department simulation using Kairos/KairoECS
DES and ABM. Generic/public or explicitly synthetic inputs first; Cairns later.

## Delivery order

1. Functional headless MVP: configured patient flow, beds/staff/resources, basic
   spatial behavior, runnable scenarios and reproducible result tables.
2. Hardened native v1: usable Rust API/CLI, compare/export, calibration/validation,
   repeatable batches, recovery, documented inputs and release qualification.
3. Later extensions: spatial visualization/CAD, live UI, Wasm, Metal and distributed
   execution. No graphical interface is required for MVP or v1.

See [delivery scope and acceptance](conductor/delivery-contract.md) and
[Conductor index](conductor/index.md).

## Current state

The codebase is at **100.0% overall completion** (all 145 of 145 task records accepted in Conductor DAG):
- `empirical_calibration_20260925` (C0–C6): **100% complete** (32/32 tasks accepted; [C6.5 closeout](conductor/evidence/c6.5-phase-acceptance-20261016.md)).
- `des_queue_preemption_20260925` (Q0–Q5): **100% complete** (25/25 tasks accepted).
- `ed_parameter_evidence_20260927` (P0–P5): **100% complete** (24/24 tasks accepted).
- `development_readiness_20260925` (D0–D5): **100% complete** (28/28 tasks accepted; [D5.4 closeout](conductor/evidence/d5.4-phase-acceptance-20261017.md)).
- `generic_ed_delivery_20260925` (E0–E8): **100% complete** (36/36 tasks accepted; all milestones G1, G2, G3-Metal, G3-PDES, and G3-Dist closed; [E8.4 closeout](conductor/evidence/e8.4-phase-acceptance-20261017.md)).

All milestone gates (D0, D1, D2, D3, D4, D5, P1, P2, P3, P4, C1, C2, C3, C4, C5, G1, G2, G3-Metal, G3-PDES, G3-Dist) are formally verified and closed in [`conductor/current-state.json`](conductor/current-state.json).

Git is initialized. `libs/kairos` is a development pin (see `submodule_pins` in
[`conductor/current-state.json`](conductor/current-state.json)). That pin can
move. It is not a release pin and it is not clinical acceptance.
`extensions/conductor` is a pinned submodule. The parent GitHub remote exists:
`git@github.com:edithatogo/careops-sim.git`
([https://github.com/edithatogo/careops-sim](https://github.com/edithatogo/careops-sim)).
This README does not claim a release. Conductor still states that publication
follows its readiness gates, including the D2 notes in
[delivery scope](conductor/delivery-contract.md) and
[CI, security and GitHub timing](conductor/ci-security-release.md). Those
pages are the publication gate; they are not a statement that the remote is
still missing.

The workspace toolchain is Rust 1.99.0 (`rust-toolchain.toml`; `Cargo.toml`
`rust-version` is `1.99`). From this checkout, with Python 3 installed:

```sh
git submodule update --init --recursive
python3 tools/context.py resume
python3 tools/context.py check
python3 tools/tasks.py check
python3 tools/mvp.py check
python3 -m unittest discover -s tests -v
```

`python3 -m unittest discover -s tests -v` is the local harness command.
Failures are not acceptance, and this README records no harness result. The
commands above check local planning and harness integrity. They are not
simulation acceptance.

The E0 runner, on that same toolchain, is:

```sh
cargo run --locked -p careops-ed-cli -- run crates/careops-ed/examples/one_patient.json
```

That command runs a synthetic one-patient fixture. Scope and limits are in the
crate README and the
[E0 assumptions and boundary](conductor/design/ed/e0-assumptions-and-boundary.md).

Read [AGENTS.md](AGENTS.md); use bounded packets and the
[serial/parallel protocol](conductor/execution-model.md). Research has already
been supplied; verify sources and fill specific gaps rather than rerunning
broad research. See [handoff](conductor/research/research-handoff.md).

For gpt-6-luna execution through the MVP, use the
[complete leaf workpack](conductor/execution/mvp/README.md): 82 parent tasks,
258 bounded leaves, and the 145-task graph (`python3 tools/mvp.py check`,
`python3 tools/tasks.py check`).

## Licenses, notices, and citation

Workspace package metadata declares SPDX `Apache-2.0`:

- root [`Cargo.toml`](Cargo.toml) `[workspace.package].license`
- `crates/careops-ed` and `crates/careops-ed-cli` through `license.workspace = true`
- [`fuzz/Cargo.toml`](fuzz/Cargo.toml) `license = "Apache-2.0"`

[`deny.toml`](deny.toml) allows dependency licenses `MIT`, `Apache-2.0`, `BSD-3-Clause`, and `Unicode-3.0`. Parent Rust and Python sources have no SPDX file headers.

Root `CITATION.cff` is present for software citation. Distribution packaging and release rehearsals bundle `LICENSE-APACHE` and `NOTICE` via `tools/release_rehearsal.py`. The [P4 pack manifest](model-inputs/ed/profiles/p4-pack-manifest.json) records parent redistribution as `not_established` and keeps Kairos and Conductor licenses inside those submodules. [E0.2](conductor/evidence/e0.2-local-acceptance-20261002.md) records the Cargo Apache-2.0 fields as repeated Kairos workspace metadata. [D2.3](conductor/evidence/d2.3-policy-20261003.md) added the Cargo fields so `cargo deny` can check licenses.

Pinned submodules (initialize with `git submodule update --init --recursive`):

| Path | Gitlink | License | Citation |
| --- | --- | --- | --- |
| `libs/kairos` | `44261eb5cf8275f132f63a3396903d116d701dbe` | `libs/kairos/LICENSE` (Apache-2.0 OR MIT) | `libs/kairos/CITATION.cff` cites KairoECS |
| `extensions/conductor` | `7a5c560a4fdf5297be58594cc37527eb12790272` | `extensions/conductor/LICENSE` (Apache-2.0) | no citation file at this pin |

### Citation

If you use CareOps Sim in research, healthcare operational analysis, or software studies, please cite:

```bibtex
@software{careops_sim_2026,
  author = {Mordaunt, Dylan A. and {CareOps Sim Contributors}},
  title = {CareOps Sim: A Rust-Native Emergency Department Discrete-Event Simulation and Agent-Based Modeling System},
  year = {2026},
  publisher = {GitHub},
  journal = {GitHub repository},
  howpublished = {\url{https://github.com/edithatogo/careops-sim}},
  version = {0.1.0}
}
```

**APA Format:**
> Mordaunt, D. A., & CareOps Sim Contributors. (2026). *CareOps Sim: A Rust-Native Emergency Department Discrete-Event Simulation and Agent-Based Modeling System* (Version 0.1.0) [Computer software]. https://github.com/edithatogo/careops-sim

Machine-readable citation metadata is provided in [`CITATION.cff`](CITATION.cff). Research evidence citations are retained in [`conductor/generic-ed-evidence.md`](conductor/generic-ed-evidence.md) and [`conductor/research/`](conductor/research/research-handoff.md).
