# B0.3 — Birthing reference profile contract tests

**Task:** B0.3, generic birthing suite delivery

**Status:** Accepted after reviewer-agent re-review and coordinator acceptance

**Base:** `3ddde7714e06543e4caf97fc09e8421446ee664c`

**Packet:** [`B0.3.profile-contract-tests.json`](../../execution/packets/B0.3.profile-contract-tests.json)

## Scope

The tests in [`test_birthing_profile_contract.py`](../../../tests/test_birthing_profile_contract.py) validate the version-1 Queensland public reference profile's structural shape, source provenance, unit semantics, nonnegative integer counts, denominator reconciliation and inference limits. Negative controls prove that negative counts, changed denominators, incorrect units, malformed onset/route rows, invented site staffing values and removed unknown-evidence records are rejected.

The contract is deliberately evaluated with the repository's standard-library `unittest` lane. The qualified Python 3.15.1 environment has no `jsonschema` module; this task therefore adds no third-party dependency or claim of full JSON Schema Draft conformance. These checks do not validate clinical guidance, establish local site policy, provide person-level hazards, or make the profile runnable.

Observed mother counts reconcile independently to 60,477; route counts reconcile to 61,404 babies. Profile source year (2022), policy revision dates, and prospective target-unit wording remain separate. Staffing, eligibility and operational targets remain unconfigured, while unknown correlations and missing individual-level evidence remain explicit.

## Verification record

Commands run from the repository root with `.venv/bin/python` 3.15.1 at the base above. The packet binds spec, plan, test matrix and profile SHA-256 inputs; see its `input_hashes` for exact values. Packet validation occurred before committing the implementation, while the base still matched `HEAD`.

| Command | Exit | Output SHA-256 | Result |
| --- | ---: | --- | --- |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/B0.3.profile-contract-tests.json` | 0 | `2abaac2832351140c25f21f6fe23ba23c80e9d02f9b5043cee4c5c0d13d2ebe5` | Packet schema, source hashes and accepted B0.2 prerequisite validate. |
| `.venv/bin/python -m unittest discover -s tests -p test_birthing_profile_contract.py -v` | 0 | `367c871b08b975f32a1c5821737a220c14a59967c635255813cda748402a9559` | 9 focused tests passed, including five mutation/negative-control cases. |
| `git diff --cached --check` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | Staged patch whitespace check passed. |
| `.venv/bin/python tools/context.py check` | 0 | `30d41575e2c2417d8858271c3a90d45af70f45b227c02f8a4ab0c2ba786e4e1d` | 25 tracks, 109 milestones, no link errors. |
| `.venv/bin/python tools/tasks.py check` | 0 | `ab8b9f02b3213a06d5ea7abe5c5c139318dcecab9d957cb97570eaa7024af7d3` | 401 task records; prerequisite and DAG checks pass. |

### Reviewer-agent finding and correction

The first reviewer-agent assessment of implementation commit `76d71d1` found that non-object rows in the labour-onset and route arrays were silently skipped. The implementation now emits path-specific errors for both arrays, with one negative-control test per array. The correction packet is [`B0.3.profile-contract-tests.review-fix.json`](../../execution/packets/B0.3.profile-contract-tests.review-fix.json). Fresh reviewer-agent re-review examined exact correction commit `ef543aa8b03a696fa900ace6be42d2f503d27987`, verified both rejection paths and their controls, reran all 11 tests and relevant planning checks, and reported no further actionable findings. B0.3 is accepted as input-contract verification only.

Correction checks ran at base `76d71d1485a85a27bb9445fa3d2c0106b34a085c`, Python 3.15.1. The focused test file passes 11 tests; the two malformed-row controls both pass. All correction packet inputs are hash-bound to the unchanged profile and B0 contract. The re-review of `ef543aa` is recorded above.

| Correction command | Exit | Output SHA-256 | Result |
| --- | ---: | --- | --- |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/B0.3.profile-contract-tests.review-fix.json` | 0 | `2abaac2832351140c25f21f6fe23ba23c80e9d02f9b5043cee4c5c0d13d2ebe5` | Correction packet, hashes and B0.2 dependency valid. |
| `.venv/bin/python -m unittest discover -s tests -p test_birthing_profile_contract.py -v` | 0 | `10757baa0284f07a023886446307b335b6135909945f9c241779c9b55bfbb656` | 11 tests passed. |
| `git diff --cached --check` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | Staged patch whitespace check passed. |
| `.venv/bin/python tools/context.py check` | 0 | `fe34659e376821c4219a9e4cad5dbe31c39cf5bd6422c6caf375ae2c100e3558` | 25 tracks, 109 milestones; no errors. |
| `.venv/bin/python tools/tasks.py check` | 0 | `ab8b9f02b3213a06d5ea7abe5c5c139318dcecab9d957cb97570eaa7024af7d3` | 401 task records; DAG valid. |

No simulation behavior, clinical validation, site-policy approval or operational acceptance is claimed.
