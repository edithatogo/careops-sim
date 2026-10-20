# Product Guidelines & Engineering Standards

## 1. Audience & Purpose

- **Target Users**: Health service executives, clinical directors, hospital operations leads, healthcare planners, and operational researchers comparing capacity, workforce, and demand-flow scenarios.
- **Decision Support, Not Clinical Directives**: Present CareOps Sim strictly as an operational scenario-planning and systems-analysis platform. Outputs must never be framed as clinical diagnostic advice, individual patient outcome predictions, or automated staffing directives.
- **Universal Applicability**: Keep all simulation models usable across multiple jurisdictions (Queensland, other Australian states, New Zealand, the UK, Canada, and international health systems). Use Cairns/CHHHS as a concrete reference profile pack, not an embedded default in the engine code.

---

## 2. Universal Abstraction & Code Hygiene Rules

- **Zero Hardcoded Regional Identifiers in Rust Crates**:
  - Domain crates (`careops-core`, `careops-ed`, `careops-birthing`, `careops-surgery`, `careops-wards`, `careops-clinics`, `careops-hospital`, `careops-network`) must **NEVER** contain hardcoded names of specific hospitals, cities, health districts, or regional ambulance providers in struct names, enum variants, or business logic.
  - *Incorrect*: `enum EmergencyProvider { Qas, Rfds }`, `struct ChhhsNetwork`, `fn route_to_cairns_ed()`.
  - *Correct*: `struct EmergencyMedicalServiceProfile`, `struct RegionalHealthNetwork`, `fn route_to_facility(id: FacilityId)`.
- **Jurisdictional & Site Profiling via Versioned Packs**:
  - All local triage conventions (ATS, MTS, CTAS, ESI), urgency guidelines (CPC, ESPI, RTT), funding parameters (NHRA NEP/NWAU, NHS PbR), facility inventories, and transit distances belong exclusively in structured configuration files under `profiles/jurisdictions/` and `profiles/sites/`.
- **Clean Architectural Layering**:
  - Keep low-level simulation engine mechanics in [Kairos](../libs/kairos).
  - Keep domain-neutral healthcare primitives in `careops-core`.
  - Keep clinical unit logic in modular crates (`careops-ed`, `careops-birthing`, etc.).
  - Keep whole-campus coordination in `careops-hospital` and regional network routing in `careops-network`.

---

## 3. Results, Reproducibility & Uncertainty

- **Provenance First**: All simulation runs must record their scenario inputs, random seeds, software revisions, hardware backend, and data version in Arrow IPC and Parquet metadata.
- **Evidence Tier Taxonomy**: Clearly distinguish synthetic test cases, public statistical distributions, user-supplied empirical data, and locally validated site observations. Source and transformation provenance must be preserved across the calibration pipeline.
- **Uncertainty & Sensitivity**: Always report confidence intervals and variance metrics with comparative scenario results. Never claim empirical calibration or operational fitness until the corresponding Conductor verification gates have formally passed.
- **Standardized Operational Metrics**: Report standardized healthcare operational indicators:
  - Patient flow: waiting times, length of stay (LOS), time-to-decision, boarding duration.
  - Resource utilization: room occupancy, midwife/nurse-to-patient ratios, surgical session utilization.
  - Access performance: triage target compliance, emergency surgery decision-to-incision intervals, and elective waitlist breach rates.

---

## 4. Privacy, Security & Data Governance

- **Zero Private Patient Data**: All checked-in test fixtures, CI benchmarks, and default examples must utilize synthetic data or appropriately aggregated public statistics (e.g. AIHW, Queensland Perinatal reports). Never commit identifiable patient data, confidential clinical notes, or restricted institutional records to Git.
- **Site CAD & Capacity Governance**: Treat detailed hospital CAD drawings, physical security blueprints, and private institutional staffing data as governed external inputs with recorded access boundaries and licensing provenance.

## Composition qualification

Apply [CC-01–CC-08](design/composition-contracts-20261020.md). Report uncertainty
when estimable and explain deterministic cases or insufficient evidence rather
than manufacture confidence intervals. Policy profiles must identify source,
version and applicability. No classification alone permits active interruption.
