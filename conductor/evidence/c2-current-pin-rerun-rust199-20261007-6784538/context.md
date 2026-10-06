# C2.4.q4-urgent-interruption-ledger

Model: gpt-6-luna; output: Test-only actual Flow/Q4 urgent preemption integration through the calibration bridge, preserving interrupted service and separately reconciling route, queue, useful service, and interruption intervals
Oracle: In paired Macro and routed nonzero-transit Micro actual-Flow cases, hold a capacity-1 blocker until tick 5, begin a service whose frozen fixture seed samples 20 ticks at tick 5, and admit a 2-tick urgent request at tick 8 that suspends the original work until tick 10. Assert the original WorkId/RequestId, sampled service duration/key/draw position, exact remaining service progress, one suspend/resume/completion lifecycle, and resource/queue conservation. Macro completes at tick 27 with queue 5 + useful service 20 + interruption 2; Micro's accepted 2-tick route precedes service submission and completes at tick 27 with transit 2 + queue 3 + useful service 20 + interruption 2. Assert intervals from actual Flow/transit callbacks, no duplicate arrival/request, no extra Transit RNG, no production API or scheduler/seed changes, and run Rust 1.99.0 only.

Instance: q4-rerun-6784538 — Refresh exact-pin local DES/ABM/calibration verification at Kairos 678453838f391f3dd869b79fd502077f63f3a067, including the C2.2 provider sampling tests. No source edits and no C2.4/C2 phase acceptance.
Interface: Do not change source, manifests, lockfiles, APIs, task catalog, parent pin, checkboxes, or owner decisions. Use only Rust/Cargo 1.99.0. Run the exact locked DES/ABM/calibration feature-set test, strict all-target Clippy, and workspace fmt-check at the bound Kairos commit. Preserve outputs in this instance artifact folder. A pass refreshes local current-pin verification only; C2.4 owner, hosted, portable restart, release, clinical and phase gates remain open.


## conductor/evidence/c2.4-closeout-rust199-20261007/current-readback-post-6303-20261007.json:1-71
{
  "schema_version": 1,
  "task_id": "C2.4.closeout",
  "instance_id": "current-readback-post-6303-20261007",
  "status": "coordinator_current_readback_phase_withheld",
  "parent_source_baseline": "3e2cf21bcde297f27888f02eb54bdf058e07e013",
  "current_kairos_pin": "6303a2552257f1fcbd2aaa002d38352a2a467592",
  "phase_acceptance": "withheld",
  "full_C2_acceptance_claimed": false,
  "phase_checkbox_changed": false,
  "q4_interruption_leaf": {
    "status": "locally_accepted_at_historical_tested_pin",
    "tested_pin": "6053d8dd2f8057c423decc929c90a8bcd10837f9",
    "receipt": "conductor/evidence/c2.4-q4-interruption-rust199-20261007/acceptance-rust199.json",
    "receipt_sha256": "88109c5dc3a35eada33d0f047ac33d5271f7a5050f138393ff4a435a265a97c1",
    "tested_flow_bridge_sha256": "e2c3b5eb5381b85a083f879a017d7be5cd573cdabd23b82eda486bab27e41959",
    "runtime_suite_rerun_at_6303a255": true,
    "current_pin_verification": {
      "status": "passed_local_exact_pin_checks",
      "receipt": "conductor/evidence/c2.4-current-pin-rerun-6303a255-20261007/acceptance-rust199.json",
      "receipt_sha256": "d8e11420de56c0938cc29abf49dc3a000503d4340d1231f8d02ea384b4e91ea8",
      "tests": "447 passed, 0 failed, 5 ignored across 43 result groups",
      "strict_clippy": true,
      "format": true
    }
  },
  "current_pin_delta": {
    "from": "6053d8dd2f8057c423decc929c90a8bcd10837f9",
    "to": "6303a2552257f1fcbd2aaa002d38352a2a467592",
    "summary": "Adds #[doc(hidden)] to Rust-public TransitContext::schedule_transit_retry; no executable code or test inputs changed.",
    "rust_toolchain": "1.99.0",
    "format_passed": true,
    "rustdoc_passed": true,
    "receipt": "conductor/evidence/c2.4-retry-doc-hidden-rust199-20261007/README.md",
    "receipt_sha256": "8ed2d2b604e3beaac8c773d5962571290bc5ae94e66c66cbadac17df20592e16",
    "runtime_suite_or_strict_clippy_at_new_pin": true
  },
  "manual_elapsed_time_readback": {
    "status": "manual_source_reconciliation_complete",
    "evidence": "conductor/evidence/c2.4-runtime-manual-readback-20261007/README.md",
    "evidence_sha256": "fcf95929debabdd56e078e01ccb9756fd7b996658c11b4f3ebf0f0829749163c",
    "base_macro_ticks": "5 queue + 20 service = 25",
    "base_micro_ticks": "2 transit + 3 queue + 20 service = 25",
    "q4_macro_ticks": "5 queue + 20 service + 2 interruption = 27",
    "q4_micro_ticks": "2 transit + 3 queue + 20 service + 2 interruption = 27",
    "at_manual_readback_creation_no_test_or_rust_command_run": true,
    "later_current_pin_verification": "conductor/evidence/c2.4-current-pin-rerun-6303a255-20261007/acceptance-rust199.json"
  },
  "owner_packets": {
    "Track03": {
      "path": "conductor/evidence/c2.4-track03-owner-review-rust199-20261007/review-packet.md",
      "status": "prepared_unsent",
      "owner_disposition": "open"
    },
    "Track22": {
      "path": "conductor/evidence/c2-track22-checkpoint-owner-review-20261007/review-packet.md",
      "status": "prepared_unsent",
      "portable_checkpoint_restart": "open"
    }
  },
  "open_gates": {
    "Track03_owner_disposition": true,
    "Track22_contract_and_portable_restart": true,
    "hosted_exact_head_checks": true,
    "release_acceptance": true,
    "clinical_acceptance": true,
    "C2_4_Conductor_phase_review": true
  },
  "prior_closeout_status": "historical_snapshot_retained",
  "current_runtime_acceptance_claim": false
}


## conductor/evidence/c2.5-decision-mismatch-rust199-20261007/acceptance-rust199.json:1-28
{
  "schema_version": 1,
  "task": "C2.5",
  "status": "local_leaf_and_parent_join_accepted_phase_gates_open",
  "source_commit": "3926d0f8f401b605ea62b15cde7b5ef1ffba89c1",
  "parent_base": "deeb7d6cf9c787895bc9c694cb83b0d114bafd37",
  "toolchain": {
    "rustc": "rustc 1.99.0 (b940084d7 2026-09-28)",
    "cargo": "cargo 1.99.0 (5f94df478 2026-08-27)"
  },
  "source_sha256": "6cb8218ff09ac51848cb449053256501e2f3c9b7ac39c0799367b5c424ec2ea4",
  "packet_sha256": "ca44e183682e74c2791625c84e3e0443b4e05f5492cada217b2e698c8bd069b0",
  "verification": {
    "focused_worker_test": "passed: 1 passed, 0 failed, 89 filtered",
    "calibration_library_suite": "passed: 88 passed, 0 failed, 2 ignored",
    "strict_clippy": "passed",
    "format": "passed"
  },
  "scope": "Test-only decision-mismatch preservation assertion; no production/API change, serialization, restart or multi-work claim.",
  "open_gates": [
    "Track03 owner review",
    "Track22 portable checkpoint/restart",
    "hosted exact-head checks",
    "C2.4 phase review",
    "release",
    "clinical acceptance"
  ]
}


## crates/kairo-ecs-calibration/src/flow_bridge.rs:2408-2500
    fn actual_flow_urgent_suspend_resumes_same_macro_and_routed_micro_service() {
        let macro_run = run_urgent_preemption_case(
            FidelityMode::Macro,
            PreemptionStrategy::Suspend,
            TransitIntent::Route,
        );
        let micro_run = run_urgent_preemption_case(
            FidelityMode::Micro,
            PreemptionStrategy::Suspend,
            TransitIntent::Route,
        );

        assert_eq!(macro_run.completion, SimTime::from_ticks(27));
        assert_eq!(micro_run.completion, macro_run.completion);
        assert_eq!(macro_run.service_duration, SimDuration::from_ticks(20));
        assert_eq!(micro_run.service_duration, macro_run.service_duration);
        assert_eq!(macro_run.service_key, micro_run.service_key);
        assert_eq!(macro_run.service_seed, micro_run.service_seed);
        assert_eq!(macro_run.service_draw_position, 1);
        assert_eq!(
            micro_run.service_draw_position,
            macro_run.service_draw_position
        );
        assert_eq!(macro_run.decision.mode, FidelityMode::Macro);
        assert_eq!(micro_run.decision.mode, FidelityMode::Micro);
        assert_eq!(macro_run.transit_elapsed, SimDuration::ZERO);
        assert_eq!(micro_run.transit_elapsed, SimDuration::from_ticks(2));
        let t = SimTime::from_ticks;
        let d = SimDuration::from_ticks;
        assert_eq!(
            macro_run.completion.duration_since(t(0)).unwrap(),
            d(5).checked_add(d(20)).unwrap().checked_add(d(2)).unwrap()
        );
        assert_eq!(
            micro_run.completion.duration_since(t(0)).unwrap(),
            d(2).checked_add(d(3))
                .unwrap()
                .checked_add(d(20))
                .unwrap()
                .checked_add(d(2))
                .unwrap()
        );
    }

    #[test]
    fn actual_flow_urgent_restart_reuses_sampled_service_for_macro_and_zero_transit_micro() {
        let macro_run = run_urgent_preemption_case(
            FidelityMode::Macro,
            PreemptionStrategy::Restart,
            TransitIntent::Zero,
        );
        let micro_run = run_urgent_preemption_case(
            FidelityMode::Micro,
            PreemptionStrategy::Restart,
            TransitIntent::Zero,
        );

        assert_eq!(macro_run.completion, SimTime::from_ticks(30));
        assert_eq!(micro_run.completion, macro_run.completion);
        assert_eq!(macro_run.service_duration, SimDuration::from_ticks(20));
        assert_eq!(micro_run.service_duration, macro_run.service_duration);
        assert_eq!(macro_run.service_key, micro_run.service_key);
        assert_eq!(macro_run.service_seed, micro_run.service_seed);
        assert_eq!(macro_run.service_draw_position, 1);
        assert_eq!(
            micro_run.service_draw_position,
            macro_run.service_draw_position
        );
        assert_eq!(macro_run.decision.mode, FidelityMode::Macro);
        assert_eq!(micro_run.decision.mode, FidelityMode::Micro);
        assert_eq!(macro_run.transit_elapsed, SimDuration::ZERO);
        assert_eq!(micro_run.transit_elapsed, SimDuration::ZERO);
        assert!(!macro_run.has_transit_carrier);
        assert!(!micro_run.has_transit_carrier);
        assert_eq!(macro_run.terminal_progress, micro_run.terminal_progress);
        assert_eq!(macro_run.terminal_request_state, RequestState::Completed);
        assert_eq!(
            micro_run.terminal_request_state,
            macro_run.terminal_request_state
        );

        let t = SimTime::from_ticks;
        let d = SimDuration::from_ticks;
        assert_eq!(
            macro_run.completion.duration_since(t(0)).unwrap(),
            d(5).checked_add(d(3))
                .unwrap()
                .checked_add(d(2))
                .unwrap()
                .checked_add(d(20))
                .unwrap()
        );
        assert_eq!(


## conductor/evidence/c2.2-provider-sampling-rust199-20261007/README.md:1-32
# C2.2 provider sampling tests — Rust 1.99.0

This receipt records additional unit coverage for the existing private intrinsic
work-duration provider. It does not change provider behavior or public APIs.

## Scope and source

- Kairos commit: `678453838f391f3dd869b79fd502077f63f3a067`
- Parent pin commit: `7b9e5bcb4e382959f08ee80a473400548d0d6a9e`
- Parent baseline during execution: `9896dd9ddd5bef614aabbd0e1522d9cbc2e2d2b5`
- Source: `libs/kairos/crates/kairo-ecs-calibration/src/work_duration.rs`
- Source SHA-256: `4c140e2b73334f6d6a0355ec53a3fe9138985029ebfaeb86da3ada9e9e3d15ef`
- Contract: `libs/kairos/conductor/design/calibration/c20-work-provider-v1.md`
- Contract SHA-256: `51ac6dc6783c280368dfd4630e4c33c5d585bf4c9657484e5e6c4ccb5c541333`
- Rust/Cargo: `1.99.0` only (`rustc 1.99.0 (b940084d7 2026-09-28)`;
  `cargo 1.99.0 (5f94df478 2026-08-27)`)

The tests cover the canonical weighted sequence and draw positions, ordered
support and the cumulative bucket boundary, fixed `u128::MAX` with no draws,
typed constructor/lookup errors with unchanged stream continuation, and real
stream rejection accounting. Independent review found no expected-value or
continuation mismatch. The contract's rejection followed by draw-position
overflow rollback remains covered by the existing seed-map test.

## Executed checks

Working directory: `/Users/doughnut/Documents/careops-sim-session-worktrees/c2-implementation-20261006`

| Command | Result | Log |
|---|---:|---|
| `rustup run 1.99.0 cargo test --manifest-path libs/kairos/Cargo.toml -p kairo-ecs-calibration work_duration::tests --locked` | exit 0; 7 passed | [focused tests](logs/work-duration-tests.log) |
| `rustup run 1.99.0 cargo test --manifest-path libs/kairos/Cargo.toml -p kairo-ecs-calibration --locked` | exit 0; 152 passed, 5 ignored | [package tests](logs/package-tests.log) |

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
