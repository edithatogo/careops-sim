# Phased implementation plan: empirical calibration and validation

Parameter evidence is owned by the [parameter track](../ed_parameter_evidence_20260927/plan.md): P0 precedes C0, P3 supplies the reviewed sampling contract for C2, and P5 qualification precedes C6. C0/C1 source/mapping work consumes the shared catalogue rather than duplicating domain research.


Execution: use the shared [serial/parallel protocol](../../execution-model.md) and
[small-packet decomposition guide](../../execution/decomposition.md). The task
catalog preserves every prerequisite and phase closeout. Prepare and validate
bounded worker packets before dispatch; gpt-6-luna is a candidate worker, not an
assumed authority for unresolved contracts or acceptance decisions.

**Status:** proposed; checkboxes describe future implementation.
**Specification:** [spec.md](spec.md) · **Workflow:** [workflow.md](../../workflow.md)
**Owners:** existing 21/04/03/22; 01/12/25/30 for shared contracts and gates.

Programme prerequisites: D1 before C0; D2 before C1 implementation. C6 reuses
the generic ED fixture delivered by E2, avoiding a second competing model. See
[development readiness](../development_readiness_20260925/plan.md) and
[generic ED delivery](../generic_ed_delivery_20260925/plan.md).


## Research already completed and remaining acceptance

Completed: requested research responses received, duplicates reconciled, embedded
material indexed and relevant findings incorporated into specifications/plans.
See [research handoff](../../research/research-handoff.md) and its evidence links.
This completed intake is distinct from the numbered implementation/qualification
tasks below. No broad repeat search or separate-file recovery is a prerequisite.

Remaining: reuse supplied tables, payloads and narrative; verify relevant primary
sources/current source code, document citations and transformations, reconcile
conflicts, and fill only demonstrated gaps. Mark locally authored structured
records as transcribed or derived; never claim recovery of an unseen original.
A missing source stays unverified or an explicit synthetic assumption with a
validity limit. Acceptance requires this track's actual outputs and tests.

## C0 — Data, fidelity and calibration contracts

Entry: specification review and refresh of existing Kairos plans; queue Q0 API
contract available for cross-track review. No queue implementation needed yet.

- [x] C0.1 Record the smallest-change ADR for a reusable Rust calibration crate,
  Arrow feature placement, runner/configuration extensions and stable model-
  adapter hooks. Resolve the missing Track 22 VVUQ contract reference with 21.
- [x] C0.2 Freeze trace/residual/metric schemas, event mapping, timestamp origin,
  missing/censoring policy, fidelity precedence and observed-versus-predicted
  ledger separation. Specify seed-purpose derivation through 01. Preserve raw task
  events, provisional/realized disposition, knowledge availability and versioned
  exact/partial/unverified mappings; published standards do not imply local conformance.
  Define risk-start/last-observed/event/cause/censor-reason semantics and
  exogenous/primitive/clamp-only/target roles with time-of-knowledge and cluster IDs.
  Preserve event occurrence versus nullable source-recorded/message-created times, location intervals, ED episode end versus physical departure, and observed/derived/defaulted lineage. Reconcile report 30 provisional IDs with this schema; freeze exact source standards editions without assuming local availability.
- [x] C0.3 Assess [public data/open examples](../../generic-ed-evidence.md),
  recording population, field coverage, licence, revision and assumptions.
  Define synthetic datasets, identifiable/confounded parameter fixtures,
  prespecified objective weights/scales, split strategy, numeric tolerances and
  inference validity rules. Record current Arrow dependency/MSRV compatibility
  options; resolve exact versions with 25/30 before adding dependencies.
  Reuse reports 12/30 and prior embedded evidence; verify primary methods/standards and document exact mappings and gaps. Author local schemas/fixtures from supplied content rather than waiting for named download files.
- [x] C0.4 Conductor — review and verify phase (workflow.md).

Exit: reviewed schemas/ADR/test oracles and library/CLI responsibility map.
Manual check: follow one case through arrival, triage, transit, bed and discharge;
identify which times are observed, inferred, clamped, predicted and censored.

## C1 — Real Arrow/Parquet ingestion and normalized traces

Entry: C0. Owners 04 (IO), 21 (semantics); can proceed alongside Q1–Q3.

- [x] C1.1 Write failing actual IPC/Parquet read/write and independent-reader
  fixtures, including wide/long mappings, timestamp units/timezones, DST
  ambiguity, nulls, duplicate IDs, invalid chronology and overflow. Include missing
  triage/cohort denominators, distinct administrative/physical departure and future-
  outcome leakage fixtures; observed task events must survive lossy standards mappings.
  Add report 30 negative transformations: meta.lastUpdated as event recording, MSH-7 as occurrence, A08 as automatic physical movement, and OMOP visit end as observed departure without lineage. Test reversed intervals, minute precision and valid boarding after episode end.
- [x] C1.2 Implement optional Arrow IPC/Parquet features, bounded RecordBatch
  readers and typed schemas. Preserve custom smoke-format compatibility without
  mislabelling it IPC. Lock dependencies and verify feature-minimal/MSRV builds.
  Acceptance: [C1.2 qualification](../../evidence/c1.2-closeout-20261004/README.md),
  development pin f18aba1; source PR #211 remains unmerged while broader gates fail.
- [x] C1.3 Implement normalization, stable external sorting, origin conversion,
  partial-order/occupancy validation, exclusions and provenance manifests. Test
  equivalent input at several batch sizes, row groups and physical row orders.
  Acceptance: [C1.3 qualification](../../evidence/c1.3-closeout-20261004/README.md),
  development pin 18ee41e; source PR #212 remains unmerged while broader gates fail.
- [ ] C1.4 Conductor — review and verify phase (workflow.md).

Exit: C-01; exact normalized canonical record hashes match across reader layouts.
C-01 passes the declared synthetic matrix: [invariance qualification](../../evidence/c01-closeout-20261005/README.md); C1.4 phase review remains open.
Manual check: inspect IPC/Parquet with a second implementation, verify units and
nullable fields, and reconcile input/accepted/excluded/censored counts. Source
identities stay out of public fixture/report artifacts.
Manual readback passes: [independent physical and count evidence](../../evidence/c1-independent-readback-20261005/README.md);
666 files verified, counts conserved, and C1.4 remains open.

## C2 — Macro/Micro execution and minimal spatial behavior

Entry: C0 and reviewed Q0 API; full integration tests require Q4. Owner 03 with
21 model adapter, 01 RNG and 22 scenario configuration review.

- [ ] C2.1 Write failing mode resolution/boundary tests and common-random-number
  fixtures. Assert Macro emits no transit, zero-transit Micro matches its paired
  Macro fixture, and changing mode never discards active or suspended work.
- [ ] C2.2 Implement empirical work-duration providers, FidelityPolicy and stable
  entity/task/purpose keys. Separate intrinsic work from queue/transit intervals;
  persist all mode and stream state for resume.
- [ ] C2.3 Implement a deterministic route-graph TransitModel with units, stable
  shortest-path ties, integer tick conversion, unreachable errors and stored
  movement progress. Supply a minimal urgency/FIFO/zone/skill staff adapter;
  cleaning/reservation remains an explicit model resource lifecycle. Require explicit
  movement-mode speeds, versioned geometry and O/D purpose; test mixed-use pauses
  and sensor-derived distance are not silently treated as path/speed ground truth.
- [ ] C2.4 Conductor — review and verify phase (workflow.md).

Exit: C-02 in standalone fixtures; integrated interruption acceptance awaits Q4.
Manual check: inspect the same synthetic case in Macro/Micro, reconcile clinical
work plus transit plus waits, and demonstrate no double-counted elapsed time.

## C3 — Trace-driven shadow runner

Entry: C1, C2 and Q4. Owners 21 (probe semantics), 22 (runner), 03 (runtime).

- [ ] C3.1 Write failing anchor/probe fixtures: early completion, late completion,
  completion exactly at anchor, missing target, impossible resource occupancy,
  repeated events, interrupted transit/work and bounded probe termination.
  Add the walk+work=5 ridge fixture and independent walk observation;
  assert objective ties do not imply uniquely identified empirical primitives.
- [ ] C3.2 Implement observed-ledger replay and isolated predictive probes with
  explicit snapshots. Preserve macro anchor ticks; allow late probes to finish
  in isolated virtual time or return censored/failed. Never rewind the macro run.
- [ ] C3.3 Capture unclamped target predictions and signed residuals, feasibility,
  missingness and replay role. Implement strict/diagnostic policy and checkpoint
  recovery of outstanding probes without duplicates or future-state leakage.
- [ ] C3.4 Conductor — review and verify phase (workflow.md).

Exit: C-03; intentionally slow walking produces positive residuals despite exact
macro anchoring. Manual check: compare observed ledger and probe logs separately;
verify one probe cannot change another's starting historical state or occupancy.

## C4 — Residual telemetry and statistical distance metrics

Entry: C0/C1 schemas; may proceed before C3 using synthetic predictions. Owner 21;
04 owns typed sidecar encoding and 12 the independent reference fixtures.

- [ ] C4.1 Write analytic W1/KS tests, tied/weighted/unequal/empty cases and
  independent reference fixtures with pinned generator provenance. Add duration
  scaling, large-tick precision, null, censoring and missing-outcome tests.
  Add equal-marginal/opposite-dependence and W1=180/KS=0.2 tail fixtures;
  specify a simple joint/conditional diagnostic before optional multivariate metrics.
- [ ] C4.2 Implement deterministic sorted-CDF W1 and KS D, paired residual
  summaries, grouping/window semantics, counts/validity flags and explicit
  insufficient-data statuses. No automatic classical KS p-value for clustered,
  tied records. Optional inference requires separate documented validity tests.
- [ ] C4.3 Emit calibration_residual.v1/calibration_metric.v1 through actual Arrow
  IO, joined to existing run/event records. Verify stable reduction order and
  compatibility without altering event_log.v1 field types.
- [ ] C4.4 Conductor — review and verify phase (workflow.md).

Exit: C-04; analytic examples match and independent oracle errors satisfy C0
absolute/relative tolerance. Manual check: reproduce one W1 and KS result by hand
and confirm every excluded/censored/unmatched observation is visible in counts.

## C5 — Reproducible fitting, CLI and local multicore runs

Entry: C3/C4. Owners 21 (study/search), 22 (runner/CLI), 01 (seed contract).

- [ ] C5.1 Write failing grid-search, invalid-candidate, tie-ranking, split-leakage,
  failed-probe penalty and resume tests. Test repeated runs and 1/2/N workers
  with randomized wall-clock completion order.
  Add censoring-rate 2/7, day-versus-row bootstrap, temporal cutoff,
  paired CRN and precision-cap fixtures from the research integration ledger.
- [ ] C5.2 Implement bounded candidate enumeration, parameter/unit constraints,
  prespecified normalized objectives, common seed schedules and fixed evaluation
  budgets. Persist all attempts; rank only complete evaluation batches.
  Record ridges/bound hits and identifiability/MC-indeterminate states;
  use prespecified batched precision checks inside fixed caps, with valid stopping
  inference or a fixed confirmation sample. Keep deterministic batch decisions.
- [ ] C5.3 Extend the existing scenario/seed manifest and CLI with trace verify,
  shadow/free replay, calibrate and compare operations. Preserve legacy inputs;
  emit full study/candidate/configuration/artifact provenance and typed failures.
- [ ] C5.4 Integrate CPU worker execution for independent replications/candidates
  through Track 22, with deterministic shard merge and crash/resume handling.
  Keep core state isolated and search decisions independent of worker order.
- [ ] C5.5 Conductor — review and verify phase (workflow.md).

Exit: C-05 recovery portion and C-06 worker/resume equivalence. Manual check: stop
mid-study, resume, and verify candidate ranking/counts/hashes against a complete
run. Legacy runner fixtures still parse and behave as before.

## C6 — Integrated ED demonstration and release evidence

Entry: C5, Q5 and generic ED E2; source and dependency versions frozen for acceptance.
Owners 21/12/25, with 03/04/22 and backend-owner handoff.

- [ ] C6.1 Execute a synthetic ED fixture covering arrivals/acuity, staff zones,
  diagnostics, urgent preemption, bed cleaning and boarding. Calibrate known
  transit parameters, recover the identifiable fixture and flag a deliberately
  confounded service/transit case instead of claiming unique recovery.
- [ ] C6.2 Freeze selected parameters and evaluate free-running Macro/Micro on
  held-out cases/time blocks. Report waits/length of stay/throughput/resource use,
  transit/interruption counts, W1/KS/residuals, uncertainty, exclusions and runtime.
  Require correct synthetic expectations; empirical superiority is not presumed.
  Separate historical-exogenous-input holdout from arrival-generator
  validation, orthogonally to Macro/Micro fidelity. Freeze thresholds and use fresh
  seeds; report marginal/joint conflicts and stochastic/input/structural uncertainty.
- [ ] C6.3 Run feature/API/manifest/Arrow compatibility, deterministic replay and
  performance gates; benchmark ingestion memory, probe overhead, metrics and
  candidate throughput with named data sizes/hardware. Document validity limits.
- [ ] C6.4 Hand off fixtures and contracts to existing Metal/PDES/distributed
  tracks. Add future visualization snapshot/Wasm parity requirements from
  [hybrid-model appraisal](../../hybrid-model-insights.md); render-rate changes
  must not change simulation results. Hardware/UI work stays with those owners.
- [ ] C6.5 Conductor — review and verify phase (workflow.md).

Exit: C-01–C-07 evidence, reviewed documentation/release notes, upstream registry
and handoff synchronization, scoped commits and compatible parent pin update.
Manual check: an independent reader reproduces the full study from its manifest
and input hashes without Python or private EHR data. Any unavailable browser/GPU/
distributed evidence is labelled unverified and does not count as a passed gate.

## Planned verification commands

Run within `libs/kairos` after new crates/features exist. The names `ipc`,
`parquet` and `kairo-ecs-calibration` are proposed and frozen in C0, not existing
commands or completed tests in this planning deliverable.

```sh
cargo fmt --all -- --check
cargo test -p kairo-ecs-arrow --features ipc,parquet
cargo test -p kairo-ecs-calibration
cargo test -p kairo-ecs-calibration --release
cargo test -p kairo-ecs-des -p kairo-ecs-abm -p kairo-ecs-cli
pwsh -File scripts/validate_conformance_fixtures.ps1
pwsh -File scripts/validate_conductor_phase_gates.ps1
pwsh -File scripts/validate_conductor_dag.ps1
```

C0 adds exact feature-minimal/MSRV/clippy and compatibility commands from existing
owner gates. C5 freezes executable CLI reproduction commands in the fixtures.
Each checkpoint stores logs, versions, hashes, tolerances, observed results and
remaining limitations; planned commands alone are not test evidence.

## MVP gpt-6-luna workpack

Every task in this track that is an ancestor of E2.4 is decomposed in the
[MVP leaf recipes](../../execution/mvp/README.md) and
[readable work breakdown](../../execution/mvp/work-breakdown.md). These recipes
are mandatory preparation inputs: freeze/bind interfaces, source slices, paths,
commands and reviewer acceptance before dispatch. Parent tasks close only after
all leaf instances and the original phase acceptance pass. Post-MVP tasks are
outside this workpack. No Luna execution or qualification is implied by coverage.
