# Specification: Generic birthing suite simulation delivery

## Summary

Deliver a standalone, modular simulation crate (`careops-birthing`) representing a universal hospital birthing unit, adaptable to international and multi-state jurisdictions, with CHHHS and Australian perinatal datasets serving as the initial reference profile pack. The crate captures maternal labor progression, linked neonate generation upon delivery, capability-based birthing spaces (LDR suites, water immersion, CTG monitoring), midwifery and medical staffing patterns, and acute escalation to Category 1 emergency caesarean with strict decision-to-delivery time boundaries. Consistent with modular design, destination departments (Operating Theatres, Postnatal Wards, SCN/NICU, Retrieval) are interfaced via decoupled Egress Sinks, allowing the birthing suite to be simulated as an autonomous unit while providing integration points for future tracks.

## Requirements

### B-R1: Perinatal Parameter Evidence & Distribution Intake
- Incorporate empirical distributions from AIHW National Perinatal Data Collection and Queensland Perinatal reports:
  - Maternal parity split (primiparous ~42%, multiparous ~58%).
  - Presentation modes: spontaneous labor, scheduled induction of labor (IOL), and elective planned caesareans.
  - Stage durations: Latent phase, First stage active labor (median 6–12h primip, 3–6h multip), Second stage delivery (median 1–2h primip, 0.5–1h multip), Third stage placenta delivery & recovery (0.5–2h).
  - Complication & escalation probabilities: failure to progress, fetal distress, postpartum hemorrhage (PPH), emergency CS rates (Cat 1: crash within 30 min, Cat 2: within 75 min).

### B-R2: Labor Progression State Machine
- Model the physiological progression of labor:
  - Antenatal assessment / triage check.
  - Latent labor / Induction monitoring.
  - Active First Stage labor.
  - Second Stage active pushing and delivery.
  - Third Stage placental management & immediate postpartum recovery.
- Competing stochastic hazards: spontaneous progression vs required intervention (augmentation, epidural, operative delivery).

### B-R3: Linked Maternal-Neonate Dynamic Entities
- At delivery event (Second Stage completion), instantiate a new `Neonate` entity linked to the `Maternal` entity via `careops-core::LinkedEntityPair`.
- Capture neonate condition (Apgar 1/5 min, resuscitation cot requirement, skin-to-skin vs immediate stabilization).
- Telemetry maintains correlated clinical episode records for mother and infant.

### B-R4: Capability-Based Birthing Spaces
- Model rooms based on capability profiles:
  - Standard LDR (Labor, Delivery, Recovery) suite.
  - Water immersion / Birthing pool suite.
  - High-dependency / continuous CTG monitoring capability.
  - Resuscitaire / neonatal stabilization station.
- Account for room turnover, terminal cleaning, and bed-blocking when egress sinks are unavailable.

### B-R5: Midwifery and Medical Staffing Models
- Dedicated 1-to-1 midwifery care during active labor (Stage 2 and active Stage 1).
- Midwifery shift changeover with continuous patient handoff.
- On-call escalation: obstetric registrar/consultant call-in for instrumental birth or complications; anaesthetic call-in for epidural or theatre transfer; paediatric team call-in for high-risk delivery.

### B-R6: Acute Escalation & Preemption
- Category 1 Emergency Caesarean:
  - Immediate preemption of non-critical staff allocations.
  - Strict 30-minute decision-to-delivery target.
  - Rapid dispatch to Theatre Egress Sink.

### B-R7: Decoupled Egress Sinks
- Transmit completed or transferred cases to abstract egress sinks without embedding recipient unit logic:
  - `PostnatalWardSink`: Standard uncomplicated mother+baby discharge to ward (with boarding delay if ward is full).
  - `OperatingTheatreSink`: Urgent or elective caesarean transfer.
  - `SpecialCareNurserySink`: Neonatal admission for premature/distressed infants.
  - `RetrievalSink`: Maternal/neonatal aeromedical transfer (relevant for regional/rural hubs like Cairns).

### B-R8: Execution, CLI, and Arrow Telemetry
- Standalone CLI `careops-birthing` supporting batch runs, parameter sweeps, and scenario files.
- Export detailed simulation events via Arrow IPC and Parquet, recording wait times, room occupancy, midwife-to-patient ratios, and decision-to-delivery intervals.

## Out of Scope

- Detailed internal surgical workflows inside the operating theatre (owned by the future Surgery track).
- Detailed postnatal inpatient stay days 2–4 (owned by future Inpatient/Ward tracks).
- 3D spatial pathfinding and lift dynamics (handled via transfer delay distributions; full spatial layout deferred to post-v1 spatial visualization).
