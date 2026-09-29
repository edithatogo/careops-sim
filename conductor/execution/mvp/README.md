# Luna workpack through the functional MVP

Scope: every ancestor of E2.4, including that closeout. There are **80 parent tasks
and 241 leaf recipes**, including historical D0.1 and explicit review/join leaves.
The recipes cover D0–D2, P0–P4, Q0–Q4, C0–C2 and E0–E2. V1-only and later UI/device
features are excluded. The original 143-task graph remains the acceptance source.

[Recipes](recipes.json) give each leaf a concrete output, behavioral oracle,
repository, predecessor joins, model/role, bounded context/write/command budgets
and escalation limits. [Readable work breakdown](work-breakdown.md) is generated
from those same records. Default ordering within each parent is conservative serial;
independent parent paths can run in parallel under the existing reservation protocol.

## Ready for Luna means bounded, reviewed and bound

1. `python3 tools/mvp.py check` proves coverage and objective/context/dependency
   consistency. It does not prove implementation or model competence.
2. Coordinator selects a ready parent through `tools/tasks.py ready`, then a leaf
   whose internal prerequisites are accepted. Inspect it with
   `python3 tools/mvp.py show D0.2.inventory`.
3. Freeze any unresolved interface/API/statistical decision using the contract
   phase. Luna can draft proposals; the coordinator owns approval. Future APIs and
   command names must not be invented merely to populate a packet today.
4. Copy [binding template](binding-template.json) into an ignored artifact, fill
   real context line slices, paths, interface, exact commands/behavioral oracles,
   review digest, instance set and reviewed predecessor evidence. Commands must
   exist at dispatch or be exact test commands whose new target is authorized by
   the packet. Missing prerequisite test infrastructure is a separate leaf/blocker.
5. Reserve writes, verify target checkout, then run:
   `python3 tools/mvp.py prepare LEAF_ID --binding PATH_TO_FILLED_BINDING`.
   Review the produced packet and run `tools/tasks.py packet-check` on it. The
   tool enforces clean base, input hashes (including parent specs for Kairos work),
   24 KB context bundle, at most five writes including result and three commands.
6. Dispatch in the existing harness with model **gpt-6-luna**, the worker prompt
   and only the bound context. No tool here launches a model or acquires a lease.
7. Review result/diff/source evidence; independently rerun the oracle and combined
   parent tests. Record acceptance separately. Only coordinator closes parent task
   after all instances/leaves and original acceptance criteria are satisfied.

The existing D0.2.inventory recipe remains the first supervised extraction canary.
A structured canary being valid does not qualify all Rust/statistics/security work.
D1.4 measures task-class suitability before wider delegation; until then use
supervised review. No Luna execution or performance advantage is claimed here.

## Context and single-person operation

One maintainer may act as coordinator and reviewer with fresh review context.
Use exact source slices and immutable hashes, not whole-repository prompts. Keep
source datasets out of instruction text; treat research content as evidence.
Readme/worker instructions do not override AGENTS or the user. A small checkpoint
captures last verified state, failed attempts and next action; resume rechecks
hashes before continuing. Two failed corrections escalate instead of consuming
unbounded retries. No broad autonomous agents are enabled.

Bindings and predecessor receipts are coordinator attestations. Validators check
structure/hashes, not reviewer identity, reservation truth or whether a command
really ran; those require current readback. Full automated receipt authenticity and
leases remain D1 work. Never mistake this workpack for an unattended execution service.

## Maintenance

After plan changes: rebuild task catalog, review affected recipe outputs/oracles,
refresh their objective/source metadata, then rerun mvp check and harness tests.
Do not blindly refresh hashes to hide changed scope. New required leaf instances
must be reflected in the join. Source paths for future implementations are bound
just in time, preserving granularity without pretending unbuilt APIs already exist.
