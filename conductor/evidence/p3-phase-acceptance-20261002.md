# P3 phase acceptance — 2026-10-02

**Disposition proposed for independent closeout:** accept P3 at bounded synthetic
sampling/uncertainty/interval-accounting scope, with empirical ED parameters still
UNKNOWN. This closes the phase method contract, not the evidence acquisition or
runtime profile gates in P4/P5.

## Evidence reviewed

- P3.1 coordinator acceptance: synthetic known-distribution, dependence,
  censoring, identifiability and joint-versus-marginal fixtures.
- P3.2 coordinator acceptance and ADR-0004: reproducible synthetic candidate fit
  and holdout method checks; no candidate winner or empirical ED family selected.
  The full test suite includes the P3.2 fitting/holdout tests.
- P3.3 coordinator acceptance: all three leaf instances (sampling, uncertainty,
  DES/ABM join) accepted after independent reviews with artifact SHA-256 values.
- P3.3 phase evidence reconciliation records source commit, toolchains, exact
  submodule pins, command results, output log hash and the known evidence boundary.

## Phase exit checks

- **One fit:** the P3.2 test suite reproducibly exercises the synthetic lognormal
  MLE versus empirical-resampling candidate comparison and the locked holdout
  check; outputs remain explicitly synthetic and do not select an ED family.
- **One conditional draw:** the P3.3 sampling oracle performs an inverse-CDF
  categorical draw with invented mode-specific probabilities and fixed uniform
  variates, including category boundaries and a near-one rounding case.
  Unknown/missing mode defers; future diagnosis/disposition mutation cannot alter
  the early route.
- **No transit/service double count:** the P3.3 join fixture reconciles 10
  invented elapsed units as active work 3, setup 1, transit 2 and wait 4. The
  active-work total remains 3; staff transit on a separate actor timeline and
  censored intervals cannot be added as completed patient service.
- **Uncertainty/range types:** within-run, parameter, data and structural
  uncertainty are distinct; hard limits, observed ranges, scenario ranges,
  uncertainty intervals and calibration-search bounds have distinct meanings and
  provenance. No empirical bounds were added.

## Executed gates

Working directory: `/private/tmp/careops-sim-p33`; phase evidence base was
`1485d2fe3a2278ce967a68d2d5103645a4f63d63`.

- `python3 -m unittest discover -s tests -v` — 296 passed, exit 0; log
  `.artifacts/p3-final-full-unittest.log`, SHA-256
  `0d9be4a065d4dd1212a325393412f10a055714c96bac8c0e594d1ab451ab51d2`.
- `python3 tools/context.py check` — 5 tracks, 34 milestones, 535 links,
  `errors: []`.
- `python3 tools/tasks.py check` — 143 records; DAG valid.
- `python3 tools/mvp.py check` — 80 parents, 244 leaves; joins/hashes valid.
- P3.4 closeout packet check — `errors: []`.
- Python 3.14.7; rustc/cargo 1.98.1. No native Kairos test was needed for this
  parent-only contract work; the exact Kairos submodule pin was hydrated locally.

## Limits retained

ADR-0004 remains authoritative: empirical ED duration family, conditional
applicability, numerical ranges, parameter uncertainty and defaults remain
UNKNOWN. No empirical ED data, priors, clinical policy, Kairos runtime sampler,
validated generic profile, or Cairns transfer is accepted. P4 example packs and
P5 actual-model coverage/load tests remain open.
