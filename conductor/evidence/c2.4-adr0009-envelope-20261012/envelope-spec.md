# ADR-0009 first-leaf envelope spec fragment (prepared, not implemented)

Status: prepared worker spec fragment for coordinator review. No Rust source changed.
Packet: `C2.adr0009-envelope` (task C2.4). Base: parent `6bdad961`, Kairos `6784538`.
Contract: `conductor/decisions/ADR-0009-c2-portable-checkpoint-subset.md` (SHA-256 `85f03da8...`).

## 1. Supported boundary (ADR-0009)

Between events, after `run` returns, with no callback batch, submission, permit,
partially applied batch, or shard result in flight and dispatch already reconciled.
Flow work is empty or a single idle-queued bridge work item. Active or suspended
transit, unknown components/contexts/handlers, and unregistered streams are rejected
before any runtime mutation. `FlowRuntimeIdentity` is never serialized (ADR-0006);
logical IDs are rebound explicitly.

## 2. Envelope fields (Track22 owns versioning/completeness/publication)

- Envelope/schema version; complete/incomplete publication state.
- Run provenance: scenario, seed manifest, parameter/config inputs, source/build identity.
- Named checkpoint boundary + logical simulation frontier; scheduler/worker compatibility per Track01.
- Complete component payload descriptors: owner/component key, payload schema version, length/hash, logical references for cross-component validation.
- Final integrity value binding envelope + payload set.
- Byte/count/depth limits checked before allocation (Track22).

## 3. Payload inventory (first leaf)

| Component | Owner | Requirement |
| --- | --- | --- |
| Calibration stream snapshot | Track21 | versioned payload + codec |
| Seed-collision registry | Track21 | new snapshot interface required |
| Fidelity policy/decisions + bridge work item | Track21/Track03 | codec + explicit rebind |
| Scheduler frontier (time, priority, sequence, event IDs, cancelled set) | Track01 | full state required |
| Entity allocator generations, engine RNG | Track01 | full state required |
| Flow requests/leases/resources | Track03 | must be empty/idle, else reject |
| Envelope, completeness, publication, runner ledger | Track22 | validation + atomic publication |

Transit progress checkpoint is a second leaf (test-matrix interrupted-edge restore), not this leaf.

## 4. Validation order (fail closed, before any runtime mutation)

1. Envelope version supported; exact payload inventory present.
2. Hashes/lengths match; source/config/schema/plugin compatibility passes.
3. Resource limits pass before allocation.
4. Required handlers/codecs registered for every payload.
5. Boundary check: zero in-flight work per section 1; Flow idle condition holds.
6. Reject on: unknown/missing required state, incompatible versions/topology, corrupt/partial artifacts, unregistered state, owner-marked-unsupported state. Never silently drop or best-effort continue.

## 5. Restore oracle (first acceptance)

Fixture: C2.1 paired Macro/explicit-zero Micro deterministic fixture.
Procedure: run to event k, publish envelope, exit; restore in a new process and continue.
Require exact equality: event IDs, canonical integer trace hash, Service draw positions vs uninterrupted run.
Negative cases (each rejected before mutation): corrupt payload, truncated payload, unknown payload, active-work boundary.
Bind: Kairos pin, toolchain (Rust 1.99.0), seed and config hashes.
Worker-count (1/2/N), permuted completion order, crash and mid-study resume remain C5 (`calibration_worker_resume`).

## 6. Explicit non-goals for this leaf

No on-disk format selection, no serialization library choice (no serde on des/rng/calibration/abm types at pin),
no tolerances, no public API exposure (Track25 review required), no crash/commit-ordering details.
No C2/C2.4 phase, hosted, release, or clinical acceptance claim.
