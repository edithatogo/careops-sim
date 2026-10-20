# CareOps Sim product context

CareOps Sim supports operational scenario comparison across clinical units,
hospitals and regional health networks, using Rust-native Kairos mechanics.
Generic/public models precede separately qualified jurisdiction/site profiles;
Cairns/CHHHS is a reference profile, not an engine default.

## Model scope

- Units: ED, birthing, surgery/PACU, inpatient/ICU/postnatal/subacute care, outpatient/procedural services and waiting lists.
- Hospitals: bed management, shared diagnostics, workforce calendars, transport, equipment and service availability.
- Networks: escalation/repatriation, retrieval, public/private hospitals, primary care/private specialists, urgent care, aged/disability care and community follow-up.
- Later consumers: mental health, rehabilitation, palliative care and hospital-at-home.
- Policy evaluation: explicit activity, cost, funding, outcome and equity measures; qualified offline NHRA exchange before coupled strategic models.

## Current increment

Reconcile original completed checklists against exact-scope evidence, qualify
Kairos candidates without forced pin upgrades, then extract small shared
healthcare contracts through ED/ward/community and birthing/theatre fixtures.
The [roadmap](roadmap.md) defines sequencing; [CC-01–CC-08](design/composition-contracts-20261020.md)
define identity, transfer ownership, allocation, progression, capacity, policy
clocks, reproducibility and observation/evaluation requirements.

Keep urgency, complexity, target accounting and safe interruption distinct.
Source local classification systems, crew permissions, numerical thresholds and
parameter distributions through versioned packs. Do not infer clinical suitability
or causal effects from administrative categories or synthetic conformance.

## Architecture and acceptance

Kairos owns neutral engine primitives; careops-core owns common healthcare
semantics; domain crates own clinical workflows; profiles own local rules/data.
Preserve one clock/state authority, stable random streams and ED compatibility.
Reuse the existing calibration and experiment contracts. No Python/Julia runtime
is introduced into the Rust operational engine by optional policy adapters.

Original 145 plan tasks are checked complete; no fresh runtime or clinical
requalification follows from that count. New R/A/B plans are pending.
Hardware acceleration, distributed execution, publication and clinical deployment
retain separate acceptance. See [delivery contract](delivery-contract.md),
[readiness map](module-readiness.md) and [related repositories](design/related-repository-boundaries-20261020.md).
