# Implementer handoff — 17 November 2026

Proposal branch: `feat/icu-liaison-anzics-economic-psa-20261117`;
draft PR: https://github.com/edithatogo/careops-sim/pull/215 .
Base main inspected: `23553b7026bc0cccce478d4523043de292a74794`.
Conductor submodule retained at `7a5c560a4fdf5297be58594cc37527eb12790272`.
No new Cargo dependency, dependency lock, CI, or gitlink changed.

R78 public/source-only evidence is documented with links and caveats.
R79 model/fixture/code/tests are a PR candidate, NOT native accepted yet;
the remaining check/review milestone is pending. R80 local hospital profile
and R81 temporal Kairos DES remain future, blocked by proper governance,
source data and independent evaluation.

The first hosted run found path inventory mismatches and missing Conductor
track standard files/noncompliant phase IDs. These were reconciled in
subsequent commits; this note does not certify the latest CI result.
Use actual current PR head checks and logs, then fix only authorised paths.

The first model is a probabilistic parameter-uncertainty analysis of expected
annual cohort consequences, not a patient-flow simulator. It yields synthetic
expected bed-days, returns, gross/net value and P(net positive). Prospective
duration/timing, available staffed beds, capacity demand and post-ICU safety
need separate event simulation and observational validation.

Before merge: independent review, exact PR-head tests, format, clippy,
Conductor context and task catalogue, privacy and policy checks. Record
any current blockers rather than altering gates to make them pass.
