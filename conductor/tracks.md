# Track registry

- [ ] **DES queues and preemption** — `des_queue_preemption_20260925`
  ([index](tracks/des_queue_preemption_20260925/index.md),
  [spec](tracks/des_queue_preemption_20260925/spec.md),
  [plan](tracks/des_queue_preemption_20260925/plan.md)). Proposed; upstream owner 03.
- [ ] **Empirical calibration and validation** — `empirical_calibration_20260925`
  ([index](tracks/empirical_calibration_20260925/index.md),
  [spec](tracks/empirical_calibration_20260925/spec.md),
  [plan](tracks/empirical_calibration_20260925/plan.md)). Proposed; upstream owner 21,
  with 03/04/22 integration.

Calibration schema/ingestion/metrics can proceed independently after contract
review. Shadow replay of interrupted work depends on the queue/Flow integration
milestone Q4. See [milestones and ownership](kairos-enhancements.md).

- [ ] **Development readiness and hardened delivery** — `development_readiness_20260925`
  ([index](tracks/development_readiness_20260925/index.md),
  [spec](tracks/development_readiness_20260925/spec.md),
  [plan](tracks/development_readiness_20260925/plan.md)). In progress: D0.1 audit/
  context bootstrap only. Remaining toolchain, skill, CI and release work planned.
- [ ] **Generic ED library delivery** — `generic_ed_delivery_20260925`
  ([index](tracks/generic_ed_delivery_20260925/index.md),
  [spec](tracks/generic_ed_delivery_20260925/spec.md),
  [plan](tracks/generic_ed_delivery_20260925/plan.md)). Proposed; ED-native, dashboard
  and existing-backend acceptance phases. No later clinical-domain tracks.

[Readiness and capability coverage](module-readiness.md) defines completion levels.
Metadata carries explicit local phase dependencies and cross-track milestone
edges; `python3 tools/context.py check` checks targets/cycles. Start D0/D1 before
formal Q0/C0 contracts; E0 unlocks D2 GitHub/CI, then Q1/C1 can proceed.
