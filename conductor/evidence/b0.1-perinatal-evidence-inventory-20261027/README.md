# B0.1 — Public perinatal evidence inventory

**Task:** B0.1, birthing-suite delivery

**Status:** Accepted after reviewer-agent assessment

**Source review date:** 2026-10-27

**Implementation base:** `717ee035a0ea3aa5756a410510999945e3803c89`

**Packet:** [`B0.1.perinatal-evidence-inventory.json`](../../execution/packets/B0.1.perinatal-evidence-inventory.json)

## Finding

Public sources support retrospective population summaries, selected stratified rates, and definitions of fields collected by perinatal data systems. They do not provide a validated person-level labour-progression or deterioration hazard model, nor do aggregate marginals identify joint event-time distributions. This inventory is suitable for choosing evidence questions and profile provenance; it does not supply clinical thresholds or model parameters.

## Source inventory

| Source | Edition / version and period | Observable source shape | Permitted use for this task | Limits retained |
| --- | --- | --- | --- | --- |
| [AIHW, Australia's mothers and babies](https://www.aihw.gov.au/reports/mothers-babies/australias-mothers-babies/contents/summary) | Web report last updated 30 June 2026; principal perinatal reporting in the summary is through birth year 2023. | Public aggregate annual trends, counts/rates and selected population breakdowns from the NPDC and related collections. | Descriptive reference rates for explicitly matched population/jurisdiction/year strata; source for identifying candidate profile variables. | These are retrospective summaries, not individual records or conditional hazards. Topic completeness, denominator, jurisdiction exclusions, and shifts in reporting must be checked per table. The report itself warns of interpretation issues for trends such as stillbirths. Aggregates do not determine correlations or event-time distributions. |
| [AIHW, NPDC data-source and technical notes](https://www.aihw.gov.au/reports/mothers-babies/australias-mothers-babies/contents/technical-notes/data-sources) | Same 2026 report; 2023 cohort finalised nationally by 1 July 2025 and published 31 July 2025. | Describes annual standard de-identified jurisdictional extracts; the Perinatal NMDS supplied for 2023 spans NMDS 2022–23 and 2023–24. | Collection lineage, scope, timeliness, standardisation and availability cautions. | State/territory systems and inclusion scope differ. Some additional fields are not standardised or not collected everywhere; reported tables can differ from jurisdictional reports. This is metadata about data supply, not access to row-level records. |
| [Queensland Health, Perinatal reports and information](https://www.health.qld.gov.au/dsb/peri) | Public page lists *Perinatal Statistics—Queensland 2022* as final, released September 2024, as of access on 2026-10-27. | Annual aggregate report PDFs and linked tables/figures for Queensland 2022. | Queensland-specific descriptive context where a table's population and denominator match the question. | The page's listed report is a 2022 data year, not a current real-time feed. Table publication does not establish individual records, full distributions, missingness properties or joint correlations. Recheck the live page before later profile versioning. |
| [Queensland Perinatal Data Collection Manual 2023–2024, v1.0](https://www.health.qld.gov.au/__data/assets/pdf_file/0032/1249556/QPDC-2023_2024-manual_v1.0.pdf) | Manual title identifies 2023–2024, Version 1.0. | Data dictionary/collection instructions. It defines first- and second-stage labour duration fields in total minutes and says clinical judgement based on subjective observation may be used if quantitative measurement was not performed. It also distinguishes spontaneous labour, induction, and no labour (including failed induction resulting in caesarean). | Field semantics, unit and coding interpretation; informs what a future governed dataset could contain. | The existence of a duration field is not evidence of a public duration distribution, consistent measurement, complete values, or an empirical hazard. Subjective observations, different measurement practices, selection and censoring matter. Failed induction must not be misclassified as established labour. |
| [AIHW, National Core Maternity Indicators 2026 metadata (METEOR)](https://meteor.aihw.gov.au/content/805209/Print) | 2026 metadata set; supersedes 2025; standard date 2 September 2026. Indicator data cover calendar years 2004–2024, with availability varying by indicator and jurisdiction. | Definitions of selected clinical management and outcome indicators with retrospective reporting. | Versioned indicator definitions and aggregate monitoring context, after checking the selected indicator's exact numerator, denominator, and available years. | Indicators measure care processes/outcomes; they are not individual-level progression hazards. AIHW warns that some data are not standardised or available for every jurisdiction/year. |
| [Queensland Clinical Guidelines publications](https://www.health.qld.gov.au/qcg/publications) | Live publication index; exact applicable guideline edition must be selected and recorded during B0.2. | Normative practice guidance, distinct from observed outcome data. | Candidate source for a jurisdiction-specific policy target or eligibility rule after exact guideline/version/scope verification. | This inventory does not infer any numeric threshold or translate a guideline into model behaviour. Guideline currency and applicability need B0.2 review. |

## Evidence classes and assumptions

- **Observed aggregates:** AIHW and Queensland annual report tables provide aggregate counts, rates, trends, and selected strata. They can inform descriptive calibration targets only when year, jurisdiction, denominator, inclusion rules, and data quality are explicit.
- **Collection metadata:** AIHW technical notes, METEOR indicator records, and the QPDC manual define sources, fields, concepts, and availability. Definitions are not observed distributions.
- **Normative policy:** Queensland Clinical Guidelines are policy/practice sources. Their targets and eligibility rules must be versioned separately from observed frequencies and reviewed for the intended scope in B0.2.
- **Synthetic assumptions:** The birthing track's earlier example timings (including 30/75-minute examples), parity/duration examples, staffing, route shares, and missing event correlations remain hypotheses unless individually tied to an eligible source and population. No such value becomes a default in B0.1.
- **Individual hazards and joint distributions:** None of the public material inventoried here identifies person-level competing risks, labour-stage transition hazards, deterioration hazards, conditional dependence, or complete event histories. A hazard model would need suitable governed person-level or appropriately published survival/distribution evidence, explicit cohort and censoring definitions, and independent model qualification. Do not derive one by treating an aggregate percentage as an individual probability or by multiplying marginal rates.

## Gaps retained for B0.2 and B0.3

1. The public Queensland annual report currently listed is 2022 data. It is not an adequate basis for assuming current site-specific population, route, staffing, eligibility, or timing parameters.
2. Public aggregates do not supply a validated labour-duration distribution or competing-risk hazard. The QPDC manual's subjective measurement allowance is a further measurement limitation, not a substitute distribution.
3. Joint distributions/correlations among parity, onset, intervention, labour stage, deterioration, route and maternal/neonatal outcomes are not identified by marginal tables.
4. Availability, coding and inclusion may differ by jurisdiction, topic and year. The 2023 national extract joined two NMDS versions and has explicit topic exclusions; each later parameter needs a table-level audit.
5. Exact Queensland policy/guideline versions, profile scope, and any permitted target semantics remain to be verified in B0.2.

No patient-level source was accessed, no restricted-data request was made, and no synthetic evidence was represented as observed. This inventory does not constitute clinical validation, local service acceptance, or permission to use a profile operationally.

## Verification record

Repository: `<B0_SOURCE_WORKTREE>`

Python: repo-local `.venv/bin/python`, Python 3.15.1

Input hashes: packet-bound hashes for B0 spec/plan/test matrix appear in the packet.

Web access: official AIHW and Queensland Health pages/PDF listed above, accessed 2026-10-27.

### Review and coordinator acceptance

- Reviewer agent `/root/b01_review` reviewed implementation commit `da7aa6fbbf26164d85c2607dc10d478cad7638da`; outcome: no actionable findings. Review covered source dates/versions, aggregate-versus-hazard claims, the QPDC subjective duration caveat, scope and classifications.
- The coordinator accepted B0.1 after that review. Acceptance/registry synchronization is recorded in a separate coordination commit.

| Command (from repository root) | Exit | Output SHA-256 | Result |
| --- | ---: | --- | --- |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/B0.1.perinatal-evidence-inventory.json` | 0 | `2abaac2832351140c25f21f6fe23ba23c80e9d02f9b5043cee4c5c0d13d2ebe5` | Packet schema and binding valid. |
| `.venv/bin/python tools/tasks.py check` | 0 | `ab8b9f02b3213a06d5ea7abe5c5c139318dcecab9d957cb97570eaa7024af7d3` | 401 task records; coverage, prerequisites and DAG valid. |
| `.venv/bin/python tools/context.py check` | 0 | `96cc372010e7645170feafb7b28ede9afc3a6a9e77011a100d94a280dba552ed` | 25 tracks, 109 milestones, 1,974 local links; no errors. |
| `git diff --check` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | No whitespace errors. |

All four commands were executed on base `717ee035a0ea3aa5756a410510999945e3803c89`, using repository `.venv/bin/python` 3.15.1 where applicable. The packet records exact source-input hashes. These are planning/inventory integrity checks; no birthing simulation or clinical behavior was run.
