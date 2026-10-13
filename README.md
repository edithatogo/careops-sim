# CareOps Sim

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

Local planning and context tooling are present. The E0 runner in
`crates/careops-ed` exists. E1.2 pathway work is accepted: seeded time-varying
arrivals, clinical states, pathway runtime, disposition, and D3.3 mutation
qualification. There is no E2 headless MVP and no release-qualified ED
simulation. Functional MVP acceptance is phase E2, and that phase is not done.

Active work is recorded in
[`conductor/current-state.json`](conductor/current-state.json). Re-read that
file before dispatch; it can move. The copy in this tree (`updated_at`
2026-10-13) names track `generic_ed_delivery_20260925`, phase E1. Next is
E1.3: admission boarding/discharge and optional abandonment/transfer policies.

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

Root `LICENSE`, `NOTICE`, `CITATION.cff`, and `CITATION.md` are absent. The [P4 pack manifest](model-inputs/ed/profiles/p4-pack-manifest.json) records parent redistribution as `not_established` and keeps Kairos and Conductor licenses inside those submodules. [E0.2](conductor/evidence/e0.2-local-acceptance-20261002.md) records the Cargo Apache-2.0 fields as repeated Kairos workspace metadata. License ownership and a public redistribution grant remain a maintainer decision. [D2.3](conductor/evidence/d2.3-policy-20261003.md) added the Cargo fields so `cargo deny` can check licenses.

Pinned submodules (initialize with `git submodule update --init --recursive`):

| Path | Gitlink | License | Citation |
| --- | --- | --- | --- |
| `libs/kairos` | `7af989c8951c232f7bc4e6a2f543c4ee0061af99` | `libs/kairos/LICENSE` (Apache-2.0 OR MIT) | `libs/kairos/CITATION.cff` cites KairoECS |
| `extensions/conductor` | `7a5c560a4fdf5297be58594cc37527eb12790272` | `extensions/conductor/LICENSE` (Apache-2.0) | no citation file at this pin |

Maintainer checklist before publishing a grant or a software citation:

1. Name the copyright holder.
2. If Apache-2.0 stays the parent license, add the standard Apache-2.0 text as root `LICENSE`, then revisit P4 `repository_licensing.status`.
3. Add `NOTICE` for third-party notices the holder chooses to carry. Dependency SPDX identifiers stay in `deny.toml` and `Cargo.lock`.
4. Add `CITATION.cff` after authors are named. This README and [`conductor/product.md`](conductor/product.md) already use the title CareOps Sim. Neither file names authors.
5. Keep research citations in [`conductor/generic-ed-evidence.md`](conductor/generic-ed-evidence.md), [`conductor/research/`](conductor/research/research-handoff.md), and `model-inputs/ed/**/evidence/`. The shortlist names AIHW, CDC NHAMCS, the HSMA SimPy examples, and the treatment-centre model docs. AIHW and the pinned HSMA example have bound evidence under `model-inputs/ed/`. CDC NHAMCS and the treatment-centre documentation page remain shortlist candidates in `conductor/generic-ed-evidence.md`; that note already requires a recorded licence and revision before reuse.
