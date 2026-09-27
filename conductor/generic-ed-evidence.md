# Generic ED: initial public evidence and example shortlist

Checked 2026-09-25. These are candidate inputs for the generic model; no dataset
has been downloaded, fitted or validated in this planning delivery.

| Source | Intended use | Limit |
| --- | --- | --- |
| [AIHW emergency department presentations](https://www.aihw.gov.au/hospitals/topics/emergency-departments/presentations) | Primary Australian aggregate reference for presentation volumes and triage mix; follow linked downloadable tables and metadata | Aggregate summaries do not provide individual staff, transit or resource-use traces |
| [CDC NHAMCS datasets and documentation](https://www.cdc.gov/nchs/nhamcs/documentation/index.html) | Optional public-use encounter-data example for ingestion and sensitivity work; consult codebook and survey weights | US context; not an interchangeable Australian/Cairns parameter source or a continuous hospital event log |
| [HSMA healthcare simulation examples](https://github.com/hsma-programme/simpy_visualisation) | Open model/example discovery, including its linked treatment-centre model; compare pathway structure and simple reproducible scenarios | Teaching/treatment-centre assumptions do not establish completeness or validity for a full ED |
| [Treatment-centre model documentation](https://pythonhealthdatascience.github.io/stars-simpy-example-docs/content/02_model_code/04_model.html) | Inspect the original model structure before selecting a small reference fixture | Exact code/data licence, revision and applicable assumptions must be recorded before reuse |

## Use in the current plans

During C0/C1, inventory usable fields, population/period, aggregation, units,
missingness, source revision, licence and reference hash. Select an Australian-
style generic baseline with explicit assumptions, then create synthetic event
traces with known ground truth for calibration/queue tests. Use public summaries
as aggregate checks, not as fabricated patient histories. If constructing a
synthetic population from marginals, label assumed dependence and distributions.

Python reference models may inform fixtures; the implemented simulation and
reusable modules remain Rust-native. Open examples are references, not new
mandatory runtime dependencies or automatically added submodules.

Keep profiles such as `generic-ed` and later `cairns-ed` conceptually separate.
For Cairns, revisit pathway/triage/bed definitions, arrival patterns, staffing and
skills, spaces/routes, diagnostics, cleaning, admission/boarding and timestamp
meaning with local evidence. These later inputs are not prerequisites for the
current framework specification or generic ED model.


The [ED parameter-evidence track](tracks/ed_parameter_evidence_20260927/plan.md)
now owns systematic source discovery/extraction, applicable ranges/distributions,
assumptions and example profiles. This shortlist remains a starting point, not a
complete parameter evidence base or validation of any numerical default.
