# CareOps Sim — Conductor

## Project context

- [Serial/parallel and smaller-model execution](execution-model.md)
- [Worker packet decomposition](execution/decomposition.md)
- [Resume state](current-state.json) — `python3 tools/context.py resume`
- [Module readiness and delivery gates](module-readiness.md)
- [Dependency/version policy](dependency-policy.md)
- [Agent and harness engineering](agent-engineering.md)
- [CI, security and GitHub timing](ci-security-release.md)
- [Product](product.md)
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
| DES queues and preemption | [Specification](tracks/des_queue_preemption_20260925/spec.md) | [Plan](tracks/des_queue_preemption_20260925/plan.md) | Research incorporated; contract review and delivery pending |
| Empirical calibration and validation | [Specification](tracks/empirical_calibration_20260925/spec.md) | [Plan](tracks/empirical_calibration_20260925/plan.md) | Research incorporated; contract review and delivery pending |

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

The [Luna MVP workpack](execution/mvp/README.md) covers all 80 MVP parent tasks with 239
bounded leaves and explicit joins. Run `python3 tools/mvp.py check`; bind reviewed
context/commands just before dispatch. No autonomous execution or model
qualification is claimed.
