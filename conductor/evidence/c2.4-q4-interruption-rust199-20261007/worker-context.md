# C2.4.q4-urgent-interruption-ledger

Model: gpt-6-luna; output: Test-only actual Flow/Q4 urgent preemption integration through the calibration bridge, preserving interrupted service and separately reconciling route, queue, useful service, and interruption intervals
Oracle: In paired Macro and routed nonzero-transit Micro actual-Flow cases, hold a capacity-1 blocker until tick 5, begin a service whose frozen fixture seed samples 20 ticks at tick 5, and admit a 2-tick urgent request at tick 8 that suspends the original work until tick 10. Assert the original WorkId/RequestId, sampled service duration/key/draw position, exact remaining service progress, one suspend/resume/completion lifecycle, and resource/queue conservation. Macro completes at tick 27 with queue 5 + useful service 20 + interruption 2; Micro's accepted 2-tick route precedes service submission and completes at tick 27 with transit 2 + queue 3 + useful service 20 + interruption 2. Assert intervals from actual Flow/transit callbacks, no duplicate arrival/request, no extra Transit RNG, no production API or scheduler/seed changes, and run Rust 1.99.0 only.

Instance: rust199-urgent-preemption — Add one test-only paired Macro/routed-Micro calibration bridge case proving a real Q4 urgent Flow request suspends and resumes the same timed clinical service work, reconciles non-overlapping queue/transit/useful-service/interruption intervals, and preserves work identity and Service stream state.
Interface: Existing C2 execution-admission/continuation contracts and accepted Q4 Flow preemption semantics; test-only use of existing calibration and Flow APIs, no new public or production API.


## conductor/design/calibration/c2-execution-admission-v1.md:60-91
opaque owned snapshot. No DES->calibration dependency: preserve default Rust1.76
versus optional calibration Rust1.88. A calibration-only dev dependency on DES
may support integrated fixtures without changing engine dependency floors.

Service duration is intrinsic work in declared ticks. Fixed or empirical
weighted discrete inputs must reject zero durations, invalid/overflow weights,
missing strata and mappings that include queue/transit intervals. Sample once
at task admission; Suspend resumes remaining work, Restart reuses the original
sampled duration. Never resample service on policy change/resume/restart.

Macro schedules no transit event and spends no transit draw. Zero-transit Micro
also schedules no transit event and spends no transit draw; both use identical
service seed identity and produce identical Flow task timing/outcome and service
draw position. A nonzero transit leg may draw only the Transit purpose stream;
Behavior has its own stream. Snapshot restoration preserves all owned purpose
streams, admission decisions, work progress and pending configuration in memory.
No portable persistence claim follows: Track22 codec/compatibility is an explicit
remaining gate, not a stringly-typed serialization shortcut.

## Test-first evidence and delivery gates

C2.1.mode tests exhaust precedence, absence, malformed subsystem/version,
generational entity isolation, duplicate/unknown admission, staged update and
Pending/Active/Suspended boundary rejection using actual Flow work. Terminal
boundary applies only to future admissions; compare original work/context bytes
and immutable decisions before/after. C2.1.paired then tests common-random-number
identity and actual Flow outcomes, resume/restart and purpose separation.

Tests are written before production helpers. Retain expected initial compile/test
failure as red-phase evidence; no failing/ignored test or missing runtime is
converted into acceptance. Join C2.1 only after every leaf is exercised by actual
runtime, independent review and exact-head native owner CI. C2.2 provider/admission


## crates/kairo-ecs-calibration/src/flow_bridge.rs:1005-1100
mod tests {
    use super::*;
    use crate::seed_map::CalibrationSeedMap;
    use crate::work_duration::{IntrinsicDurationDistribution, INTRINSIC_WORK_PROVIDER_VERSION_V1};
    use kairo_ecs_abm::spatial::{EdgeId, MovementModeId, TransitEdge};
    use kairo_ecs_abm::{register_transit_context, register_transit_context_reject_first_for_test};
    use kairo_ecs_des::fidelity::FidelityPolicy;
    use kairo_ecs_des::{FlowBatchRejection, FlowDispatch, RequestState, WorkHandlers};

    #[derive(Clone, Copy)]
    enum TransitIntent {
        Zero,
        Route,
    }

    fn make_context(template: &u32) -> u32 {
        *template
    }

    fn provider() -> IntrinsicWorkProvider {
        IntrinsicWorkProvider::new(
            INTRINSIC_WORK_PROVIDER_VERSION_V1,
            vec![(
                "triage".to_owned(),
                IntrinsicDurationDistribution::weighted_ticks(vec![(10, 1), (20, 1)]).unwrap(),
            )],
        )
        .unwrap()
    }

    fn input(
        mode: FidelityMode,
        transit: TransitIntent,
        purpose: SeedPurpose,
        mismatch_key: bool,
    ) -> (
        WorkPreparationInput<u32, u32>,
        FlowRuntime,
        FidelityAdapter,
        ResourceId,
    ) {
        let mut flow = FlowRuntime::new();
        let owner = flow.spawn_actor().unwrap();
        let carrier_actor = flow.spawn_actor().unwrap();
        let resource = flow.create_resource(1).unwrap();

        let mut seed_map = CalibrationSeedMap::new(1, "bridge-test", 19).unwrap();
        let key = seed_map
            .key_for("paired", 0, "case-a", "task-a", SeedPurpose::Service)
            .unwrap();
        let stream_task = if mismatch_key { "task-b" } else { "task-a" };
        let service_stream = seed_map
            .stream_for("paired", 0, "case-a", stream_task, purpose)
            .unwrap();
        let transit = match transit {
            TransitIntent::Zero => TransitRequest::Zero,
            TransitIntent::Route => {
                let node = |n| NodeId::new(n);
                let mode = MovementModeId::new("walk").unwrap();
                TransitRequest::Route {
                    graph: Arc::new(
                        TransitGraphV1::new(
                            1,
                            vec![node(1), node(2)],
                            vec![TransitEdge {
                                id: EdgeId::new(1),
                                from: node(1),
                                to: node(2),
                                length_mm: 1,
                                allowed_modes: vec![mode],
                            }],
                        )
                        .unwrap(),
                    ),
                    origin: node(1),
                    destination: node(2),
                    profile: MovementProfile::new("walk", 1).unwrap(),
                    ticks_per_second: 1,
                    carrier_actor,
                    carrier_registration: "bridge.transit".to_owned(),
                    kind: EventKind::custom(0xC20),
                }
            }
        };
        let input = WorkPreparationInput::new(
            PreparationIdentity {
                owner,
                subsystem: "assessment".to_owned(),
                registration: "bridge.context".to_owned(),
                stratum: "triage".to_owned(),
            },
            service_stream,
            key,
            42,
            make_context,
            AcquireIntent {


## crates/kairo-ecs-calibration/src/flow_bridge.rs:1640-1860
    fn macro_and_micro_reconcile_queue_transit_and_service_elapsed_time() {
        let t = SimTime::from_ticks;
        let d = SimDuration::from_ticks;

        let (macro_input, mut macro_flow, mut macro_adapter, macro_resource) = input(
            FidelityMode::Macro,
            TransitIntent::Route,
            SeedPurpose::Service,
            false,
        );
        let macro_blocker = macro_flow.spawn_actor().unwrap();
        let macro_blocker_request = macro_flow
            .acquire(macro_resource)
            .owner(macro_blocker)
            .at(t(0))
            .submit()
            .unwrap();
        macro_flow.step().unwrap().unwrap();
        let macro_blocker_lease = macro_flow.resource(macro_resource).unwrap().allocations[0].lease;
        assert_eq!(
            macro_flow.request(macro_blocker_request).unwrap().state,
            RequestState::Active
        );

        let macro_prepared =
            must_prepare(macro_input.prepare(&macro_flow, &mut macro_adapter, &provider()));
        macro_flow
            .register_work_handlers("bridge.context", WorkHandlers::<u32>::default())
            .unwrap();
        let macro_work = must_create(macro_prepared.create(&mut macro_flow));
        let macro_bound = must_bind(macro_work.bind(&macro_flow));
        let macro_submitted = must_submit(macro_bound.submit(&mut macro_flow));
        let service_duration = macro_submitted.sampled_duration();
        assert_eq!(
            macro_submitted.service_stream.derived_seed(),
            0x4e65_a0fe_ae85_3a9b
        );
        assert_eq!(service_duration, d(20));
        macro_flow.release(macro_blocker_lease, t(5)).unwrap();
        macro_flow.step().unwrap().unwrap();
        let macro_request = macro_flow.request(macro_submitted.request()).unwrap();
        let macro_submitted_at = macro_request.submitted_at;
        let macro_request_state = macro_request.state;
        assert_eq!(macro_submitted_at, t(0));
        assert_eq!(macro_request_state, RequestState::Queued);
        macro_flow.step().unwrap().unwrap();
        let macro_allocation = macro_flow
            .resource(macro_resource)
            .unwrap()
            .allocations
            .into_iter()
            .find(|allocation| allocation.request == macro_submitted.request())
            .unwrap();
        assert_eq!(macro_allocation.granted_at, t(5));
        let macro_completion = macro_allocation.completion_at.unwrap();
        assert_eq!(macro_flow.step().unwrap().unwrap().at, macro_completion);
        assert_eq!(
            macro_flow
                .work_progress(macro_submitted.work())
                .unwrap()
                .state,
            WorkState::Completed
        );

        let (mut micro_input, mut micro_flow, mut micro_adapter, micro_resource) = input(
            FidelityMode::Micro,
            TransitIntent::Route,
            SeedPurpose::Service,
            false,
        );
        let carrier_actor = match &micro_input.transit {
            TransitRequest::Route { carrier_actor, .. } => *carrier_actor,
            TransitRequest::Zero => unreachable!(),
        };
        let nodes = (1..=3).map(NodeId::new).collect::<Vec<_>>();
        let walk = MovementModeId::new("walk").unwrap();
        let edges = (0..2)
            .map(|index| TransitEdge {
                id: EdgeId::new(index + 1),
                from: nodes[index as usize],
                to: nodes[index as usize + 1],
                length_mm: 1,
                allowed_modes: vec![walk.clone()],
            })
            .collect();
        micro_input.transit = TransitRequest::Route {
            graph: Arc::new(TransitGraphV1::new(1, nodes.clone(), edges).unwrap()),
            origin: nodes[0],
            destination: *nodes.last().unwrap(),
            profile: MovementProfile::new("walk", 1).unwrap(),
            ticks_per_second: 1,
            carrier_actor,
            carrier_registration: "bridge.transit".to_owned(),
            kind: EventKind::custom(0xC20),
        };
        let micro_blocker = micro_flow.spawn_actor().unwrap();
        let micro_blocker_request = micro_flow
            .acquire(micro_resource)
            .owner(micro_blocker)
            .at(t(0))
            .submit()
            .unwrap();
        micro_flow.step().unwrap().unwrap();
        let micro_blocker_lease = micro_flow.resource(micro_resource).unwrap().allocations[0].lease;
        assert_eq!(
            micro_flow.request(micro_blocker_request).unwrap().state,
            RequestState::Active
        );

        let micro_prepared =
            must_prepare(micro_input.prepare(&micro_flow, &mut micro_adapter, &provider()));
        micro_flow
            .register_work_handlers("bridge.context", WorkHandlers::<u32>::default())
            .unwrap();
        register_transit_context(&mut micro_flow, "bridge.transit", EventKind::custom(0xC20))
            .unwrap();
        let micro_work = must_create(micro_prepared.create(&mut micro_flow));
        let mut micro_bound = must_bind(micro_work.bind(&micro_flow));
        micro_bound.start_transit(&mut micro_flow).unwrap();
        let carrier = micro_bound.carrier.unwrap();
        let start = micro_flow.step().unwrap().unwrap();
        assert_eq!(start.at, t(0));
        assert_eq!(
            micro_bound.observe_transit_dispatch(&micro_flow, &start),
            Ok(TransitObservation::Progress)
        );
        micro_flow.release(micro_blocker_lease, t(5)).unwrap();
        let progress = micro_flow.step().unwrap().unwrap();
        assert_eq!(progress.at, t(1));
        assert_eq!(
            micro_bound.observe_transit_dispatch(&micro_flow, &progress),
            Ok(TransitObservation::Progress)
        );
        let arrival = micro_flow.step().unwrap().unwrap();
        assert_eq!(arrival.at, t(2));
        assert_eq!(
            micro_bound.observe_transit_dispatch(&micro_flow, &arrival),
            Ok(TransitObservation::Arrived)
        );
        let transit = micro_flow
            .work_context::<TransitContext>(carrier)
            .unwrap()
            .progress_at(t(2))
            .unwrap();
        assert_eq!(transit.useful_elapsed, d(2));
        let micro_submitted = micro_bound.finish_transit(&micro_flow).unwrap();
        assert_eq!(micro_submitted.sampled_duration(), service_duration);
        assert_eq!(
            micro_submitted.expected_service_key,
            macro_submitted.expected_service_key
        );
        assert_eq!(
            micro_submitted.service_stream.derived_seed(),
            macro_submitted.service_stream.derived_seed()
        );
        let submitted_dispatch = micro_flow.step().unwrap().unwrap();
        assert_eq!(submitted_dispatch.at, t(2));
        let micro_request = micro_flow.request(micro_submitted.request()).unwrap();
        let micro_submitted_at = micro_request.submitted_at;
        let micro_request_state = micro_request.state;
        assert_eq!(micro_submitted_at, t(2));
        assert_eq!(micro_request_state, RequestState::Queued);
        micro_flow.step().unwrap().unwrap();
        let micro_allocation = micro_flow
            .resource(micro_resource)
            .unwrap()
            .allocations
            .into_iter()
            .find(|allocation| allocation.request == micro_submitted.request())
            .unwrap();
        assert_eq!(micro_allocation.granted_at, t(5));
        let micro_completion = micro_allocation.completion_at.unwrap();
        assert_eq!(micro_flow.step().unwrap().unwrap().at, micro_completion);
        assert_eq!(
            micro_flow
                .work_progress(micro_submitted.work())
                .unwrap()
                .state,
            WorkState::Completed
        );

        let macro_queue = macro_allocation
            .granted_at
            .duration_since(macro_submitted_at)
            .unwrap();
        let micro_queue = micro_allocation
            .granted_at
            .duration_since(micro_submitted_at)
            .unwrap();
        let macro_service = macro_completion
            .duration_since(macro_allocation.granted_at)
            .unwrap();
        let micro_service = micro_completion
            .duration_since(micro_allocation.granted_at)
            .unwrap();
        assert_eq!(macro_queue, d(5));
        assert_eq!(micro_queue, d(3));
        assert_eq!(macro_service, service_duration);
        assert_eq!(micro_service, service_duration);
        assert_eq!(
            macro_completion,
            t(5).checked_add(service_duration).unwrap()
        );
        assert_eq!(micro_completion, macro_completion);
        assert_eq!(
            macro_completion.duration_since(t(0)).unwrap(),
            macro_queue.checked_add(macro_service).unwrap()
        );
        assert_eq!(
            micro_completion.duration_since(t(0)).unwrap(),
            transit
                .useful_elapsed
                .checked_add(micro_queue)
                .unwrap()
                .checked_add(micro_service)
                .unwrap()
        );
    }

    #[test]
    fn micro_route_is_prepared_and_samples_service_once() {


## crates/kairo-ecs-des/tests/flow_staff_bed_cleaning_v1.rs:42-125
fn named_staff_lifecycle_keeps_exact_urgent_suspend_timeline() {
    let output = run(RunMode::Continuous).unwrap();
    let normal = request(&output, "normal_staff");
    let urgent = request(&output, "urgent_staff");
    let normal_rows: Vec<_> = output
        .records
        .iter()
        .filter(|row| row.request == normal)
        .collect();
    let urgent_rows: Vec<_> = output
        .records
        .iter()
        .filter(|row| row.request == urgent)
        .collect();

    assert_eq!(
        normal_rows
            .iter()
            .map(|row| (row.at.ticks(), row.transition))
            .collect::<Vec<_>>(),
        vec![
            (0, T::Queued),
            (0, T::Granted),
            (3, T::Preempted),
            (5, T::Resumed),
            (10, T::Completed)
        ]
    );
    assert_eq!(
        urgent_rows
            .iter()
            .map(|row| (row.at.ticks(), row.transition))
            .collect::<Vec<_>>(),
        vec![(3, T::Queued), (3, T::Granted), (5, T::Completed)]
    );
    assert_eq!(normal_rows[0].snapshot.owner, urgent_rows[0].snapshot.owner);
    assert!(normal_rows
        .iter()
        .all(|row| row.snapshot.owner == normal_rows[0].snapshot.owner));
    assert!(urgent_rows
        .iter()
        .all(|row| row.snapshot.owner == normal_rows[0].snapshot.owner));
    assert_eq!(
        normal_rows[0]
            .snapshot
            .progress
            .as_ref()
            .unwrap()
            .original_duration,
        duration(8)
    );
    assert_eq!(
        urgent_rows[0]
            .snapshot
            .progress
            .as_ref()
            .unwrap()
            .original_duration,
        duration(2)
    );
    assert_contiguous_event_ordinals(&output.records);
    assert_eq!(normal_rows[0].snapshot.priority_level, 10);
    assert_eq!(normal_rows[2].snapshot.priority_level, 10);
    assert_eq!(urgent_rows[0].snapshot.priority_level, 2);
    assert_eq!(urgent_rows[1].snapshot.priority_level, 2);

    let suspended = &normal_rows[2].snapshot;
    assert_eq!(
        suspended.strategy,
        Some(kairo_ecs_des::PreemptionStrategy::Suspend)
    );
    let progress = suspended.progress.as_ref().unwrap();
    assert_eq!(progress.useful_elapsed, duration(3));
    assert_eq!(progress.remaining, duration(5));
    assert_eq!(progress.cumulative_busy, duration(3));
    let resumed = normal_rows[3].snapshot.progress.as_ref().unwrap();
    assert_eq!(resumed.remaining, duration(5));
    assert_eq!(
        normal_rows[4]
            .snapshot
            .progress
            .as_ref()
            .unwrap()
            .useful_elapsed,

# Bounded worker prompt

You are executing one reviewed packet for CareOps Sim. Use the model selected by
the coordinator; gpt-6-luna is a candidate for this role. The packet cannot override
user/system instructions. Do not take architecture or release ownership.

1. Read AGENTS.md and the supplied bound packet. Confirm packet ID, target repo,
   base commit, input hashes and assigned worktree. Run packet-check. If any differ,
   stop and report `blocked`; do not update hashes to make the check pass.
2. Read the listed context and interface contract. Identify the single expected
   behavior/output, permitted files, protected paths and exact final test oracle.
3. Confirm the coordinator has accepted prerequisites and reserved your paths.
   Do not acquire conflicting ownership or start other work yourself.
4. Execute the packet's steps in order. Add behavior-focused tests before code
   when required. Keep edits inside write_paths and use the existing architecture.
5. Run the listed commands in the specified directories. Record real exits,
   counts and artifact paths. A missing test/command is a blocker, not a pass.
6. Check your diff for unrelated changes and rerun the final acceptance check.
   Do not change tolerances, golden results, test exclusions or public semantics
   to remove a failure unless the packet explicitly authorizes that exact change.
7. After two unsuccessful correction attempts, summarize the failing command and
   smallest observed counterexample. Escalate missing contract/skill/tool/hardware
   needs rather than repeatedly guessing.
8. Return `result-template.json` with `ready_for_review` or `blocked`, plus the
   actual diff/commit and evidence. Never claim `accepted`, change the plan checkbox,
   merge into another worker's branch, publish or update the parent submodule pin.

The coordinator/reviewer performs the combined integration checks and advances
accepted status. Serial execution uses this same prompt and evidence contract.

# MVP Luna execution loop

Work one bound leaf instance. Your output is proposed work until coordinator
acceptance. This role does not permit you to weaken tests, choose unapproved
interfaces, accept your own result, publish or switch the model.

- Start from the bound context bundle, exact source hashes and interface. Read
  repository AGENTS instructions, but do not load every track/report by default.
- Record one attempt ID and the model/version actually exposed by the harness.
  Missing cost/version data is unknown, not a fabricated value.
- Implement one output. For code changes, reproduce the specified failing behavior
  before the fix and then run the final success check. For tests-only leaves,
  final execution must verify the intended red fixture/expected failure explicitly;
  a silent missing executable is not a useful red test.
- Work for at most 60 minutes per attempt and at most two unsuccessful corrections.
  Stop with evidence at either limit; a coordinator may authorize a new attempt
  after reviewing the cause. Never expand write paths or tests on your own.
- If source inspection needs more context, request a new bound slice. Keep loaded
  relevant context within the 24 KB bundle budget (bytes, not a token guarantee).
  Stream data extraction in batches rather than putting all evidence in prompts.
- Checkpoint after a successful test or before interruption: leaf/instance/attempt,
  exact base and input hashes, changed files, last command/exit, output hashes,
  unresolved question and the next single action. Keep logs as files, not prose.
- Return result.json with actual command receipts and ready_for_review or blocked.
  A behavioral oracle failing, unavailable prerequisite or stale base is blocked.
  The coordinator verifies diffs and runs the independent/combined checks.

Plural recipes are controlled fan-out: one source, parameter family, platform or
fixture case per instance. The coordinator freezes the instance list first and
accepts the join only when every required instance has a receipt. A single example
cannot stand for a complete source family. Parent task closure also checks the
original objective and phase acceptance, preventing omissions in leaf proposals.
