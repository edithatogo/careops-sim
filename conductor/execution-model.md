# Serial, parallel and smaller-model execution

The four tracks share one task graph and one acceptance standard. Choose
`serial` with one worker, or `parallel` with a bounded worker count. No separate
parallel implementation plan, weakened test suite or reordered simulation events
is introduced. Development-agent parallelism is distinct from simulated PDES.

## Operating roles

- **Coordinator:** prepares bounded packets, freezes interfaces, assigns work,
  manages reservations and integrates commits. It owns the registry, task status,
  shared manifests/lockfiles, submodule pins and phase closeouts.
- **Worker:** executes one approved packet. `gpt-6-luna` is the intended initial
  smaller-model candidate for bounded code/test/docs tasks. Use existing configured
  settings or explicit session choices; the planner does not change model settings.
- **Reviewer/integrator:** verifies scope, evidence and acceptance against the
  integrated commit. Review may be another agent or a fresh context of the same
  single agent in serial mode. Independence of evidence matters more than agent
  count. Contract/statistical/security decisions need an appropriately capable
  reviewer, rather than an assumed model capability.

Architecture and ambiguous modeling decisions are coordinator work. A smaller
model may implement a settled contract but must not invent scheduler semantics,
change RNG algorithms, relax metrics/tolerances or waive security/release gates.
The user can run the entire workflow serially; no task requires multiple agents.

## Task catalog and source of truth

`conductor/execution/tasks.json` indexes all 119 existing plan tasks. Each entry
contains its exact objective, source plan/phase, prerequisite tasks, conservative
write reservations, required context and phase acceptance text. Plan checkboxes
remain the accepted task-status source; a worker report does not check them off.

Refresh the catalog with `python3 tools/tasks.py build` after editing a plan or
phase metadata. Review the generated diff. `python3 tools/tasks.py check` fails if
catalog content diverges from the plans, required edges are missing, or a cycle
exists. Phase-entry dependencies resolve to the prerequisite phase's closeout
**task**, not merely its first coding task. Tasks inside a phase default to serial
order; an approved packet may split a task into independent leaves with a join.

The catalog is a planning/preparation schedule, not 119 preapproved code jobs.
Future APIs do not yet exist: turning speculative file names into runnable
commands would be misleading. Packet preparation is a mandatory executable step,
with validation before worker dispatch. Preparation can read disjoint areas in
parallel; source mutations follow the reviewed packet's narrower reservation.

## Worker packet: definition of ready

Use `conductor/execution/packet-template.json` and this checklist. The coordinator
must bind every field; a template cannot pass `tools/tasks.py packet-check`.

1. One behavior/output, normally ≤5 owned files, one repository and ≤3 meaningful
   verification commands. Target 30–90 minutes and ~4k task-context tokens; split
   larger work instead of depending on long implicit reasoning. Limits are sizing
   defaults; exceptions require a recorded reason and reviewed narrower interface.
2. Exact base commit for the **target repository**, parent Kairos pin where
   relevant, and readable context paths. Existing input/contract file SHA-256s
   bind the packet to the source examined. New output paths may be absent.
3. Fixed inputs/public signatures, expected output shape, allowed write paths,
   protected paths, dependencies, no-change constraints and ordered microsteps.
   New interfaces require a reviewed ADR/contract reference before code work.
4. Exact command argv and working directory; expected exit codes and behavioral
   oracles. Tests must detect the relevant bug, not merely compile. A TDD packet
   can explicitly expect an initial failure and a final pass; never leave the
   final acceptance at a failing test. No invented command/test targets.
5. Expected artifacts and a structured result contract. Return commit/diff,
   changed paths, command results, test counts/log paths, unresolved issues and
   status `ready_for_review` or `blocked`; a worker cannot mark itself accepted.
6. Stop conditions: missing prerequisite/API, changed input hash, out-of-scope
   edit, unclear oracle, unsupported hardware, repeated failed approach or need
   for secrets/publication. Return the smallest reproducer/question and escalate;
   do not silently broaden scope or modify the oracle.

A task with several behaviors is decomposed into packet IDs such as
`Q3.2.accounting`, `Q3.2.eviction`, `Q3.2.tokens`. Each has explicit dependencies;
`Q3.2` closes only when every leaf is integrated and the combined checks pass.
The packet template supports `leaf_dependencies`; the coordinator must verify
accepted leaf receipts before dispatch. Leaf examples are planning boundaries,
not claims that those future APIs already exist.

## Serial procedure

1. Run context check and task-catalog check.
2. `python3 tools/tasks.py ready --mode serial` returns one dependency-ready
   **preparation candidate** from unchecked tasks.
3. Bind/split/validate its worker packets. Claim the packet in the coordinator's
   run ledger, with base commit, worker and reservation before source edits.
4. Execute one leaf, review it, integrate and rerun required checks. Continue its
   remaining leaves and task-level integration check.
5. Record accepted evidence and then update its plan checkbox, refresh catalog
   and current-state. Proceed to the next ready task.

## Parallel procedure

1. Use the same checks and `ready --mode parallel --workers N`. Selection is
   deterministic and conservative: exact/ancestor path reservations conflict.
   The command is a read-only proposal, **not a lock acquisition or agent launch**.
2. The sole coordinator checks active reservations, subtracts already assigned
   work, creates isolated target-repository worktrees and writes the run ledger
   **before** launching workers. No independent coordinators may schedule writes.
3. All packet dependencies and contract versions must be accepted. Sharing a
   read-only contract is allowed; writing a contract used by another worker forces
   a barrier/rebind. Workers do not edit task metadata, shared Cargo manifests,
   lockfiles, CI permissions or parent submodule pins unless that exact edit is
   their exclusive reviewed packet.
4. Integrate in deterministic dependency order; rerun affected and combined tests
   on the integrated head. Rebase/rebind stale packets; never resolve semantic
   conflicts by blindly accepting one side. Cherry-pick success alone is not proof.
5. Close the task and release reservations only after accepted evidence. A timeout
   does not free a reservation until the old writer is confirmed stopped. Retry
   uses the same packet ID plus a new attempt ID and a verified base.

For Kairos worktrees, create them in the Kairos Git repository; a parent worktree
alone does not isolate the existing submodule working directory. Initialize the
parent's recorded submodule when using a parent worktree. Never have two agents
mutate the same `libs/kairos` checkout or update the parent pin concurrently.

## Examples of safe parallel opportunities

| After accepted milestone | Candidate work | Constraint |
| --- | --- | --- |
| D1 | Q0 queue-contract proposal, C0 data/fidelity-contract proposal, E0 ED skeleton | Separate proposal/output paths; shared type/manifest changes integrate serially |
| D2 + respective contracts | Q1 DES resource work and C1 Arrow ingestion | Different crates; dependency/lockfile changes owned by one integration packet |
| C1 | C4 pure distance-metric work alongside eligible Q tasks | Metrics stay in calibration/Arrow ownership; do not edit DES source |
| E4 | E5 dashboard and E6 native Metal work | Separate parent UI/native-backend worktrees; fixed API and seed/telemetry contracts |

C2 and Q work both touch DES/ABM and are conservatively serialized until finer
packets prove disjoint ownership. Extra agents cannot remove prerequisite gates.

## Smaller-model qualification

Start with a supervised packet such as the provided D0.2 inventory extraction.
Evaluate the same packets in serial and parallel modes and with gpt-6-luna against
fixed expected outputs. Include negative cases (stale base, missing dependency,
false pass, out-of-scope write, wrong units/RNG order) and one integration conflict.
Compare acceptance accuracy, false-pass rate, human corrections, elapsed time and
cost. Promote a task class only after it passes; contract/security/statistical
review remains separately qualified. No gpt-6-luna run has been performed by this
setup change, and no model-capability or speedup claim follows from it.

## Durable result/assignment format

The coordinator keeps a run-specific ledger outside worker-owned files, with:
packet/task ID, attempt, target repo, base SHA, input hashes, worker/model, worktree,
reserved paths, start/heartbeat, state, result path and accepted integration SHA.
Active states: prepared → running → ready_for_review → integrated, or blocked/
failed/cancelled. Only integrated evidence unlocks dependent work. This manual
single-coordinator protocol works now; autonomous concurrent claiming would need
an atomic lease-store implementation and its own failure tests before use.


## Ready-to-bind first packet

From a clean committed checkout, prepare the small inventory trial:

```sh
python3 tools/tasks.py bind conductor/execution/packets/D0.2.inventory.json
python3 tools/tasks.py packet-check .artifacts/packets/D0.2.inventory.json
```

The tracked recipe intentionally has no current base SHA: `bind` writes a local
packet with the exact target HEAD and input hashes. Rebinding requires coordinator
review, not a worker shortcut around drift. No model is launched by these commands.
Use [the worker prompt](execution/worker-prompt.md),
[result contract](execution/result-template.json) and
[phase decomposition guide](execution/decomposition.md). The output verifier is
`tools/verify_inventory.py`; this tests source extraction, not model reasoning
about architecture or actual simulation readiness.
