# Phased implementation plan: DES queues and preemption

Execution: use the shared [serial/parallel protocol](../../execution-model.md) and
[small-packet decomposition guide](../../execution/decomposition.md). The task
catalog preserves every prerequisite and phase closeout. Prepare and validate
bounded worker packets before dispatch; gpt-6-luna is a candidate worker, not an
assumed authority for unresolved contracts or acceptance decisions.

**Status:** in progress; checked Q0–Q3 tasks record evidence-backed experimental development acceptance. Q4/Q5 remain open; the programme and release are not complete. Parent integration is governed by exact-head hosted checks and native merge readback.
**Specification:** [spec.md](spec.md) · **Workflow:** [workflow.md](../../workflow.md)
**Owners:** upstream 03, with 01/04/12/25 as recorded per milestone.

Programme prerequisites: D1 establishes reviewed toolchain/agent contracts before
Q0; D2 establishes useful remote CI before Q1 implementation. See
[development readiness](../development_readiness_20260925/plan.md).


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

## Q0 — Freeze contracts and compatibility decisions

Entry: reviewed specification, current upstream source/registry refresh. This
phase resolves design details before shared code is changed.

- [x] Q0.1 Record an ADR for additive FlowRuntime, legacy DESContext/Resource
  compatibility, one authoritative DES/ABM world, owned in-memory continuation
  context and public handle/error types. Defer portable checkpoint codecs to
  Track 22; review with 01/03/25.
- [x] Q0.2 Freeze one-unit queue semantics, independent preemption flags, deadline
  boundaries, completion-at-T behavior, cancellation/repriority order, victim
  ties, restart draw reuse and zero-time budgets in executable fixture specs.
  Reconcile report 27 against the local contract: reject shrink below active count, allow idle capacity zero, expire deadline-at-T before grant, complete at T before eviction, and reuse restart draws. Record deliberate SimPy differences; pin reference source before conformance scripts.
  Use report 27 embedded matrix/timelines as completed research input; verify primary reference semantics and convert accepted cases into local executable oracles. Include a latest-admission victim-tie trace, zero-duration grant/completion, and configured same-tick budget exact-boundary/overflow/preserved-pending/reset cases including delivered notifications. Test same-tick cancel/reprioritize order for both equal-priority insertion orders and scheduler-priority precedence. A cancelled claim stays terminal; resubmission uses a new claim identity. No separate test_cases.json is required to begin.
- [x] Q0.3 Reserve Flow event-kind IDs without collisions; define the lifecycle
  sidecar with 01/04. Align checkpoint integration with Track 22's contract if
  available; do not define a competing snapshot format. Fix stale contract
  links through their owners and agree event/transition join semantics.
- [x] Q0.4 Conductor — review and verify phase (workflow.md).

Exit evidence: reviewed ADR/schema/API examples and expected traces for all tie
cases. Manual check: trace the 0/3/5 interruption example and timeout-at-release
example on paper; all reviewers derive the same results. No new core ordering.

## Q1 — ECS state, checked commands and resource invariants

Entry: Q0. Owners 03, shared contracts reviewed by 01.

- [x] Q1.1 Write failing component/invariant tests for live/recycled entities,
  capacity=0, overflow, duplicate release, resource removal and capacity shrink.
- [x] Q1.2 Add ResourceCapacity, ResourceRequest, ClaimQueue<PriorityKey>,
  ActiveAllocations, work/context handles and checked command admission to the
  new runtime. Reuse ComponentRegistry; enforce liveness and despawn cleanup.
- [x] Q1.3 Implement resource/manual lease lifecycle and canonical inspection;
  reject past commands and invalid partial builder state transactionally.
- [x] Q1.4 Conductor — review and verify phase (workflow.md).

Exit: Q-01 and basic Q-02 unit/property tests; no leaked claims after failed
commands. Manual check: inspect components across spawn/acquire/release/despawn
and recycle the owner entity; old handles cannot release a new lease.

## Q2 — Priority queue, deadlines and deterministic dispatch

Entry: Q1. Owner 03; conformance review 12.

- [x] Q2.1 Write failing queue/tie fixtures covering FIFO within priority,
  reprioritization retaining admission sequence, cancellation, deadlines and
  growth/drain. Permute component insertion/removal and event insertion orders.
- [x] Q2.2 Implement ordered queue index, checked admission sequence, deadline
  events and resource arbitration. Separate resource priority from scheduler
  priority; preserve core order and local boundary semantics.
- [x] Q2.3 Add state-machine property tests asserting capacity conservation and
  mutually exclusive queue/active membership after every generated operation.
  Verify timeout-at-T forbids granting at T in both event insertion orders.
- [x] Q2.4 Conductor — review and verify phase (workflow.md).

Exit: Q-01/Q-02/Q-04 for non-preemptive resources; legacy FIFO fixture unchanged.
Manual check: use a small capacity-two example to inspect queue rekeying and all
terminal reasons. Snapshot records match committed state after each event.

## Q3 — Suspend, Abort and Restart

Entry: Q2. Owner 03 with 01 interruption-state/RNG review.

- [x] Q3.1 Write failing three-strategy fixtures with low work(10) at 0 and urgent
  work(2) at 3; cover nested interruptions, multiple victims, equal priorities,
  non-preemptible holders and zero-remaining completion at the interruption tick.
  Add report 27 independent oracle: low work(10) at 0, urgent work(3) at 4; urgent ends 7, low ends 13 Suspend / 17 Restart / aborts 4. Original completion at 10 must be a stale no-op.
- [x] Q3.2 Implement eviction selection and atomic lease replacement, elapsed/
  remaining/cumulative effort, suspended context, attempt revisions, cancel-token
  invalidation and explicit cancellation while suspended.
- [x] Q3.3 Implement resume/restart handlers and exactly-once transition emission.
  Add fault/stale-event injection tests; assert restart reuses the initial draw
  and aborted tasks never resume. Property-test repeated preempt/resume cycles.
- [x] Q3.4 Conductor — review and verify phase (workflow.md).

Exit: Q-03 completion ticks are 12/15/absent; urgent completion=5 for all three;
zero duplicate grants/releases/completions. Manual check: audit every busy and
waiting interval and the owned in-memory interruption context for the fixture.

## Q4 — Declarative Flow, ABM adapter and lifecycle telemetry

Entry: Q3; reviewed 04 lifecycle-sidecar contract. Owner 03; 04 owns telemetry
encoding. Portable checkpoint work is deferred to Track 22 and is not required
to enter or exit this queue phase.

- [x] Q4.1 Write API-level integration tests for acquire/priority/can_preempt/
  preemptible/deadline/work builders, non-reentrant notifications, behavior
  callbacks and a single shared DES/ABM time/world.
- [x] Q4.2 Implement builders, domain dispatch hooks for typed in-memory
  continuation context, deterministic notification order and bounded
  zero-duration feedback handling. Do not add portable codecs without an
  accepted Track 22 contract. Add a migration example retaining the old FIFO API.
- [x] Q4.3 Add resource_lifecycle.v1 telemetry with owning tracks and preserve
  event_log.v1. Portable checkpoint/resume integration is a separate Track 22
  handoff; if its contract is unavailable, record it as deferred rather than
  blocking the queue API or claiming save/restore support.
- [x] Q4.4 Add a synthetic workflow fixture: named staff, urgent interruption,
  staged staff/bed claims and bed cleaning before reavailability. Keep clinical
  rules in the adapter and avoid atomic multi-resource claims not supported by v1.
- [x] Q4.5 Conductor — review and verify phase (workflow.md).

Exit: Q-05/Q-06 integration evidence; calibration C3 can consume the reviewed,
versioned experimental API. Stable API and release acceptance remain Track 25/Q5 gates.
Manual check: create an interruptible task using the fluent API with no manual
component attachment; pause at an event boundary in the same runtime, continue,
and compare terminal state and canonical lifecycle records to uninterrupted
execution. Portable checkpoint/restore requires separate Track 22 evidence.

## Q5 — Conformance, performance and upstream handoff

Entry: Q4; legacy compatibility baseline and benchmark environment recorded.
Owners 03/12/25; coordinate 22/32/34/35 without rewriting backend plans.

- [x] Q5.1 Add deterministic conformance fixtures for all boundary/strategy cases.
  Compare canonical output hashes on repeated serial runs and independent
  replications at 1/2/N local workers. Run property seeds in debug and release.
  Run a bounded pinned SimPy comparison for shared semantics; intentional differences use local expected traces. AllOf is not atomic acquisition; queued cancel, active lease release and scheduled-event cancellation have separate oracles.
- [x] Q5.2 Benchmark FIFO/priority queues, rekey/cancel churn, high interruption
  rates, and many resources at representative queue/active sizes. Record latency,
  throughput, memory and active-victim scan costs against the legacy baseline;
  retain upstream acceptance targets and review any regressions.
- [ ] Q5.3 Run feature-minimal/legacy/API compatibility gates; document migration,
  starvation, one-resource limits, the Track 22 checkpoint boundary and result
  provenance. Do not claim queue-owned portable checkpoint compatibility.
  Hand off single-LP ownership/zero-lookahead restrictions and queue fixtures to
  existing PDES/distributed owners; flag Metal queue execution as unsupported.
- [ ] Q5.4 Conductor — review and verify phase (workflow.md).

Exit: Q-01–Q-07 evidence, risk disposition, release notes, updated upstream records
and clean scoped commits. Manual check: independent reader follows the example,
replays the fixtures and verifies hashes. An unavailable backend remains unverified,
not a passing compatibility result. Parent pin moves only after upstream gates.

## Planned verification commands

Run from `libs/kairos` after the corresponding code/test targets exist. These
commands have **not** been run for this planning deliverable. Exact optional
feature/test names are finalized by Q0/Q4.

```sh
cargo fmt --all -- --check
cargo test -p kairo-ecs-des -p kairo-ecs-abm
cargo test -p kairo-ecs-des --release
cargo test -p kairo-ecs-core -p kairo-ecs-state -p kairo-ecs-rng
cargo test -p kairo-ecs-arrow -p kairo-ecs-cli
pwsh -File scripts/validate_conformance_fixtures.ps1
pwsh -File scripts/validate_conductor_phase_gates.ps1
pwsh -File scripts/validate_conductor_dag.ps1
```

Also run affected clippy/API/feature gates as required by upstream workflow.
Record exact commands, versions, fixture seeds, output hashes and benchmark
artifacts. A documentation checklist does not substitute for their execution.

## MVP gpt-6-luna workpack

Every task in this track that is an ancestor of E2.4 is decomposed in the
[MVP leaf recipes](../../execution/mvp/README.md) and
[readable work breakdown](../../execution/mvp/work-breakdown.md). These recipes
are mandatory preparation inputs: freeze/bind interfaces, source slices, paths,
commands and reviewer acceptance before dispatch. Parent tasks close only after
all leaf instances and the original phase acceptance pass. Post-MVP tasks are
outside this workpack. No Luna execution or qualification is implied by coverage.

Q4.5 development acceptance (2026-10-04): final Kairos pin `1125b5268bb349a5befe12f5789045042faab3e3` passed exact-head two-host native-owner CI [37192692770](https://github.com/edithatogo/kairos/actions/runs/37192692770), phase/DAG/strict clean-tree gates and independent source/governance reviews. Canonical Rust is current stable 1.99.0; separate 1.88/1.76 compatibility floors remain. Parent PR delivery is pending. Q5, Track 22 portability, Track 25 experimental API and release holds are unchanged.

## Q5.2 execution update — 2026-10-04

Q5.2 execution update (2026-10-04): supplementary kernel/runtime harnesses and
exact-key removal are implemented for development qualification. The first
matrix timed out all 13 cases with 100,000 requests; a corrected immutable
rerun and source/CI receipts are retained in the Q5.2 evidence document.
Q5.2 remains unchecked until those cases and canonical regression gates pass.
Next source work follows the reviewed [scaling design](../../design/queue/q5.2-scaling-followup-20261004.md):
delta staging, waiting-deadline lookup and ordered replacement selection,
with source-bound Luna packets and independent failure-atomicity review.

## Q5.2 acceptance — 2026-10-05

[Development qualification](../../evidence/q5.2-completion-20261005/README.md): all 39 runtime cases/five repeats (195 processes) completed, including all 13 100k cases. Correctness and source-equivalence review passed. Both isolated Ubuntu native-owner runs passed 12/12 jobs and six canonical metrics under unchanged thresholds. Original local four-failure and later two-failure comparisons are preserved; source/binary-identical controls do not establish a speedup. Parent development pin advances to 8daa0978578b8a5b5b6427e84db1a3e6c54a1123 while retaining accepted C-01 and C1 readback. Q5.3 compatibility/API/migration and Q5.4 phase review remain open; broader child PR checks are not all green and child PR #214 remains draft/unmerged.
