# C2 portable checkpoint contract — coordinator proposal for owner review

**Status:** nonbinding design proposal. It is not an accepted ADR, Track22 disposition, implementation packet, or permission to add a public Rust API. Track22 and contributing owners must review the scope and choices below before codec or restore code is authorized.

## Target outcome

Meet C2 specification C-06: candidate rankings, canonical integer traces, and IDs survive repeat runs, checkpoints, and 1/2/N workers; numeric comparisons use declared tolerances. The plan leaves portable checkpoint/restart open. This proposal treats fresh-process restoration from a supported boundary as a candidate acceptance oracle for that gate, subject to Track22 and contributing-owner disposition; it is not stated as an already accepted C2.4 criterion. Existing `resume-plan` and CLI scaffold files do not demonstrate portable restoration.

## Proposed contract shape

Use a versioned outer checkpoint envelope with owner-defined state payloads. Track22 owns run orchestration, envelope completeness, compatibility preflight, durable publication, and restore coordination. State owners define their payload versions, codecs, logical references, and compatibility rules; no central encoder reaches into their private runtime fields.

Candidate envelope metadata, subject to Track22 decision:

- Envelope/schema version and complete/incomplete publication state.
- Run provenance: scenario, seed manifest, parameter/config inputs, source/build identity, and the versions/hashes owners need to reject incompatible restore.
- A named checkpoint boundary and logical simulation frontier, with scheduler/worker compatibility recorded according to Track01's contract.
- A complete set of component payload descriptors: stable owner/component key, payload schema version, content length/hash, and the logical references needed for cross-component validation.
- A final integrity value binding the envelope and payload set. Track22 must decide artifact representation, atomic publication and resource/path limits.

Track22 should define completeness rules for required, optional, unknown, duplicate, missing, and unreferenced payloads, plus byte/count/depth limits checked before allocation. Integrity hashes detect accidental changes; they do not establish authenticity. Compatibility policy must say which source, lockfile, feature, configuration, handler, and input drift is rejected or supported. Matching hashes alone do not prove semantic compatibility.

The envelope describes the checkpoint; it does not define scheduler heap encoding, RNG algorithm/state layout, component serialization, entity-ID remapping, or Flow callback rebinding.

## Restore contract to review

1. Validate the complete artifact and all cross-payload references before mutating a new runtime: envelope version, exact payload inventory, hashes/lengths, source/config/schema/plugin compatibility, resource limits, and required handlers/codecs. A failed restore must not leave a partially admitted or runnable runtime.
2. Reject unknown or missing required state, incompatible versions/topology, corrupt or partial artifacts, unregistered state, and any state an owner marks unsupported. Do not silently drop state or best-effort continue.
3. Construct a fresh process/runtime, register the exact reviewed component handlers, restore owner payloads, and run explicit owner-defined logical-ID rebinding. Never serialize or compare `FlowRuntimeIdentity`; ADR-0006 makes it process-local.
4. Capture all component state at one coherent logical instant across scheduler, Flow/world, calibration streams/ledger, and runner completion ledger. Resume only at an owner-approved boundary where callback/submission code has returned, dispatch results are observed and reconciled, and no callback batch, submission, permit, partially applied batch, or shard result is in flight. The owners must determine how the boundary is detected and how external mutable aliases are constrained or rejected. In particular, a checkpoint cannot split Flow dispatch from bridge observation.
5. Define the commit relationship between checkpoint publication and result/artifact publication. The crash oracle must cover interruption on both sides of that boundary and show that resumed candidate/replication work is neither lost nor counted twice.
6. Compare resumed and uninterrupted runs with exact equality for IDs, canonical integer traces, candidate order/budgets/seeds, and other owner-declared canonical outputs; compare numeric objectives using C0-declared tolerances. Do not choose tolerances, tie-breaking, reduction order, or worker-topology compatibility here. One fresh-process fixture cannot establish every C-06 requirement. The plan assigns worker/resume equivalence to C5, whose test matrix adds permuted completion order and crash/resume cases.

## Ownership decisions required before implementation

| Owner | Contract decision needed |
| --- | --- |
| Track22 | Supported run boundary; envelope versioning/completeness and pre-allocation limits; compatibility preflight; atomic file publication; recovery after corrupt/partial writes; checkpoint/result commit ordering and runner ledger semantics. |
| Track01 | Scheduler frontier/order state, entity allocator identity/generation, event ID/sequence preservation or a reviewed stable mapping, and engine RNG state. Decide whether restored IDs/generations remain identical or use an equivalence-preserving mapping. |
| Track03 | Flow request/work/resource/callback state, safe capture points, supported typed contexts/handlers, alias policy, and explicit runtime/work identity rebinding. |
| Track21 | Calibration provider/ledger state, seed-map/collision-registry state, each stream's logical identity/state/draw position, bridge continuation state, and cross-payload consistency oracle. |
| Track04 | Whether any persisted trace/Arrow payload participates in the checkpoint contract and the schema/version compatibility rules if so. |
| Track25 / compatibility owner | Public API, feature/MSRV, compatibility and release review for any exposed checkpoint surface. |

## Suggested first bounded sequence

1. Track22 and the state owners accept/revise this contract and record an ADR with one explicit supported C2 boundary plus a fail-closed unsupported-state matrix.
2. Bind a fixture and interruption point to exact scenario, seed/config hashes, source/toolchain, owner payload versions, and an uninterrupted-run oracle. Decide the worker-count cases and tolerances before coding.
3. Implement only the approved envelope validation/publication or first owner codec leaf. Keep the C2 gate open until the fresh-process restore and suffix-equivalence oracle pass at the integrated exact pin.
4. Complete C5 worker/resume equivalence, including permuted completion-order and mid-study resume acceptance; do not infer it from one checkpoint fixture.

## Open alternatives for owner disposition

- **Broad runtime snapshot:** captures every supported engine/Flow/calibration component. Highest coverage and cost; only viable with codecs and restore ownership across all state owners.
- **Explicitly supported subset with fail-closed rejection:** supports a reviewed C2 state subset and rejects all unknown contexts/handlers/state. Lower initial coverage; it cannot waive C-06 requirements and must have a declared scope and oracle reviewed by the owners.
- **No portable checkpoint until all state owners expose a contract:** leaves the current CLI scaffold in place and keeps C2 acceptance open. Valid if the owner review concludes cross-track restoration is not ready.

These alternatives do not choose an on-disk format or runtime serialization strategy. The owner disposition must select or replace them and bind the result to a concrete first implementation packet.

## Evidence boundary

Prepared from the C2 specification/plan, Track22 CLI/handoff, C2.5 in-memory contract, ADR-0006, and the read-only source inventory in [the Track22 owner packet](../c2-track22-checkpoint-owner-review-20261007/review-packet.md). Parent source baseline at preparation: `c0ced34ae24c10b46ea010b979767ae4a293c33a`; Kairos pin: `6303a2552257f1fcbd2aaa002d38352a2a467592`. No source/API changes, Rust commands, or tests are included in this proposal.
