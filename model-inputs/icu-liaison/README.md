# ICU liaison illustrative economic scenario (adult ICU)

This directory holds replaceable, versioned JSON inputs for a Rust-native
probabilistic cost-consequence simulation. All values other than the CCR
20.4% delay benchmark are explicitly synthetic illustrations, not CHHHS
observations, forecasts or validated treatment effects.

## Run (from repository root)

    cargo run --locked -p careops-core --example icu_liaison -- model-inputs/icu-liaison/illustrative.json > icu_liaison_output.json
    cargo test --locked -p careops-core --lib icu_liaison

The output is JSON with its input SHA-256, seed, scenario ID, distributions,
quantiles and simulated probability of non-negative net resource benefit.
Every input is editable without modifying Rust. Keep any local CHHHS profile
outside this public repository; the example rejects direct patient-record inputs.
No patient identifier or episode-level data are required for this phase.

## Interpret parameters correctly

- annual_eligible_discharges: number of live adult ICU-to-ward discharges
  actually within the proposed service scope. The example's 1,000 is synthetic.
- delayed_discharge_ge12h_rate: 0.204, from ANZICS CCR 2024/25 pooled reporting
  units. This is NOT the Cairns hospital rate.
- delayed_cases_addressable_fraction: share of qualifying delayed transfers
  for which coordination/ward support could plausibly change the delay.
- addressable_cases_improved_fraction: proportion actually shortened under the
  proposed roster; includes reach and effectiveness, so do not multiply again.
- hours_released_per_improved_case: mean INDEX ICU discharge-ready hours saved.
- unplanned_72h_readmission_risk: a NEW locally defined outcome per eligible
  live ICU discharge, not the published CCR 4.5% or CORE/APD 4.0%.
- relative_readmission_reduction: negative values allow an increase in
  readmissions because recognition/rescue may improve.
- additional_cost_per_readmission_aud: costs attributable to a later
  readmission, NET of expected care without the return. Exclude INDEX delay
  hours to prevent double counting.
- net_avoidable_cost_per_icu_day_aud: avoidable cost of an INDEX discharge
  delay day less additional ward care. Do not substitute average ICU bed cost.
- annual_incremental_service_cost_aud: full extra annual workforce and running
  costs relative to the current comparator.

Each scalar has modal value and low/high bounds, sampled using triangular
uncertainty. low=value=high locks a parameter. Public benchmarks should remain
fixed until local inputs replace them; nominal intervals are judgemental, not
confidence intervals. Status tracks public_benchmark, illustrative_assumption,
or local_input_unvalidated; source is mandatory and public benchmarks require URL.

## Model and boundary

Annual cohort expected values are calculated within each of 5,000 uncertainty
draws using the repository's pinned deterministic RNG. Reported 5th/95th
percentiles are sensitivity intervals, NOT calibrated predictive intervals.
No patient-level binomial variation, dependence, seasonality, bed-shift capacity,
post-discharge rescue benefits, mortality/QALYs, or change to ward length of
stay is modelled. Do not treat released bed-days as cash savings or automatically
as staffed capacity. This is not a Kairos discrete-event simulation.

Use actual APD/CCR timestamps, hospital movements, liaison activity, discharge
delay causes, patient costing, staffing, and capacity demand to calibrate later.
Only then consider the independent Kairos DES and CareOps costing contracts.
