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
| Development readiness | [Specification](tracks/development_readiness_20260925/spec.md) | [Plan](tracks/development_readiness_20260925/plan.md) | Local audit/bootstrap implemented; remaining work planned |
| Generic ED delivery | [Specification](tracks/generic_ed_delivery_20260925/spec.md) | [Plan](tracks/generic_ed_delivery_20260925/plan.md) | Proposed |
| DES queues and preemption | [Specification](tracks/des_queue_preemption_20260925/spec.md) | [Plan](tracks/des_queue_preemption_20260925/plan.md) | Proposed; ready for review |
| Empirical calibration and validation | [Specification](tracks/empirical_calibration_20260925/spec.md) | [Plan](tracks/empirical_calibration_20260925/plan.md) | Proposed; ready for review |

These tracks cover parent ED delivery and changes owned by existing Kairos tracks.
They do not declare new upstream IDs or change upstream completion records.
The local audit/context bootstrap is implemented; simulation implementation and
full development/release acceptance remain outstanding. Earlier product discovery
remains available in
[the product discovery document](../docs/product-draft.md).
