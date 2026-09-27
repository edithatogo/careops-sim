# Specification: empirical calibration and validation

**ID:** `empirical_calibration_20260925` · **Status:** proposed
**Upstream owner:** 21 (VVUQ), integrated with 03 (Flow), 04 (Arrow), 22 (runner).
**Baseline:** Kairos `fae901558f07b7b717a676adbafbe2cdc78dea1c`.

## 1. Objective and boundaries

Provide reusable Rust tooling that turns timestamp observations into trace replay,
explicit macro/micro execution, residual diagnostics and reproducible parameter
calibration. Use the existing Arrow schema pipeline and experiment manifests.
Demonstrate whether added agent detail improves held-out predictions over a macro
baseline, rather than assuming a hybrid model is inherently more accurate.

Inputs: local Arrow IPC/Parquet tables, explicit column/time mappings, scenario
and seed manifests, model adapter, fidelity policy, route graph when required,
parameter bounds, objective configuration and train/validation/test partitions.
Outputs: validated normalized traces, replay diagnostics, predicted/unclamped
residuals, distribution metrics, parameter-search history and reproducible run
artifacts, with provenance and data-quality exclusions.

In scope: simple interpretable staff heuristics, deterministic graph transit,
interruptible tasks, empirical duration sampling, shadow anchors, W1/KS metrics
and a bounded candidate/grid-search calibration runner. Initial delivery uses
generic ED synthetic fixtures informed by public data/open examples where suitable; real EHR access, clinical policy validation, CAD conversion,
full pedestrian dynamics, fatigue models, learned policies and a Pixi.js dashboard
are separate work. The renderer must never own simulation time.

## 2. Existing plan alignment and module boundaries

Propose `kairo-ecs-calibration`, a Rust library owned by existing Track 21. It owns
normalized trace semantics, model-adapter traits, replay/fitting logic and pure
metrics. Real Arrow/Parquet readers/writers belong in `kairo-ecs-arrow` (Track 04).
Flow/fidelity execution hooks belong in DES/ABM (03). CLI and experiment/resume
orchestration belong in `kairo-ecs-cli` (22). Core (01) owns seed/order contracts;
conformance (12) owns portable fixtures. The new crate needs a C0 ADR; it is not
a new Git submodule or a replacement experiment framework.

Current Arrow support is a schema plus custom smoke bytes; real Arrow IPC and
Parquet interoperability is a prerequisite, not an assumed capability. Add
optional IO features without introducing Arrow or statistics into the core
scheduler. Preserve `event_log.v1` and legacy scenario/seed parsing. Integrate
new configuration by a versioned extension/reference from the existing manifest;
unknown versions fail explicitly. Replace or extend the current simplistic
manifest reader only with backward-compatibility fixtures.

## 3. Ingestion and trace schema

`TraceMapping` explicitly maps source columns to event semantics. Wide timestamp
tables may be unpivoted under a declared mapping; long tables are direct input.
Never infer that bed-assignment time equals physical bed occupancy or that a
charting timestamp is the time an activity actually occurred.

### Normalized `trace_event.v1`

| Field | Type / rule |
| --- | --- |
| schema_version, dataset_id, mapping_version | Versioned strings; immutable within one ingestion artifact |
| case_key | Pseudonymous Utf8 identifier; never used directly as entity seed |
| source_event_key | Utf8 unique per dataset; reject duplicate keys, or apply an explicitly recorded dedup policy |
| event_kind | Versioned controlled vocabulary: arrival, triage_start/end, bed_assigned, bed_entered, service_start/end, discharge, etc.; mapping may register extensions |
| occurrence | UInt32 occurrence/task index to disambiguate repeated events |
| observed_at | Event occurrence timestamp, not entry/message/update time; Arrow Timestamp with explicit units/timezone, canonical UTC representation; original precision retained in provenance |
| recorded_at, message_created_at | Nullable source-recorded and message-creation timestamps, separately mapped; never substitute for occurrence |
| time_lineage | Versioned observed/derived/defaulted/unknown classification and mapping provenance for each timestamp |
| relative_ticks | FixedSizeBinary(16), little-endian u128, same tick representation as event_log.v1 |
| source_order | UInt64 stable source row/event ordinal used only as a final tie-break |
| resource_key, actor_key, location_key | Nullable Utf8; absence is meaningful and reported |
| quality_flags | Explicit bitset/dictionary with version; missing/imputed/ambiguous provenance, not free-form silent repairs |

Dataset manifest: input file hashes, selected columns, source timestamp meaning,
original timezone/units, origin timestamp, tick resolution and rounding rule,
ordering/mapping version, normalization hash, observation window, exclusions and
row counts. Source identifiers remain local; synthetic data populate test assets.

### Validation and normalization

- Read real RecordBatches from IPC or Parquet with bounded batch sizes. Reject
  wrong physical types, corrupt files, unsupported schema and invalid mappings.
- UTC conversion precedes subtraction from one dataset origin. Naive timestamps
  require an explicit timezone and DST ambiguity policy. Never guess or truncate
  silently. Checked signed subtraction/conversion rejects pre-origin, overflow or
  unrepresentable values; sub-tick rounding is declared and counted.
- Stable canonical order: `(relative_ticks, case_key, occurrence, event_kind rank,
  source_event_key, source_order)`. Domain precedence ranks are declared by the
  mapping, validated for cycles and included in its hash. External sorting/spill
  handles inputs exceeding memory; file/row-group/chunk order must not change
  normalized output when event keys are stable.
- Validate per-case partial order, occurrence pairing, location/resource keys and
  observation boundaries. Equal timestamps can be valid; do not invent causal
  precision the records do not contain. Required missing anchors fail strict
  mode. Optional exclusions are counted with reasons and sensitivity summaries.
- Distinguish left/right censoring, missing timestamps and out-of-order recording.
  A discharge outside the observation window is not a zero duration. No silent
  complete-case-only success claim; report coverage and outcome counts.
- If observed resource allocations overlap beyond declared capacity, reject
  resource-clamped replay with an actionable diagnostic. Do not create phantom
  beds or silently increase capacity. A timing-only exploratory mode may report
  the inconsistency without claiming resource-feasible replay.

## 4. Dual-fidelity execution

Add a versioned `FidelityPolicy` to scenario configuration. `FidelityMode` is
`Macro` or `Micro`. Precedence: `(entity, subsystem)` override, entity override,
subsystem default, global default. Resolve and freeze the policy at task admission
and record the decision. Switching applies at the next quiescent task boundary;
reject switching an active/suspended task, changing resources or discarding work
mid-attempt. Checkpoints store resolved modes and pending future policy changes.

| Concern | Macro | Micro |
| --- | --- | --- |
| Service | Empirical duration distribution/sample conditional on declared strata | Same clinical work-duration model plus explicit transit and interruptible task logic |
| Spatial transit | No spatial event or transit cost | Deterministic route plus walking-speed delay; progress/context survive interruption |
| Resource queues | Same queue/preemption rules where configured | Same queue/preemption rules; explicit agent identity/eligibility may select resource |
| Staff behavior | Declared aggregate dispatch policy | Initially urgency, stable FIFO and assigned-zone/skill eligibility; deterministic tie-breaks |
| Observability | Task start/end, wait, busy time, outcomes | Same metrics plus transit, route, interruption and policy-decision records |

Clinical/ED component types (`AcuityLevel`, `AssignedZone`, skills, bed cleaning or
reservation states) belong in the model adapter/CareOps. General resource gates
are reusable: a bed is not released to its patient queue until its cleaning work
finishes; reservations must name an owning claim/state. Queue conservation must
hold across these states. Individual staff entities can own capacity-one
resources, while a model dispatcher selects eligible staff. General atomic
staff+bed acquisition remains outside Track 1; examples avoid hold-and-wait
cycles through explicit staged allocation.

**Avoid double counting:** an arrival-to-treatment interval often includes waiting
and transit. Do not fit it as intrinsic service time then add both again. Mapping
must label each estimated distribution's start/end events and included processes.
When timestamps cannot identify service or walking separately, return an
identifiability warning, constrain parameters with independent observations, or
keep the subsystem Macro. EHR timestamps alone may be insufficient even for a
DES service model.

### Spatial contract

`TransitModel` receives actor/task identity, origin/destination, route graph,
parameter set and deterministic random stream; returns path and tick duration or
a structured unreachable/invalid error. First implementation: immutable graph
with stable node/edge IDs and integer distances, deterministic shortest-path
tie-breaking and positive speed in declared units. Use checked integer/rational
conversion with ceiling to ticks. No per-render-frame walking integration.

Record edge progress and route/graph version in suspended contexts. Routing
changes take effect at a declared movement boundary. Initial routing does not
claim pedestrian congestion; occupancy-dependent edge costs are a later,
versioned policy requiring observations and new fixtures. Layout assets must
include scale, connectivity, walkable edges and destinations; raw CAD is not an
executable routing model.

### Randomness

Version logical keys for case/task/purpose/replication seed derivation under Track
01. Assign synthetic entity IDs from stable normalized identities. Separate
service, transit and behavior streams so optional Micro work cannot shift service
draws. Persist draw positions and keys on resume. Common random numbers align
comparable candidate runs; do not reseed by thread, input row order or wall time.

## 5. Trace-driven shadow replay

`ReplayMode` distinguishes `FreeRunning` (only declared exogenous inputs such as
arrivals are fixed) from `ShadowAnchored` (specified observed macro transitions
are fixed). A model adapter declares permitted anchors and predicted endpoints.
Never force all events and use the resulting perfect timestamp match as evidence
of calibration quality.

Shadow mode uses an **observed macro ledger** plus **isolated predictive probes**.
The ledger follows validated historical occupancy/transitions. Each probe runs
micro behavior from a documented anchor/context snapshot to predict a target
transition. Probe IDs/seeds are stable and their allocations cannot mutate the
observed ledger. Within a probe, queue/preemption state follows Track 1. If staff
occupancy is unobserved, the assumption used to initialize it is recorded and the
result is labelled conditional on that assumption.

Execution order at an observed tick:

1. Dispatch anchors in normalized trace order through the standard scheduler;
   validate the expected macro state and apply the observed ledger transition.
2. Capture the declared post-anchor/pre-probe context; start or continue the
   appropriate isolated micro probes with deterministic identities.
3. Run probe events through the same Flow transaction order and bounded virtual
   scheduler. Store the first predicted target time **before** any clamp.
4. At the observed target, compute paired diagnostics if the probe is complete.
   If it is still running, keep the macro anchor fixed and continue the isolated
   probe to its bounded target/horizon. Once complete, report positive lateness.
   Probe future state cannot leak into an earlier macro snapshot.
5. Record observed target, predicted target, signed residual, feasibility and
   censoring. Publish transitions/residuals through Arrow sidecar streams.

For paired target j, `residual_ticks = predicted_j - observed_j` with checked signed
arithmetic (encode sign plus u128 magnitude to avoid i128 overflow). Early arrival
produces negative residual and observed slack; late arrival produces positive
residual and an anchor-feasibility violation. No negative service time, simulated
time reversal or fabricated completion. A probe that never reaches its endpoint
within its horizon is censored/failed, not silently omitted or assigned zero.

A configurable strict policy can reject candidates exceeding a preset infeasible
anchor fraction; diagnostic mode reports every mismatch. Both preserve fixed
observed anchors. Fitting uses unclamped predictions, not ledger timestamps.
Shadow replay isolates hypotheses; final performance assessment uses **free
running held-out simulation** to test coupled bottlenecks and feedback.

## 6. Telemetry and statistical metrics

Maintain existing `event_log.v1`. Add Track 04 schemas with matching ID/time
encodings and explicit joins:

- `calibration_residual.v1`: schema_version, dataset/scenario/run/candidate IDs,
  case/task/occurrence, probe_id, endpoint, fidelity, observed/predicted ticks?,
  residual_sign/magnitude?, anchor role, feasibility/censoring reason, seed-map
  version, mapping/parameter/graph hash, causal event reference where available.
- `calibration_metric.v1`: schema_version, run/candidate IDs, metric name/version,
  endpoint/stratum/window, units, reference/simulation counts, excluded/censored/
  unmatched counts, value?, status, uncertainty method/interval?, validity flags,
  input/provenance hashes. Group definitions include mode and replay role.

Use duration or relative-event distributions with explicit comparability, not
unrelated calendar timestamps. Group by task/acuity/shift only when prespecified;
report sparse strata as insufficient data. Paired residual summaries and unpaired
distribution distances answer different questions and must be labelled separately.

### Deterministic algorithms

- One-dimensional Wasserstein-1: sort samples, merge empirical CDF breakpoints and
  integrate absolute CDF difference. Normalize sample masses independently when
  sizes differ. Default equal weights; optional finite nonnegative weights need
  positive total mass and their provenance recorded. Units match the input.
- Two-sample KS statistic D: maximum absolute empirical CDF difference, advancing
  all ties together. Report D, sample counts and tie counts. Default **no p-value**:
  repeated patient/task data and coarse timestamp ties may violate standard test
  assumptions. Optional inference must name its assumptions and use a documented
  method such as seeded cluster resampling with appropriate cluster units.
- Normalize exact integer durations near a documented origin before f64 analysis;
  detect conversion precision loss and reject/declare a permitted scale change.
  No f64 event ordering. Fixed sorting/reduction order and algorithm versions
  produce repeatable CPU results; use declared numerical tolerance across
  architecture/compiler/backend boundaries.
- NaN/infinity, invalid weights or empty distributions produce typed invalid/
  insufficient-data results, never a misleading distance of zero.
- Compare censoring and missing/outcome rates separately. First release does not
  claim a survival-analysis correction; distances on uncensored subsets carry
  coverage warnings. Censoring or dropped cases cannot improve an objective
  unnoticed.

Analytic fixtures: A=[0,2], B=[1,3] has W1=1 and KS D=0.5; identical arrays yield
zero; A=[0,0], B=[1,1] yields both 1. Include unequal sizes, ties, weights, unit
scaling and independently generated reference cases.

## 7. Calibration and validation procedure

`CalibrationStudy` provides bounded parameters with units, constraints, a fixed
candidate enumeration, objective terms, split manifest, seed/replication budget,
probe limits and an explicit stopping policy. Start with deterministic grid or
supplied-candidate search. Optimization algorithms beyond that are extensions.

1. Validate data and partition by patient/episode and preferably held-out time
   blocks. Training, model-selection validation and final test sets are disjoint;
   preprocessing/distribution fitting use training data only.
2. Fit or configure macro service distributions and a minimal agent policy. State
   externally fixed parameters and what the data can actually identify.
3. Evaluate parameter candidates using shadow-probe residuals and configured
   distribution metrics. Common random numbers reduce comparison noise. A
   unit-normalized objective records weights/scales; add explicit penalties or
   rejection thresholds for missing targets, anchor infeasibility and constraint
   violations. No silent optimizer-selected exclusions.
4. Rank candidates only after a fixed evaluation batch completes; tie-break by
   stable candidate ID. Store every attempted/failed candidate, config and seed.
   A resume restores completed candidate/replication identities without duplicate
   weighting. Report sensitivity/non-identifiability if several settings fit.
5. Choose parameters on validation data; freeze them before final test evaluation.
   Compare Macro and Micro in free-running mode on the same held-out cases and
   seed schedule. Report accuracy, uncertainty and execution cost, including
   waits, throughput, resource use, transit and interruption statistics.

Synthetic recovery fixture uses a route of known distance, independent observed
transit anchors, known walking speed and injected noise (plus a noiseless case).
The identifiable case recovers the specified candidate within declared tolerance;
a deliberately confounded service/transit fixture must report ambiguity. A lower
training error alone does not establish empirical validity or justify Micro.

## 8. Proposed public APIs and runner integration

Illustrative Rust surface, pending C0 review:

```rust
let trace = TraceReader::parquet(path, mapping)?.normalize(time_policy)?;
let runner = ReplayRunner::new(model_adapter, scenario, seed_manifest)?;
let shadow = runner.run(&trace, ReplayMode::ShadowAnchored(anchors), fidelity)?;
let metrics = ResidualAnalyzer::new(metric_spec).compare(&shadow, &trace)?;
let study = CalibrationStudy::new(parameters, split, objective, budget)?;
let result = study.fit(&runner, &trace)?;
```

Expose model-adapter hooks for event mapping, context checkpoints, macro inputs,
probe target detection and validation. Built-in model fixtures require no Python.
Proposed CLI operations extend the existing `kairoecs` runner: `verify trace`,
`replay` with a replay-mode/config extension, `calibrate`, and `compare-runs`.
Reconcile exact flags with Track 22's current command surface in C0; names here
are proposed, not current working commands.

Artifact bundle: existing scenario/seed manifests plus versioned calibration
configuration, normalized-trace reference, parameter table, candidate history,
residual/metric tables, exclusion report, replay/checkpoint evidence and final
comparison report. Include engine Git SHA, Cargo.lock hash, toolchain, feature
flags, host/backend, input hashes, mode, schemas and objective versions. Never
embed raw patient records in a public report by default.

## 9. Execution, dependencies and acceptance

CPU comes first. Independent candidates/replications may run on local workers;
each owns its world and Arrow output shards. Merge by candidate/replication and
stable record key. Test 1/2/N workers and resume equivalence. Floating reductions
use canonical ordering; do not use completion-order-sensitive adaptive search.
GPU metric kernels, Metal measurements, browser rendering and PDES transport are
separate existing backend milestones. Pure metrics and task contracts must allow
later integration without changing their meaning.

Calibration C0/C1/C4 can start independently of queue implementation. C2 needs the
reviewed Flow API; C3 with interruption needs Q4, and final C6 needs Q5. Existing
upstream contracts/owners remain dependencies even when their registry says Done.

Acceptance criteria:

- C-01 Real Arrow IPC and Parquet round trips plus independent-reader fixtures;
  equivalent tables normalize identically across batch/row-group sizes.
- C-02 Macro produces no transit events; Micro routes deterministically and
  interrupts/resumes correctly. A zero-transit/no-extra-policy Micro fixture
  matches Macro task outcomes/draws. Mixed modes obey precedence and boundaries.
- C-03 Shadow macro anchors equal observed ticks, while deliberately biased
  predictions have nonzero residuals. Late probes, impossible occupancy and
  missing endpoints have explicit outcomes. No double clamp or time reversal.
- C-04 W1/KS analytic/reference fixtures pass; invalid/empty/tied/censored inputs
  are correctly classified. Precision and inference assumptions are reported.
- C-05 Synthetic identifiable recovery passes, confounded recovery reports
  ambiguity, and disjoint held-out free-running Macro/Micro comparisons exist.
- C-06 Candidate rankings, canonical integer traces and IDs survive repeat runs,
  checkpoints and 1/2/N workers. Numeric comparisons obey declared tolerances.
- C-07 Legacy event schema/manifests and feature-minimal core builds remain valid;
  no mandatory Python runtime, GPU hardware, new submodule or renderer dependency.

See [plan](plan.md), [test matrix](test-matrix.md), [risks](risk-register.md).
Release implications: new optional Rust crate/features and versioned sidecars;
API/compatibility review, documentation and dependency/MSRV gates are required.

## Sources and related decisions

- [Arrow columnar format](https://arrow.apache.org/docs/format/Columnar.html) for
  timestamp/columnar interoperability; [Rust Parquet Arrow reader](https://docs.rs/parquet/latest/parquet/arrow/arrow_reader/index.html)
  for the native RecordBatch ingestion route.
- [Wasserstein distance reference](https://docs.scipy.org/doc/scipy/reference/generated/scipy.stats.wasserstein_distance.html)
  and [two-sample KS reference](https://docs.scipy.org/doc/scipy/reference/generated/scipy.stats.ks_2samp.html)
  inform independent metric fixtures and interpretation. SciPy is not a runtime
  dependency of this Rust design.
- [Hybrid-model and browser appraisal](../../hybrid-model-insights.md) records
  accepted insights and qualified claims from the user's supplied note.

## Domain and site profile boundary

The toolkit is a general DES/ABM framework capability. Generic ED model code,
public-source/synthetic parameter profiles and later Cairns ED mappings are
separate artifacts. Reuse the same validated toolkit for subsequent domains in
[the roadmap](../../roadmap.md); do not hard-code Cairns/CHHHS assumptions into
queue, fidelity, telemetry or calibration components. Add other simulation
methods only through a concrete, reviewed extension to shared time/state/output
contracts. Public aggregate statistics can inform a generic baseline but cannot
be treated as patient-level traces or direct evidence of staff walking behavior.

## Research integration requirements

C0/C2 must distinguish measured task events from fitted model parameters, RTLS observations from legal paths, and clamped final outcomes from predictive knowledge. Staff/shift clustering and eligible-choice-set availability constrain identifiability; elapsed EHR intervals alone do not identify walking or switching costs.

See [reports 5–8 incorporation](../../research/ed-research-incorporation-20260927.md).

## Reports 9–12 integration

The [integration decisions and candidate oracles](../../research/ed-research-incorporation-9-12-20260927.md)
apply to the tasks in this track. Proposed policies/versions remain review inputs;
no reported research check substitutes for locally executed acceptance.

## Reports 26–30 integration

The [research decisions and acceptance cases](../../research/ed-research-incorporation-26-30-20260927.md)
refine this track without completing implementation gates. External versions and
missing bundle contents remain unverified; existing ownership and DAG apply.

C0 also freezes location-interval representation and distinct `episode_end` and
`physical_departure` event kinds. Source minute precision and unknown timezones
remain explicit; normalization must not fabricate observed precision. The report
30 field IDs are provisional and do not replace case/event keys above.
