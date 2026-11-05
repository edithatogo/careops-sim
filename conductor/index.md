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
| Repository reconciliation & Kairos upgrade | [Specification](tracks/repo_reconciliation_kairos_upgrade_20261020/spec.md) | [Plan](tracks/repo_reconciliation_kairos_upgrade_20261020/plan.md) | Complete: R0–R3 accepted; pin 44261eb retained, with missing E6/E8 hardware capabilities kept as external dispatch gates |
| Multi-domain core abstractions & boundaries | [Specification](tracks/careops_core_domain_abstractions_20261020/spec.md) | [Plan](tracks/careops_core_domain_abstractions_20261020/plan.md) | A0–A7 bounded engineering complete at544a1e2; PR197 unmerged; broader acceptance remains separate |
| Generic birthing suite delivery | [Specification](tracks/birthing_suite_delivery_20261020/spec.md) | [Plan](tracks/birthing_suite_delivery_20261020/plan.md) | B0–B7 bounded engineering accepted; full specification and clinical/site/operational gates open |
| ED parameters and example inputs | [Specification](tracks/ed_parameter_evidence_20260927/spec.md) | [Plan](tracks/ed_parameter_evidence_20260927/plan.md) | Historical P0–P5 generic catalogue/model coverage accepted; Cairns-specific values and empirical hospital validity remain separate |
| Development readiness | [Specification](tracks/development_readiness_20260925/spec.md) | [Plan](tracks/development_readiness_20260925/plan.md) | Historical plan tasks checked; dated evidence retained |
| Generic ED delivery | [Specification](tracks/generic_ed_delivery_20260925/spec.md) | [Plan](tracks/generic_ed_delivery_20260925/plan.md) | Historical E0–E8 checks retained; E9 CPU/CLI capability repair accepted at 18b6e8f; G3 modes unqualified |
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

## Completed Polars migration

[Archived CareOps migration](archive/polars_data_processing_20261022/index.md)
records accepted R10–R14 engineering. Source258bb6b passed both787-test native
hosts,815+127 Python,26 frozen benchmarks, full117 mutation accounting, coverage,
policy and the native Linux package. [Final source receipt](evidence/polars-r14-qualification-20261104/README.md)
preserves original failures, actual source/merge distinctions, rollback and the
net+3645 maintenance cost (qualified source+3615; archive fixture+6; CI helper/tests+24). All20 task IDs and three downstream prerequisites
remain; runtime074/data473/Polars964 and ownership are unchanged. The archive
preserves dated source acceptance. PR199 passed final gates and merged into the
existing Generic ED integration target; [post-archive receipt](evidence/polars-r14-qualification-20261104/post-archive-final-gates.json)
records exact source/tested/final merge distinctions. PR197/198/main and release/clinical gates remain separate.

## Separate library adoption tracks

[Library adoption programme](library-adoption.md) registers rand, rand_distr, ECharts, petgraph and Three.js separately, after accepted Polars delivery in both repositories. All implementation remains pending.

[SHA-2 R50.1 inventory](evidence/sha2-r501-inventory-20261104/README.md) verifies
its historical upstream prerequisite and current consumers. [R50.2 qualification](evidence/sha2-r502-qualification-20261104/README.md)
accepts exact0.11.1/default features, bounded value and build-identity compatibility
as a replacement candidate. R50/R51 bounded engineering is accepted; [implementation and guard](evidence/sha2-r51-implementation-20261104/README.md)
are source-bound. R52 bounded engineering is complete at source1d559a6, including Linux `/dev/full`, full hosted/integration/quality/security/cost gates. [Archived track](archive/sha2_integrity_20261022/index.md) and [final source receipt](evidence/sha2-r52-acceptance-20261104/README.md) preserve all9tasks and distinguish this later documentation closeout.

## Robustness-first dependency decisions

[Separate dependency tracks and sequencing](robust-dependencies.md) prioritise existing-system robustness. [Value gates](dependency-value-policy.md) prevent adoption for its own sake. SUMO conditional; Rapier deferred.

- [Dependency sequencing and prerequisite gates](dependency-sequencing.md)

[Validation/tooling planning review](validation-tooling-review.md)

## Python runtime migration

[Python 3.15 migration and qualification](tracks/python315_runtime_qualification_20261023/index.md) records accepted runtime implementation at main `43ea690` (PRs #178/#179), final Python 3.15.1 handover and eight guard controls. Conductor documentation review is tracked separately in PR #180.

## Tempfile prerequisite inventory

[Tempfile R53.1](evidence/tempfile-r531-inventory-20261105/README.md) verifies historical Track94 and inventories current retained/migrated consumers. That inventory did not qualify current-version/runtime or phase acceptance. R53.2 candidate evidence follows below; ownership/pins and Polars durability gates remain.

[Tempfile R53.2 qualification](evidence/tempfile-r532-qualification-20261105/README.md) records exact current dependency, private compatibility/value/security and model-build/lock effects. R53 and bounded R54 are locally accepted; [R54 phase proof](evidence/tempfile-r54-implementation-20261105/phase-review.json) records actual controls and strict immutable successor. R53–R55 bounded engineering is complete at qualified sourcee86c992. [R55 acceptance](evidence/tempfile-r55-acceptance-20261105/acceptance.json) binds all configured hosted/native/quality/security/package/cost gates. Later Conductor closeout is documentation only; PR201 remains draft/frozen with no merge/release/pin promotion.

## thiserror candidate qualification

[Accepted R56 candidate](evidence/thiserror-r562-qualification-20261105/qualification.json) qualifies eight nontrivial error enum conversions with exact2.0.21/default std and preserved source(None), messages/variants/FFI. [Track](tracks/thiserror_contracts_20261022/index.md) proceeds to R57 actual-source tests and closed successor guard; R58 full acceptance remains. No parent adoption or pin/merge/release change follows from qualification.
