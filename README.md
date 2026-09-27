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
The native bootstrap/toolchain commands will be documented when D1/E0 deliver them.

Next preparation candidates: D0.2 capability/owner/gate reconciliation and P0.1
supplied-evidence reconciliation. Read [AGENTS.md](AGENTS.md); use bounded packets
and the [serial/parallel protocol](conductor/execution-model.md). Research has
already been supplied; verify sources and fill specific gaps rather than rerunning
broad research. See [handoff](conductor/research/research-handoff.md).

For gpt-6-luna execution through the MVP, use the [complete leaf workpack](conductor/execution/mvp/README.md).
