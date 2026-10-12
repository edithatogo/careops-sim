# ADR-0009: C2 portable checkpoint — supported subset, fail closed

- **Status:** accepted (scope decision); implementation not yet started
- **Date:** 2026-10-12
- **Decider:** single maintainer, acting as Track22 owner and contributing Track01/Track21 state owners
- **Inputs:** [coordinator proposal](../evidence/c2-track22-checkpoint-contract-options-20261007/proposal.md), [Track22 owner packet](../evidence/c2-track22-checkpoint-owner-review-20261007/review-packet.md), C2 [spec](../tracks/empirical_calibration_20260925/spec.md) C-06, [plan](../tracks/empirical_calibration_20260925/plan.md) C2/C5, [test matrix](../tracks/empirical_calibration_20260925/test-matrix.md)
- **Pins at decision:** parent `98c306a977a88cce6c039a9774b7d2f2676fdf7b`, Kairos `678453838f391f3dd869b79fd502077f63f3a067`

## Decision

Choose the proposal's **explicitly supported subset with fail-closed rejection**.
The C2 portable-checkpoint gate is satisfied only by a fresh-process restore of a
reviewed C2 state subset at one supported boundary, matching an uninterrupted run.
Worker-count (1/2/N), permuted completion order, crash and mid-study resume
equivalence remain C5 acceptance (plan C5 exit; test-matrix
`calibration_worker_resume`). This allocation is what the plan already states; it
is not a waiver of C-06.

Rejected alternatives: a broad runtime snapshot (needs codecs from Tracks 01/03/04/21
including per-process type-erased handlers; slowest), and no portable checkpoint
(leaves C2 open indefinitely).

## Supported boundary

Between events, after `run` returns, with no callback batch, submission, permit,
partially applied batch or shard result in flight and dispatch already reconciled.
Flow work is empty or a single idle-queued bridge work item. Active or suspended
transit, unknown components/contexts/handlers and unregistered streams are rejected
before any runtime mutation. `FlowRuntimeIdentity` is never serialized (ADR-0006);
logical IDs are rebound explicitly.

## Subset state and owners

| Component | Owner | Starting point at pin |
| --- | --- | --- |
| Calibration stream snapshot | Track21 | in-memory `CalibrationStreamSnapshot` + versioned `restore()`; needs byte codec |
| Seed-collision registry | Track21 | no snapshot; new |
| Fidelity policy/decisions + bridge work item | Track21/Track03 | C2.5 in-memory contract; needs codec + rebind |
| Scheduler frontier (time, priority, sequence, event IDs, cancelled set) | Track01 | none |
| Entity allocator generations, engine RNG | Track01 | none |
| Flow requests/leases/resources | Track03 | must be empty/idle; otherwise rejected |
| Envelope, completeness, publication, runner ledger | Track22 | CLI scaffold only |

Transit progress (`TransitProgressCheckpoint`, crate-private) is a second leaf tied
to test-matrix interrupted-edge restore, not part of the first leaf.

## First acceptance oracle

Reuse the C2.1 paired Macro/explicit-zero Micro deterministic fixture. Run to event
*k*, publish the envelope, exit; restore in a new process and continue. Require exact
equality of event IDs, canonical integer trace hash and Service draw positions with
the uninterrupted run. Negative cases: corrupt, truncated and unknown payloads, and
an active-work boundary, each rejected before mutation. Bind to Kairos pin,
toolchain, seed and config hashes.

## Not decided here

On-disk format, serialization library (no serde on des/rng/calibration/abm types at
the pin), tolerances, public API exposure (Track25 review required) and the
crash/commit-ordering details, which the first implementation packet must bind.
