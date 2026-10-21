# Cross-service composition contracts

Planning revision authorised on 20 October 2026. These are implementation
requirements and acceptance recipes, not delivered capabilities. Historical
receipts retain their exact scope and pins.

## Ownership and extraction

Kairos owns neutral scheduling, relationships, allocation mechanics, cancellation,
random streams and experiment machinery. CareOps owns healthcare identity,
clinical progression, eligibility, staffing, classification and outcome semantics.
Jurisdiction/site profiles supply sourced, versioned policies and parameters.
A configurable clinical policy remains a domain policy.

Reuse existing Kairos owners: 01/03/59/64 for entity/resource contracts,
21/22/66 for experiments and observation mathematics, 04/62 for interoperable
outputs and 25 for compatibility. Parallel readback found Tracks 67–69 registered in local Kairos commit
`0157d0cb60932b3cda86761954e1d0d9efebd55c`: linked entities (67),
boundary channels (68), competing risks (69). Reuse these pending owners with
01/03/59/64/21 review; registration is not delivered runtime. Resolve exact
source and owner status at dispatch.
Require a neutral fixture and two concrete consumers before extraction. Preserve
the ED adapter while extracting small contracts; a universal workflow framework
is not a prerequisite.

## Required contracts

| ID | Contract | Minimum semantics | CareOps phase |
| --- | --- | --- | --- |
| CC-01 | Person and episode identity | Stable synthetic person ID; distinct referral, encounter, admission and transfer IDs; typed one-to-many relationships; recycled ECS handles are not longitudinal identities | A0/A5 |
| CC-02 | Transfer lifecycle | Requested, offered, reserved, departing, in-transit, arrived, handover-complete, rejected, cancelled, expired; operation ID, ownership, resource-release points and compensation | A4 |
| CC-03 | Resource bundle | Compatible team/space/equipment; all-or-none commit or explicitly named staged approximation; deterministic contention, expiry, rollback, cancellation and starvation policy | A3 |
| CC-04 | Independent progression | Patient state evolves while waiting/boarding/travelling; progression, service completion and intervention compete under explicit event ties; stale events cancelled/versioned | A1/B1 |
| CC-05 | Effective capacity | Physical, equipped, staffed, open, reserved, occupied, cleaning and unavailable states; calendars, handovers, shared staff and call-in latency | A2/A3 |
| CC-06 | Policy and clocks | Scheme/version/jurisdiction/source/effective dates; start/stop/pause/reassessment events; urgency, priority, target and interruption eligibility distinct | A1 |
| CC-07 | Reproducible composition | Stable module/person/process random stream namespaces and draw ownership; event ordering; checkpoint and worker equivalence | A0/A6 |
| CC-08 | Observation/evaluation | Latent vs observed state; missing/delayed timestamps, censoring, exclusions, unmet demand; separate activity, cost, funding, outcome and equity ledgers | A7 |

### Transfer invariants

An accepted message is not a completed transfer. Destination reservation may
coexist with origin occupancy; count physical occupancy, reserved capacity and
clinical responsibility separately. Specify where the patient resides during
travel, when beds/escorts are released, and when responsibility transfers.
Rejection after departure needs a named fallback. Retries and late acknowledgements
must not duplicate admissions, births or discharges.

Conserve people using explicit creation/birth, entry, exit and death events.
Conserve episodes and resource claims separately: a person may have concurrent
service episodes. Readmission retains person identity with a new episode.
Multiple births require one-to-many links. Birth may follow vaginal or operative
delivery; if fetal state is modeled, define its transition to post-birth identity.

### Allocation and priority

`AllOf`/`AnyOf` join events; they do not establish atomic resource acquisition.
Atomic command admission is also not an all-or-none grant. Qualify bundle
allocation with contending requests, partial failure, timeout and cancellation.
Urgent queued work may displace other queued work without interrupting active
surgery. CareOps specifies safe interruption points, handover and resource
retention; no urgency category alone grants preemption permission.

Targets are measured outcomes, not forced success. Retain breaches. Define
immediate-service accounting for zero-duration targets without division by zero.
Wait/target ratios are not calibrated clinical hazard rates.

### Time, progression and observation

Use one virtual-time system with explicit units. Map local calendars and daylight
saving through adapters. Administrative clock pauses do not stop progression.
Hazard models declare covariates, censoring, RNG ownership and rescheduling rules;
state changes must not redraw unrelated service times. Decision agents see only
available observations, not latent future outcomes or pre-sampled event times.

Extend existing RNG/calibration contracts. Preserve ED compatibility mode and
version intentional schema/behaviour changes. Byte parity applies to unchanged
baseline scenarios, not scenarios deliberately given new clinical behaviour.

## First acceptance scenarios

| Fixture | Demonstrates | Negative/control cases |
| --- | --- | --- |
| ED → minimal ward → community | Ready-for-discharge differs from departure; downstream scarcity feeds back into boarding | Rejected/expired offer, transport cancellation, duplicate acknowledgement, delayed community place, horizon censoring |
| Birthing → bounded theatre → maternal/infant destinations | Bundle allocation, operative birth callback, linked identities and independent recovery | Multiple birth, split destinations, infant escalation, cancellation, non-interruptible surgery, target breach |

Run both in one deterministic process first. Neutral Kairos fixtures use factory
jobs needing machine/operator/tool bundles and parcels crossing two depots.
Require conservation after every event, exclusive ownership, no double booking,
same-seed replay, retry idempotency, unrelated-module noninterference,
serial/parallel replication equivalence and interruption/recovery readback.
Cross-process transport is separately qualified; LP-per-hospital is a candidate
execution mapping rather than domain architecture.

## Evaluation and later domains

Before game-theoretic optimisation, expose separate ledgers for activity,
resource use, economic costs, payments, outcomes and access/equity. Record units,
currency/price year, horizon, perspective and aggregation/discounting choices.
Monthly policy changes affect operational capacity with explicit lag, avoiding
circular feedback and double-counted operational responses. Use paired scenarios,
held-out validation, uncertainty decomposition and identifiability limits.
Aggregate public statistics need not identify micro-level hazards or causal effects.

Mental health, rehabilitation, palliative care and hospital-at-home are future
consumers for recurrent contact, long episodes and community constraints.
Equipment maintenance/sterilisation, inventory and diagnostic order/specimen/result
lifecycles are later candidates; detailed tracks require a concrete consumer.

## Sources and unresolved parameters

Read on 20 October 2026:

- [SimPy conditions](https://simpy.readthedocs.io/en/stable/topical_guides/events.html)
  and [resource requests](https://simpy.readthedocs.io/en/stable/topical_guides/resources.html):
  event joins do not define a transactional allocator.
- [Queensland Emergency Surgery Access Guideline](https://www.health.qld.gov.au/__data/assets/pdf_file/0033/635784/qh-gdl-440.pdf):
  published 2017 A–E categories differ from the earlier draft. Verify applicable
  current site policy; this source is not a current local authority assertion.
- [AIHW lower-urgency ED methodology](https://www.aihw.gov.au/reports/primary-health-care/use-eds-lower-urgency-care-2020-21-and-2021-22/contents/lower-urgency-care/summary):
  urgency alone does not establish suitability for diversion.
- [RANZCOG urgency statement](https://ranzcog.edu.au/wp-content/uploads/Categorisation-Urgency-Caesarean-Section.pdf):
  retain source version and local policy when selecting obstetric targets.

Unsourced transport speeds, crew mappings, response targets, parity shares and
clinical-duration distributions from earlier drafts remain candidate assumptions,
to resolve during B0 or future profile work. They are not validated defaults.
