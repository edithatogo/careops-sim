# B0.2 — Versioned Queensland public-reference profile

**Task:** B0.2, birthing-suite delivery

**Status:** Accepted after reviewer-agent re-review and coordinator acceptance

**Review date:** 2026-10-27

**Implementation base:** `ed4c9cea11dfb2cff4b218530c1fde0fca1c3d5b`

**Packet:** [`B0.2.qld-reference-profile.json`](../../execution/packets/B0.2.qld-reference-profile.json)

**Profile:** [`qld-public-reference-2022-v1.json`](../../../model-inputs/birthing/profiles/qld-public-reference-2022-v1.json)

## Profile scope

The new JSON is a versioned *reference-only* profile. It records the Queensland 2022 published aggregates for 60,477 mothers and 61,404 babies, including pregnancy-history categories, plurality, maternal-age groups, labour onset and birth method. Each count uses the table's named mother/baby denominator. The method-of-birth categories reconcile to 61,404; labour-onset categories reconcile to 60,477.

Observed aggregate tables, current policy-source metadata, and synthetic assumptions have separate fields. The observed QPDC table cross-classifies selected attributes (for example onset by method and method by plurality), but it does not contain linked person histories, complete joint distributions or individual time-to-event data.

## Current Queensland policy-source readback

As accessed 2026-10-27:

- The Queensland Clinical Guidelines publication index lists *Intrapartum fetal surveillance* last amended September 2026, with review date February 2030. The guideline history records V11 on 4 September 2026 and V12 on 25 September 2026. The currently linked PDF identifies `MN25.15-V12-R30`, amended September 2026, and explicitly replaces V11. The later history entry and PDF agree. No clinical decision rule or operational turnaround target is encoded.
- The current *Normal birth* guideline PDF identifies `MN22.25-V5-R27` (published December 2022); the index says last amended July 2023 and review December 2027. Its one-to-one midwifery-support statement is specifically within the low-risk-at-term context and does not define a general site roster ratio.
- The publication index lists *Instrumental vaginal birth* as last amended October 2026 and review due April 2029. Its linked PDF identifies `MN24.49-V5-R29`, effective October 2026, with review due December 2029. The two sources conflict on the review date; both dates are retained as unresolved. *Prolonged and/or obstructed labour* is listed as last amended September 2026. These have distinct populations and clinical scopes. None is converted into a generic rule by this reference profile.
- These versions are Queensland references. A specific site model, applicable population, local capability, and service governance have not been selected. Accordingly eligibility is not encoded, staffing is unavailable as a public site parameter, and operational decision-to-transfer/incision/delivery targets remain unconfigured. Clinical observation or stage-duration guidance is not treated as a service turnaround SLA.

Earlier 30/75-minute and parity/duration examples remain hypotheses. This profile does not make them clinical constants or thresholds.

## Source links and source versions

- [Queensland Perinatal Statistics 2022 tables](https://www.health.qld.gov.au/__data/assets/pdf_file/0027/1378017/tables_2022x.pdf), final report released September 2024; selected variables and Tables 6.01, 6.05 and 6.12. The report flags form changes as a potential influence on reporting.
- [Queensland Perinatal Statistics report index](https://www.health.qld.gov.au/dsb/peri), accessed 2026-10-27; identifies the 2022 report as final and its release date.
- [Queensland Clinical Guidelines publications](https://www.health.qld.gov.au/qcg/publications), live policy index accessed 2026-10-27.
- [Queensland Clinical Guidelines history](https://www.health.qld.gov.au/qcg/guidelinehistory), version-change record accessed 2026-10-27.
- [Normal birth guideline PDF](https://www.health.qld.gov.au/__data/assets/pdf_file/0014/142007/g-normalbirth.pdf).
- [Intrapartum fetal surveillance guideline PDF](https://www.health.qld.gov.au/__data/assets/pdf_file/0012/140043/g-ifs.pdf).
- [Instrumental vaginal birth guide](https://www.health.qld.gov.au/__data/assets/pdf_file/0033/736917/g-instrumental.pdf).
- [Queensland Clinical Guidelines history](https://www.health.qld.gov.au/qcg/guidelinehistory) records the surveillance V11 and V12 dates and is used to preserve source chronology.

## Validation record

Python commands used repository `.venv/bin/python` 3.15.1. Output hashes are SHA-256 of combined stdout/stderr.

| Command | Exit | Output SHA-256 | Result |
| --- | ---: | --- | --- |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/B0.2.qld-reference-profile.json` | 0 | `2abaac2832351140c25f21f6fe23ba23c80e9d02f9b5043cee4c5c0d13d2ebe5` | Packet and B0.1 dependency validate. |
| Packet-bound Python profile assertions (all maternal category totals 60,477; route total 61,404; explicit unconfigured states) | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | JSON parses and denominators reconcile. |
| `.venv/bin/python tools/context.py check` | 0 | `a77e6364b7072a4988ae67186985563a6993060b1347dc0a0dfcba7a0e591340` | 25 tracks, 109 milestones, 1,978 local links; no errors. |
| `.venv/bin/python tools/tasks.py check` | 0 | `ab8b9f02b3213a06d5ea7abe5c5c139318dcecab9d957cb97570eaa7024af7d3` | 401 task records; coverage, prerequisites and DAG valid. |
| `git diff --check` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | No whitespace errors. |

Checks ran at base `ed4c9cea11dfb2cff4b218530c1fde0fca1c3d5b` before implementation commit. No simulation behavior, runtime loading, clinical validation or operational acceptance is claimed.

### Reviewer-agent findings and corrections

The reviewer agent examined `a8f3e08294626ec40f78ec8924ed64047b3e740b` and found three metadata issues: unnumbered summary tables had been assigned unsupported table numbers; V11/V12 history was described as a mismatch instead of a sequence; and the instrumental-birth PDF's review date conflicted with the publication index and its document identifier was omitted. The correction packet is [`B0.2.qld-reference-profile.review-fix.json`](../../execution/packets/B0.2.qld-reference-profile.review-fix.json). Fixes are awaiting a fresh reviewer-agent assessment.

Review-fix verification ran in the isolated worktree at base `a8f3e08294626ec40f78ec8924ed64047b3e740b` using `.venv/bin/python` 3.15.1. `tools/tasks.py packet-check conductor/execution/packets/B0.2.qld-reference-profile.review-fix.json` exited 0 with `errors: []`; the packet's inline JSON source-integrity assertions exited 0; `git diff --check` exited 0 with no output. The exact commands and acceptance oracles are retained in the packet. The reviewer packet is bound to the implementation base, so a post-commit packet-check correctly reports base drift at the correction commit; the recorded pre-commit validation remains the packet result.

### Independent reviewer-agent re-review

Reviewer-agent re-review of exact correction commit `6b37893e82c343c6f7863852ce95e1e52f20c10c` found no substantive regression. It verified the official summary titles, V11/V12 chronology and current V12 PDF, and `MN24.49-V5-R29` with both review dates retained as unresolved. Profile assertions and `git diff --check` passed; no fixes were requested. B0.2 is accepted as a reference-evidence task only; no clinical validation, site-policy approval, runtime loading, or operational acceptance is claimed.
