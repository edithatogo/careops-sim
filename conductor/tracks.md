# Track registry

Current reconciliation: [R0.1 evidence](evidence/r0.1-reconciliation-20261021/README.md).
The merged baseline is careops-sim `ea42fcf` and Kairos planning main `55b20e7`;
the careops-sim runtime pin remains `44261eb`. Historical plan checkboxes are
preserved and are not fresh runtime, hardware or clinical qualification.
`[x]` below records the stated historical closeout scope; `[~]` retains an active
reconciliation or capability gap. Metadata and scoped receipts explain the boundary.
The separate careops portfolio repository is outside this work.

- [x] **DES queues and preemption** — `des_queue_preemption_20260925`
  ([index](tracks/des_queue_preemption_20260925/index.md),
  [spec](tracks/des_queue_preemption_20260925/spec.md),
  [plan](tracks/des_queue_preemption_20260925/plan.md)). Q0–Q5 retain accepted experimental queue development under owner 03, per [Q5.4 receipt](evidence/q5.4-completion-20261005/README.md). Its source/child PR/parent-publication statements describe 5 October, not current dispatch. Portable checkpoints, advanced backends, release/security and clinical/MVP acceptance remain separately scoped; R0.2 inventories the pinned consumer contracts.
- [x] **Empirical calibration and validation** — `empirical_calibration_20260925`
  ([index](tracks/empirical_calibration_20260925/index.md),
  [spec](tracks/empirical_calibration_20260925/spec.md),
  [plan](tracks/empirical_calibration_20260925/plan.md)). C0–C6 complete and accepted; upstream owner 21, with 03/04/22 integration. Qualification gates C-01 through C-07 passed: Arrow IPC/Parquet ingestion invariance, Macro/Micro fidelity policy and route graph transit, shadow replay with unclamped residuals and probe isolation, deterministic Wasserstein-1/KS distance metrics, candidate grid search with worker/resume equivalence, synthetic ED fixture calibration with identifiable recovery and confounded ridge identification, free-running Macro/Micro holdout evaluation with 3-way uncertainty decomposition, 1/2/4 worker invariance, and contract handoffs to Metal/PDES/distributed/Wasm. Retained evidence: [C6.5 phase acceptance](evidence/c6.5-phase-acceptance-20261016.md). Real-world hospital clinical validation, physical GPU compute execution, distributed MPI clustering, and browser frontend UI remain separate downstream gates.

- [x] **Development readiness and hardened delivery** — `development_readiness_20260925`
  ([index](tracks/development_readiness_20260925/index.md),
  [spec](tracks/development_readiness_20260925/spec.md),
  [plan](tracks/development_readiness_20260925/plan.md)). D0–D5 complete and accepted per [D5.4 Phase Acceptance](evidence/d5.4-phase-acceptance-20261017.md) and [D5.4 Phase Evidence](evidence/d5.4-phase-evidence-20261017.md). Clean bootstrap, context harness, evaluated agent skills, GitHub/CI required check validation, quality/mutation defenses, trust boundary reviews, clean-consumer release packaging with SPDX 2.3 SBOM and SLSA v1 provenance, pin/dependency rollback rehearsals, and bounded maintenance automation (schedules, scope restrictions, quality debt tracking, performance drift surveillance, and workflow rollbacks) are fully delivered. Track closed; development handoff complete.
- [~] **Generic ED library delivery** — `generic_ed_delivery_20260925`
  ([index](tracks/generic_ed_delivery_20260925/index.md),
  [spec](tracks/generic_ed_delivery_20260925/spec.md),
  [plan](tracks/generic_ed_delivery_20260925/plan.md)). E0–E8 have historical checked tasks and dated [E4.4](evidence/e4.4-phase-acceptance-20261016.md) through [E8.4](evidence/e8.4-phase-acceptance-20261017.md) receipts. Keep the track in progress for capability reconciliation: historical E6/E8 hardware claims are disputed by the [merged audit](design/kairos-capability-reconciliation-20261020.md). R0.3 audits their exact scope; R2.3 requires actual device/multi-host receipts if consumed. Do not restart E5 from this registry or treat a 100% historical headline as operational acceptance.

[Readiness and capability coverage](module-readiness.md) defines completion levels.
Metadata carries explicit local phase dependencies and cross-track milestone
edges; `python3 tools/context.py check` checks targets/cycles. Historical D/Q/C/E
entry sequences remain in their plans; current dispatch follows the unchecked
R/A/B DAG and reviewed packets.

- [x] **ED parameters, example inputs, ranges and distributions** — `ed_parameter_evidence_20260927`
  ([index](tracks/ed_parameter_evidence_20260927/index.md),
  [spec](tracks/ed_parameter_evidence_20260927/spec.md),
  [plan](tracks/ed_parameter_evidence_20260927/plan.md)). P0–P5 retain historical acceptance at the generic catalogue/model coverage boundary per [P5.4](evidence/p5.4-phase-acceptance-20261015.md). This does not establish empirical hospital validity or Cairns-specific values. The former P0-first proposal is superseded; use the R0.2 inventory for current source qualification.

- [x] **Repository status reconciliation and Kairos upgrade** — `repo_reconciliation_kairos_upgrade_20261020`
  ([index](tracks/repo_reconciliation_kairos_upgrade_20261020/index.md),
  [spec](tracks/repo_reconciliation_kairos_upgrade_20261020/spec.md),
  [plan](tracks/repo_reconciliation_kairos_upgrade_20261020/plan.md)). Completed: R0 reconciliation, R1 local CPU/IO/recovery baseline, R2 candidate qualification, and R3 deterministic conformance and retain-pin decision accepted. Working pin 44261eb retained with zero drift; missing hardware capabilities E6/E8 classified as external dispatch gates.

- [x] **Multi-domain core simulation abstractions and egress boundaries** — `careops_core_domain_abstractions_20261020`
  ([index](tracks/careops_core_domain_abstractions_20261020/index.md),
  [spec](tracks/careops_core_domain_abstractions_20261020/spec.md),
  [plan](tracks/careops_core_domain_abstractions_20261020/plan.md)). Acceptance history: A0–A4.4 are accepted locally; the integrated A4 transfer and ward/community contracts passed workspace checks and reviewer-agent review per [A4 phase review](evidence/a4.4-phase-review-20261027/README.md). A5.1–A5.4 are accepted locally; the A5 phase passed independent reviewer-agent review and full workspace/Conductor checks per [A5.4 phase review](evidence/a5.4-phase-review-20261027/README.md). A6.1 is accepted at `e547b62` after independent reviewer-agent re-review; exact nominal fixture output parity and separate schema-version documentation are recorded in [A6.1 evidence](evidence/a6.1-ed-compatibility-20261027/README.md). A6.2 is accepted at `24f55b1` after independent reviewer-agent review; the synthetic ED→ward→community fixture and bounded limits are recorded in [A6.2 evidence](evidence/a6.2-ed-ward-community-20261027/README.md). A6.3 is accepted at `deba8a2` after reviewer-agent review and correction of the shared-scheduler wording; see [A6.3 evidence](evidence/a6.3-composition-scheduler-parity-20261027/README.md). A6.4 passed integrated A6 review, final workspace gates and reviewer-agent re-review at source `f2e09a3`; see [A6 phase review](evidence/a6.4-phase-review-20261027/README.md). A7.1–A7.4 are accepted at `ef613be` after exact-source independent review,766 Python tests and all17hosted successes plus expected main-only attestation skip; see [A7 acceptance](evidence/a7-two-consumer-20261103/README.md). Core A0–A7 is complete for bounded generic synthetic engineering; PR197 remains unmerged and broader acceptance gates stay separate.

- [~] **Generic birthing suite simulation delivery** — `birthing_suite_delivery_20261020`
  ([index](tracks/birthing_suite_delivery_20261020/index.md),
  [spec](tracks/birthing_suite_delivery_20261020/spec.md),
  [plan](tracks/birthing_suite_delivery_20261020/plan.md)). B0–B7 generic synthetic engineering prerequisites are accepted at `154ba1a` after final independent logic/determinism review, actual Linux/macOS CLI/PyArrow checks and all17hosted successes plus expected main-only attestation skip; see [B7 acceptance](evidence/core-a7-prerequisites-20261102/B7.md). All32engineering tasks are checked. Staffing/timing remain profile-owned and safe active-phase retention is preserved. Track remains in progress because full specification approval is false and clinical/site/operational acceptance is open. PR197 is unmerged.

## Research intake completed

All requested research themes have responses; embedded content is available.
Historical intake and P5 generic coverage are accepted within their dated scope.
New source/profile gaps remain separately qualified work. See [research handoff and next steps](research/research-handoff.md).
D0–D5 retain their dated development closeout; no D4 restart is inferred.
No broad new research round is a prerequisite. Track checkboxes retain only their explicitly stated acceptance scope.

## Authoritative delivery sequence

[Functional MVP → hardened native v1 → extensions](delivery-contract.md) defines the
required features and acceptance recipes. E2 is the functional headless MVP; E4
is native v1. Visual/spatial import, live UI and advanced backends remain post-v1.

---

---
- [ ] **Track: rand deterministic RNG adoption** — Proposed; implementation not started. [Track](tracks/rand_rng_adoption_20261022/index.md).

---
- [ ] **Track: rand_distr statistical sampling adoption** — Proposed; implementation not started. [Track](tracks/rand_distr_sampling_20261022/index.md).

---
- [ ] **Track: Apache ECharts analytical dashboard adoption** — Proposed; implementation not started. [Track](tracks/echarts_dashboard_20261022/index.md).

---
- [ ] **Track: petgraph spatial topology and routing adoption** — Proposed; implementation not started. [Track](tracks/petgraph_spatial_routing_20261022/index.md).

---
- [ ] **Track: Three.js unified spatial viewer adoption** — Proposed; implementation not started. [Track](tracks/threejs_spatial_viewer_20261022/index.md).

---
- [ ] **Track: Valhalla road-route qualification** — Proposed, conditional value gate unresolved. [Track](tracks/valhalla_transport_20261022/index.md).

---
- [ ] **Track: MapLibre regional transport viewer** — Proposed, conditional value gate unresolved. [Track](tracks/maplibre_regional_viewer_20261022/index.md).

---
- [ ] **Track: Conditional SUMO traffic co-simulation** — Proposed, conditional value gate unresolved. [Track](tracks/sumo_traffic_qualification_20261022/index.md).

---
- [~] **Track: CareOps sha2** — R50.1 historical prerequisite and current consumer inventory accepted; R50.2 exact current dependency/value qualification next. No runtime adoption.
  - Link: [sha2_integrity_20261022](tracks/sha2_integrity_20261022/index.md)

---
- [ ] **Track: CareOps tempfile**
  - Link: [tempfile_publication_20261022](tracks/tempfile_publication_20261022/index.md)

---
- [ ] **Track: CareOps thiserror**
  - Link: [thiserror_contracts_20261022](tracks/thiserror_contracts_20261022/index.md)

---
- [ ] **Track: CareOps proptest**
  - Link: [proptest_invariants_20261022](tracks/proptest_invariants_20261022/index.md)

---
- [ ] **Track: CareOps jsonschema new-dependency-qualification**
  - Link: [jsonschema_validation_20261022](tracks/jsonschema_validation_20261022/index.md)

---
- [ ] **Track: CareOps ajv new-dependency-qualification**
  - Link: [ajv_dashboard_validation_20261022](tracks/ajv_dashboard_validation_20261022/index.md)

---
- [ ] **Track: CareOps criterion existing-tooling**
  - Link: [criterion_benchmark_consolidation_20261022](tracks/criterion_benchmark_consolidation_20261022/index.md)

## Inventory-first Polars sequencing — 23 October 2026

Polars inventory leads; sha2/tempfile/proptest are separate tracks with gates only on demonstrated dependent migration tasks. See [sequencing](dependency-sequencing.md). SHA2 R50.1 inventory is accepted; other implementation/acceptance statuses remain unchanged. No runtime adoption follows from the inventory.

## Python runtime migration — 23 October 2026

- [x] **Track: Python 3.15 migration and qualification** — [Accepted parent runtime](tracks/python315_runtime_qualification_20261023/index.md); main `43ea690`, protected PRs #178/#179, hosted and repo-local Python 3.15.1 handover verified.

## Current Generic ED capability repair — 3 November 2026

Generic ED remains in progress. E9.1–E9.4 are accepted at 18b6e8f after independent review, both diagnostic fixes,
766 Python tests and15 hosted successes plus expected skip. The repair adds truthful
executable mode reporting and fail-closed backend selection, with current CPU
API/batch/recovery/export checks. PR198 is draft/unmerged, stacked on PR197.
The current07410266 runtime contains native wgpu/TCP mechanisms, but ED does not
consume them; device provenance/readback accounting, protocol identity and failed
collective/frontier handling remain exact upstream integration blockers.
See [current E9 evidence](evidence/ed-capability-qualification-20261103/README.md).
Historical E0–E8 task marks are preserved and cannot certify current G3 modes.
