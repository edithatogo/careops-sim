# Incorporation of reports 5–8

Status: incorporated into planning on 2026-09-27; source evidence and runtime acceptance remain open.
Original narratives are preserved byte-for-byte in [the input manifest](supplied/20260927/manifest.json).
Raw Markdown is archived with a .txt suffix so unresolved external sandbox links
are preserved as source text, not presented as working project navigation.
Embedded document instructions were treated as proposals. No code, skills, datasets or remote repositories were installed.

## Intake and provenance

| Input | Contribution | Artifact status |
| --- | --- | --- |
| [5: spatial/behavior](supplied/20260927/report-5.md.txt) | Movement, task choice, interruption and observation design | Claims 48 evidence rows and 8 rules; CSV/JSON/protocol bundle not supplied |
| [6: DES primitives](supplied/20260927/report-6.md.txt) | Australian task means, international comparators, synthetic model assets | Claims 63 evidence rows, 17 assets, 14 gaps, 19 sources; files not supplied |
| [7: model audit](supplied/20260927/report-7.md.txt) | Corrected predictive DAG, planned/realized outcomes, recovery failure | New 51-row reconstruction; original 248-family count remains unverified |
| [8: standards audit](supplied/20260927/report-8.md.txt) | Denominators, versioned/partial mappings, recovery failure | New 68-row/272-mapping reconstruction; original 340/1,700 counts remain unverified |

Reports 5/6 explicitly did not inspect the local project specs. Their proposed gaps
were checked against our specs here, not accepted as proof of missing implementation.
Reported remote syntax/test passes are not local verification receipts. No downloadable
bundles matching these reports were found in Downloads; sandbox links are not local files.

## Incorporated requirements and task ownership

- P0.1/P0.3: reconcile all inventories by semantic IDs; preserve original versus
  reconstructed provenance, numerical evidence class and source-verification state.
- P1.2/P1.3: retain Australian task means as candidate mean anchors only; missing
  family/shape/tails stay unknown. Distinguish procedure clean-up, bed cleaning,
  notification, clean-bed idle, active consultation and response delay. Never average
  synthetic, elicited and observed triage values. Separate principal-activity-only
  observations from complete concurrent workload.
- P2.1/P2.2: record persistent staff assignments, eligible action sets, interruption
  parent/child chains, first subsequent task and eventual resumption; distinguish
  interruption arrival rate from switch response rate. Spatial records need mode,
  metric graph, O/D purpose, restrictions, geometry revision and walk/wait/work labels.
- P3.3/E1.2: generate every conditioning variable in an acyclic case-mix model;
  distinguish latent truth, agent-known information and recorded observations.
  Provisional disposition occurs after relevant information; realized disposition
  is an outcome. Historical final diagnosis/disposition cannot leak into prediction.
- C0.2/C1.1: preserve raw events and versioned semantic mappings; retain unknown ATS
  as an observation state, not a sixth clinical priority; source-population and
  denominator consistency are explicit tests. Published standards do not prove local deployment.
- C2.3/E2.2: configure movement modes and persistent assignment without importing
  site-specific coefficients. RTLS preprocessing/smoothing and map matching belong
  in observation adapters; straight-line displacement is not a legal route or speed fit.
- Q0/E0: review assisted transfers, contested equipment and supervision against
  one-resource scope. Preserve atomic bundles and fractional multitasking as deferred.
  ED task reselection must use the approved queue contract; it cannot override Suspend.

## Candidate sources and explicit limitations

The detailed numeric leads remain in the archived reports pending primary extraction.
Priority retrieval: Gerdtz/Bucknall ATS-conditioned triage table (DOI
10.1046/j.1365-2648.2001.01871.x) and the IHACPA Emergency Care Clinician Time
Consensus Study Report. Report 6's Zhu intern-study means are historical population-
specific anchors, not full fitted distributions. Lim's task-family labels lack
required parameters for several fits. Flex Track bed-placement-to-exit durations
are occupancy comparators, not clinician active work. STARS defaults are synthetic;
any Normal duration port must explicitly handle impossible negative durations.

Report 5's Cole adjusted effect is not a universal additive switch delay; Meng
layout coefficients are site-specific associations. Mode-specific speeds, supply-trip
matrices and supervision durations remain unknown. Missing values are not zero;
disabling a mechanism is a separate, explicit synthetic policy setting.

Candidate asset audit: STARS revision `161fa56392a836f1b3f6f32b448ae18ef29c9972`
and AMR-Hub revision `4f3f6c32872c4b0a1486b47f82c8272c34f70db5` as reported,
with independent license/source review before reuse. Geometry extraction is a future
adapter/example lead, not a new CAD implementation obligation. MIMIC-IV-ED demo
is a schema-fixture candidate; controlled full data require their own access terms
and neither provides direct triage start/stop evidence merely by having triage fields.

## Targeted primary-source checks and unresolved conflicts

The [AIHW appendix](https://www.aihw.gov.au/getmedia/159d833f-6f0f-4bd7-9563-c090a1ab8030/emergency-department-care-2024-25-appendixes.pdf)
confirms that its emergency-presentation table includes unreported triage.
Keep that population distinct from all ED presentations. Report 8's arithmetic
separates 9,091,132 known-category counts from 9,094,312 total (residual 3,180).
The exact attribution of the all-presentation residual still needs the corresponding
source-table metadata; do not infer it solely from a different table's footnote.
Conditional-on-known and all-record probability vectors require different denominators.
Report 7's rounded Queensland ATS vector summing to one does not prove an exact
empirical categorical distribution; preserve its emergency-visit restriction.

[Official OMOP 5.5 documentation](https://ohdsi.github.io/CommonDataModel/cdm55.html)
was independently located, updating the older report's 5.4-only reference.
This is a reference candidate, not an installed dependency or local system version.
Other standards/terminology versions reported in 8 remain pending exact source/
deployment verification. Mapping quality is exact/partial/proposed-extension/
simulation-only/unverified; measured task events are observational data, while
fitted duration distributions are model inputs. Do not classify every task timestamp
as simulation-only merely because a standard lacks a lossless field.

## Next research handoff

F1/F2 have returned narrative outcomes: original recovery failed; reconstructions
were reported. R2/R3 have returned narratives and asset leads. Do not rerun these
broad searches. Obtain the actual four sets of CSV/JSON/YAML/source/manifest outputs,
then run F3 reconciliation on available reconstructions plus DES/ABM evidence,
explicitly leaving historical original coverage unresolved. Never expand 51 or 68
rows artificially to match 248 or 340. R4–R8 research remains useful as previously scoped.
No plan checkbox or acceptance gate has been closed by document incorporation.
