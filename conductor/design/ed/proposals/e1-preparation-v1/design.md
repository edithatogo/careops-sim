# Generic ED pathway proposal

## Boundaries and inputs

Keep reusable scheduling, claims, interruption and fidelity in Kairos. CareOps owns clinical pathways, patient policy, site input mappings and endpoint meanings. Start generic/synthetic; public data requires the existing verified P catalogue; Cairns remains a later profile. E1 uses reviewed Macro execution initially and preserves the C2 fidelity choice in its inputs. Staff dispatch, route transit, cleaning and composite-team realism belong to E2; CAD/visualization remain post-v1.

P4 packs are data-only, use exact decimal-second source quantities, and are not E0 executable inputs. A future adapter must declare tick resolution, checked exact conversion or an explicit reviewed rounding policy, overflow handling, source provenance and an input-byte hash. Never copy these tick fixtures into empirical profile defaults. Root seed, named logical streams, event identities and draw positions must consume the accepted C2 contract; no new seed derivation is invented here.

## Arrival and case-mix preparation

Support exact supplied arrivals for deterministic fixtures. For the first synthetic P4 projection, propose an explicitly declared count-preserving generator: retain each interval/mode/category count and use the accepted logical RNG streams to place its arrivals within that interval. The within-interval law, time quantization, identity order and draw consumption must be reviewed and recorded; no uniform-placement or rate assumption is implicit. P4 counts are not Poisson intensities. A stochastic intensity-driven process may be selected only when the profile explicitly supplies its process family, rates, units and provenance. Do not infer rates, branching probabilities or durations from these oracle traces. Unknown case mix or diagnostic policy fails validation rather than silently choosing a clinical default.

Dispatch must add a repeatability oracle for the chosen arrival projection using the actual C2 stream contract, including zero demand, time-bin boundaries, same-tick ordering, changed root seed and unchanged inputs. This preparation package does not invent its expected random sequence before that API is qualified.

## Patient state and endpoints

Proposed states: arrived → waiting_triage → triage → waiting_assessment → assessment → optional diagnostics → optional treatment → disposition_pending → discharge_pending/awaiting_compatible_bed/awaiting_transfer → in_transfer when applicable → terminal_after_physical_departure. Explicit alternative terminal reasons are abandonment and transfer only when enabled by a reviewed profile. A patient can have several resource/task episodes while remaining one patient. Discharge-pending can proceed directly to physical departure; the branch list is not one mandatory linear pathway. Proposed terminal reasons are discharged (departure after discharge decision), admitted (physical departure to the accepted inpatient destination), transferred (physical departure to another facility) and abandoned (profile-defined physical leaving). Admission/discharge decisions alone never create a terminal reason/time.

Maintain distinct latent clinical state, agent knowledge (with observation time/provenance), provisional disposition and realized outcome. A decision reads only knowledge available at its simulation time; pre-sampled future latent events and eventual realized outcomes must not enter that view. Provisional admission/discharge intent can change and never substitutes for a physical terminal outcome. Test two runs whose visible histories match at decision time but whose hidden future outcomes differ: the decision must match, with no future information in its recorded inputs.

Store stable patient ID, arrival, declared acuity scale/category, pathway ID, stage/task/queue episode IDs, fidelity, clinical event history and exactly one terminal reason/time or a current unfinished state. Record queue entry, grant, active-work start/end, interruption/resumption and physical movement separately. Triage/administrative discharge/admission decisions are not interchangeable with physical departure. Null means absent/unobserved; it never means zero. Episode ID and event ordinal provide identity for repeated tests/tasks.

Acuity categories and proportions are supplied explicitly; never infer service times, deterioration, or staff interruption policy from category alone. Diagnostic branches are configured ordered stages for E1; parallel clinical orders require later reviewed composition. Optional stages skip without acquiring a resource. Zero-duration work is a bounded same-tick transition, not a polling loop; enforce an event budget and reject cyclic zero-work paths. Queue priority maps, preemptible task strategies and resource eligibility are explicit policies. Use the accepted Q tie ordering, generation/stale-event guard and completion-at-interruption semantics; do not override them with patient-ID ordering.

## Capacity and allocation

Each work item claims one unit of one named Q resource. ED occupancy, reservation, task claim and physical departure are distinct lifecycles. No hidden atomic bed-plus-nurse/doctor claim or nested composite-team allocation is introduced. Before implementing simultaneous occupancy and service, review how the domain occupancy ledger constrains eligibility without consuming the same bed twice. Multi-role resuscitation/sedation/assisted transfer is visibly unsupported unless a reviewed dedicated composite approximation is supplied.

Retain ED occupancy while a patient boards and throughout transfer; release on physical departure only. Compatible bed offers have identity, destination, expiry and single-consumption state. A late/expired/withdrawn/incompatible offer cannot release occupancy; repeated notification cannot duplicate departure. Ongoing boarder-care work remains an E1.3/E2 integration gate. External bed-offer scenarios do not claim causal ward or ambulance-community effects. Horizon expiry is a measurement boundary, not automatic patient discharge or resource release.

## Proposed observation decision

Recommend an E0-compatible initial E1 convention: positive horizon H; arrivals in [0,H), completion/departure events at H included; optional observation cutoff W satisfies 0 ≤ W < H. State/event rates use W ≤ event_time ≤ H and duration H−W. Arrival-cohort durations use W ≤ arrival < H, with unfinished/right-censored counts reported separately. This differs from the earlier half-open metric proposal. It must be versioned and approved before implementation; the existing proposal and E0 are unchanged here. No default H or W is supplied.

Queue wait (matching the existing metric proposal): active service start minus queue entry for one named episode. Also expose resource_claim_wait = grant minus queue entry; never alias these when grant precedes active work, such as during Micro transit. In the Macro fixtures grant and active work start coincide. Interrupted active work and requeue episodes remain separate, with any total-patient wait explicitly named. Time-to-care: first profile-declared care contact minus profile-declared eligible start; proposed synthetic fixtures use assessment start minus arrival and label that choice. LOS: physical departure minus arrival. Boarding: physical departure minus explicit boarding-start event (synthetic examples use admission decision). Throughput: unique terminal patients by reason, plus count/(H−W); do not sum task completions as patients. Utilization: summed nonoverlapping busy intervals per unit clipped to the observation window, divided by integrated known available staffed capacity for the same resource set. Zero denominator is undefined with counts/validity reason, never zero or infinity; unknown availability is unavailable. Zero-length observation windows are invalid. Report numerator/denominator and units, not just ratios.

Conservation at every snapshot: unique arrived = terminal + unfinished; unfinished states partition waiting/in_service/boarding/transfer. Terminal patients never reenter. Allocation counts stay between zero and known eligible capacity. Queue admission and preemption must neither duplicate a request nor release another patient's allocation. Conservation does not establish clinical plausibility or absence of starvation; report censored waits explicitly.

## Required decisions before implementation

| Decision | Proposed choice / gate |
| --- | --- |
| Time and horizon | Inclusive departures at H for compatibility; review/version the difference from the half-open metric proposal. |
| First care / boarding start | Explicit profile endpoint fields; fixture meanings are not national reporting definitions. |
| Stage duration | Active work excludes queue and transit; consume C2 duration/fidelity API, without resampling on resume. |
| Acuity and diagnostics | Declared scale/mix/priority map; ordered optional diagnostics initially. |
| Input projection | Review P4-to-E1 adapter and checked seconds-to-ticks policy; no assumed E0 compatibility. |
| Occupancy plus task claims | Review domain-ledger/Q integration before implementation; no implicit atomic multi-resource acquisition. |
| Admission and abandonment | Named, versioned policies; optional abandonment disabled unless supplied, with same-tick ordering reviewed. |
| Replay / empirical outputs | Consume accepted C1 event semantics and Arrow mappings; do not rename administrative clocks as physical events. |

Decision review belongs to the consuming E1 owner. This package proposes bounded choices; it does not create a new approved public API or bypass unmet engine gates.
