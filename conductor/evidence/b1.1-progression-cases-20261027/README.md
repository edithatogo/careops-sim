# B1.1 — Synthetic birthing progression case oracles

**Task:** B1.1, generic birthing suite delivery

**Status:** Accepted synthetic case-oracle contract after reviewer-agent review and coordinator acceptance

**Base:** `0af509ba163cda925533f0b0f25f463dff1cdffd`

**Packet:** [`B1.1.progression-cases.json`](../../execution/packets/B1.1.progression-cases.json)

## Scope

[`b1-progression-cases.synthetic.json`](../../../model-inputs/birthing/fixtures/b1-progression-cases.synthetic.json) defines deterministic expected traces for nominal spontaneous, induction and protracted pathways. Each event records synthetic simulation tick, stable priority/id, patient state and service state. The traces cover labour progression during a suite/assessment wait and during transit, induction/intervention decisions distinct from progression and deterioration, deterioration while waiting or travelling, and invalidation of a scheduled progression event after a state change.

[`test_birthing_progression_cases.py`](../../../tests/test_birthing_progression_cases.py) checks the fixture shape, scenario coverage, event identity/order, wait/transit expectations and negative controls. The fixture uses no fitted distributions, clinical thresholds, hazard rates, probabilities, target times or site defaults.

## Verification boundary

These tests validate the **case oracle fixture**, not an executable birthing model. The `careops-birthing` crate does not yet exist. B1.2 must bind these cases to actual patient/service state and event APIs, demonstrate the failing cases against the initial runtime, then implement until the same cases pass. Do not mark this case-corpus check as clinical behavior or runtime acceptance.

| Command | Result |
| --- | --- |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/B1.1.progression-cases.json` | exit 0; accepted A5.4/B0.4 prerequisites and hashes valid; `2abaac2832351140c25f21f6fe23ba23c80e9d02f9b5043cee4c5c0d13d2ebe5`. |
| `.venv/bin/python -m unittest discover -s tests -p test_birthing_progression_cases.py -v` | exit 0; 8 fixture-contract tests passed; `a5cfc077387319a6dbcb350372c45d5e340ca0d1ee1f9337d771a8b063491bdd`. |
| `git diff --cached --check` | exit 0; no whitespace errors; `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`. |
| `.venv/bin/python tools/tasks.py check` | exit 0; 401 task records and valid DAG; `ab8b9f02b3213a06d5ea7abe5c5c139318dcecab9d957cb97570eaa7024af7d3`. |
| `.venv/bin/python tools/context.py check` | exit 0; 25 tracks, 109 milestones, no link errors; `8dcaa8068b15cbf2ad929b18ab1092b01ea25e8a9278c48796bea7081ec4bbb7`. |

No patient-level evidence, hazard model, clinical prediction, site applicability, operational claim or birthing runtime is introduced by B1.1.

## Reviewer-agent assessment

Reviewer-agent `/root/b11_cases_review` reviewed exact commit `6cf1a2f275fddd16a0cb10572c2bf9a1d90e74e1`, reran the focused suite (8/8) and Conductor task/context checks, and recommended accepting B1.1 strictly as the synthetic case-oracle task. Its low-severity follow-up is carried to B1.2: tests must schedule the stale progression event and prove it is rejected after deterioration changes state. The agent did not treat the fixture-contract tests as runtime evidence.
