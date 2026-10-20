# Implementation Plan: Generic birthing suite simulation delivery

## B0 — Perinatal Evidence & Parameter Calibration Intake

- [ ] Task: Compile AIHW and Queensland Perinatal parameter sets (parity, induction, labor duration distributions, intervention rates)
- [ ] Task: Create structured JSON/YAML profile packs for baseline Australian Public Birthing Unit
- [ ] Task: Validate evidence distributions and unit tests for parameter loaders
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## B1 — Birthing Suite Episode Lifecycle & Labor Progression

- [ ] Task: Implement `LaborStage` state machine (Antenatal Triage, Latent/Induction, Active First Stage, Second Stage Delivery, Third Stage Recovery)
- [ ] Task: Model stochastic competing risks (progression, failure to progress, augmentation, fetal distress)
- [ ] Task: Unit test single-mother clinical trajectories under nominal and prolonged labor
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## B2 — Neonatal Spawning & Dual-Patient Tracking

- [ ] Task: Implement delivery event trigger dynamically instantiating `NeonateEntity`
- [ ] Task: Link mother and infant via `careops-core::LinkedEntityPair` and model shared vs divergent care paths
- [ ] Task: Test dual-entity telemetry recording, Apgar evaluation, and cot resuscitation allocation
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## B3 — Birthing Space & Equipment Allocation

- [ ] Task: Configure capability-based birthing rooms (LDR rooms, pool suites, continuous CTG stations)
- [ ] Task: Model room reservation, continuous labor occupancy, cleaning turnover, and boarding hold
- [ ] Task: Test space contention, surge demand, and capacity exhaustion behavior
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## B4 — Midwifery & Obstetric Staffing Dynamics

- [ ] Task: Implement 1:1 active labor midwifery care allocation and shift changeover logic
- [ ] Task: Implement on-call medical team response (Obstetric, Anaesthetic, Paediatric) with call-in distributions
- [ ] Task: Verify staffing ratio invariants and break/relief coverage
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## B5 — Acute Escalation & Preemption

- [ ] Task: Implement Category 1 Emergency Caesarean escalation protocol with 30-minute target deadline
- [ ] Task: Preempt lower-priority staff/equipment allocations using Kairos preemption primitives
- [ ] Task: Test time-to-theatre metrics and breached threshold accounting
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## B6 — Decoupled Egress Handover Routing

- [ ] Task: Connect Birthing Unit to `PostnatalWardSink`, `OperatingTheatreSink`, `SpecialCareNurserySink`, and `RetrievalSink`
- [ ] Task: Implement transfer delay, clinical handover communication duration, and downstream boarding holds
- [ ] Task: Test split egress (e.g. mother to Postnatal, baby to SCN) and verify patient conservation
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## B7 — Scenario CLI, Arrow Telemetry & Replay Verification

- [ ] Task: Implement `careops-birthing` CLI runner supporting nominal and surge scenario configurations
- [ ] Task: Export simulation runs to Arrow IPC and Parquet formats with complete episode telemetry
- [ ] Task: Verify deterministic replay, benchmark throughput, and record closeout evidence
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)
