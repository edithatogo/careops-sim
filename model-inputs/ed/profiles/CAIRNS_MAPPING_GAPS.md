# Future Cairns ED evidence mapping and acquisition plan

**Status:** future-work plan. **Local availability:** UNKNOWN. No Cairns source,
feed, profile, data extract, or approval route has been evidenced in this work.
This document records no local values and establishes no Cairns conformance.
No private data or private patient data is included. The generic P4 fixtures remain separate
from any future Cairns profile; they are not local evidence.

The supplied report-30 temporal crosswalk and incorporation note provide
candidate standard semantics and mapping-strength labels. Those labels describe
standards semantics, not local availability. See
`conductor/research/supplied/20260927/report-30.md.txt` (overall finding,
provisional crosswalk, and timestamp/location implications) and
`conductor/research/ed-research-incorporation-26-30-20260927.md` ("Trace
semantics and candidate crosswalk"). Verify the pinned research baselines and
site applicability during future acquisition. The P4 guide's synthetic pack
and profile vocabulary are structural references only.

## Candidate standards crosswalk for later source verification

The supplied report-30 audit compared the following research baselines: FHIR R4
4.0.1 with AU Base 6.0.0 / AU Core 2.0.0, HL7 v2.5.1, AIHW NAPEDC NMDS
2026–27, and OMOP CDM 5.5. These are versioned research references to verify
before use, not claims about the versions implemented at Cairns or the latest
available standards. The mapping-strength labels below describe semantic fit in
those standards only. They do not indicate that Cairns uses or exposes a source.

| Candidate source profile | Candidate field mapping to retain for review | Limits to verify before treating a value as observed |
|---|---|---|
| FHIR R4 / AU profiles | Event-specific Encounter/status periods for occurrence; `Encounter.location.location` and `.period` for a location interval; linked `AuditEvent.recorded` for audit recording time | `meta.lastUpdated` is resource-version time, not generic source event-recording time. Encounter end is not automatically the non-admitted episode end or physical exit. Establishment-level location is not necessarily an internal ED location. |
| HL7 v2.5.1 | `EVN-6` event occurrence; `EVN-2` recorded/entered time; `MSH-7` message creation time as a separate clock; `PV1-3` plus local PL dictionary for location; applicable movement events for interval reconstruction | Confirm the implemented message profile and field population. An A08/PV1-3 update alone does not prove movement; movement reconstruction needs sequence, cancellation and local location rules. A03/`PV1-45` does not by itself establish the physical ED boundary. |
| AIHW NAPEDC NMDS 2026–27 | Milestone-specific date/time pairs; preserve non-admitted episode end separately from physical departure (report-30 cites METEOR 799780/799782 and 799756/799758) | Relevant time precision is `hhmm`; do not manufacture seconds or timezone offsets. Aggregate reporting does not supply internal location, staff, or bed intervals, and does not prove availability of Cairns source records. |
| OMOP CDM 5.5 | `VISIT_OCCURRENCE` / `VISIT_DETAIL` as an analytic projection where ETL lineage identifies their meaning; retain source IDs and transformations | Visit/detail bounds are partial mappings. Derived/defaulted visit end is unsafe as physical departure without ETL provenance; the CDM has no generic raw-source recording-time field. `care_site_id` may be coarser than an ED location. |

Before using any row, verify the exact source version/profile, field definition,
population, available precision, location dictionary, mapping transformation,
and endpoint semantics from the actual Cairns source and its accountable owner.
If local source evidence is absent, record `UNKNOWN` and do not backfill a
candidate standards field as an observation. These candidate relationships are
from report 30's provisional crosswalk (including its lines 123–150); they are
not local conformance findings.

## Mapping and evidence acquisition matrix

Every source and value below is a candidate for investigation, not a claim that
a named system exists or supplies a field. For every row, current local status
is **UNKNOWN**. Source owner and approval route must be identified and approved
before access or use.

| Generic input/evidence group | Candidate local source and required fields | Owner / approval route | Status | Transformation, validation, and review |
|---|---|---|---|---|
| Arrival and case mix | Candidate registration/arrival or encounter feed; event/case identifier, arrival channel, arrival time, triage/category fields if defined, disposition, cohort and observation window | To be identified; service owner, data custodian and governance approval | UNKNOWN | Preserve source meaning, units, event and recorded clocks; reconcile counts and denominators; clinical and data review before parameterization |
| Acuity and pathways | Candidate triage, clinical pathway, or encounter classification source; source-coded acuity, pathway/category, effective time, mapping version | To be identified; clinical owner and terminology/data governance approval | UNKNOWN | Retain source codes and versioned crosswalk; no invented category mapping; clinical review of applicability and missingness |
| Work and elapsed durations; censoring | Candidate event, task, or encounter records; start/end event definitions, observation window, censor indicator/reason, clock precision | To be identified; process owner and data custodian approval | UNKNOWN | Keep work duration distinct from elapsed time; derive only from documented endpoints; preserve incomplete/censored observations and review exclusions |
| Physical, open, staffed, equipment capacity and calendars | Candidate space/capacity registers, operational calendars, equipment/capability records, and staffing availability evidence; resource/location IDs, capacity state and effective intervals, capability, schedule and exceptions | To be identified; operational, facilities, workforce and equipment owners with governance approval | UNKNOWN | Distinguish physical from open, staffed and usable capacity; model effective intervals and calendars only after as-operated operational validation |
| Staff roles, skills and rosters | Candidate role/skill catalogues and roster or availability records; role/skill IDs, coverage intervals, zone/resource association and applicable rules | To be identified; workforce and clinical operations owners with privacy/governance approval | UNKNOWN | Use only authorized, appropriately aggregated or de-identified information; distinguish rostered from available/staffable coverage; workforce and clinical review |
| Locations, route graph, CAD and operational validation | Candidate verified CAD/as-operated location records, movement or location events, route/transfer rules; `location_id`, boundaries, adjacency/route and effective intervals | To be identified; site operations/facilities owner and data custodian approval | UNKNOWN | Define exact ED boundary and internal locations from verified evidence; validate routes and operational rules with site operations; no inferred paths |
| Inpatient bed offers and boarding boundary | Candidate bed offer/acceptance and transfer records; offer/decision times, status, destination, cancellation/correction and physical movement evidence | To be identified; bed management and ED/inpatient operational owners with governance approval | UNKNOWN | Separate offer, acceptance, admission/episode milestones, and physical movement; agree boarding boundary and validate with both services |
| Observation and measurement denominators | Candidate source population definitions, event coverage/monitoring metadata and measurement records; eligible population, inclusion window, capture status, unit, precision and missingness | To be identified; clinical measurement owner and data governance approval | UNKNOWN | State denominator and coverage window; distinguish unobserved from zero; assess representativeness and missing/censored data before use |

## Time, location, and event handling

Keep these concepts separate whenever the source exposes them:

- `source_event_time`: when the source says the event occurred.
- `recorded_time`: when the source system recorded the event.
- Message creation time, where available, such as a source message timestamp.
- `location_id`, `location_start_time`, and `location_end_time`: location identity
  and the interval during which it applied.
- `episode_end_time`: non-admitted ED episode end, where defined by the source.
- `physical_departure_time`: physical exit from the ED, where recorded.

Never equate event occurrence, message creation, data entry, `lastUpdated`,
episode end, and physical exit. A derived value must retain its derivation and
must not be presented as an observed source timestamp. Preserve source minute
precision; timezone is UNKNOWN until evidenced. Document any date/time
combination rule, DST (daylight saving) handling, source clock correction, and
the authority for each rule. Define how late, cancelled, corrected, and duplicate
events are handled, including sequence and tie rules. Maintain source-level
provenance for every retained or transformed event and location interval.

## Required source/feed profile and ETL lineage

For each independently governed source or feed, record a versioned profile with:

- system and version (UNKNOWN until verified), source owner, source/feed identity,
  extraction/access route and authorization;
- authoritative field definition and source code set, population and time window,
  units and precision, timestamp and location semantics;
- licensing, privacy and governance constraints; known and unknown values;
- correction, cancellation, duplicate, late-arrival and clock-correction policy;
- immutable source snapshot hash and capture date, with a reproducible reference
  that does not expose restricted records.

Maintain ETL lineage from each raw source field through every transformation to
the normalized parameter, event, location or profile ID and output hash. Record
the transformation version, units, timestamp derivation, code mappings,
reversible audit reference, and missingness/censoring rules. Preserve raw-to-
normalized traceability in an authorized environment. Do not silently impute,
collapse clocks, or replace missing/censored endpoints with defaults.

## Site operational validation

Before a site profile is proposed, site reviewers must establish from verified
evidence the exact ED boundary, internal location dictionary, route graph and
as-operated movement rules. Validate active capacity against closed,
repurposed, temporary/surge and otherwise unavailable spaces; confirm monitored
capability, equipment availability, staffable schedules, and operational rules
for each applicable interval. Use CAD/as-operated review only when a verified
source is obtained. Record reviewer, evidence reference, effective date and
unresolved discrepancies. No bed counts, arrival values, local paths, source
schema, timezone, or operating rule is established here.

## Privacy, governance, and staged acceptance

Identifiable or raw private records must not be committed to this repository or
public CI artifacts. Use synthetic fixtures only until authorized access to
appropriately de-identified data is approved. Document the data custodian,
purpose, permitted fields, access controls, retention, disclosure controls and
approval evidence before extraction. Governance permission is currently
UNKNOWN.

1. **Scope and authority:** identify source owners, custodians, clinical and
   operational reviewers, intended population/window, lawful/authorized access
   route, and privacy/governance approvals. Stop if any are unresolved.
2. **Source discovery:** inventory candidate feeds and versions without
   presuming availability; obtain field definitions, licensing and correction
   policies; capture immutable, access-controlled source hashes when authorized.
3. **Schema and contract gate:** before representing observed Cairns values,
   review and version a schema/provenance contract that supports sourced local
   observations and their privacy/licensing metadata. P4 schema v1 currently
   requires `provenance.class: synthetic`; the P4 structural/semantic validators
   therefore cannot accept an observed Cairns profile. Do not mislabel local
   observations as synthetic or bypass those validators. Keep v1 unchanged and
   preserve its fixtures/manifest; implement and review a backward-compatible
   schema, validator, manifest and acceptance-contract revision before adding a
   site-profile fixture. The schema change does not itself authorize private data
   or external redistribution.
4. **Semantic mapping:** only after the schema/contract gate is accepted, create
   a separate proposed Cairns profile with explicit source-to-field lineage.
   Keep unknowns explicit and preserve generic fixture files and manifest.
5. **Operational and clinical review:** validate boundary, locations, capacity,
   staffability, pathways, denominators, and timestamp/correction semantics with
   accountable site reviewers; record evidence and unresolved gaps.
6. **Transformation and quality review:** reproduce ETL from authorized source
   evidence; verify units, precision, provenance, missingness, censoring,
   corrections, output hashes and privacy controls independently.
7. **Acceptance:** obtain independent data, clinical and operational review;
   confirm the proposed site profile and its evidence meet the revised, versioned
   schema/contract and validators. Record explicit acceptance before using it
   for Cairns analysis or calibration. A completed checklist or standards
   crosswalk alone is not acceptance.

Until these gates pass, local availability and all site-specific values remain
**UNKNOWN / not established**. This is an acquisition plan, not a site
specification or completed acquisition.
