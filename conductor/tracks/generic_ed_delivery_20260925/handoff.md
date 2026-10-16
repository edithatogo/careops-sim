# Handoff: Generic ED library delivery

Status: Phase E0–E4 accepted; Milestone G1 (Hardened native v1) qualified.
Read [the plan](plan.md), [module readiness](../../module-readiness.md) and
[development audit](../../evidence/development-audit.md) before starting.

Owners: CareOps; Kairos 03/21/22/05/09/32/34/35. Respect milestone dependencies, source pins and the existing
Kairos contract/compatibility rules. Keep source changes in their owning repo.
No later clinical-domain tracks have been introduced.

## Accepted Milestone G1 Closeout Record (2026-10-16)

- **Phase E4 Review & Closeout**: ACCEPTED per [E4.4 Phase Acceptance](../../evidence/e4.4-phase-acceptance-20261016.md) and [E4.4 Phase Evidence](../../evidence/e4.4-phase-evidence-20261016.md).
- **Milestone G1 Boundary**: Headless emergency department DES/ABM simulation engine, validation taxonomy distinguishing verification from clinical validation, sensitivity/identifiability surfaces, held-out Macro/Micro evaluation, multi-day ED benchmarks (>540k steps/sec), leak-free heap soak, executable documentation (`docs/USER_GUIDE.md`), intentional error troubleshooting, and Stage 2 delivery contract qualified.
- **Manual Verification Passed**: Fresh consumer installed `target/release/careops-ed`, validated `nominal_mvp.json` and `surge_mvp.json`, executed runs, compared metrics (+2,177.8% wait inflation), and exported CSV without private data dependency or code edits.
- **Next Entry Gate**: Phase E5 (Interactive scenario dashboard, WebSocket snapshot protocols, and spatial presentation bindings).

