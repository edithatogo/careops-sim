# Optional strategic-information and operational-policy evaluation

Date: 8 November 2026. Status: planning addition; implementation and qualification
pending. This refines R76.5 and paired-outcome/valuation handoffs under the existing
Kairos domain-contract track. It does not take over ED MVP, native-core or routing work.

## Parallel-owner reconciliation

Read the [shared reconciliation](https://github.com/edithatogo/paignia/blob/87b4bda345b85310e85239d4845b0f85bf1c6439/docs/integrations/parallel-plan-reconciliation-20261108.md),
[issue #205](https://github.com/edithatogo/careops-sim/issues/205) and
[Kairos #381](https://github.com/edithatogo/kairos/pull/381). Paignia is the actual
standalone provider; PR #2 merged its bounded core/solve seed at
`12b22ce7571b9c590405bf53c2c1c5862b49cd64`. This is not full package, consumer or
scientific qualification. Current T17.09/.11 belong to reimbursement; .10 is the
selected second application, currently Innovate. Historical issue text does not
make CareOps a required first or third consumer. CareOps remains an optional
candidate under existing R75–R77 rather than a second T17/provider catalogue.

Keep clinical pathways, operational policies, valuation assumptions and domain
validation here. Route generic clock/RNG, pathway execution and shared-resource
mechanism gaps through existing Kairos/CareOps owners; adding a game adapter must
not create a competing loop or hide those gaps behind solver diagnostic traces.
The current gitlink, locks, advisory claims and ED MVP priorities are unchanged.

Reuse Voiage O01–O08 paired inputs and optional SI qualification. Keep parameter,
model, measurement, simulation-replicate and solver identities distinct. The shared
noise control with one parameter state and A=[-1,1], B=[1,-1] replicates must not
be exported as two learnable parameter states: parameter EVPI is zero, not one.
Voiage owns the arithmetic; the application owns input truth and valuation.
This planning reference does not admit a provider pin or implement a new schema.

## Preserve ownership and independent acceptance

CareOps owns operational games, actors, workflow policies and model assumptions.
Paignia's separate Rust game packages own generic solving, UOGTO owns semantics and Kairos
owns native time/state/events/resources/RNG. Valuation providers apply explicit HEOR
assumptions; Voiage evaluates evidence and research choices. Do not put generic
solvers into careops-core or Kairos, or VOI formulas into CareOps.

R76.5 remains optional and independently pending. Preserve R76.6's runtime/valuation
fixture prerequisites and R77 acceptance; do not add game-provider availability or
strategic VOI to the native consumer or generic ED MVP release gates. A bounded
static policy game can qualify without dynamic execution. Dynamic claims require
actual native dispatch and replay, not printed imports or a Python fallback.

## Pending integration requirements

- [ ] Within R76.5, declare the selected operational game, lawful information and
  solver profile; preserve solution concept, selection policy, catalogue coverage,
  algorithm/version, residuals and failure status from qualified installed artifacts.
- [ ] Keep analyst draws, hidden operational state, actors' observations, simulation
  replications and solver runs distinct. Shared aggregate reports must not leak
  private workload/capacity or patient-level state to a policy that cannot observe it.
- [ ] Reuse paired strategy/draw contracts with explicit common-world coupling,
  weights, units, population/horizon and provenance. Equal seeds or row counts are
  insufficient. Keep activity, resource use, payments, economic costs, health and
  access/equity outcomes in separate ledgers before declared valuation.
- [ ] Feed qualified, valued outcomes to optional Voiage interfaces. Preserve signed
  actor/social values and declared welfare weights; reconcile transfers rather than
  treating every payment as resource consumption. Reject broken pairing, stale
  evidence and nonconverged/unsupported solvers instead of inserting zeroes.
- [ ] For dynamic games, preserve selected-policy/chance causal traces, native
  generational identity, lawful observations, purpose-isolated solver/exogenous RNG
  and joint policy/native replay. Solver diagnostics are not simulated service events.

## Pilot and acceptance

First use a tiny synthetic two-organisation information/shared-capacity game with
an independently enumerated reference. Compare fixed responses with supported
recomputed responses, research-only with disclosure-only interventions, and no-action
choices where actually feasible. Then bind an application pilot to the existing
CareOps programme only when its clinical/operational assumptions are reviewed.
A numerical game fixture is not evidence of real organisational behaviour, misconduct
or safe clinical practice. No private patient data enter Git or public benchmark artifacts.

Check multiple equilibria and reordered inputs, nonanticipative policies, signed
value/transfer accounting, invalid units/timebases, solver failure, deterministic
native replay and installed operation outside provider source trees. Report wrong
policy/design probability and regret, numerical/sampling/behavioural uncertainty,
resource impacts and equity assumptions. Do not tune on confirmatory cases.

The next implementation leaf must freeze exact adapter/fixture/test paths, add and
observe failing witnesses, implement minimally and retain scoped results before
broader repository/native/hosted gates. Preserve current task IDs/statuses, advisory
ownership, Kairos gitlink, locks and roadmap deferrals. No source, provider pin,
public schema, release, dataset publication or task completion is made here.

## Coordinated Voiage plan

See the [shared SI01–SI04 plan and SI-V01–SI-V08 cases](https://github.com/edithatogo/voiage/blob/c414752a228e7af5d689d9d5876f8f7372c42447/conductor/tracks/scientific_ecosystem_alignment_20261029/strategic-information-integration.md).
That source commit is a planning proposal, not a dependency pin or an acceptance receipt.
