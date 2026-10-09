# C2.4.evidence

Model: gpt-6-luna; output: Phase evidence reconciliation
Oracle: Every parent task has integrated output and executed gates; unresolved failures prevent acceptance

Instance: current-native-review-20261009 — Current-pin C2 manual interval and internal owner review; record withheld phase gates without changing oracles
Interface: ADR-0009 internal review; existing C2 acceptance specification and unmodified plan exit


## conductor/tracks/empirical_calibration_20260925/plan.md:96-153
## C2 — Macro/Micro execution and minimal spatial behavior

Entry: C0 and reviewed Q0 API; full integration tests require Q4. Owner 03 with
21 model adapter, 01 RNG and 22 scenario configuration review.

- [x] C2.0 Freeze the provider/admission/transit-observation seams and write
  independently reviewed test-first mode and paired integration fixtures. Record
  expected red evidence for absent runtime APIs without claiming runtime acceptance;
  this preparation gate enables C2.2 implementation, not C2.1 completion.
- [x] C2.2 Implement empirical work-duration providers, FidelityPolicy and stable
  entity/task/purpose keys. Separate intrinsic work from queue/transit intervals;
  persist all mode and stream state for resume.
- [x] C2.3 Implement a deterministic route-graph TransitModel with units, stable
  shortest-path ties, integer tick conversion, unreachable errors and stored
  movement progress. Supply a minimal urgency/FIFO/zone/skill staff adapter;
  cleaning/reservation remains an explicit model resource lifecycle. Require explicit
  movement-mode speeds, versioned geometry and O/D purpose; test mixed-use pauses
  and sensor-derived distance are not silently treated as path/speed ground truth.
  Connect the calibration bridge to actual route carriers: validate actor/task/
  resource association, track exact owned event IDs and Flow callback receipts,
  reconcile only accepted arrival with the original timed work request, and allow
  one explicit retry only for the matching consumed rejected event. Macro and
  explicit-zero Micro continue to emit no transit event or Transit draw.
- [x] C2.1 Write failing mode resolution/boundary tests and common-random-number
  fixtures. Assert Macro emits no transit, zero-transit Micro matches its paired
  Macro fixture, and changing mode never discards active or suspended work.
- [x] C2.5 Implement and test the reviewed same-live-Flow in-memory
  continuation contract for fidelity policy/decisions, the owned Service stream
  and one bound bridge work item. Preserve state across a one-shot in-process
  handoff; portable serialization and process restart remain Track22 gates. The decision-mismatch rejection now has a test-only exact-value preservation oracle. Local
  same-live-Flow continuation acceptance does not close the Track03/Track22, hosted,
  portable checkpoint/restart, release, clinical or C2.4 phase gates.
- [ ] C2.4 Conductor — review and verify phase (workflow.md).

After preparation C2.0, C2.2 provider/policy, C2.3 generic routing, and C2.1
mode/paired behavior have local exact-pin joins under Rust 1.99.0. The earlier
C2.1 paired receipt is historical; its superseding paired Restart and parent
integration receipt are recorded at Kairos pin `51b5e5d`. C2.5 decision-mismatch
coverage is locally accepted at exact Kairos pin `3926d0f`; see [decision-mismatch
evidence](../../evidence/c2.5-decision-mismatch-rust199-20261007/README.md). The paired comparison covers full Flow outcomes for its deterministic
fixture; the no-Transit-draw observation is scoped to that fixture and its
TransitContext, which has no RNG field. The separate C2.3.dispatch leaf remains
the staff-policy adapter. Integrated runtime-ledger leaf accepted at `5cc8250`; see
[C2.4 runtime-ledger evidence](../../evidence/c2.4-runtime-ledger-rust199-20261007/README.md).
The test-only Q4 urgent-interruption integration is locally accepted at exact
Kairos pin `6053d8d`; see [its Rust 1.99.0 evidence](../../evidence/c2.4-q4-interruption-rust199-20261007/README.md).
Standalone synthetic accounting remains in [the manual ledger evidence](../../evidence/c2-manual-ledger-20261007/README.md).
C2.4 phase review and Track03 owner review remain open. Hosted checks, portable
checkpoint/restart, release, and clinical acceptance also remain open.
Dependency readiness does not grant parallel writes: overlapping path claims
remain serial. All C2.1 original oracles remain mandatory; a fixed-duration
fixture, disconnected RNG check, compile failure or missing transit API cannot
satisfy paired acceptance. See [the test-first join decision](../../decisions/ADR-0008-c2-test-first-join.md).

Exit: C-02 has standalone fixtures and a locally accepted integrated Q4
urgent-interruption runtime leaf. Q4.5 is accepted and integrated. Track03
owner/phase review and the remaining hosted, portable-checkpoint, release, and
clinical gates remain required before this phase can pass.


## conductor/tracks/empirical_calibration_20260925/spec.md:318-340
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



## conductor/decisions/ADR-0009-internal-owner-review-and-learning.md:1-35
# ADR-0009 — Internal owner reviews and evidence-backed procedural learning

Date: 2026-10-09. Status: coordinator decision under the user's instruction to
proceed with the recommended internal reviews and set up a self-improving system.

## Problem

C2 progress stalled after repeated audits found the same contract questions.
Track responsibilities were treated as presumed external people, although the
prepared execution packet named internal DES/ABM agent roles. Repeated audits,
wrong-worktree reads, and retries of over-budget claim inputs consumed effort
without opening the next implementation step.

## Coordination decision

The coordinator may assign qualified internal reviewers to the named engineering
roles within the user's authorized scope. Each disposition records the role,
reviewed repository/commit, evidence, decisions and remaining action. An internal
review is never represented as an external maintainer response. Genuine external
approval needs an identified authority and a concrete action requiring it; a
track number or unfilled packet alone does not establish that requirement.

Resolve architecture in bounded ADRs before dispatch. Workers implement settled
contracts in separate worktrees; the coordinator owns shared manifests, parent
pins and acceptance. Keep the Rust 1.99.0 session constraint for verification.
Track/API, hosted, portable-restart, release and clinical evidence retain their
declared scopes. This decision does not mark C2.4 or change its prerequisites.

Dispatch reviews to answer a specified decision, not to repeat a completed audit.
After a no-progress result, identify the missing decision or changed evidence
that would permit another attempt. Before reporting a blocker, verify source
state, check existing user authorization and distinguish internal coordination
from external action. Tests use behavioral oracles; planned checks are not passes.

## Learning decision

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
