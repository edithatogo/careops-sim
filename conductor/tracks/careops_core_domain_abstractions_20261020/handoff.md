# Planning handoff

20 October 2026: revised plan incorporates [CC-01–CC-08](../../design/composition-contracts-20261020.md).
All implementation tasks remain unchecked. User authorised planning incorporation;
this is not spec/runtime/clinical acceptance. Resolve current upstream owners,
exact revisions and capability receipts before binding a worker packet.

The first proof is a small ED/ward/community fixture, followed by a birthing/theatre
fixture and A7 evidence join. B0 evidence intake is independent of core delivery.
Preserve ED replay, existing source pins, jurisdiction-neutral code and versioned
profiles. Do not infer atomic grants from conditions or delivery from transfer.

Historical proposal is retained in the planning baseline commit. Current plan,
specification and test matrix supersede its unconditional capability assumptions.

## 21 October follow-up

PR173 C-R0–C-R7 and its review refinements are now mapped into this branch's
numbered plans and derived catalog; see the [integration record](../../evidence/pr173-roadmap-integration-20261021.md).
The original checkout's divergent uncommitted proposals are preserved and are not
reported as integrated. R0 must reconcile them before dispatch from that checkout.
This incorporation is local planning only; no PR merge, pin change or hardware
qualification occurred. Existing unchecked tasks remain unchecked.

## Core integration through A6 — 1 November 2026

Chat 01a19709-fd7b-7a61-a1e7-1ed85239eba0 owns the integration at
`<managed-worktrees>/core-a6-integration/careops-sim`.
The packet joins 717ee03 to remote main c8ac417 and covers A2–A6 Rust source,
tests, manifests, phase receipts and Conductor records. There is no native
object scope. Next operation is PR verification. A7.1 awaits birthing B7.4.

Integration exception: main pins Kairos 0741026. Its kairo-ecs-des does not
export FlowRuntime, RequestId, RequestState or ResourceId required by the A3.2
conformance target. The original A3 phase receipts describe the prior pin.
Keep the PR in draft pending a compatible, qualified Kairos interface and a
successful locked run including allocation_kairos_conformance.

## 2 November 2026 — A7 prerequisite recovery

Retained B0/B1 scoped local source/evidence is reconciled at `2294973` from
frozen `f568909`; original checkout remains unchanged. B2 source `c251f9b`
received reviewer fixes `d3c8fc5` (per-delivery recovery) and `9702fac` (birth-time
observation provenance). Exact final review found no remaining actionable issues.
Full core/birthing tests and Clippy pass; hosted checks are the next acceptance
gate. B2 checkboxes remain unchecked until that readback. B3–B7 and A7 remain
incomplete. [Evidence](../../evidence/core-a7-prerequisites-20261102/B2.md).

### B6 scoped acceptance; B7 next — 3 November 2026

Current chat01a19b9f-69e7-7353-9b75-445d5653feb4 owns source lane
`<managed-worktrees>/careops-pr194-checks/careops-sim`, branch
`codex/core-a7-prerequisites`, PR197. Owned paths are birthing consumer and
coordinator Conductor records; no native objects. Earlier owners/assignments
remain history. Sourcefd9ee4c is independently re-reviewed after application-time
and escort-exclusivity corrections. Scoped runtime81d1e5c passes current hosted
native Linux/macOS and completed quality/security checks. Seven transfer and46
birthing local tests pass. ED mutation graph is unchanged from fully green948500b;
reuse that receipt while current context completes, without relabeling it green.
Final combined-head full gates remain mandatory. B7 native batch IPC/Parquet,
actual composition, worker/recovery/performance and A7 remain open. B7 dependency
candidate4134/Polars964 keeps runtime074 separate; any actual graph/config change
invalidates narrow reuse. No clinical/site/full-spec approval follows.

### B7 full hosted acceptance; A7 eligible — 3 November 2026

Runtime154ba1a is accepted for the generic synthetic B7 contract after final
independent implementation/determinism review and all17hosted successes plus
expected main-only attestation skip. Actual native Linux/macOS CLI/PyArrow,
current-source ED88.1834%coverage/116canonicalmutants (111caught,2exactequivalents,
3unviable,0timeouts), full quality/security and lock gates pass. Model/build
identity now binds completed-replication recovery. B7.1–B7.4 are checked; full-spec,
clinical/site/operational acceptance remains open. Existing chat/worktree/branch
ownership is unchanged; no native-object scope. Next operation: bind reviewed A7
join/evaluation/accounting contract packet. Its private preflight corrected false
censoring and prediction-time leakage before dispatch. No A7 task is yet accepted.
