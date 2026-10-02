# P2.3 ABM catalogue and hand-worked checks contract

Coordinator contract, 2026-10-02. This is a proposed evidence catalogue and
representation check, not a calibrated or executable ED profile. It uses the
accepted schema-v1 parameter record and accepted usage-matrix-v2 decisions.

## Exact scope and ownership

Three disjoint catalogue outputs cover all 22 registered rows in the following
families exactly once:

- `staff_behavior.json`: all 9 `abm_staff_behavior` IDs.
- `spatial.json`: all 9 `spatial_abm` IDs, including deferred congestion.
- `optional_complexity.json`: all 4 `optional_complexity` IDs, explicitly
  deferred by the usage matrix.

The coordinator owns the registry, schema, usage matrix, this contract, merged
ABM README, cross-family validator, synthetic fixture/check, shared profile and
P3 join decisions. Workers may write only their one catalogue file. Every row's
semantic name and owner come from the ID registry; consumer, unit contract,
decision, profile stage, value role and gates come from the accepted usage
matrix. Catalogue `family` uses the exact registry family key.

## Record and annotation requirements

Each family file has `schema_version: 1`, `status: proposed`, exact `family`, a
`records` array, and an `annotations` object keyed by parameter ID. Every record
must validate against `parameter-record.schema.json`; each ID appears exactly
once in its family and no unregistered ID appears. Annotation keys are exactly
the family's IDs. Required annotation members are `unit_contract`,
`value_role`, `evidence_refs`, `acquisition_plan`, `candidate_families`,
`candidate_status`, and `downstream_gates`.

Evidence refs must point only to accepted P2.1/P2.2 records that explicitly map
to that exact ID. Store repo-relative paths and current SHA-256. Do not imply
that nearby evidence establishes an uncovered parameter. For uncovered rows,
use an empty evidence list and a concrete acquisition plan (owner, required
fields/denominator, access route, impact and review gate). Preserve the source's
bounded population, period, setting and method in the linked source record; do
not pool or transfer its numeric summaries into these model inputs.

All five range slots (`physical_limits`, `observed_sample_range`,
`generic_scenario_range`, `uncertainty_interval`, `calibration_bounds`) stay
`unknown` with an explicit reason for active candidate inputs. No numeric
`value`, range endpoint, `reference_default`, distribution parameters or fitted
coefficients are permitted. Use `deferred` with no value for matrix-deferred
rows. Non-numeric candidate families are research candidates only; their status
must begin with canonical `unselected`. Do not claim statistical independence
when dependence is unidentified.

## Semantic boundaries

- Record observed source result, author interpretation, and CareOps heuristic
  as distinct evidence classes. A heuristic may be proposed only as a clearly
  labelled synthetic test rule; it is not an empirical default.
- DES owns patient arrival, pathway, queue/wait and service/diagnostic elapsed
  time inputs. ABM movement owns explicit edge travel; typed interruption,
  pause, and resumption are separate event components. P3 must reconcile these
  so Macro elapsed intervals are not added to Micro subintervals.
- Keep graph geometry, coordinate frame/scale, metres-per-edge, access and
  reachability, movement mode, speed, route choice, supply-trip purpose and
  interruption cause distinct. One evidence record cannot stand in for another.
- Synthetic fixtures use labeled invented values solely to test units, graph
  arithmetic and accounting invariants. They may not be copied into catalogue
  values/ranges or described as typical ED behavior.
- Optional complexity records remain deferred; no fatigue, learning, crowding
  or detailed physiology rule may leak into the MVP.

## Required hand-worked verification

Coordinator-owned fixture/check must show: (1) an explicitly synthetic,
versioned graph edge in metres and speed in metres/second gives a travel time in
seconds by distance/speed; (2) reverse reachability is not inferred from a
one-way edge; (3) inaccessible/unreachable edges are rejected; (4) active work,
travel, queue/wait and interruption/pause are separate labeled intervals and
the sum is accounted once; (5) Macro elapsed time cannot be summed with its
Micro decomposition; and (6) a no-graph Macro case remains valid. Hand-worked
arithmetic is explicitly illustrative, not an ED parameter estimate.

## P3 handoff and acceptance limits

P3 owns candidate fitting, dependence/identifiability decisions, conditional
sampling order, shared factors, uncertainty, and DES/ABM boundary resolution.
This task records candidate families only and flags which joint observation is
needed to identify them. Catalogue acceptance means schema, provenance,
coverage, unknown/deferred, and representation invariants pass. It does not
establish transferable ED values, behavior, spatial validity, empirical fit,
runtime consumption, patient safety or Cairns applicability.
