# Track registry

Current planning revision: R0 reconciles historical labels and runtime scope.
Original plans contain 145 checked tasks; the new R/A/B plans remain pending.
[Composition contracts](design/composition-contracts-20261020.md) govern the new work.
Registry prose below retains historical handoffs until R0 evidence reconciliation.

- [x] **DES queues and preemption** — `des_queue_preemption_20260925`
  ([index](tracks/des_queue_preemption_20260925/index.md),
  [spec](tracks/des_queue_preemption_20260925/spec.md),
  [plan](tracks/des_queue_preemption_20260925/plan.md)). Q0–Q5 accepted for experimental queue development; upstream owner 03. Independent Q5.4 replay, evidence/risk disposition, release notes and governance review passed with exact-head native and child hosted gates. Preserves C1.4/Q5.2; child PR219 stays stacked/draft and formal release/security, stable API, advanced backend, checkpoint and clinical/MVP gates remain separate. Parent local gates passed. At source preparation, the committed-pin check, hosted parent CI and merge readback await publication; this registry does not assert a completed parent merge.
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
  [plan](tracks/generic_ed_delivery_20260925/plan.md)). E0–E4 complete and accepted; Milestone G1 (Hardened native v1) qualified per [E4.4 Phase Acceptance](evidence/e4.4-phase-acceptance-20261016.md) and [E4.4 Phase Evidence](evidence/e4.4-phase-evidence-20261016.md). Headless ED DES/ABM, verification vs clinical validation taxonomy distinction, sensitivity/identifiability surfaces, held-out Macro/Micro evaluation, multi-day ED benchmarks (>540k steps/sec), leak-free heap soak, executable documentation, and fresh consumer delivery contract qualified. Phase E5 dashboard, E6 Metal, E7 PDES, and E8 distributed execution remain downstream. No later clinical-domain tracks.

[Readiness and capability coverage](module-readiness.md) defines completion levels.
Metadata carries explicit local phase dependencies and cross-track milestone
edges; `python3 tools/context.py check` checks targets/cycles. Start D0/D1 before
formal Q0/C0 contracts; E0 unlocks D2 GitHub/CI, then Q1/C1 can proceed.

- [~] **ED parameters, example inputs, ranges and distributions** — `ed_parameter_evidence_20260927`
  ([index](tracks/ed_parameter_evidence_20260927/index.md),
  [spec](tracks/ed_parameter_evidence_20260927/spec.md),
  [plan](tracks/ed_parameter_evidence_20260927/plan.md)). Proposed. P0 can start
  immediately; P1 DES and P2 ABM evidence run independently, joining at P3.
  E0/C0 consume P0, C2 consumes P3, E1 consumes P4, and C6 consumes P5.

- [ ] **Repository status reconciliation and Kairos upgrade** — `repo_reconciliation_kairos_upgrade_20261020`
  ([index](tracks/repo_reconciliation_kairos_upgrade_20261020/index.md),
  [spec](tracks/repo_reconciliation_kairos_upgrade_20261020/spec.md),
  [plan](tracks/repo_reconciliation_kairos_upgrade_20261020/plan.md)). Proposed. Reconciles capability/status evidence, audits the harness and qualifies a bounded Kairos candidate; retains the working pin if required gates fail.

- [ ] **Multi-domain core simulation abstractions and egress boundaries** — `careops_core_domain_abstractions_20261020`
  ([index](tracks/careops_core_domain_abstractions_20261020/index.md),
  [spec](tracks/careops_core_domain_abstractions_20261020/spec.md),
  [plan](tracks/careops_core_domain_abstractions_20261020/plan.md)). Proposed. Extracts dynamic stage graphs, capability-based rooms, multi-role procedural staffing, decoupled egress handover boundaries, and linked entity pairs into `careops-core`.

- [ ] **Generic birthing suite simulation delivery** — `birthing_suite_delivery_20261020`
  ([index](tracks/birthing_suite_delivery_20261020/index.md),
  [spec](tracks/birthing_suite_delivery_20261020/spec.md),
  [plan](tracks/birthing_suite_delivery_20261020/plan.md)). Proposed. Modular birthing suite simulation crate (`careops-birthing`) with labor progression, dynamic neonate spawning, LDR suites, 1:1 midwifery staffing, acute CS preemption, and decoupled outbox egress routing.

## Research intake completed

All requested research themes have responses; embedded content is available.
Remaining tasks verify sources, resolve conflicts, fill specific gaps and deliver
tested outputs. See [research handoff and next steps](research/research-handoff.md).
D3 development review is complete; D4 awaits its release entry dependencies.
No broad new research round is a prerequisite. Track checkboxes continue to represent full delivery acceptance.

## Authoritative delivery sequence

[Functional MVP → hardened native v1 → extensions](delivery-contract.md) defines the
required features and acceptance recipes. E2 is the functional headless MVP; E4
is native v1. Visual/spatial import, live UI and advanced backends remain post-v1.
