# E1.4 manual native trace readback

Integrated source: `af3dd4a2e0d152ba4e3eda2caf185c3adeaf7181` (E1.3 horizon acceptance). The 23 rows below come from the retained actual native JSON records; values were read from `actual-endpoint-traces.json`, not generated from fixture expectations. Each acuity/pathway row is a separate one-patient `run_terminal` call.

## 20 acuity and pathway rows

The four timing templates repeat at acuities 1–5. Every actual row preserves its input acuity as both pathway acuity and known acuity. The individual case IDs are enumerated in the crosswalk JSON.

For every endpoint, the native test derives active work, completed LOS, first-care wait, observed elapsed and total stage queue wait from actual patient episodes; it compares denominators and frozen means, and checks interval accounting and count conservation. These are independent test reducers over runtime rows, not a native aggregate-metrics API.

| Pathway | Actual clinical intervals | Decision and physical endpoint | LOS / observed elapsed |
|---|---|---|---|
| Discharge, no diagnostics | Triage [1,3), Assessment [3,6), Treatment [6,11) | Decision at 11; discharged at 11; ED space [1,11) | Active work 10; first-care wait 0; stage-queue wait 0; LOS/observed 10 |
| Discharge, diagnostics | Triage [1,3), Assessment [3,6), Diagnostics [6,10), Treatment [10,15) | Decision at 15; discharged at 15; ED space [1,15) | Active work 14; first-care wait 0; stage-queue wait 0; LOS/observed 14 |
| Admission, no diagnostics | Triage [1,3), Assessment [3,6), Treatment [6,11) | Decision/boarding start 11; boarding [11,13); transfer [13,15); physical transfer at 15; ED space [1,15) | Active work 10; first-care/stage-queue wait 0; LOS/observed 14; offer wait 2; decision-to-departure 4 |
| Admission, diagnostics | Triage [1,3), Assessment [3,6), Diagnostics [6,10), Treatment [10,15) | Decision/boarding start 15; boarding [15,17); transfer [17,19); physical transfer at 19; ED space [1,19) | Active work 14; first-care/stage-queue wait 0; LOS/observed 18; offer wait 2; decision-to-departure 4 |

- Acuity 1: `endpoints_acuity_1_discharge_no_diagnostics, endpoints_acuity_1_discharge_diagnostics, endpoints_acuity_1_admission_no_diagnostics, endpoints_acuity_1_admission_diagnostics`; all four actual native rows match the timing and endpoint row above.
- Acuity 2: `endpoints_acuity_2_discharge_no_diagnostics, endpoints_acuity_2_discharge_diagnostics, endpoints_acuity_2_admission_no_diagnostics, endpoints_acuity_2_admission_diagnostics`; all four actual native rows match the timing and endpoint row above.
- Acuity 3: `endpoints_acuity_3_discharge_no_diagnostics, endpoints_acuity_3_discharge_diagnostics, endpoints_acuity_3_admission_no_diagnostics, endpoints_acuity_3_admission_diagnostics`; all four actual native rows match the timing and endpoint row above.
- Acuity 4: `endpoints_acuity_4_discharge_no_diagnostics, endpoints_acuity_4_discharge_diagnostics, endpoints_acuity_4_admission_no_diagnostics, endpoints_acuity_4_admission_diagnostics`; all four actual native rows match the timing and endpoint row above.
- Acuity 5: `endpoints_acuity_5_discharge_no_diagnostics, endpoints_acuity_5_discharge_diagnostics, endpoints_acuity_5_admission_no_diagnostics, endpoints_acuity_5_admission_diagnostics`; all four actual native rows match the timing and endpoint row above.

Additional endpoint controls:

- `endpoints_empty`: actual result has zero patients and zero outcome counts; observed resource busy/queue counts are zero while configured capacity remains present. The test reducer derives zero denominators and checks that the five frozen means are null; the native terminal result itself does not expose aggregate metrics.
- `endpoints_one_discharge`: acuity 3 arrives at 1, stages [1,3), [3,6), [6,11), discharge at 11; LOS and observed elapsed are 10.
- `endpoints_admission_no_offer`: acuity 3 stages [1,3), [3,6), [6,11); decision at 11; boarding remains open at horizon 30. Active work is 10; first-care and stage-queue waits are 0. Observed boarding and elapsed time are 19 and 29; no offer allocation, transfer, terminal tick, realized outcome, or completed LOS. Space remains held through horizon (observed occupancy 29).

## E0 single-stage extremes (six assertion rows)

These are actual `run_scenario` E0 projections and were not serialized as full terminal endpoint traces. Native assertions compare the runtime rows with the frozen one-stage fixture values:

- `extremes.saturated` and `extremes.saturated_permuted`: four patients; p0 [1,3), p1 [3,5), p2 [5,7); p3 is granted at 7 with zero observed work at the horizon, and the result is invariant to input permutation.
- `extremes.long_censored`: one patient starts at 1, observes six work ticks to horizon 7, has no completion tick or completed LOS.
- `extremes.long_exact_horizon`: one patient completes at tick 9 exactly at the horizon, with eight work ticks and LOS 8.
- `extremes.queued_censored`: four arrive, only p0 starts at 1 and observes one work tick; p1–p3 remain unstarted and unfinished at horizon 2.
- `extremes.zero_service_rejected`: the actual E0 call returns an error containing `work_ticks must be positive`. The fixture specifies `scheduled_events: 0`, but E0 exposes no event counter and this native test asserts the error only; zero scheduled events is not a separately observed runtime value.

## Invalid input controls (17 assertion rows)

For each of the 17 IDs in the crosswalk, the native test first runs an otherwise-valid full terminal baseline, applies the one fixture mutation, asserts `InvalidInput` contains the recorded diagnostic substring, repairs that semantic boundary, and asserts a valid full terminal result plus conservation/interval accounting. The individual error strings are not present in the saved actual trace JSON; they are test assertions. This distinction preserves the native evidence boundary.

## Seeded arrival-to-terminal composition (actual trace)

The composition test generated five seed-11 arrivals with IDs/modes/acuities preserved, bound two-stage templates, appended Diagnostics and Treatment, and ran the real terminal scheduler with capacity 5 and horizon 10. The actual result in `actual-composition-trace.txt` has `arrival-v1-b0000-m00-a5-n000000` (alpha, acuity 5) arrive at 5 and terminate at 9; `arrival-v1-b0000-m01-a4-n000000` (beta, acuity 4) arrive at 5 and terminate at 9; `arrival-v1-b0000-m01-a4-n000001` (beta, acuity 4) arrive at 6 and terminate at 10; `arrival-v1-b0000-m00-a1-n000000` (alpha, acuity 1) arrive at 9 and remain unfinished; and `arrival-v1-b0000-m01-a2-n000000` (beta, acuity 2) arrive at 9 and remain unfinished. The three terminal patients complete one-tick triage, assessment, diagnostics and treatment stages in sequence. The two late patients complete triage [9,10), then start Assessment at 10 with zero observed work and remain unfinished with no decision, realized outcome or LOS. Conservation is 5 = 3 discharged + 2 unfinished; transferred and abandoned are both 0. The test also asserts seeded repeatability and input permutation invariance.

## Optional abandonment (actual traces)

The native run emitted two actual synthetic traces, read back in `e1.3-exits-completion-20261011/manual-trace-readback.md`. Outside scenario: p1 occupies ED space [1,20) and is unfinished boarding; p2 abandons at tick 2 without acquiring space or starting clinical work. Queued scenario: p1 triage runs [1,3) then assessment starts at horizon 3 with zero work; p2 acquires space at 1, is queued for triage, then abandons at 2 and releases space; p3 arrives/acquires at 2 and receives triage at deadline 3, so the first-care grant wins with zero observed work. Counts reconcile 2 = 1 abandoned + 1 unfinished, and 3 = 1 abandoned + 2 unfinished.

## Evidence limits

This is native library/test evidence for synthetic generic profiles. The six extreme rows are E0-only projections; negative diagnostic messages are asserted but not individually printed. The current CLI remains on E0 schema 1. This readback does not qualify E-R4 warmup/uncertainty/operational utilization, MVP behavior, Cairns profiles, clinical policy, hosted execution, release, or operational acceptance.
