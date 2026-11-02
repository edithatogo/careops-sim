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

## 2 November 2026 — A7 prerequisite recovery

Retained B0/B1 scoped local source/evidence is reconciled at `2294973` from
frozen `f568909`; original checkout remains unchanged. B2 source `c251f9b`
received reviewer fixes `d3c8fc5` (per-delivery recovery) and `9702fac` (birth-time
observation provenance). Exact final review found no remaining actionable issues.
Full core/birthing tests and Clippy pass; hosted checks are the next acceptance
gate. B2 checkboxes remain unchecked until that readback. B3–B7 and A7 remain
incomplete. [Evidence](../../evidence/core-a7-prerequisites-20261102/B2.md).

### B2 acceptance and B3 candidate

B2 runtime is accepted at frozen hosted `871c478`; B2 source review passed at
`9702fac`. Test-only `b01c885` controls have13local passes/review and retain the
final combined-head hosted gate. B3 candidate `6292b3b` has5spaces/25corecapacity
and18fullbirthing tests passing plus no-findings logic review. Reviewer-requested
clock/in-memory-history documentation is applied. B3.1–B3.4 remain unchecked until
its hosted milestone; B4–B7/A7 remain open. Earlier pending statements above are
historical stages.

### 3 November — B3 scoped acceptance and B4 dispatch

B3 runtime `e3bc8ba` is scoped-accepted using its current native Linux/macOS and
other completed checks, independent review, and frozen871c478's unchanged ED
mutation receipt. Current context/aggregate remains pending; final exact combined
head gates remain mandatory. B4.1 staffing/bundle cases are next. Reassess evidence
reuse if shared/core/dependency/schema/fixture/config/ED-path inputs change.

### B3 full hosted result; reviewed B4 candidate

B3 e3bc8ba completed its full hosted run41457106192:16 successes and the expected
main-only attestation skip. B4 implementation84c9edf received fixes12477e2 and
ebf680e after independent review. Exact final re-review found no remaining issues;
26 birthing tests and all-target Clippy pass. B4 boxes remain unchecked pending
its scoped hosted milestone. Responsibility audit must run after coordinator
events; dynamic early termination, queued atomic grants and durability remain
excluded. B5–B7/A7 are still open.

### B4 scoped acceptance; B5 next

Static runtime2c13ec0 is scoped-accepted after current native/quality checks and
exact-source re-review at ebf680e. Full26birthing tests pass. ED verification
graph is unchanged from fully green e3bc8ba; its mutation receipt is reused while
current context/aggregate runs. Final actual combined-head gates remain required.
Next B5.1: urgency reassessment/queue displacement/non-interruption cases, then
bounded theatre acceptance/start/birth/completion callbacks and separate metrics.

### B4 fully hosted; B5 independently reviewed candidate

B4 exact2c13ec0 full hosted run41461436139 passed. B5 source1252b3e received
resource-pairing corrections4197d3d/8ba15d5. Final logic and deterministic-fixture
reviews are ready_for_review at8ba15d5;13 theatre tests independently passed.
Full local birthing milestone:39 passed. B5 boxes remain unchecked until its
scoped hosted milestone. Next B6 covers lifecycle routing/split destinations/
fallback and conservation; B7/A7 remain open. No durability, parallel or clinical
acceptance is inferred from B5.

### B5 scoped acceptance; B6 next

B5 runtime948500b is scoped-accepted after current native/quality checks and
independent logic/determinism review at8ba15d5. Thirteen theatre tests passed
independently; full local birthing milestone39 passed. ED verification graph
is unchanged from fully green2c13ec0; that mutation receipt is reused while
current context/aggregate completes. Final actual combined-head gates remain.
Next B6.1–B6.3 lifecycle/split-destination/fallback/escort/person-episode-resource
conservation, using qualified core transfer contracts and explicit ownership.

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
