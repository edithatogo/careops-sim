# Ownership contract — queues and preemption

Primary upstream owner: Track 03 (`kairo-ecs-des`, coordinated `kairo-ecs-abm`,
`examples/flow/`). Local track coordinates extensions to that owner's existing
plan; it does not change its current upstream status.

| Boundary | Owner / rule |
| --- | --- |
| Scheduler order, shared handles, registry or RNG change | Track 01 review and ADR; no incidental redesign |
| Lifecycle schema/Arrow encoding | Track 04 owns implementation; DES exposes typed records without Arrow dependency |
| Runner/checkpoint orchestration | Track 22 owns manifest and resume integration |
| Fixtures/benchmarks | Track 12 owns conformance integration |
| Public API/version policy | Track 25 reviews additive surface and migration |
| Parallel/GPU/distributed | Existing 32/34/35 own backend code and acceptance |

Blocked paths absent a coordinated owner handoff: FFI/language bindings, GPU/
PDES/distributed crates, unrelated tracks and dashboard code. No clinical policies
in general core components. Planning does not authorize new remote publication.

Parallel-safe work: calibration schema design, pure metrics and Arrow ingestion
after contract review. Shared Cargo manifests, Arrow schemas and scheduler/type
files require one coordinated owner. Deliver tests and compatibility evidence
before integration; maintain spec/plan/risk/test/handoff records at each phase.


## Execution modes

This track supports serial execution or bounded parallel subagents through
[the shared execution protocol](../../execution-model.md). Workers receive one
reviewed packet, exact source hashes, write reservations, commands and behavioral
oracles. The coordinator owns shared files, integration and accepted task status.
Use [the decomposition guide](../../execution/decomposition.md) to size work for
simpler models; unresolved design decisions escalate before implementation.
