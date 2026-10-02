# P4 synthetic example-pack contract

Status: coordinator-frozen data-only contract, 2026-10-02. This contract exists
to make P4.1 profiles complete and machine-checkable while the current E0 runner
supports only supplied work items and static capacity. It is not a runtime API.

## Versions and execution boundary

- `profile_pack_schema_version: 1` identifies this P4 example-data envelope.
- E0 scenario JSON remains a distinct executable projection. A pack is not
  accepted by `careops-ed run`; P5 will bind it to the completed model and
  calibration runner.
- The existing P0.2 `capacity-location.schema.json` v3 is reused for the
  capacity/location binding. Routes in the pack remain distance metadata; no
  movement, route choice or visualization is implied.
- Every P4 fixture is synthetic. It cannot establish a public-data estimate,
  generic operational baseline, clinical recommendation, or Cairns value.

## Shared time and provenance

- All profile times are offsets from relative origin zero, represented as
  `{ "value": "<nonnegative decimal>", "source_unit": "s" }`. Decimal strings
  preserve source precision and avoid binary floating point. Calendar dates,
  local timezone rules and daylight-saving transitions are outside this schema.
- Each pack has an unsigned 64-bit run seed and a positive horizon. A seed is
  recorded for reproducibility; this contract does not define the E2 named RNG
  streams or promise that the data-only pack can be executed.
- Provenance identifies `synthetic` and carries a human-readable assumption
  note. No numeric value is presented without that classification.

## Required model-input shapes

- `arrival_table` is a time-ordered set of half-open intervals with explicit
  counts by named arrival mode. Counts are prescribed fixture inputs, not fitted
  rates or a generated point process.
- `acuity_scales` name the category set used by the fixture (for the Australian
  examples, synthetic ATS categories). `case_mix_table` partitions each arrival
  interval and mode into category counts. P4.2 checks category references and
  reconciles these counts with `arrival_table`; no observed prevalence or
  clinical default is implied.
- `resource_calendars` give time-varying open and staffed counts separately for
  each declared resource bucket; physical counts remain in the P0.2 binding.
- `staffing_calendars` give role-specific present counts over relative intervals.
- `capacity_location` uses the accepted P0.2 schema and retains explicit named
  zones, locations, resource buckets, task classes, staff roles and eligibility.
- `route_graph` contains named nodes that refer to capacity/location IDs and
  directed edges in metres. It is a graph input example, not imported geometry.
- `distributions` use explicit finite discrete outcomes with a unit and
  probability mass per outcome. P4 examples label these distributions synthetic;
  P4.2 performs probability-sum, support and cross-field semantic checks. No
  empirical family, fit, prior or independence assumption is selected here.
- `initial_state` is explicit at a relative time and lists any occupants and
  remaining work. An empty list means known empty initialization for that
  fixture, not an unknown state.

## Validation and later work

P4.1 adds a JSON Schema and local validator for structural validation, including
the referenced P0.2 capacity/location schema. P4.2 adds stable semantic
diagnostics for probability sums, ordering, capacity feasibility, schedule
coverage, initialization and provenance, and negative examples. P4.3 documents
the profiles and maps future Cairns acquisition separately. P5 alone validates
profile loading and consumption by the full model/calibration runner.
