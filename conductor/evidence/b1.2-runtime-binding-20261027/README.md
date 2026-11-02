# B1.2 runtime binding — candidate evidence

Status: accepted local synthetic runtime conformance at reviewed integrated commit `700341336dec51e4aed774065dc3c0a5d9f282e9`.

## Scope and boundary

Added the domain-owned `careops-birthing` crate. It composes the accepted `careops-core` event ordering, stale-generation validation, competing-risk sampler and named RNG streams. Patient and service states are separate. Fixture string event/person IDs map to numeric runtime IDs by lexicographic rank. Progression events are generation-bound; service callbacks remain eligible after patient deterioration. Episodes censor at an inclusive synthetic tick horizon.

All event ticks are synthetic. The only nonzero hazard in tests is an intentionally extreme synthetic control used to make an event certain. No clinical rate, threshold, target, site default, empirical validity, clinical prediction or operational acceptance is asserted.

## Red-to-green record

- Base: `a6e8d8f1d4550d8400d811d6bd10991c9a0a2b36`.
- Expected red before runtime implementation: `cargo test -p careops-birthing --test runtime_conformance`; exit 101 because the requested `BirthingEpisode`, `EventEffect`, `PatientState`, and `ServiceState` APIs were not implemented. The test target existed and compiled to an unresolved-import failure.
- Green after initial implementation: `cargo test --locked -p careops-birthing`; exit 0, four runtime conformance tests passed.
- Hazard-reschedule regression red before correction: `cargo test -p careops-birthing --test runtime_conformance updated_hazard_invalidates_the_old_progression_before_resampling`; exit 101 because the old event remained eligible and invalidated the replacement.
- Green after correction: the same focused command; exit 0, one test passed. The full `cargo test --locked -p careops-birthing` then passed all five runtime conformance tests.
- Core compatibility: `cargo test --locked -p careops-core --test pathway_failing_cases_tests`; exit 0, 13 tests passed.
- Formatting: `cargo fmt --all -- --check`; exit 0.
- `git diff --check`; exit 0.

The runtime conformance tests execute nominal, induction and protracted B1.1 event traces; pre-schedule the named stale progression before deterioration and prove it is rejected when popped; verify a shuffled tie-order control across tick/priority/event/person keys; verify hazard changes use only the progression stream; and verify an event after the censoring horizon does not execute.

## Reproducibility

- Rust toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`.
- Simulation seed: `41` for fixture traces, `77` for hazard/RNG separation, `4` for horizon censoring.
- B1.1 fixture SHA-256: `ba6e039a32ca3c5499a1fdc5e79008a061ac23e005b8ab72b59b4894429f8e45`.
- Packet SHA-256: `47364964b6a58c7c68a09de4ae20cb3c4c1606a37ee108a0f5e187a21946e913`.

- `crates/careops-birthing/src/lib.rs`: `a6f888aeaef762dffac2c86f2905b49a6c792c464919f312cbff160474fbc006`.
- `crates/careops-birthing/tests/runtime_conformance.rs`: `7a91f1763a668753969ef0e01ea5163bbd960c5f0eafb02cbc8929ffc4e820fc`.
- `Cargo.toml`: `6cd0fb6ef6cd21028b203a73579b06ffb7a4309764d299bf03fab6c2f77e4e4b`.
- `Cargo.lock`: `b21dc623039d7cc0f1598aec88f1dc1dbbbc11b74fda284f26bd07bb14310315`.

## Review and acceptance

A reviewer-agent independently reviewed commit `5ce4036f742e9af5f814d38fcf68399f73bd34c7`, found one P1 hazard-rescheduling defect, and requested a fix. The coordinator added a red-to-green regression and corrected the reschedule path in commit `700341336dec51e4aed774065dc3c0a5d9f282e9`. Fresh reviewer-agent review of that integrated commit passed with no remaining findings. Coordinator acceptance is limited to this local synthetic runtime contract.

## Limitations

This is local synthetic contract conformance, not empirical calibration, clinical validation, site policy acceptance, runtime integration with ED, or operational approval. Current case execution is single-episode and discrete-tick; broader trajectory validation remains B1.3.

## Reviewer-agent finding and correction

Reviewer-agent reviewed integrated commit `5ce4036f742e9af5f814d38fcf68399f73bd34c7` and found that a new hazard sample could leave an older progression event eligible. The new regression failed before correction (focused test exit 101): the old event executed and invalidated the replacement. `sample_progression_tick` now invalidates queued progression generations before replacement sampling. The regression then passed, recording the old event stale and applying only the replacement event. Full focused crate suite passes 5 tests; core pathway compatibility passes 13 tests. Fresh reviewer-agent review of that integrated commit passed with no remaining findings at commit `700341336dec51e4aed774065dc3c0a5d9f282e9`; coordinator acceptance is recorded at `5135a26f0729926cfe33f31ad8e8429764a2bd22`.
