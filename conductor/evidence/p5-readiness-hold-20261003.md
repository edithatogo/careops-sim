# P5 readiness hold — 2026-10-03

Disposition: **hold for prerequisite implementations**. P5.1–P5.4 remain open.
Readback base: `305135d` in `/private/tmp/careops-sim-p33`.

## Source and acceptance readback

The [parameter plan](../tracks/ed_parameter_evidence_20260927/plan.md) requires
accepted P4, E2 and C5 before P5. P4.4 is accepted; E2.4 and C5.5 are unchecked
and unaccepted in the task graph. Recursive graph readback finds 54 unaccepted
prerequisite tasks before P5.1, including development readiness, queue/runtime,
calibration and pathway work.

`Cargo.toml` contains the `careops-ed` and `careops-ed-cli` workspace members.
The actual `ScenarioConfig` in `crates/careops-ed/src/lib.rs` accepts supplied
synthetic work items under the E0 contract. The full E2 hybrid model configuration,
Macro/Micro execution and C5 calibration runner are absent from this integrated
source. Static P4 validators and an E0 smoke run cannot establish P5 consumer
coverage, actual-model loading or conditional/dependence preservation.

## Required implementation gates

1. Complete D2 and the Q1–Q4 runtime dependencies.
2. Complete C0–C2 interfaces/runtime and E1–E2; accept E2.4 against the actual
   hybrid model and composed synthetic scenario.
3. Complete C3–C5; accept C5.5 against the actual reproducible calibration runner.
4. Bind P5.1 to exact accepted model/runner commits, exported configuration keys,
   parameter IDs, units, supported profile versions and the frozen P4 inputs.

P5.1 must then reject a temporary unregistered field, missing IDs, unexplained
defaults, unused active parameters, unit drift and unsupported versions. P5.2
must load the example corpus through both consumers and demonstrate fixed-seed
repeatability, Macro/Micro consistency, dependence, diagnostics and sensitivity.
P5.3 and P5.4 require executed evidence and unresolved acquisition/validity limits.
No commands for these future APIs are invented here.

## Executed preparation checks

- `python3 tools/context.py resume`: clean integrated source; active D2 context.
- `python3 tools/tasks.py check`: 143 task records; coverage/prerequisites/DAG valid.
- `python3 tools/mvp.py check`: 80 parents, 248 leaves; joins and hashes valid.
- Direct source, plan and task-graph readback confirms the missing gates above.

The [execution contract](../execution-model.md) requires stopping at a missing
prerequisite or undefined API. This hold preserves that boundary; it is not P5
acceptance and does not request new user authorization. The active D2 state is
preserved for its coordinator.
