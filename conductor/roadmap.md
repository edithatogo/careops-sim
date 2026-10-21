# Product expansion roadmap

20 October 2026 planning revision. Preserve generic/public baselines, separate
jurisdiction/site profiles and evidence-qualified delivery. The original five
track checklists are complete; broader capability claims require R0 reconciliation.

| Milestone | Scope | Sequence / evidence |
| --- | --- | --- |
| M0 | Reconciliation and baseline | R0–R3: exact-source capability audit, harness and ED baseline, bounded candidate qualification; retain working Kairos pin if required gates fail |
| M1 | Small shared healthcare contracts | A0–A5: identity, progression/policy clocks, effective capacity, team bundles, transfer lifecycle and relationships |
| M2a | First cross-service fixture | A6: ED → minimal ward → community; discharge readiness, backpressure, transfer and conservation proof |
| M2b | Second consumer and shared join | B0 evidence intake may start independently; B1–B7 birthing → bounded theatre → maternal/infant destinations; A7 joins both and observation/evaluation contracts |
| M3 | Surgery and inpatient flow | Full theatres/PACU/day surgery and acute/ICU/postnatal/subacute wards; shared staff, equipment and downstream constraints |
| M4 | Ambulatory/procedural/waitlist services | Clinics, endoscopy/cath lab, appointments, recurring contacts, waitlist eligibility/pause/reassessment/cancellation clocks |
| M5 | Hospital orchestration | Bed management, shared diagnostics, order/specimen/result lifecycle, workforce calendars, inventory/sterilisation/maintenance where justified |
| M6 | Network and policy evaluation | Transfers and retrieval, public/private/community boundaries; single-process reference first, distributed mappings measured later; offline fiscal exchange before coupled optimisation |

The new birthing slice is an early second consumer; full surgical delivery remains
M3. Its bounded theatre callback is required for birth/completion metrics, not a
claim that the complete surgical model exists.

## Common contracts and owners

[CC-01–CC-08](design/composition-contracts-20261020.md) define required semantics
and two acceptance fixtures. [Related repository boundaries](design/related-repository-boundaries-20261020.md)
cover Kairos, the CareOps portfolio, NHRA and the NWAU wrapper.

Kairos 01/03/59/64 own candidate entity/resource mechanisms; 21/22/66 own reusable
experiment/observation mechanisms; 04/62 own format interoperability; 25 owns API
compatibility. Resolve live IDs and ownership at dispatch. Parallel local commit `0157d0cb60932b3cda86761954e1d0d9efebd55c` registers
67 (relationships), 68 (channels) and 69 (competing risks). Reuse those pending
tracks; their registration does not certify runtime or justify pin promotion.

Keep clinical priorities and operator names in versioned profiles. A shared
priority representation does not equate classification systems or confer safe
interruption permission. Event conditions do not imply atomic resource grants.
Use [priority policy](design/universal-triage-priority-framework.md) and
[transfer boundaries](design/transport-retrieval-interfacility-flow.md).

## Data and qualification

Build generic reproducible models with public data and labelled synthetic
assumptions; qualify CHHHS/Cairns profiles separately. Preserve source versions,
units, transformations, included/excluded observations and parameter uncertainty.
Aggregate data alone may not identify micro-level hazards or causal policy effects.

Every addition requires conservation, replay, failure/recovery and appropriate
observation validation. Keep latent patient state, available observations and
policy actions separate. Preserve working ED adapters until replacements pass
baseline parity and rollback checks. CPU composition is independently useful;
GPU, PDES and distributed execution each need their own runtime evidence.

## Later domain candidates

Mental health, rehabilitation, palliative care and hospital-at-home extend the
same identities and boundaries to long episodes, recurrent contact and community
constraints. Reserve detailed tracks until a concrete requirement and evidence
source exist. No speculative plugin framework is required.

Spatial capture, CAD, shared floor plans and live UI remain governed by the
[spatial plan](spatial-visualization.md). Evaluate typed event namespaces if
composition demonstrates collisions; coordinate with Kairos 01/25 rather than
creating another event registry. Preserve the [post-MVP reuse review](design/post-mvp-kairos-abstraction-review.md)
extraction and compatibility gates.

## Parallel audit alignment

The 20 October capability audit is retained separately at CareOps commit
`356f54b` and Kairos commit `0cdc9ac`. R0 consumes its findings; no registry
Done label overrides missing runtime evidence. Preserve the qualified Q/C,
physical IO and checkpoint capabilities of the current pin when considering
mainline. Inventory reusable experiment orchestration first (seed partitioning,
worker invariance, progress/cancel, manifests and recovery), then extract only
small neutral mechanisms with a second consumer and parity proof. This reuse
lane can proceed independently of domain resource implementation; it is not a
new prerequisite for the headless ED model. Continue hardware qualification
through the audit's Mac/NAS/CUDA plan, separately from composition acceptance.
