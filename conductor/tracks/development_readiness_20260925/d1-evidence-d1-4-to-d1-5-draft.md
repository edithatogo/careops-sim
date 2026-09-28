# D1.4–D1.5 evidence reconciliation draft

Status: `ready_for_review` (worker proposal; coordinator acceptance remains required).

## Reconciled phase state

The bound `conductor/current-state.json` says D1.1–D1.4 are accepted, D1.5 is closed locally after receipt, drift, recovery and ownership work, and D1.6 phase review is next. The D1 plan excerpt agrees: D1.4 and D1.5 are checked, D1.6 remains unchecked, and D1 remains open until D1.6 review. The next action is therefore D1.6 review; this draft does not accept D1.6 or close D1.

The ownership prerequisite receipt in the packet records the expected `boundary-fixture` instance accepted, with artifact hashes matching the packet-bound ownership contract and implementation evidence. This satisfies the prerequisite for this bounded reconciliation only; the coordinator retains parent acceptance authority.

## D1.4 pilot evidence and limits

The supervised synthetic pilot reports baseline 6/7 and skill-assisted 7/7 over seven unique cases, with known-bad acceptance 0/12. The determinism candidate improved on one case (0/1 to 1/1); calibration (4/4) and dependency (2/2) tied. All three candidates remain `unpromoted` because the small pilot does not establish repeatable task-class benefit. Repeated DET-01 is excluded from independent-case denominators while its serial/parallel outcome was preserved.

This was context-level separation on a shared host and filesystem, not technical oracle isolation. No actual repository patch, CI run, clinical data, or implementation was evaluated. Exact response sizes, tool cost/latency and model build identity were unavailable in the bound evidence; record them as unknown. The evidence makes no hidden-evaluation claim. A follow-on needs more independently written variants, a second matched serial/parallel case per class, response-size and reviewer-intervention measures, model build identity where available, and technical isolation before any hidden-evaluation claim.

## D1.5 implementation and limits

The receipt validator, dispatch/acceptance drift verifier, synthetic recovery fixture, and ownership/reservation boundary evidence are recorded as locally integrated/accepted in their evidence summaries. The evidence supports metadata-only receipt validation, packet/input/output drift checks, synthetic kill/restart and stale-context behavior, and planner-level path reservation rejection under the single-coordinator protocol.

These results do not demonstrate atomic multi-coordinator leasing, real-process recovery or external-side-effect idempotency, coordinator identity, or shared-host hidden-oracle isolation. Hidden evaluation remains off. D1.5's local closure and the ownership prerequisite do not establish hosted CI or Windows verification; both remain unverified until the D2 GitHub gate. D2 is not pulled into this phase, and the packet does not authorize hosted setup or release claims.

## Coordinator review points

Confirm the local status/next-action reconciliation against the integrated source, retain the pilot and recovery/ownership limitations above, and keep hosted Actions/Windows as unverified pending D2. This is a bounded review draft only; no plan, current-state, or acceptance record was changed.
