# Ownership contract

CareOps owns domain input/evidence/profile files under proposed `model-inputs/ed/`.
P0 freezes exact paths. P1 writes `des/` and P2 writes `abm/`; both read the same
accepted schema. The coordinator owns shared catalogue IDs/schema, merged
profiles, dependence model and statistical decisions. Workers never rewrite
sources or invent missing values to satisfy completeness checks.

Follow [serial/parallel execution](../../execution-model.md). Assign one parameter
family/source to a bounded worker packet: exact source/revision, fields/units,
allowed output, extraction oracle, uncertainties and escalation. Smaller models
can extract and validate data; applicability, identifiability and distribution
selection require reviewed decisions. Independent source readback verifies entries.

Kairos 03/21/22 review model/sampling/calibration interfaces and 04 owns reusable
IO. Blocked: scheduler/RNG changes, private data import, clinical prescriptions,
new later-domain tracks and unsourced numeric defaults. Research can proceed
before D1/GitHub readiness; runtime changes still follow existing development gates.
