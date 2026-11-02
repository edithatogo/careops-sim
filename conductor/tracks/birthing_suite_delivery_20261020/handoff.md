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
