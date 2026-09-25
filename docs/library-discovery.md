# Library discovery for CareOps Sim

Discovery date: 2026-09-25.

## Initial concept

The user wants an emergency department simulation using their Kairos library and
other relevant libraries from their GitHub account. They also authorized source
submodules where these enable library development.

## Repository preparation

- Conductor tooling: `extensions/conductor`, release `conductor-v0.4.1`, commit
  `7a5c560a4fdf5297be58594cc37527eb12790272`.
- Kairos source: `libs/kairos`, commit
  `fae901558f07b7b717a676adbafbe2cdc78dea1c` (the observed default-branch HEAD;
  no tags were returned by the remote tag query).
- The project-level `conductor/` path is reserved for this project's context.
- Neither a simulation implementation nor library compatibility has been tested.

Submodule checkout instructions:

```sh
git submodule update --init --recursive
```

The recorded commits fix the source versions. For development within Kairos,
create a branch inside `libs/kairos`, commit upstream changes there, and record the
new submodule commit in the parent repository. Merely editing the submodule does
not record those changes in the parent repository.

## Candidate reuse

| Repository | Observed capability | Proposed role and remaining work |
| --- | --- | --- |
| [kairos](https://github.com/edithatogo/kairos/blob/fae901558f07b7b717a676adbafbe2cdc78dea1c/README.md) | Rust scheduler, world state, random streams, DES resources and ABM APIs; active prerelease with preview bindings | Required simulation engine. Check the selected interfaces and reproducibility before implementation. |
| [microcosting_healthworkforce](https://github.com/edithatogo/microcosting_healthworkforce/blob/main/README.md) | Python labour-cost calculations, productive-time adjustments, rate conversion, projections and sensitivity analysis | Candidate staffing-cost adapter. Assess reusable functions and required spreadsheet inputs; parameter provenance and rates remain to be supplied. |
| [mchs](https://github.com/edithatogo/mchs/blob/master/README.md) | Python NWAU calculator including ED activity; support and parity depend on calculator and year | Candidate funding-output adapter. Choose the applicable year and inputs and check its evidence before use; funding and resource costs are distinct outputs. |
| [voiage](https://github.com/edithatogo/voiage/blob/main/README.md) | Value-of-information analysis with Python orchestration and selected Rust kernels | Candidate decision-analysis adapter once scenarios, uncertainty draws and a decision-value model are defined. |
| [careops-process](https://github.com/edithatogo/careops-process/blob/main/README.md) | OCEL 2.0 contracts and process discovery, delay and conformance analysis | Candidate event-export/analysis adapter. Its documented domain is administrative and governance workflows; an ED domain mapping would need to be designed. |
| [careops-costing](https://github.com/edithatogo/careops-costing/blob/main/README.md) and [hwcc](https://github.com/edithatogo/hwcc) | Workbook migration context and Power Platform artefacts | Possible requirements references; no reusable runtime library established in this review. |

Except for the pinned Kairos code described below, this is repository-documentation
discovery, not an API audit or a successful integration claim. Additional libraries
have not been added as dependencies.

## Kairos implementation observations

At the pinned revision, [`kairo-ecs-des/src/lib.rs`](../libs/kairos/crates/kairo-ecs-des/src/lib.rs)
uses a FIFO `VecDeque` for resource waiting queues. It provides scheduler event
priorities, but this does not implement acuity-based resource allocation for an ED.
The ED model will need an explicit allocation policy.

`DESContext::new(_seed)` does not use its seed argument. Reproducible stochastic
arrivals and service durations therefore require explicit wiring to the random
stream APIs and verification of the resulting model. A constructor seed alone is
not evidence of reproducible stochastic behaviour.

## Existing simulation repositories

The complete default-branch file trees returned by GitHub contained only
`README.md` and `LICENSE` for each of:

- [sim_hospital_emergencydepartment](https://github.com/edithatogo/sim_hospital_emergencydepartment)
- [sim_hospital_wholeofhospital](https://github.com/edithatogo/sim_hospital_wholeofhospital)
- [sim_hospital_wholeofdistrict](https://github.com/edithatogo/sim_hospital_wholeofdistrict)

No reusable implementation was found in those default branches. Other branches
were not examined.

## Proposed vision — awaiting user confirmation

**CareOps Sim — Emergency Department:** Build a reproducible Kairos-based
simulation of patient arrivals, triage, assessment, treatment, and discharge or
admission. Compare staffing, treatment-space capacity, demand, and patient-flow
scenarios using waiting times, length of stay, throughput, and resource use.
Costing, funding, process analysis, and uncertainty libraries are candidate
optional integrations.

The intended users, setting, model detail, input data, interface, initial scenario,
and success criteria remain to be defined through Conductor setup. The technology
stack and first implementation track have not been approved.
