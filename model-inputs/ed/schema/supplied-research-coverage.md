# ED supplied-research coverage and gap map

Status: coordinator draft for P0.1 review. This maps supplied narratives and
indexed embedded content to the declared generic-ED input scope. It does not
accept numerical claims as empirical defaults, verify external citations, or
establish that referenced downloadable bundles exist. Report hashes are in the
[archive manifest](../../../conductor/research/supplied/20260927/manifest.json);
embedded excerpt hashes and source line ranges are in the
[embedded-content manifest](../../../conductor/research/embedded-content/manifest.json).

## Disposition vocabulary

- **Retain**: keep a distinct input family or boundary in the local catalogue.
- **Merge**: preserve under a shared semantic ID only after checking meaning,
  conditions and population; retain each source lineage.
- **Conflict**: retain both claims and an explicit resolution rule; do not average
  or silently select one.
- **Defer**: outside the generic MVP or unsupported at the declared fidelity.
- **Gap**: required content is absent, unverified, or not identified by available
  observations; record owner/acquisition route and impact.

## Coverage by model-input family

| In-scope family | Supplied coverage and report provenance | P0 disposition | Remaining gap / guardrail |
|---|---|---|---|
| Demand, arrival mode, temporal profile, referrals/transfers, case mix and acuity | R3 catalogue (83–106); R4 parameterisation (85–140); R7 corrected case mix/disposition (23–58); R8 denominator audit (44–89); R9 candidate structure (32–88) | Retain time-varying demand and conditional case mix; merge synonymous labels only after stable-ID/denominator review | Values are candidates. National marginals do not identify patient-level dependence. Keep R3/R4 synthetic annual profiles separate. Verify definitions/denominators (R8 44–89; R9 89–131). |
| Arrival pathway, ambulance, triage, reassessment and offload | R3 pathway tables (41–52, 87–106); R4 data/parameter tables (93–140, 220–231); R6 primitive evidence (17–46); R8 triage audit (44–89); R29 ED boundary/offload (21–88, 130–151) | Retain arrival, triage, ambulance arrival, handover/offload and crew release as distinct events | Public summary clocks do not establish primitive service or crew-release distributions. R29 offer/withdrawal semantics are proposed and need P1 review; no fixed POST surrogate as a primitive. |
| Clinical pathway, diagnostics, treatment, observation, consultation and disposition | R3 catalogue/pathways (83–106); R4 core/minimum boundary (93–140, 265–288); R6 primitive work anchors (17–46); R29 boundary cases (21–36, 130–151) | Retain pathway stages, route probabilities, loops, and provisional versus realized disposition | Verify source rows and clinical pathway definitions. Do not leak final diagnosis/disposition into agent-known state. Exact multi-resource bundles exceed the approved minimal queue boundary (R29 102–128). |
| Work, queue, transit, turnaround and elapsed durations | R3 dependence/EHR fields (150–218, 327–351); R4 candidate distributions (120–140); R6 active-work cautions (17–46); R12 fitting/identifiability (19–65, 97–130, 216–266) | Keep active work, queue delay, transit, response delay, cleaning, boarding and total elapsed time distinct | EHR elapsed intervals cannot identify intrinsic work without direct observations or justified decomposition. Mean-only anchors are not fitted distributions. Fitting/holdout belongs to P1/P3. |
| Physical treatment capacity, beds, rooms, equipment, staffing and calendars | R3 capacity/workforce (89–106, 291–351); R4 parameterisation/data request (93–140, 220–231); R29 physical bed retention/offer boundary (28–36, 62–101) | Retain physical count separately from staffed/open/compatible capacity; represent time-varying resources | Generic/public sources do not establish operational availability. No Cairns as-operated capacity or CAD is supplied. Site validation belongs to a later profile task. |
| Resource queues, priority, timeout, preemption and interruption state | R3 queue/resource family (83–100); R5 task choice/interruption (37–58, 80–113); R27 conformance/timelines (60–190); R29 concurrency boundary (102–128) | Retain queue policy, priority direction, deadlines, cancellation and interruption ancestry as separate contract dimensions | R27 is a proposal, not an accepted Kairos API. Q0 owner disposition and Track25 root registration remain gates. Atomic bundles/fractional multitasking are deferred. |
| Staff ABM attributes, task choice, zones, skills, handover and interruptions | R3 actor rules/dependence (150–218); R5 observations/gaps/protocol (19–59, 80–145); R6 coupled tasks (27–46) | Retain eligibility, persistent assignment, task choice, interruption lineage, resumption, handover and switching overhead separately | EHR timestamps alone do not identify heuristics, concurrency or eligible choices. Use transparent baseline rules; label unobserved choices as assumptions (R5 80–113). |
| Spatial layout, named locations, route graph, scale, walking and congestion | R3 EHR/spatial data request (327–351); R5 movement/trips (21–36, 101–115); R9 spatial limitations (132–149); R26 workload/benchmark candidates (52–90); R29 location boundary (28–36, 88–101) | Retain minimal named locations and optional graph/distance/travel inputs; allow spatial fidelity to be disabled | No real ED CAD or route graph is supplied. Public geography is not indoor routing. Geometry import, visualization and congestion are deferred; no unsupported walking-speed defaults. |
| Patient behavior, mobility, assistance, patience and abandonment | R3 behavior catalogue (100–104, 150–166); R4 minimum/advanced boundary (275–288); R5 EHR limits (80–100) | Retain only behaviors enabled by the declared model; permit explicit unknown/deferred states | No supplied evidence establishes generic patience, mobility or compliance distributions. Do not add behavioral detail merely to fill the catalogue. |
| Hospital boundary, admission offer, boarding and transfer | R3 boarding/inpatient boundary (50–51, 91–104); R4 local data request (133–140, 220–231); R29 offer/boarding/finite vacancy semantics (21–68, 88–101) | Retain bounded external admission/bed-offer boundary and ED resource retention through physical departure | No whole-hospital ward, elective, discharge or bed-release mechanism in ED MVP. Specialty-conditioned finite offers/persistence need P1 review. |
| Initial state, time horizon, warm-up, replications, seeds and experiments | R3 simulation/observation family (104–106, 217–218); R10 determinism/evaluation proposal (39–131); R26 numerical/benchmark contract (90–151); R28 CI/quality proposals (66–148) | Retain initialization, time units, seed purposes, horizon, warm-up, replications and scenario controls | Reported commands/harnesses are unexecuted proposals. Do not infer backend support from crate/feature names; GPU, PDES and distributed execution remain phased. |
| Measurement/event definitions, EHR mapping, missingness and censoring | R3 EHR field inventory (327–351); R4 standards/crosswalk and data requests (29–79, 211–245); R7 standards mapping (110–150); R8 audit (90–180); R12 protocol/censoring (19–130); R30 minimal crosswalk/fixtures (119–177) | Merge identical concepts only with versioned mappings; distinguish observed, clamp-only, exogenous and validation-target roles | Excerpts are not complete external bundles. R30 crosswalk is partial/provisional. Timestamp semantics, site mapping, missingness and censoring need verification before calibration. |
| Costs and outcomes | R3 optional costs/outcomes (104–105); R4 optional cost/resource row (140); R26 calibration workloads (58–68) | Defer from minimal functional model; preserve extension points | No agreed perspective, local prices, costing year or outcome valuation. |
| Fatigue, learning, crowding feedback, detailed physiology and whole-hospital behavior | R3 optional domains (100–106); R4 advanced ABM boundary (275–288); R5 evidence limits (80–113); R29 out-of-boundary cases (102–128) | Defer unless evidence and decision relevance justify a later scoped change | Avoid unidentifiable agent complexity and later-domain tracks before generic ED MVP/v1. |

## Cross-report conflicts and merges to preserve

1. **Register completeness:** R3 describes 248 families and R4 reports 340
   parameters/1,700 mappings (R3 389–404; R4 265–295). Inline excerpts do not
   authenticate complete standalone registers. Use local declared scope and
   consumer coverage, not those counts, as completeness oracle.
2. **Synthetic demand scenarios:** R3 and R4 give different illustrative annual
   demand profiles (R3 243–291; R4 120–134). Keep separate and label synthetic.
3. **Triage denominators:** R4 candidate marginals are audited by R8 (R4 120–128;
   R8 44–89). Preserve unknown/missing categories and denominator definitions;
   do not normalize conflicting rounded vectors into a clinical priority vector.
4. **Observed duration versus primitive work:** R6 task means and R12 partial
   observation methods do not justify converting EHR elapsed time to active work
   (R6 17–46; R12 32–65, 97–130).
5. **Clamped calibration:** R12 supports clamping only for identifiable inputs or
   anchors and requires unclamped downstream validation (R12 19–30, 97–170).
6. **Queues and concurrency:** R27 semantics and R29 cases inform tests but do not
   override the pending Kairos owner contract or widen Q v1 (R27 60–190; R29
   102–128).
7. **MLX/Metal and parallelism:** R10/R26/R28 give staged evaluation proposals,
   not evidence of accepted implementation or speedup (R10 39–131; R26 90–175;
   R28 66–148). Keep Rust-native authority and planned CPU-first phases.

## Gaps and ownership handoff

| Gap | Owner/phase | Closure evidence |
|---|---|---|
| Primary-source verification, versions/licences, extraction and transfer limits | P1/P2 | Exact page/table/field, population, period, units, licence and transformation; verification outcome. |
| Intrinsic service-time families and tails | P1/P3 | Direct primitive observations or transparent synthetic assumptions; fit diagnostics and chronological holdout. |
| Joint case mix and denominator definitions | P1/P3 | Cohort-specific source vectors and dependence evidence; explicit unknown/other/missing handling. |
| Staff heuristics, interruptions and movement | P2/P3 | Observation/elicitation or transparent baseline policy; no inference from undifferentiated timestamps. |
| Generic effective capacity, rosters and space capability | P1/P4 | Public-backed example or synthetic fixture distinguishing open/closed/staffed/compatible states. |
| CAD/as-operated geometry | P2 and later E5 | Not required for generic MVP; if supplied later, retain CAD revision/units and clinical operational validation separately. |
| Admission offer and competing demand | P1/P3 | Bounded external boundary with finite offer/withdrawal semantics; no whole-hospital mechanism in this track. |
| Consumer-to-catalogue mapping and load tests | P5 after E2/C5 | Runtime rejects missing, unexplained-default and orphan active inputs. |
| Cairns values and event mappings | Later site-profile phase | Separate locally evidenced profile; does not block generic inputs. |

## Source traceability

The archive manifest records SHA-256 for supplied report aliases and identifies
byte-identical duplicates. The embedded-content manifest records each indexed
excerpt's source report, report hash, excerpt file, source lines and excerpt hash.
Line references above point to archived report files. Reports 13–25 add no
distinct content where their hashes alias reports 5–12; see archive manifest.
Citation tokens inside reports remain unresolved until P1/P2 verify primary
sources.

## P0.1 disposition

This is a scope-wide reconciliation draft, not the final catalogue/schema. It
supports P0.2 schema design and P0.3 bounded primary-source verification. P0.1
may close only after coordinator review confirms rows against versioned scope and
indexed excerpts, corrects provenance defects, and records an acceptance receipt.
P0.2–P5 remain open.
