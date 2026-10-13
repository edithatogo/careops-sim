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

Local planning and context tooling are present; the ED runtime is not implemented.
Git is initialized. `libs/kairos` and `extensions/conductor` are pinned submodules.
No parent GitHub remote exists yet; creation is deliberately gated at D2.
Do not run an unimplemented simulation command copied from a proposed plan.

From this checkout, with Python 3 installed:

```sh
git submodule update --init --recursive
python3 tools/context.py resume
python3 tools/context.py check
python3 tools/tasks.py check
python3 tools/mvp.py check
python3 -m unittest discover -s tests -v
```

These checks verify local planning/harness integrity, not simulation acceptance.
The first native E0 runner is available in `crates/careops-ed`: `cargo run --locked -p careops-ed-cli -- run crates/careops-ed/examples/one_patient.json`. It runs a synthetic one-patient fixture only; see the crate README and E0 assumptions contract for scope and limits.

Next preparation candidates: D0.2 capability/owner/gate reconciliation and P0.1
supplied-evidence reconciliation. Read [AGENTS.md](AGENTS.md); use bounded packets
and the [serial/parallel protocol](conductor/execution-model.md). Research has
already been supplied; verify sources and fill specific gaps rather than rerunning
broad research. See [handoff](conductor/research/research-handoff.md).

For gpt-6-luna execution through the MVP, use the [complete leaf workpack](conductor/execution/mvp/README.md).

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
