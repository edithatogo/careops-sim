# E1 behavior-first fixture contract v1

Coordinator frozen 2026-10-11. Synthetic oracle specification only; no native
pathway runtime or clinical acceptance. See ADR-0010. Rust execution remains
1.99.0 only; Python standard-library tests independently check fixture arithmetic.
This is a test fixture format, not a public application schema or runtime API.

## Shared semantics

All times are integer ticks. No clinical conversion, empirical acuity mix,
stochastic fit, warm-up or default operational parameters are implied. Acuity
labels are synthetic integers 1..5, not validated ATS performance standards.
Patient identifiers are unique and stable. Inputs arrive in [0,horizon); stage
work intervals are half-open, completion exactly at horizon is included. Positive
service durations only: zero duration is explicitly rejected before scheduling.
Finite ordered stages: triage, assessment, optional diagnostics, treatment.
No cycles or implicit parallel patient attention. One independent resource per
stage; capacity is a positive integer. FCFS ties use arrival/readiness then
patient ID. No acuity priority is asserted by these synthetic fixtures.

Discharge becomes terminal only at actual discharge after treatment. Admission
intent is provisional: patient remains boarding after decision until compatible
external bed offer and actual transfer completion. Offers are finite, consumed
once, valid on [available_tick,withdrawn_tick); no ward model is implied. Fixtures
use a single offer when testing a single admission; transfer duration is positive.
No offer means unfinished boarding at horizon, not admitted. E1.3 must preserve
occupancy until actual transfer and recurrent boarder care; these full behaviors
are runtime debt, not implemented by fixture validation. Distinguish declared
latent condition, agent knowledge, provisional disposition and realized outcome.

## Format and metrics

Each family JSON has fixture_version=1, family, provenance='synthetic_test_only',
time_unit='tick', runtime_status='unbound', cases. Case IDs are prefixed by family.
Every case has case_id, inputs, expected, runtime_owner (E1.2 or E1.3), and an
explicit purpose. Family-specific inputs/expected fields must be documented in
the test module and strictly validated. Expected values must be recomputed from
inputs and checked against independently hand-fixed case assertions; simply
comparing a fixture to itself is insufficient. Mutation probes must reject
changed durations, endpoints, counts, identities and relevant validation inputs.

Arrivals = terminal + unfinished, with each arrived ID represented exactly once.
Started <= arrivals. Unfinished includes queued, active or boarding and has no
realized terminal tick/outcome. First-care wait = first stage start - arrival.
Total stage queue wait = sum(stage start - preceding stage end), first predecessor
is arrival. Work = sum completed/observed active stage intervals, excluding waits
and boarding/transfer; unfinished work is clipped at horizon. Completed LOS =
actual terminal tick - arrival; unfinished LOS is null (separate observed elapsed
may equal horizon-arrival). Boarding and transfer are separate intervals. Counts
are counts, not rates; completed LOS denominator excludes unfinished patients;
wait denominator includes patients with a first start; zero denominator => null.
Throughput is terminal count in the stated cohort. No uncertainty claim.

## Required independent families

- endpoints: empty run; one discharged patient; full synthetic acuity1..5 x
  discharge/admission x diagnostics absent/present (20 independent one-patient
  cases); no-offer admission remains unfinished. Triage2, assessment3,
  diagnostics4 when present, treatment5 ticks, arrival1; admission decision at
  treatment end, offer available2 ticks later, transfer2 ticks. Horizon30.
  Empty metrics must have zero counts and null undefined averages.
- extremes: capacity1, four same-time arrivals at1, single positive service2,
  horizon7 => starts1,3,5,7; completions3,5,7,null. Long work8 at1/horizon7
  unfinished; horizon9 complete. Include queued censoring (horizon2, same four),
  exact-horizon completion, zero-service rejection and deterministic input-order
  permutation. Single-stage analytic projections explicitly exclude full pathway
  runtime coverage. Their contract is also expressible by current E0.
- invalid: zero/unknown/inconsistent capacity, unresolved resource/task,
  incompatible resource class, missing eligibility, invalid acuity, repeated or
  disordered stages, duplicate patient IDs, nonpositive horizon, arrival at horizon,
  zero duration. Each has exact invalid payload/mutation, invariant, diagnostic
  category and native owner. Model validation must precede mutable execution;
  fixture tests alone cannot prove transactional runtime behavior.

## Acceptance boundary and runtime debt

Tests must detect independent mutations, missing required cases, wrong units,
changed provenance and false runtime_status. E1.1 accepts specifications only.
E1.2 owns arrivals/stage/acuity/diagnostic native binding; E1.3 owns actual terminal,
boarding/transfer binding. E1.4 cannot close with any fixture unbound, skipped,
ignored, unexpectedly failing or satisfied by an oracle masquerading as runtime.
Keep E0 schema1 compatibility. Freeze application schema/API separately at E1.2.
