# Plan: generic ED library delivery

Input evidence is owned by the [parameter track](../ed_parameter_evidence_20260927/plan.md): P0 precedes E0 and P4 supplies the frozen example profiles before E1. E2 supplies the actual model for P5 coverage checks. E0 source discovery reuses this work rather than maintaining a second catalogue.


Execution: use the shared [serial/parallel protocol](../../execution-model.md) and
[small-packet decomposition guide](../../execution/decomposition.md). The task
catalog preserves every prerequisite and phase closeout. Prepare and validate
bounded worker packets before dispatch; gpt-6-luna is a candidate worker, not an
assumed authority for unresolved contracts or acceptance decisions.

Every phase uses [workflow.md](../../workflow.md). Task-level acceptance below is
mandatory; budgets/thresholds are frozen in E0/D3 before measuring acceptance.

## E0 — Domain contract and minimal buildable skeleton

Entry: D1 tool/compatibility decisions; can review domain details alongside Q0/C0.

- [ ] E0.1 Write tests for scenario/schema validation, module boundary violations
  and a single deterministic patient fixture before implementing the skeleton.
- [ ] E0.2 Create a minimal Rust workspace with ED library/CLI boundary, Kairos path
  dependency and locked builds; define public API/errors, time units and profile
  schema. Keep domain/site/presentation distinct. One-patient example must run.
- [ ] E0.3 Inventory public sources/open examples, exact revisions/licences and
  fields; specify assumptions for arrivals/acuity/work/diagnostics/boarding and
  generic layout. Define metric formulas, horizon/warm-up and release support.
- [ ] E0.4 Conductor — review and verify phase (workflow.md).

Exit: buildable skeleton and synthetic fixture unlock GitHub D2; not a full ED.
Manual check: fresh checkout produces the declared one-patient outcome.

## E1 — Generic patient pathway on shared DES/ABM state

Entry: D2, Q4, C1/C2 reviewed implementation; no C6 dependency yet.

- [ ] E1.1 Write pathway tests for empty run, one patient, each acuity/pathway,
  saturated arrival demand, zero/long service, impossible resource config and
  endpoint count conservation. Define explicit metrics and expected intervals.
- [ ] E1.2 Implement seeded time-varying arrivals, triage/assessment/diagnostics/
  treatment/disposition states and stable patient/task IDs; configure service
  distributions with provenance. Do not hide queue time in service samples.
- [ ] E1.3 Implement admission boarding/discharge and optional abandonment/transfer
  policies, with terminal reason/censoring and endpoint timestamps. Version policy
  choices and test time bounds, run-horizon endings and unfinished work.
- [ ] E1.4 Conductor — review and verify phase (workflow.md).

Exit: generic flow conserves all patients/outcomes under E-R1/4. Manual verification:
trace a patient through every pathway and reconcile event times with its summary.

## E2 — Staff, capacity, spatial tasks and realistic operational constraints

- [ ] E2.1 Write tests for shifts/breaks/skill-zone constraints, urgent preemption,
  staff handover, cleaning/reservation, blocked diagnostics and boarding capacity.
- [ ] E2.2 Implement minimal agent dispatch, Macro/Micro route/work execution and
  phased resource claims avoiding unsupported atomic multi-resource acquisition.
  Shift reductions must honor the resource shrink/in-flight-work contract.
- [ ] E2.3 Add a composed synthetic ED fixture, simultaneous events and restart/
  resume edge cases; verify no double allocation, leaks, starvation hidden by
  censoring, or movement driven by animation time. Make this the shared C6 fixture.
- [ ] E2.4 Conductor — review and verify phase (workflow.md).

Exit: E-R2/3; C6 can calibrate/validate the actual generic model. Manual check:
inspect staff occupancy and patient availability across a shift and interruption.

## E3 — Usable experiment API, runner, outputs and recovery

Entry: C5. Owners parent ED adapter plus upstream 22/04/01.

- [ ] E3.1 Write public API/CLI tests for validate/run/compare/export, cancellation,
  malformed config, deterministic error codes, output overwrite protection,
  disk-full/interrupted output, seed/replication allocation and crash recovery.
- [ ] E3.2 Integrate existing runner/scenario formats, independent CPU workers,
  checkpointing and progress. Write artifacts atomically; a partial run is clearly
  incomplete and cannot be mistaken for a valid finished experiment.
- [ ] E3.3 Export Arrow/Parquet and summaries with metric denominators, uncertainty,
  censored/unfinished counts and complete provenance; test 1/2/N workers and
  checkpoint equivalence through ED+queue+calibration state.
- [ ] E3.4 Conductor — review and verify phase (workflow.md).

Exit: E-R4/5 and reproducible usable headless API. Manual check: cancel/resume a
batch and compare the completed artifact bundle with an uninterrupted baseline.

## E4 — Complete and harden the native ED library

Entry: Q5, C6, E3 and D3/D4 release evidence.

- [ ] E4.1 Add analytic limiting cases (no contention, deterministic single server,
  known queue fixture), invariant/differential tests, public-reference comparisons
  and independent review. Distinguish software verification from domain validation.
- [ ] E4.2 Run sensitivity/identifiability, held-out Macro/Micro and warm-up/
  observation-window checks; establish uncertainty and profile-specific validity
  limits. Benchmark representative demand, memory and long-run leak behavior.
- [ ] E4.3 Publish executable docs/examples and migration/support tables; package
  dry-run and test from a clean downstream Rust project at promised toolchains.
  Verify release evidence, dependency pins, licences/SBOM and restore/rollback.
- [ ] E4.4 Conductor — review and verify phase (workflow.md).

Exit: G1 complete only after evidence; no production/Cairns validity assertion.
Manual verification: new consumer installs the candidate and reproduces example
outputs from checked-in public/synthetic inputs without a private data dependency.

## E5 — Interactive scenario dashboard and binding qualification

Entry: G1; upstream 05/02/09 coordinated scope, public API review before binding work.

- [ ] E5.1 Freeze a narrow snapshot/command ABI and implement native/Wasm parity
  fixtures for IDs/u128 time, configuration, progress, cancellation and errors.
  Review actual FFI needs, memory/buffer lifetime and supported browser targets.
- [ ] E5.2 Build scenario edit/run/compare/export UI; choose presentation components
  after prototype measurement (Pixi optional). Run simulation in a worker or
  explicit local runner; keep engine and frame clocks independent.
- [ ] E5.3 Test render rates off/30/60, slow consumers/backpressure, memory growth,
  restart/route state, accessibility and useful error reporting. Measure actual
  transfer copies/frame latency; never infer WebGPU compute from rendering.
- [ ] E5.4 Conductor — review and verify phase (workflow.md).

Exit: G2 usable product with verified exports and interaction; reapply D4 gates
for introduced browser/input/FFI surfaces. Manual check: user runs and compares
staffing scenarios and obtains the same results in headless mode.

## E6 — Real Apple Metal acceleration through existing Track 32

Entry: E4 CPU correctness/performance baseline; preserve upstream performance
gates. E6 may proceed alongside E5 dashboard work in disjoint owner paths;
metadata records this explicitly. It does not wait for browser development.

- [ ] E6.1 Profile batch numeric workloads and choose a kernel with measurable
  end-to-end benefit (initial candidate: batched numeric metrics/transit evaluation).
  Write CPU/device oracle, precision/ranking and fallback/error tests first.
- [ ] E6.2 Implement real wgpu/WGSL device path in owner 32, pin tested dependencies
  and record device/driver/compiler details. Keep queue arbitration CPU-owned.
- [ ] E6.3 Run actual Apple hardware parity/throughput/transfer/memory tests,
  unavailable-device behavior and representative end-to-end benchmark. If no
  material gain, retain experimental status and document findings rather than
  declaring acceleration successful. MLX replacement requires comparative ADR.
- [ ] E6.4 Conductor — review and verify phase (workflow.md).

Exit: G3-Metal only for measured supported kernels. Manual check: prove backend
selection dispatched on hardware, not a CPU fallback with a GPU label.

## E7 — Within-run parallel DES through existing Track 34

- [ ] E7.1 Define ED LP partition/resource ownership and lookahead constraints;
  write serial-oracle, in-flight message/GVT and zero-lookahead failure tests.
  Certify each cross-LP minimum causal advance, including same-time
  cycles; empirical duration means/quantiles are never lookahead guarantees.
- [ ] E7.2 Integrate actual scheduler/Flow state with conservative PDES and real
  worker communication; preserve task RNG and snapshot contracts. Restrict shared
  resources to a defined owner and model explicit inter-LP messages.
- [ ] E7.3 Test threaded 1/2/N execution, deadlock/causality/migration boundaries,
  slow workers, checkpoint and final observable state parity. Do not impose an
  identical global cross-LP trace beyond Track 34's contract. Measure speedup.
  Distinguish trace/state/statistical equivalence per upstream contract;
  statistical similarity alone cannot pass deterministic CPU acceptance. Record
  safe-frontier blocking, communication overhead and negative scaling results.
- [ ] E7.4 Conductor — review and verify phase (workflow.md).

Exit: G3-PDES supported partition/profile, explicit unsupported zero-lookahead
cases and retained performance acceptance. Manual check: exercise a cross-LP
resource interaction and reconcile all claims/messages with the serial oracle.

## E8 — Real distributed execution through existing Track 35

- [ ] E8.1 Specify and test versioned transport envelopes, allocation/entity
  migration ownership, deduplication, reconnect/checkpoint and failure policy;
  extend D4 threat review before network execution.
  Account for in-flight traffic and persisted deduplication at coordinated
  checkpoint publication; define event identity, ownership-map revision and restart
  incarnation compatibility without replacing existing RNG/envelope contracts.
- [ ] E8.2 Implement real gRPC and MPI adapters within existing owners using
  evaluated current tonic/prost/MPI dependencies at that phase. Pin runtime and
  library versions then; do not add them to the initial native-ED dependency graph.
- [ ] E8.3 Run actual multi-process/multi-node tests: loss/delay/retry, duplicate
  message/migration, worker failure, bounded shutdown and telemetry aggregation.
  Verify serial/PDES observable parity and profile-specific speed/memory evidence.
  Exercise crashes before/after checkpoint publication, corrupt snapshots
  and obsolete incarnation traffic; include independent-replication transport
  profiles without making multiple physical nodes a C5/E7 prerequisite.
- [ ] E8.4 Conductor — review and verify phase (workflow.md).

Exit: G3-distributed only for executed runtime profiles. Cloud deployment and
browser WebGPU continue through existing 39/43/33 owner plans if needed; this phase
does not create later clinical-domain tracks. Manual verification: reproduce a
network failure and recover or terminate as specified without duplicated patients.
