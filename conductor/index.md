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
- [Product guidelines](product-guidelines.md)
- [Technology and compatibility](tech-stack.md)
- [Workflow](workflow.md)
- [Track registry](tracks.md)
- [Product expansion roadmap](roadmap.md)
- [Composition contracts and fixture gates](design/composition-contracts-20261020.md)
- [Reconciled proposals and integration](design/inter-agent-coordination-20261020.md)
- [Related repository interfaces](design/related-repository-boundaries-20261020.md)
- [Universal triage & priority framework](design/universal-triage-priority-framework.md)
- [Transport, retrieval & boundary flows](design/transport-retrieval-interfacility-flow.md)
- [Generic ED public evidence shortlist](generic-ed-evidence.md)
- [Kairos enhancement programme and dependency map](kairos-enhancements.md)

## Active delivery programme

| Track | Specification | Phased implementation plan | Status |
| --- | --- | --- | --- |
| Repository reconciliation & Kairos upgrade | [Specification](tracks/repo_reconciliation_kairos_upgrade_20261020/spec.md) | [Plan](tracks/repo_reconciliation_kairos_upgrade_20261020/plan.md) | In progress; R0/R1 accepted on development branch, R2 candidate-owner gate next |
| Multi-domain core abstractions & boundaries | [Specification](tracks/careops_core_domain_abstractions_20261020/spec.md) | [Plan](tracks/careops_core_domain_abstractions_20261020/plan.md) | Planned; R3 baseline gate before implementation |
| Generic birthing suite delivery | [Specification](tracks/birthing_suite_delivery_20261020/spec.md) | [Plan](tracks/birthing_suite_delivery_20261020/plan.md) | Planned; B0 evidence intake independent, B1 after A5 |
| ED parameters and example inputs | [Specification](tracks/ed_parameter_evidence_20260927/spec.md) | [Plan](tracks/ed_parameter_evidence_20260927/plan.md) | Historical plan tasks checked; capability scope still subject to R0 |
| Development readiness | [Specification](tracks/development_readiness_20260925/spec.md) | [Plan](tracks/development_readiness_20260925/plan.md) | Historical plan tasks checked; dated evidence retained |
| Generic ED delivery | [Specification](tracks/generic_ed_delivery_20260925/spec.md) | [Plan](tracks/generic_ed_delivery_20260925/plan.md) | Historical plan tasks checked; capability scope still subject to R0 |
| DES queues and preemption | [Specification](tracks/des_queue_preemption_20260925/spec.md) | [Plan](tracks/des_queue_preemption_20260925/plan.md) | Q0–Q5 experimental development accepted; release and ED MVP separate |
| Empirical calibration and validation | [Specification](tracks/empirical_calibration_20260925/spec.md) | [Plan](tracks/empirical_calibration_20260925/plan.md) | Historical plan tasks checked; dated evidence retained |

These tracks cover parent ED delivery and changes owned by existing Kairos tracks.
They do not declare new upstream IDs or change upstream completion records.
The original five plans contain 145 checked tasks. Their source/runtime scope is
reconciled in R0; new composition plans do not certify broader acceptance. Earlier product discovery
remains available in
[the product discovery document](../docs/product-draft.md).

Latest research: [reports 26–30 integration](research/ed-research-incorporation-26-30-20260927.md); reports 23–25 are verified duplicates.

Current dispatch: [resume state](current-state.json) and the unchecked task DAG.
The [research handoff](research/research-handoff.md) is historical intake context,
not a competing dispatch queue. [R0.1 reconciliation](evidence/r0.1-reconciliation-20261021/README.md) records source and evidence boundaries.

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

## Planned Polars migration

[CareOps migration](tracks/polars_data_processing_20261022/index.md) follows accepted Kairos Track 70. Planning registered; implementation not started.

## Separate library adoption tracks

[Library adoption programme](library-adoption.md) registers rand, rand_distr, ECharts, petgraph and Three.js separately, after accepted Polars delivery in both repositories. All implementation remains pending.

## Robustness-first dependency decisions

[Separate dependency tracks and sequencing](robust-dependencies.md) prioritise existing-system robustness. [Value gates](dependency-value-policy.md) prevent adoption for its own sake. SUMO conditional; Rapier deferred.

- [Dependency sequencing and prerequisite gates](dependency-sequencing.md)

[Validation/tooling planning review](validation-tooling-review.md)

## Python runtime migration

[Python 3.15 migration and qualification](tracks/python315_runtime_qualification_20261023/index.md) records the Python 3.15.1 requirement, retained qualification and pending main/runtime handover gates.
