# Specification: Multi-domain core simulation abstractions and egress boundaries

## Summary

Extract domain-neutral operational healthcare modeling primitives from `careops-ed` into an architectural core (`careops-core`), providing:
1. Dynamic, data-driven clinical pathway stage graphs (replacing hardcoded ED-specific `StageKind` enums).
2. Capability-based spatial and equipment modeling (`SpaceCapability`, `EquipmentCapability`, capability pooling) where rooms declare capabilities rather than rigid clinical unit types.
3. Multi-role procedural staffing engine supporting simultaneous co-allocation via Kairos SimPy-parity `ConditionWait` (`AllOf`/`AnyOf`).
4. Decoupled unit egress sinks and handover boundaries (`EgressSink`, `HandoverPayload`, inter-unit transit distributions, and downstream backpressure/boarding hooks) enabling unit simulations (Birthing, ED, Surgery, Postnatal) to dispatch and receive patients without tight module coupling.
5. Linked entity lifecycle management (`LinkedEntityPair`, `MotherChildPair`) supporting dynamic neonate generation upon delivery and paired tracking.
6. Refactoring `careops-ed` to consume `careops-core`, enforcing byte-identical regression replay.

## Requirements

### A-R1: Dynamic Pathway Stage Engine
- Replace fixed enum `StageKind` (Triage, Assessment, Diagnostics, Treatment) with a configurable stage graph (`StageDefinition`, `PathwayGraph`, `TransitionRule`).
- Support sequential, branching, and looping stages with custom clinical task payloads.

### A-R2: Capability-Based Spatial & Resource Model
- Abstract physical rooms into spaces defined by capabilities (e.g., `CapabilitySet`: monitoring, resuscitation, negative pressure, birthing pool, operative delivery) and capacity.
- Allow clinical pathway steps to request capability requirements rather than fixed bay identifiers.

### A-R3: Procedural Staffing & Multi-Role Co-Allocation
- Formalize procedure staffing specs (`ProcedureSpec`) requiring coordinated team presence (e.g. 1 Midwife + 1 Obstetrician + 1 Neonatal Pediatrician).
- Leverage Kairos `ConditionWait::AllOf` and `ConditionWait::AnyOf` for atomic reservation, avoiding allocation deadlocks and race conditions.

### A-R4: Decoupled Unit Egress & Handover Boundaries
- Define `EgressSink` trait and `HandoverPayload`:
  - When an episode concludes or escalates to external units (e.g., Operating Theatre, Postnatal Ward, SCN, ICU, Transfer), the patient entity is dispatched to a registered egress sink.
  - Sinks model transfer delay, handover communication duration, and capacity acceptance/rejection (boarding backpressure).
  - Enables Birthing Suite simulation to run independently while maintaining clean integration hooks for future surgical and ward modules.

### A-R5: Linked Entity Lifecycles
- Support creation of dependent/linked entities during a simulation run (e.g. delivery event generates `Neonate` entity linked to `Maternal` entity).
- Support joint or independent progression through pathways, joint discharge, or separate escalation (e.g., neonate to SCN while mother recovers in LDR/postnatal).

### A-R6: Backward Compatibility & Replay Parity for ED
- Refactor `careops-ed` to import and implement `careops-core` traits.
- Run deterministic regression test suite; assert zero deviation in arrival timings, queue progressions, and telemetry outputs.

### A-R7: Universal Triage & Priority Schema
- Define a unified priority contract (`PriorityRecord`, `DomainPriorityCategory`) mapping across ED (ATS 1–5), Acute Theatres (Cat 1–4), Elective Surgery (Cat 1–3), Specialist Outpatients (CPC Cat 1–3), and Retrieval Urgency (Priority 1–3).
- Implement target window accounting, breach hazard scoring, and preemption tier authority.

### A-R8: Multi-Modal Transport & Retrieval Stream Abstractions
- Define `TransportModality` (Road Ambulance, Patient Transport Van, Rotary Rescue Helicopter, Fixed-Wing RFDS, Taxi/Rideshare, Private Vehicle).
- Define `CrewStream` (Patient Transport Officer / EMT, Advanced Care Paramedic, Critical Care Paramedic, Medical Retrieval Team) with escort constraints.

### A-R9: External Boundary & Community Interface Definitions
- Define interfaces for district inflows/outflows: internal inter-facility escalation/repatriation, neighboring HHS referrals, interstate quaternary transfers, international/tourist presentations, and community/private sector interfaces (Private Hospital transfers, GP referrals/admissions, Medicare UCC diversion/escalation, and RACF emergency transfers/discharge boarding blocks).

### A-R10: Layered Profile Pack Architecture
- Implement schema and deserialization for `JurisdictionProfile` (`profiles/jurisdictions/`) and `SiteProfile` (`profiles/sites/`).
- Enforce strict abstraction: all facility topologies, triage categories, staffing levels, and transit corridors load from profile data; core Rust code remains 100% domain-neutral and free of hardcoded regional identifiers.

## Out of Scope

- Detailed interior CAD/geometrical pathfinding (reserved for post-v1 spatial visualization).
- Implementation of the full operating theatre suite or postnatal ward internal pathways (only the egress sink interface is implemented here).
