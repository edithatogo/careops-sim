# CareOps Sim — Conductor

## Project context

- [Accepted Kairos parent pin transition](evidence/e2.2-parent-pin-transition-20261012/README.md)

- [Joined Flow V4 / Transit V2 consumer acceptance](evidence/e2.2-v4-v2-joined-consumers-completion-20261012/README.md)

- [V4 portable Handoff acceptance](evidence/e2.2-v4-portable-completion-20261012/README.md)

- [V4 coherent Acquire acceptance](evidence/e2.2-v4-coherent-completion-20261012/README.md)

- [V4 preview acceptance](evidence/e2.2-v4-preview-completion-20261012/README.md)

- [Parallel V4 / Transit V2 consumer dispatch](evidence/e2.2-v4-v2-consumer-dispatch-20261012/README.md)

- [Flow V4 routing acceptance](evidence/e2.2-flow-v4-routing-completion-20261012/README.md)

- [Transit V2 direct wire acceptance](evidence/e2.2-transit-v2-direct-wire-completion-20261012/README.md)

- [Parallel routing and ABM rebound dispatch](evidence/e2.2-routing-abm-rebound-dispatch-20261012/README.md)

- [Context schema prerequisite acceptance and parallel successors](evidence/e2.2-context-schema-versions-completion-20261012/README.md)

- [Context schema prerequisite dispatch and ABM dependency stop](evidence/e2.2-context-schema-prerequisite-dispatch-20261012/README.md)

- [Flow V4 direct wire acceptance and routing successor](evidence/e2.2-flow-v4-direct-wire-completion-20261012/README.md)

- [Parallel direct wire dispatch](evidence/e2.2-direct-wire-dispatch-20261012/README.md)

- [Transit V2 native acceptance and parallel wire successor](evidence/e2.2-transit-v2-native-completion-20261012/README.md)

- [Flow V4 native acceptance and immediate wire successor](evidence/e2.2-flow-v4-native-completion-20261012/README.md)

- [Serial/parallel and smaller-model execution](execution-model.md)
- [Worker packet decomposition](execution/decomposition.md)
- [Resume state](current-state.json) — `python3 tools/context.py resume`
- [Module readiness and delivery gates](module-readiness.md)
- [Dependency/version policy](dependency-policy.md)
- [Agent and harness engineering](agent-engineering.md)
- [Procedural learning loop](harness/learning-loop.md)
- [Internal owner review and learning decision](decisions/ADR-0009-internal-owner-review-and-learning.md)
- [CI, security and GitHub timing](ci-security-release.md)
- [Product](product.md)
- [Product guidelines](product-guidelines.md)
- [Technology and compatibility](tech-stack.md)
- [Workflow](workflow.md)
- [Track registry](tracks.md)
- [Product expansion roadmap](roadmap.md)
- [Generic ED public evidence shortlist](generic-ed-evidence.md)
- [Kairos enhancement programme and dependency map](kairos-enhancements.md)

## Active delivery programme

| Track | Specification | Phased implementation plan | Status |
| --- | --- | --- | --- |
| ED parameters and example inputs | [Specification](tracks/ed_parameter_evidence_20260927/spec.md) | [Plan](tracks/ed_parameter_evidence_20260927/plan.md) | Research intake complete; delivery pending |
| Development readiness | [Specification](tracks/development_readiness_20260925/spec.md) | [Plan](tracks/development_readiness_20260925/plan.md) | Local audit/bootstrap implemented; remaining work planned |
| Generic ED delivery | [Specification](tracks/generic_ed_delivery_20260925/spec.md) | [Plan](tracks/generic_ed_delivery_20260925/plan.md) | Research intake complete; delivery pending |
| DES queues and preemption | [Specification](tracks/des_queue_preemption_20260925/spec.md) | [Plan](tracks/des_queue_preemption_20260925/plan.md) | Q0–Q5 experimental development accepted; release and ED MVP separate |
| Empirical calibration and validation | [Specification](tracks/empirical_calibration_20260925/spec.md) | [Plan](tracks/empirical_calibration_20260925/plan.md) | In progress; C2/C3/C5 synthetic implementation accepted; ED/C6 and release gates pending |

These tracks cover parent ED delivery and changes owned by existing Kairos tracks.
They do not declare new upstream IDs or change upstream completion records.
The local audit/context bootstrap is implemented; simulation implementation and
full development/release acceptance remain outstanding. Earlier product discovery
remains available in
[the product discovery document](../docs/product-draft.md).

Latest research: [reports 26–30 integration](research/ed-research-incorporation-26-30-20260927.md); reports 23–25 are verified duplicates.

Current status and execution order: [research handoff](research/research-handoff.md).

Later capability: [spatial capture, shared floor plan and live state sync](spatial-visualization.md).

Additional design review: [CAD, operational capacity and replay](research/cad-replay-appraisal.md).

## Authoritative delivery sequence

[Functional MVP → hardened native v1 → extensions](delivery-contract.md) defines the
required features and acceptance recipes. E2 is the functional headless MVP; E4
is native v1. Visual/spatial import, live UI and advanced backends remain post-v1.

## Complete MVP worker decomposition

The [Luna MVP workpack](execution/mvp/README.md) covers all 82 MVP parent tasks with 258
bounded leaves and explicit joins. Run `python3 tools/mvp.py check`; bind reviewed
context/commands just before dispatch. No autonomous execution or model
qualification is claimed.

E1.1 fixture specifications accepted on 11 October 2026: [evidence](evidence/e1.1-completion-20261011/README.md).
E1.2.arrivals accepted: [native evidence](evidence/e1.2-arrivals-completion-20261011/README.md).
Triage and diagnostics accepted: [latest evidence](evidence/e1.2-diagnostics-completion-20261011/README.md).
E1.2 accepted: [combined evidence](evidence/e1.2-completion-20261011/README.md).
E1.3 offers and boarding accepted: [boarding evidence](evidence/e1.3-boarding-completion-20261011/README.md).
Exits including optional abandonment accepted: [joined evidence](evidence/e1.3-exits-completion-20261011/README.md).
Parallel horizon fixture/composition qualification and E1.4 manual audit accepted: [E1 phase closeout](evidence/e1.4-closeout-20261011/README.md).
E2.1 staff/calendar specifications accepted: [evidence](evidence/e2.1-staff-completion-20261011/acceptance.json).
Interruption/handover specifications accepted: [evidence](evidence/e2.1-interrupt-completion-20261011/acceptance.json).
Space/cleaning/reservation specifications accepted: [joined E2.1 evidence](evidence/e2.1-space-completion-20261011/acceptance.json).
All88 E2.1 specifications are accepted; E2.2 native binding/runtime and MVP remain open.
The user has renewed autonomous continuation across task and phase boundaries; MVP and subsequent delivery gates remain open.

E2.2 pure dispatch policy accepted: [native evidence](evidence/e2.2-dispatch-policy-completion-20261011/acceptance.json). Engine/checkpoint/consumer join and E2 native runtime binding remain open.

E2 engine grant-gate runtime qualified: [evidence](evidence/e2.2-engine-grant-gate-runtime-20261011/acceptance.json). Checkpoint V2, consumers and parent native binding remain open.

Native V2 checkpoint acceptance: [local Rust 1.99 evidence](evidence/e2.2-checkpoint-native-completion-20261011/README.md). Wire and consumer qualification remain open.

Version-aware preview acceptance: [Rust 1.99 local evidence](evidence/e2.2-checkpoint-preview-completion-20261011/README.md). Wire and coherent adapter remain active.

Native coherent-cut adapter acceptance: [Rust 1.99 evidence](evidence/e2.2-checkpoint-coherent-completion-20261011/README.md). Portable V2 routing remains open.

V2 wire support accepted: [Rust 1.99 evidence](evidence/e2.2-checkpoint-wire-completion-20261011/README.md). Combined consumer qualification and portable routing remain open.

Combined checkpoint consumers accepted: [Rust 1.99 and legacy envelope evidence](evidence/e2.2-checkpoint-consumers-completion-20261011/README.md). Portable V2 routing is next.

Portable V2 checkpoint acceptance: [local Rust 1.99 evidence](evidence/e2.2-checkpoint-portable-completion-20261011/README.md). Deferred allocation and V3 persistence remain open before parent integration.

Explicit V3 wire/native continuation accepted: [Rust 1.99 evidence](evidence/e2.2-wire-v3-completion-20261012/README.md). Routing, actual V3 consumers, parent pin and ED binding remain open.

V3 routing contract/handoffs accepted and bounded source packet authorized: [dispatch record](evidence/e2.2-routing-v3-dispatch-20261012/README.md). Source and final clean-head qualification remain pending.

Common V3 routing accepted after separately reviewed lint repair: [Rust1.99 evidence](evidence/e2.2-routing-v3-completion-20261012/README.md). Three consumer lanes, parentpin and ED binding remain open.

V3 consumers accepted at the joined Kairos head: [E2.2 V3 consumer evidence](evidence/e2.2-v3-consumer-completion-20261012/README.md). Public Flow preview review precedes the parent pin and later ED adapter work.

Public Flow preview source dispatched after reviewed API design: [bounded packet](evidence/e2.2-public-preview-dispatch-20261012/README.md). Runtime qualification and ED integration remain pending.

Public Flow preview source qualification accepted at the reviewed Kairos head: [evidence archive](evidence/e2.2-public-flow-preview-completion-20261012/README.md). The parent pin remains unchanged pending queue-order/ABM contracts and native work, versioned persistence, and joined consumer qualification.

Native DES stable queue ordering and ABM arrival handoff accepted at the joined Kairos head: [Rust 1.99 evidence](evidence/e2.2-native-order-handoff-completion-20261012/README.md). Flow V4 and Transit V2 owner contracts are committed; native persistence packets are in review. Persistence, joined consumers, parent pin and ED integration remain open.

Native Flow V4 and Transit V2 persistence workers dispatched in parallel: [reviewed packets](evidence/e2.2-native-persistence-dispatch-20261012/README.md). Native source qualification is pending; each wire stage follows its own accepted predecessor.
