# R0.2 — exact-source capability inventory

Scope: careops-sim at `40c713f483c4fbc7b2a3f8a489fbf30bd2b09051`, following independently accepted R0.1 (PR174 publication is separate from merge). This evidence inventories source and historical receipts; R0.2 acceptance is reserved to the coordinator after independent review.

## Immutable revisions and disposition

- Qualified runtime pin: `44261eb5cf8275f132f63a3396903d116d701dbe`.
- Inspected merged Kairos planning main: `55b20e77eb9ca0df5a95b9e82c4fb815d3919a87`.
- Common ancestor: `384e8546d69f9cbf2746fcb2ab646263256e6dec`. The pin is not an ancestor of the inspected candidate.
- All **321** source/evidence paths in the [reviewed pin contract](../d2.4-kairos-contract.json) match the pinned bytes. Of those paths, the candidate lacks **273**, changes **34**, and preserves **14**. These counts include evidence/docs as well as source; they are not counts of missing APIs.

[inventory.json](inventory.json) retains per-path SHA-256s, crate manifests/features/dependencies, Rust public-item inventory, test-file function names and selected symbol line readbacks. The lexical symbol list aids discovery; it does not prove API reachability, complete semantics, or runtime execution. The complete-tree searches in the command receipts check for relocated selected equivalents.

## Consumed capability and evidence map

| Capability / owner | Pin source and available evidence | Candidate source / qualification limit |
| --- | --- | --- |
| Event scheduler, identities and RNG /01,03,21,25 | core/types/rng present; core scheduler checkpoint and preview fixtures; reviewed owner CI recorded in the pin contract | Basic scheduler/RNG remain, but the reviewed SchedulerCheckpointV1 implementation/fixtures are absent. New source requires replay and compatibility qualification |
| Queue, cancellation, preemption and allocation /03 | DES flow.rs, FlowRuntime, FlowAllocationMode, callback lifecycle and portable flow checkpoint v1–v4; Q5.4 receipts | Reviewed flow/fidelity source and test targets absent, no FlowRuntime/FlowCheckpointV4 equivalent identified in whole-crate search. Resource/ConditionWait names do not satisfy the consumed lifecycle |
| Fidelity and spatial/transit adapter /03,21,64 | FidelityAdapter, flow_adapter.rs and spatial.rs with checkpoint/lineage/transit tests | Reviewed source/fixtures absent; basic ABM remains, without demonstrated consumer equivalence |
| Official Arrow IPC/Parquet and resource telemetry /04,62 | arrow-io crate has pinned official Arrow60 dependencies and ipc/parquet features; resource-lifecycle(-io), trace_time modules; C4.3 retained native/independent-reader receipts | arrow-io absent; arrow-native/parquet-native are empty features with only types dependency. Current Arrow crate provides custom smoke/contract serialization, not proof of official-format interoperability |
| Calibration/replay/holdout/recovery /21,22,66 | calibration crate with flow/ipc/parquet features; C3/C4/C5/C6 historical receipts and fixtures; shadow ledger/native/wire modules | calibration crate and its feature/test targets absent; generic ensemble names do not establish retained Q/C/ED and checkpoint equivalence |
| Condition waits versus joint resource acquisition /03,64 | No AllOf/AnyOf or claim_atomic_multi_resource public contract identified at the pin | ConditionWait::AllOf/AnyOf is a declaration with no call sites outside its definition in candidate crates. No claim_atomic_multi_resource implementation identified. Neither revision establishes atomic bundle grants from those names |
| Device / multi-host /32–35 | Existing historical evidence requires exact-source scope audit R0.3 | Planning registrations and source inventory establish no hardware or operational acceptance; R2.3 applies if consumed |

The [Q5.4](../q5.4-completion-20261005/README.md), [C4.3](../c4.3-completion-20261005/README.md), [C6.3](../c6.3-compatibility-replay-performance-20261016.md) and [C6.5](../c6.5-phase-acceptance-20261016.md) records are historical scoped evidence. They are inputs for the R1 baseline freeze, not fresh passes of current-source tests. The pin contract records exact-head owner CI on Linux and macOS; this inventory does not rerun it.

## Qualification sequence

Retain the current pin while R0.3 classifies disputed evidence and R0.4 reviews this phase. R1 must audit the actual harness and freeze ED, Q/C, physical IO and checkpoint/recovery baselines, including seed/input/source/output hashes and rollback. R2 selects a capability-preserving candidate with owners and exercises real consumer APIs and independent official readers. R3 compares against frozen baselines and records an advance-or-retain decision. No automatic upgrade to latest and no acceptance from a version, enum, metadata flag or empty feature.

## Reproduction and negative controls

The read-only [collector](collect_inventory.py) accepts only immutable 40-character revisions and a pin matching the reviewed contract. It uses git objects; the separate dirty Kairos checkout is not read as source. Run receipts and retained output checksums are in [receipts.json](receipts.json). The self-comparison control must report zero candidate differences; a mismatched reviewed pin must reject before output. No source, ref, dependency, original proposal, shared status or gitlink is mutated by the collector.

The initial lease-input attempt rejected the full contract because it exceeded the harness byte budget. The successful bounded context uses spec/plan/ownership; the collector separately binds the entire contract by SHA-256. No budget limit was widened. Python/Cargo runtime tests and hardware runs are deferred to their planned qualification tasks.
