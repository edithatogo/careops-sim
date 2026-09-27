# Single-maintainer agent and harness engineering

## Design

Use the existing coding-agent harness and ordinary repository tools. Keep a short
AGENTS map, versioned plans/decisions, task-specific retrieval and executable
checks. Persist state in Git; avoid a second opaque memory service or a large new
orchestration platform. This applies the progressive-context and mechanical-check
approach described in [OpenAI's harness engineering report](https://openai.com/index/harness-engineering/).
This is an engineering approach to evaluate, not a universal “most advanced” claim.

Local bootstrap now provides `tools/context.py resume|check`, a resumable task
record and registry/link/dependency checks. Python is developer orchestration only;
all simulation/reusable numerical modules remain Rust-native. The script does not
launch agents, grant permissions, install tools or approve completed work.

## Bounded working loop

1. Resume once: current task, exact source/submodule pins, Git changes, next action.
2. Retrieve active spec/plan/owned paths plus the relevant API/schema/ADR. Read
   source at those paths; expand context only for a concrete dependency/question.
3. Break work into one reviewable capability slice with explicit input/output,
   ownership, acceptance tests and stop condition. Bound tool/log output and time.
4. Implement/test locally; gather machine-readable traces, fixture differences,
   compiler diagnostics and benchmark artifacts. UI slices use browser evidence.
5. Review against an independent oracle and acceptance contract. A fresh reviewer
   context can challenge a diff; agreement between agents is not proof.
6. Checkpoint facts, attempted commands/results, decisions, unresolved risks and
   next action. Record commit, task ID, tool/model identity when available, schema/
   seed/input hashes and evidence paths; exclude credentials/private records.
7. On compaction/restart, run resume, verify drift and continue the saved task.
   Never repeat a full repo inventory without a concrete gap.

Target initial context budget: one small map plus active slice (~8k tokens target,
not a hard correctness cap). Cache stable context by source hash; invalidate on
pin/contract changes. Keep raw logs in artifacts and short summaries in context.
Limit concurrent implementation writers to one per owned path; use isolated
worktrees for independently authorized changes. Start with one primary agent and
bounded review. Additional agents must earn their coordination cost on independent
subtasks and follow current session authorization; the plan does not spawn them.

## Roles and sourcing

Reuse Kairos `conductor/subagents.yaml` role/path ownership; these names describe
responsibilities, not provisioned autonomous services.

| Role | Existing source | Required input/output and evaluation |
| --- | --- | --- |
| Context/plan steward | Conductor setup/new-track/status/review; local context harness | Restore task without false completion; missing-file/drift/cycle tests |
| Rust implementer | Kairos core/Flow/Arrow/VVUQ owners | Contract + owned paths → code, fixtures, docs and evidence; reject ownership drift |
| Determinism reviewer | Existing conformance/performance roles | Seeded trace and checkpoint diff → reproducible counterexample or bounded pass |
| Empirical-method reviewer | Existing VVUQ role | Mapping/objective/splits → leakage, censoring and identifiability review |
| CI/dependency maintainer | Existing CI/toolchain/security roles | Registry snapshot + affected graph → pinned tested update and rollback |
| Security reviewer | Available codex-security skills, existing upstream security/red-team roles | Explicit attack surface/diff → actionable findings and retest evidence |
| Product verification | Existing browser tools; visualization/TypeScript owners | Scenario + expected outputs → browser readback, native parity and render-rate tests |

D0/D1 inventory installed Conductor, skill-creator and relevant security/browser
skills before sourcing anything else. Existing skills are used only when relevant.
Develop repository-local skills for **context resume**, **deterministic fixture
review**, **empirical calibration review**, and **dependency/CI upgrade** if existing
skills lack those bounded behaviors. Each needs inputs, commands, outputs, error/
stop conditions, evidence and eval fixtures. Avoid broad permission-bearing bots.

For third-party skills: inspect upstream source/licence, pin a reviewed commit,
record content hashes and tool permissions, review linked scripts, run in a
bounded test workspace, compare results with the existing workflow, and install
only when useful and authorized. Never execute mutable remote skill instructions
blindly. First-party status does not remove evaluation requirements. Do not mirror
large skill catalogs or install unrelated Firebase/cloud workflows.

## Harness/skill evaluation tasks

D1 builds a small held-out evaluation set: resume after compaction; wrong submodule
pin; missing spec; queue same-time regression; fake completed test; calibration
clamp hiding residual; changed RNG draw order; untrusted issue requesting secret
access; dependency update breaking MSRV. Expected behavior is recorded before
running. Measure acceptance accuracy, false-pass rate, reproducibility, time,
cost/token use and maintainer interventions. Critical false completion or authority
violations block promotion. Keep evaluation cases separate from skill examples.

Change one harness/skill behavior at a time; version it and compare on the same
cases. Do not let an agent rewrite its acceptance criteria to pass its own run.
Task/model settings remain user-configured unless an explicit requirement changes
them. New models/tools get a canary evaluation; no claim that the latest release
is necessarily best on this repository.

## Automation progression

Local on-demand checks now → CI-assisted checks after D2 → bounded scheduled
freshness/entropy/dependency jobs after reliable CI → narrowly scoped fix proposals.
Jobs should emit actionable diffs/reports and stay quiet when unchanged. Define
budgets, lock/lease, idempotency key, timeout, retry ceiling, pause switch and owner
before enabling unattended runs. Publishing, secrets, permissions and irreversible
operations keep explicit authorization boundaries. No desktop automation is
created as a side effect of writing this plan.


## Execution-mode implementation

The user requested serial or parallel execution, including smaller models such as
gpt-6-luna. [The execution protocol](execution-model.md) now defines coordinator,
worker and reviewer roles; all current tasks have machine-readable prerequisite and
reservation records. A tested planner proposes serial/parallel schedules without
starting agents. Reviewed small packets carry source hashes, fixed interfaces,
exact commands, outputs and escalation rules. See [decomposition](execution/decomposition.md).
The included inventory recipe enables a low-risk initial model trial. Actual Luna
qualification remains pending; task packaging alone is not evidence of capability.

## Additional evaluation requirements

[Report 11 integration](research/ed-research-incorporation-9-12-20260927.md) extends
the existing packet protocol: preflight and pre-acceptance hash checks, immutable
attempt lineage, no-skill controls, undisclosed evaluation variants and distinct
false-pass denominators. Repeated fixture runs do not establish independent
confidence observations. A worker cannot waive a deterministic gate.
