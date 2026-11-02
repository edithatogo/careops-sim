# ADR-0014: Person, episode and relationship identity

- Status: Proposed contract for A5.2–A5.3 implementation
- Date: 2026-10-27
- Track: `careops_core_domain_abstractions_20261020`
- Supersedes: none
- Related: CC-01 and CC-07 in `conductor/design/composition-contracts-20261020.md`; [ADR-0012](ADR-0012-transfer-lifecycle-and-conservation.md)

## Context

CareOps needs longitudinal people, multiple service episodes, births and independently progressing maternal/infant entities. ECS handles and entity storage are implementation details: they may be recycled and cannot stand in for persistent identities. A parent/child link describes a relationship; it does not define ownership or destruction. This decision freezes the behavior A5.2–A5.3 must implement without claiming any new runtime or Kairos capability.

## Decision

### Identity namespaces and cardinality

1. `PersonId` identifies one longitudinal person for the entire simulation run. It is immutable, globally unique within the run, never recycled, and never derived from an ECS slot/generation handle. A stale handle cannot resolve to a different `PersonId`.
2. `EpisodeId` is a distinct, immutable identifier globally unique within the run. Each episode belongs to exactly one person. A person has zero or more episodes, including overlapping episodes across services. Referral, encounter, admission, transfer operation, birth event, spawn event, and ECS handle are separate typed namespaces and are never converted into `PersonId` or `EpisodeId` by cast or value reuse.
3. Readmission reuses the existing `PersonId` and allocates a new `EpisodeId`; closed episode history remains queryable. Concurrent episodes for one person are permitted but do not imply duplicate people or multiple physical locations. Physical location and transfer authority remain governed by CC-02/ADR-0012 and the A4 transfer model.
4. Every relationship has an explicit typed edge and endpoints. `BirthParentOf` is a directed, one-to-many relation: one parent can have multiple distinct infant persons, and an infant may have multiple recorded birth parents where a consumer explicitly models them. It is acyclic and cannot self-reference. Cycle rejection is atomic. No global acyclicity constraint is imposed on unrelated edge types unless their own contract requires it.
5. Relationship deletion/closure changes only that edge. It does not delete either person, an episode, event history, or another edge. Episode closure, service exit and person death are separate lifecycle events. Entry/exit events do not create or destroy longitudinal persons. Death is explicit and terminal for new activity; identity and historical relationships remain addressable. The contract makes no clinical determination about death or survival.

### Creation and births

6. Creation is keyed by a typed stable `SpawnEventId`. Exact replay with the same canonical payload is idempotent and produces no duplicate person, episode, relationship or accounting event. Reuse of that event ID with different payload rejects atomically. A different spawn event cannot claim an already-created `PersonId`.
7. A `BirthEventId` is distinct from every person/episode/event namespace. One birth event may link to one or multiple distinct newborn `PersonId`s. Operative versus non-operative delivery does not change identity/cardinality rules. A5.3 owns fetal-to-post-birth identity mapping, birth-message de-duplication and split destinations; this ADR does not prescribe clinical terminology or delivery workflow.
8. Person creation/birth, episode entry/closure, service exit and death are explicit, uniquely identified lifecycle facts. Accounting applies each fact once; retries do not duplicate counts. People, episodes, links, resource claims and physical occupancy are conserved independently. A parent link or episode close is never a substitute for a death event.

### Determinism, streams and checkpoints

9. Identity-affecting commands are applied in a canonical total order derived only from logical simulation time, a versioned event-kind ordering and stable typed event ID. Worker index, message arrival order, ECS traversal and hash-map iteration never break ties. A stable creation event ID submitted after its logical time has been sealed is rejected atomically: it creates no identity, relationship, lifecycle or accounting fact, does not mark the event ID as consumed, and consumes no RNG draws. Recovery that replays from a checkpoint before the affected time is a separate explicit operation over the canonical full event set; it is not an alternate per-command acceptance path. Competing claims to one PersonId are resolved by this canonical order, so every permutation of the same event set has the same accepted/rejected events and final state.
10. Logical identity is generated from a stable run namespace and the logical creation event, not worker index, ECS traversal order, hash-map iteration, scheduling race or storage handle. For a fixed seed and event history, IDs and relationship results are invariant to worker count and input traversal order. Rejected or retried commands do not perturb unrelated model RNG streams.
11. Random draws belong to explicit stable module/person/episode/process namespaces. A newborn's stream is independent of the maternal stream; creating or resolving a relationship does not consume a clinical draw. This ADR does not choose or alter the RNG algorithm.
12. A supported checkpoint round-trip must retain all identity namespaces, relationships, lifecycle facts, spawn/birth deduplication keys and deterministic allocation state needed for future IDs. Resuming from a checkpoint and replaying the same suffix must match uninterrupted execution. This is a CareOps behavior requirement only; it does not assert portable Kairos checkpoint support.

## Required implementation oracles

The machine-readable matrix at [A5.1 identity/cardinality cases](../evidence/a5.1-identity-contract-20261027/identity-cardinality-cases.json) defines positive and negative cases for A5.2–A5.3. Each rejection must preserve the complete pre-command identity, relationship, lifecycle, accounting and RNG state. Runtime implementations must add executable tests against the actual APIs; the matrix itself is not runtime proof.

## Consequences and boundaries

- A5.2 may select concrete CareOps types and storage, but must preserve these cardinalities and atomicity rules in both neutral and clinical adapters.
- A5.3 supplies fetal mapping, duplicate birth-message and split-destination behavior.
- The existing A4 transfer operation and single-active-transfer rule are not widened by this ADR.
- No public API stability, clinical validation, cross-process delivery guarantee, or portable Kairos checkpoint claim follows from this contract.
