# E0 synthetic skeleton contract

Status: coordinator-frozen local implementation contract, 2026-10-02. This is
an E0 smoke model, not an ED pathway policy, operational profile, or clinical
parameterization. Its only runnable input is explicitly synthetic.

## Workspace and public surface

- `crates/careops-ed` owns generic scenario types, validation, deterministic
  serial execution, result types, and the public Rust API.
- `crates/careops-ed-cli` owns only command-line parsing, input loading, and
  output rendering. It does not own model policy or event scheduling.
- The parent workspace depends on the pinned Kairos `kairo-ecs-core` and
  `kairo-ecs-types` crates by path. No Kairos source is modified by this task.
- `careops-ed run <scenario.json>` validates, executes, and writes a JSON result
  to stdout. Diagnostics go to stderr and invalid input exits nonzero.

## Scenario schema v1

The strict JSON object contains `schema_version: 1`, `scenario_id`, an unsigned
`seed`, `time_unit: "tick"`, positive `horizon_ticks`, synthetic provenance with
a nonempty note, a P0-shaped capacity/location profile, and a nonempty list of
one or more supplied patient work items. Unknown fields, invalid units, invalid
counts or durations, duplicate IDs, and unresolved location/zone/resource/task
references are errors. One `work_ticks` value is a synthetic event duration; it
does not stand for a clinical service time. The seed is recorded but no random
draws are used in E0.

The capacity/location profile follows `capacity-location.schema.json` version
3: treatment-space summary, zones, locations, resource buckets, task classes,
staff roles/counts, eligibility, and optional routes. The runtime only consumes
known `open_count` and `staffed_count` for the referenced resource bucket. It
fails closed when either is unknown. Route distances preserve P0's explicit
`m` unit but are not used to calculate movement in E0.

Each patient work item has a stable `patient_id`, arrival tick, positive
`work_ticks`, resource bucket ID, and task-class ID. Arrival and completion are
scheduled through Kairos using integer `SimTime` ticks. Each resource bucket is
a single independent FIFO pool with capacity equal to the lower of its known
open and staffed counts. Ties are deterministic by arrival tick and patient ID.
No triage, branching, clinical category, interruption, transit, discharge,
boarding, or stochastic arrival behavior is implied.

The run summary reports input SHA-256, seed, horizon, tick unit, event counts,
each patient's arrival/start/completion ticks, wait/work/elapsed ticks, and
unfinished cases. The input hash is over the exact source file bytes. Horizon
events at the horizon tick are processed; arrivals must be strictly earlier
than the horizon. Work extending beyond the horizon remains unfinished.

## Synthetic example boundary

`crates/careops-ed/examples/one_patient.json` uses invented IDs, one tick of
arrival offset, two ticks of work, one synthetic resource slot, and synthetic
capacity/staff counts. It demonstrates parsing, checked references, event order,
and summary generation only. These values are not observations, defaults,
distributions, or a representation of Cairns or any other ED.

## Known non-goals and gates

This E0 contract does not claim the E1 pathway, E2 operational constraints, a
calibrated generic ED profile, Arrow/Parquet, multi-run experiments, checkpoints,
publication readiness, or supported-platform qualification. Those remain under
their existing owners and acceptance gates. Parent GitHub/hosted-CI readiness is
still governed by D2 after E0 review.
