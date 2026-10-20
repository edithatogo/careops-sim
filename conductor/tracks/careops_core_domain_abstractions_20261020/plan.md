# Implementation Plan: Multi-domain core simulation abstractions and egress boundaries

## A0 — Domain Separation & Boundary Contract

- [ ] Task: Create `careops-core` workspace crate and establish dependency hierarchy with `libs/kairos`
- [ ] Task: Define clinical domain abstraction boundaries distinguishing core runtime from domain policies
- [ ] Task: Implement schema and serde loaders for layered JurisdictionProfile and SiteProfile packs
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## A1 — Dynamic Clinical Pathway & Stage Engine

- [ ] Task: Implement `StageDefinition`, `StageId`, and directed `PathwayGraph` in `careops-core`
- [ ] Task: Implement `PriorityRecord` and universal triage mapping across ED (ATS 1–5), Acute Theatres (Cat 1–4), ESWL (Cat 1–3), and Outpatients (CPC Cat 1–3)
- [ ] Task: Add validation for pathway reachability, cycle bounds, and terminal state transitions
- [ ] Task: Unit test dynamic pathway execution against synthetic multi-stage trajectories
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## A2 — Capability-Based Spaces & Equipment Registry

- [ ] Task: Implement `SpaceCapability`, `EquipmentCapability`, and `SpaceProfile` in `careops-core`
- [ ] Task: Implement capability matching engine (`request_capabilities(set)` against available inventory)
- [ ] Task: Add unit tests for space allocation, capability pooling, and contention resolution
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## A3 — Multi-Role Procedural Staffing Engine

- [ ] Task: Implement `ProcedureSpec`, `TeamRole`, and composite staffing requirements
- [ ] Task: Integrate Kairos `ConditionWait::AllOf` and `ConditionWait::AnyOf` for atomic team co-allocation
- [ ] Task: Verify deadlock freedom and preemption priority in multi-role procedural allocations
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## A4 — Decoupled Egress Sinks & Handover Boundaries

- [ ] Task: Implement `EgressSink` trait, `HandoverPayload`, and `EgressEvent` in `careops-core`
- [ ] Task: Implement `TransportModality` (Road, Air Rescue, RFDS, Taxi, Private) and `CrewStream` (EMT, ACP, CCP, Medical Retrieval Team)
- [ ] Task: Implement standard sink behaviors: immediate release, delayed transit, and backpressure/boarding holds
- [ ] Task: Provide mock sinks for Operating Theatres, Inpatient/Postnatal Wards, and External/Boundary Transfers (Inter-district, Private Hospital, GP, UCC, RACF, Interstate, International)
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## A5 — Linked Entity Lifecycle Abstractions

- [ ] Task: Implement `LinkedEntityPair`, `EntitySpawningRelationship`, and `ParentChildBinding`
- [ ] Task: Implement split/join lifecycle logic (independent vs synchronized progression and discharge)
- [ ] Task: Unit test maternal-neonate dynamic spawning and dual-entity telemetry recording
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## A6 — `careops-ed` Refactoring & Deterministic Replay Parity

- [ ] Task: Refactor `careops-ed` to build on `careops-core` dynamic pathways and space capabilities
- [ ] Task: Execute full ED regression tests (`cargo test -p careops-ed`)
- [ ] Task: Verify 100% byte-identical replay parity on reference ED fixtures via `deterministic-fixture-review`
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)
