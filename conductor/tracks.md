# Track registry

- [~] **DES queues and preemption** — `des_queue_preemption_20260925`
  ([index](tracks/des_queue_preemption_20260925/index.md),
  [spec](tracks/des_queue_preemption_20260925/spec.md),
  [plan](tracks/des_queue_preemption_20260925/plan.md)). In progress; upstream owner 03. Q0–Q4 and Q5.1 bounded native conformance are accepted. Q5.2 development benchmark qualification and Q5.3 compatibility/migration are accepted; Q5.4 phase closeout remains open. Accepted integrated development pin preserves C1.4; release and advanced backend holds remain. Parent integration is established separately by exact-head hosted checks and native merge readback.
- [ ] **Empirical calibration and validation** — `empirical_calibration_20260925`
  ([index](tracks/empirical_calibration_20260925/index.md),
  [spec](tracks/empirical_calibration_20260925/spec.md),
  [plan](tracks/empirical_calibration_20260925/plan.md)). Proposed; upstream owner 21,
  with 03/04/22 integration.

Calibration schema/ingestion/metrics can proceed independently after contract
review. Shadow replay of interrupted work depends on the queue/Flow integration
milestone Q4. See [milestones and ownership](kairos-enhancements.md).

- [~] **Development readiness and hardened delivery** — `development_readiness_20260925`
  ([index](tracks/development_readiness_20260925/index.md),
  [spec](tracks/development_readiness_20260925/spec.md),
  [plan](tracks/development_readiness_20260925/plan.md)). In progress: D0.1–D0.4
  audit and phase closeout complete; D1.1 is accepted. D1.2 is active: the Kairos
  owner approved its policy direction, and Rust 1.76/1.98.1/beta checks plus
  toolchain/CI updates are implemented and locally reviewed. Kairos and parent commits are integrated; D1.2 local acceptance is recorded,
  with formal phase review at D1.6. D1.3 is next; CI and release work remain.
- [ ] **Generic ED library delivery** — `generic_ed_delivery_20260925`
  ([index](tracks/generic_ed_delivery_20260925/index.md),
  [spec](tracks/generic_ed_delivery_20260925/spec.md),
  [plan](tracks/generic_ed_delivery_20260925/plan.md)). Proposed; ED-native, dashboard
  and existing-backend acceptance phases. No later clinical-domain tracks.

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

## Research intake completed

All requested research themes have responses; embedded content is available.
Remaining tasks verify sources, resolve conflicts, fill specific gaps and deliver
tested outputs. See [research handoff and next steps](research/research-handoff.md).
D1.2 is the current development-readiness task; no broad new research round is a
prerequisite. Track checkboxes continue to represent full delivery acceptance.

## Authoritative delivery sequence

[Functional MVP → hardened native v1 → extensions](delivery-contract.md) defines the
required features and acceptance recipes. E2 is the functional headless MVP; E4
is native v1. Visual/spatial import, live UI and advanced backends remain post-v1.
