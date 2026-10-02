# ADR-0004: Generic ED duration-family status

- **Status:** Accepted at bounded method-and-gap-inventory scope (2026-10-02).
- **Decision:** **UNKNOWN — no empirical family selected.** No parametric duration family or empirical-resampling method is an ED default. P3.2 uses synthetic fixtures only.

## Context and evidence

P3.1 established deterministic known-distribution, censoring, identifiability, and joint-versus-marginal fixtures. Its coordinator acceptance is explicitly limited to synthetic fixture behavior and evidence lineage; it validates no empirical family, parameters, ranges, dependence structure, policy, or ED defaults.

P3.2 compares two synthetic candidates using a predeclared split. The empirical-resampling candidate had a selection score of 6.4227 versus 7.2629 for the lognormal candidate. It was therefore selected for the synthetic locked-test demonstration. On the locked test, the descriptive score was lower for lognormal (3.1868) than empirical resampling (4.1602). These scores are synthetic design diagnostics, not ED evidence, and the test was not used to retune or change the selection. The synthetic lognormal generating truth and empirical-resampling method remain harness fixtures only.

Report-12 (lines 20–36) describes distinct data-audit, primitive-fitting, calibration, validation-selection, locked-test, and uncertainty stages. It calls for fitting from directly observed primitive intervals, prespecified candidate families, handled censoring, identifiable fits, and uncertainty assessment. The supplied evidence does not establish real ED primitive-duration data or an ED family. No condition-labeled dataset was used, so empirical conditional-family candidates and their strata remain unassessed rather than assumed unnecessary.

## Rationale and consequences

Selecting a family or promoting synthetic parameters would assert evidence that is not present. Keep the empirical family, conditional-family applicability, and numerical ranges unknown until suitable observations and a reviewable validation record exist. Synthetic candidates may continue to exercise the harness but must not become defaults or be described as empirical ED results.

P3.3 may define sampling order, shared factors, seed purposes, and separation of stochastic variability from parameter uncertainty without deciding an empirical duration family. Any later choice must remain compatible with those contracts and preserve evidence lineage.

## Evidence required to revisit

A future proposal must include:

- Source, revision, licensing, provenance, and an independently readable evidence artifact.
- Direct primitive durations with a stated semantic boundary (work versus elapsed, transit, or queue time), units, and timestamp availability.
- Event, censor, competing-event, and missingness status, with the handling and observation window documented.
- Cohort, site, and time coverage; missingness and dependence assessment; and limits on transfer.
- Prespecified candidate families and rationale, with chronological or justified grouped fitting, selection, and locked-test splits.
- Support and tail, conditional and joint diagnostics, fit and holdout uncertainty, and a frozen comparison protocol that reports uncertainty without retuning on the locked test.
- Independent readback and coordinator review before any family or assumption is accepted.

No thresholds, family, durations, or Cairns applicability are established by this proposal. Coordinator acceptance is limited to this evidence-boundary decision and gap inventory. The empirical family decision remains UNKNOWN; no ED assumption is approved.
