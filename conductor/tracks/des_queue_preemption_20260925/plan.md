# Phased implementation plan: DES queues and preemption

**Status:** proposed; every checkbox below is future implementation work.
**Specification:** [spec.md](spec.md) · **Workflow:** [workflow.md](../../workflow.md)
**Owners:** upstream 03, with 01/04/12/25 as recorded per milestone.

## Q0 — Freeze contracts and compatibility decisions

Entry: reviewed specification, current upstream source/registry refresh. This
phase resolves design details before shared code is changed.

- [ ] Q0.1 Record an ADR for additive FlowRuntime, legacy DESContext/Resource
  compatibility, single authoritative DES/ABM world, task/context codecs and
  public handle/error types; review with 01/03/25.
- [ ] Q0.2 Freeze one-unit queue semantics, independent preemption flags, deadline
  boundaries, completion-at-T behavior, cancellation/repriority order, victim
  ties, restart draw reuse and zero-time budgets in executable fixture specs.
- [ ] Q0.3 Reserve Flow event-kind IDs without collisions; define lifecycle
  sidecar and snapshot extension with 01/04/22. Fix relevant stale contract
  links through their existing owners. Agree event/transition join semantics.
- [ ] Q0.4 Conductor — review and verify phase (workflow.md).

Exit evidence: reviewed ADR/schema/API examples and expected traces for all tie
cases. Manual check: trace the 0/3/5 interruption example and timeout-at-release
example on paper; all reviewers derive the same results. No new core ordering.

## Q1 — ECS state, checked commands and resource invariants

Entry: Q0. Owners 03, shared contracts reviewed by 01.

- [ ] Q1.1 Write failing component/invariant tests for live/recycled entities,
  capacity=0, overflow, duplicate release, resource removal and capacity shrink.
- [ ] Q1.2 Add ResourceCapacity, ResourceRequest, ClaimQueue<PriorityKey>,
  ActiveAllocations, work/context handles and checked command admission to the
  new runtime. Reuse ComponentRegistry; enforce liveness and despawn cleanup.
- [ ] Q1.3 Implement resource/manual lease lifecycle and canonical inspection;
  reject past commands and invalid partial builder state transactionally.
- [ ] Q1.4 Conductor — review and verify phase (workflow.md).

Exit: Q-01 and basic Q-02 unit/property tests; no leaked claims after failed
commands. Manual check: inspect components across spawn/acquire/release/despawn
and recycle the owner entity; old handles cannot release a new lease.

## Q2 — Priority queue, deadlines and deterministic dispatch

Entry: Q1. Owner 03; conformance review 12.

- [ ] Q2.1 Write failing queue/tie fixtures covering FIFO within priority,
  reprioritization retaining admission sequence, cancellation, deadlines and
  growth/drain. Permute component insertion/removal and event insertion orders.
- [ ] Q2.2 Implement ordered queue index, checked admission sequence, deadline
  events and resource arbitration. Separate resource priority from scheduler
  priority; preserve core order and local boundary semantics.
- [ ] Q2.3 Add state-machine property tests asserting capacity conservation and
  mutually exclusive queue/active membership after every generated operation.
  Verify timeout-at-T forbids granting at T in both event insertion orders.
- [ ] Q2.4 Conductor — review and verify phase (workflow.md).

Exit: Q-01/Q-02/Q-04 for non-preemptive resources; legacy FIFO fixture unchanged.
Manual check: use a small capacity-two example to inspect queue rekeying and all
terminal reasons. Snapshot records match committed state after each event.

## Q3 — Suspend, Abort and Restart

Entry: Q2. Owner 03 with 01 snapshot/RNG review.

- [ ] Q3.1 Write failing three-strategy fixtures with low work(10) at 0 and urgent
  work(2) at 3; cover nested interruptions, multiple victims, equal priorities,
  non-preemptible holders and zero-remaining completion at the interruption tick.
- [ ] Q3.2 Implement eviction selection and atomic lease replacement, elapsed/
  remaining/cumulative effort, suspended context, attempt revisions, cancel-token
  invalidation and explicit cancellation while suspended.
- [ ] Q3.3 Implement resume/restart handlers and exactly-once transition emission.
  Add fault/stale-event injection tests; assert restart reuses the initial draw
  and aborted tasks never resume. Property-test repeated preempt/resume cycles.
- [ ] Q3.4 Conductor — review and verify phase (workflow.md).

Exit: Q-03 completion ticks are 12/15/absent; urgent completion=5 for all three;
zero duplicate grants/releases/completions. Manual check: audit every busy and
waiting interval, interruption record and saved/restored context for the fixture.

## Q4 — Declarative Flow, ABM adapter and lifecycle telemetry

Entry: Q3; reviewed 04 sidecar and 01/22 checkpoint contracts. Owner 03;
04 owns encoding; 22 owns runner/snapshot integration.

- [ ] Q4.1 Write API-level integration tests for acquire/priority/can_preempt/
  preemptible/deadline/work builders, non-reentrant notifications, behavior
  callbacks and a single shared DES/ABM time/world.
- [ ] Q4.2 Implement builders, domain dispatch hooks, registered context codecs,
  deterministic notification order and bounded zero-duration feedback handling.
  Add a migration example retaining the old FIFO API.
- [ ] Q4.3 Add resource_lifecycle.v1 telemetry and checkpoint/resume support with
  owning tracks. Preserve event_log.v1. Restore queue sequences, leases, pending
  commands, completion revisions, RNG state and suspended work.
- [ ] Q4.4 Add a synthetic workflow fixture: named staff, urgent interruption,
  staged staff/bed claims and bed cleaning before reavailability. Keep clinical
  rules in the adapter and avoid atomic multi-resource claims not supported by v1.
- [ ] Q4.5 Conductor — review and verify phase (workflow.md).

Exit: Q-05/Q-06 integration evidence; calibration C3 can consume the stable API.
Manual check: create an interruptible task using the fluent API with no manual
component attachment; pause while suspended, restore and compare terminal state
and canonical lifecycle records to uninterrupted execution.

## Q5 — Conformance, performance and upstream handoff

Entry: Q4; legacy compatibility baseline and benchmark environment recorded.
Owners 03/12/25; coordinate 22/32/34/35 without rewriting backend plans.

- [ ] Q5.1 Add deterministic conformance fixtures for all boundary/strategy cases.
  Compare canonical output hashes on repeated serial runs and independent
  replications at 1/2/N local workers. Run property seeds in debug and release.
- [ ] Q5.2 Benchmark FIFO/priority queues, rekey/cancel churn, high interruption
  rates, and many resources at representative queue/active sizes. Record latency,
  throughput, memory and active-victim scan costs against the legacy baseline;
  retain upstream acceptance targets and review any regressions.
- [ ] Q5.3 Run feature-minimal/legacy/API compatibility gates; document migration,
  starvation, one-resource limits, checkpoint compatibility and result provenance.
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
