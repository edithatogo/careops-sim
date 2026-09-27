# Embedded research content audit

The supplied Markdown reports are substantive research artifacts. Earlier intake
wording about unavailable bundles was too broad: a missing standalone CSV/ZIP does
not mean its subject matter or even all its rows are missing. No additional upload
is presumed to exist or required merely because a report names a file.

A local scan found 15 distinct reports among supplied reports 3–30, with 88 inline
Markdown tables and fenced blocks. Their verbatim excerpts are indexed with source
line numbers and SHA-256 hashes in the [embedded manifest](embedded-content/manifest.json).
This is a content index, not verification of citations, executable code or claimed
external artifact completeness. Tables listing file names also count as tables;
88 is not a count of usable datasets. Narrative outside these excerpts remains
available in the supplied reports and existing archives.

## Findings

| Reports | Already present | Remaining distinction |
| --- | --- | --- |
| 3–4 | Domain/parameter tables, examples, mapping tables and explanatory narrative | Report 3 explicitly calls its inline tables condensed relative to 248 families; report 4 claims a larger 340-row register/1,700 mappings. Do not infer those complete registers are present. |
| 5–9 | Evidence findings, corrected inputs, mapping summaries and gap/audit findings | Named row-level registers are not proven complete from filenames/counts; use the inline evidence now. |
| 10–11 | Explicit CSV, YAML, JSON, command and Markdown payloads | Already extracted previously; executable examples remain unexecuted proposals. Separate files are unnecessary to access these payloads. |
| 12 | Fourteen MD-labelled decisions in a table, methods and worked synthetic fixture material | Narrative describes 18 decisions/14 fixtures; exact full machine-readable versions are not reproduced as such. |
| 26 | Options/workload tables, benchmark guidance and commands | Enough for planning; full source manifest and exact standalone contract bytes are not established. |
| 27 | Eleven conformance rows and worked timelines | The advertised 11-row matrix is represented inline. Fourteen standalone test objects and full file metadata are not proven byte-equivalent. |
| 28 | Staged controls, tool choices, fixture/command examples | Use embedded material; do not infer full advertised JSON/YAML payloads from summary counts. |
| 29 | All nine ST-labelled acceptance-case summaries and boundary reasoning | These support local Given/When/Then authoring; exact YAML and all 21 decision records are not established. |
| 30 | Eleven candidate field names, partial crosswalk tables, positive chronology and six negative cases described in prose | Much of the intended content is present. A 44-row CSV with complete provenance columns is not reproduced in full. |

## Corrected workflow

Use embedded material directly. Where a structured local artifact is useful,
transcribe supplied rows or derive reviewed test cases from the narrative, preserving
source lines and labelling local additions. Do not represent reconstructed files
as recovered originals. Request further research only for specific substantive
facts or rows that are actually needed and absent, not for file packaging alone.

F3 cannot certify historical register completeness from headline counts. This does
not block planning or developing a new source-traceable local catalogue from the
material actually supplied. The earlier intake ledgers' unavailable-bundle notes
refer only to standalone artifacts, not absence of all underlying content.
