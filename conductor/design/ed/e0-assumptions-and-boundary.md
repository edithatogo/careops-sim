# E0 assumptions, evidence and boundary

Status: E0 implementation inventory, 2026-10-02. This is a bounded handoff to
E1/E2/P1/P3, not acceptance of clinical assumptions, sources, or a generic ED
profile.

## Inputs and source register

E0 reuses the existing source and parameter catalogue. It does not copy external
source rows into a second inventory. Scope anchors are the accepted P0 research
coverage and schemas (`model-inputs/ed/schema/supplied-research-coverage.md`,
`parameter-ids.json`, `parameter-usage-matrix.json`,
`capacity-location.schema.json`), the Reports 26–30 reconciliation, and the
accepted P0.4 closeout. The report bundle is preserved in
`conductor/research/supplied/20260927/manifest.json`; the archived reports are
research inputs, not independent primary-source verification. No external
example/profile was imported into this E0 executable.

The P0 capacity/location shape is version 3 and marked `proposed`. It preserves
aggregate treatment-space capacity, resource bucket capacity, equipment
capabilities, staff counts/eligibility, named zones/locations, and optional
route distances in metres. E0 requires known positive open/staffed resource
counts for every supplied work item and does not infer missing values. These
synthetic test counts do not establish generic or Cairns operational capacity.

## Executable assumptions

- Inputs are explicit synthetic work items with stable IDs, arrival ticks,
  positive work ticks, one resource bucket and one task class. No arrivals,
  acuity mix, pathways, triage, diagnostics, disposition, admission or boarding
  are generated.
- One work item consumes one unit from its named, independent FIFO resource
  bucket for its full synthetic work duration. Capacity is the smaller of known
  open and staffed counts. This is a narrow scheduling fixture, not a staffing
  model or a claim about real-world allocation policy.
- Kairos's deterministic scheduler orders same-time events by its declared
  priority and insertion sequence. E0 inserts supplied arrival events in
  arrival-tick/patient-ID order; completion events enter when work starts.
  Therefore, at a shared tick, already-queued events run before later-inserted
  events, including completions scheduled during that tick. This observable
  ordering is an E0 implementation rule for the smoke slice, not a clinical
  priority rule.
- All times are integer `tick`; no conversion to seconds/minutes is implied.
  The horizon is inclusive for completions. Arrivals at or after the horizon
  are rejected; jobs whose completion exceeds the horizon are reported
  unfinished and are not counted complete.
- A required seed is recorded for reproducibility metadata, but E0 makes no
  random draws. Repeated runs with identical input bytes produce identical
  summaries. The CLI hashes the exact source file bytes using SHA-256.
- Route distance values preserve the P0 `m` unit but do not affect E0 execution.
  Equipment metadata is preserved and validated structurally by strict typed
  deserialization but is not allocated by this skeleton.
- The example's identifiers, 1-tick arrival offset, 2-tick work, one resource
  slot and one staff count are invented fixture values only. They are not
  defaults, sampled durations, distributions, or ED observations.

## Measurements in the smoke output

For a supplied work item, `wait_ticks = start_tick - arrival_tick` when it
starts; `elapsed_ticks = completion_tick - arrival_tick` only when complete.
`work_ticks` is the supplied synthetic value, not observed active clinical work.
`arrivals` counts all supplied work items whose arrivals fall before the
horizon; `started` counts those with a start tick; `completed` counts those
with a completion at or before the horizon; `unfinished = arrivals - completed`.
There is no warm-up exclusion, replication aggregation, utilization estimate,
LOS estimate, throughput rate, censoring adjustment or stochastic interval in
E0. Those definitions belong to later scoped work before those metrics are
reported.

## Explicitly unresolved / later gates

- No empirical arrival, acuity, pathway, diagnostic, treatment, boarding,
  staffing, equipment or layout distributions are accepted. No clinical defaults
  are supplied. Use the shared P catalogue and P1/P3 evidence gates; keep
  unsupported/missing inputs unknown until evidenced or explicitly declared as
  synthetic assumptions for a named experiment.
- The Report 29 boundary is not implemented in E0. Later work must model finite,
  destination-compatible ED-eligible bed offers with declared offer lifetime,
  persistence/withdrawal, unused-offer and competing-demand assumptions;
  accepting an offer must not release ED occupancy before physical departure.
- Ambulance arrival, triage, responsibility/handover, offload and crew release
  are distinct event clocks. E0 implements none of these and makes no fleet,
  receiver-staff, handover, offload or community-response claim.
- Assisted transfer, supervision, contested equipment, multi-person teams,
  cleaning, shared occupancy and simultaneous resource claims remain unsupported
  here. Q v1 permits one unit of one resource per work item; no hidden atomic
  resource-plus-space claim is implied.
- Public source revisions/licences and the exact primary evidence used for any
  later scenario must be traced through the shared catalogue and independently
  verified at the consuming task. The supplied reports and their citations do
  not themselves close that verification.
- E0's support is limited to the locked native Rust workspace and its declared
  Rust 1.76 MSRV on the tested host. This does not qualify other platforms,
  release support, clean external consumers, or hosted CI.
