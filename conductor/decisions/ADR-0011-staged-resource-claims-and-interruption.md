# ADR-0011: Staged resource claims and safe interruption

- Status: accepted contract for A3 implementation after reviewer-agent review
- Date: 2026-10-27
- Scope: CareOps-side allocation contract against the pinned Kairos capability
  surface `44261eb5cf8275f132f63a3396903d116d701dbe`
- Does not qualify an atomic multi-resource allocator, portable checkpoint
  restore, clinical eligibility policy, or upstream Kairos acceptance.

## Context

CC-03 requires deterministic handling of compatible team, space and equipment
claims, including contention, partial failure, timeout, cancellation, rollback
and starvation. The pinned Kairos API exposes experimental single-resource
requests and leases. Each request identifies one resource; joins coordinate
events but do not make the resulting grants transactional. Its per-resource
eligibility and continuation hooks are experimental, and the pinned checkpoint
formats do not preserve all such state. The Track 03 handoff also explicitly
limits its synthetic staff/bed/cleaning example and does not claim atomic bundles
or portable checkpoint/restore.

## Decision

1. Treat each Kairos lease as a claim on exactly one resource request and its
   request revision. CareOps may coordinate a bounded set of claims, but labels
   this as **staged acquisition**, never an atomic bundle grant.
2. Use a deterministic total order for staged claims. Keep each acquired claim
   visible as provisional until all required claims are acquired. If any claim
   is refused, times out, expires, or is cancelled, release the claims already
   acquired in reverse acquisition order. The operation reaches `released`
   only after all releases are observed; otherwise retain an explicit
   `rollback_pending`/`partial` state with the residual claim identities. Never
   report success or erase a residual claim on a failed release.
3. Cancellation and expiry are idempotent requests to end staged acquisition
   and release its known claims. A late grant racing with cancellation is
   recorded and immediately included in rollback. A late completion from an
   earlier request revision cannot satisfy the current staged operation.
4. The permitted fallback is a bounded **composite resource** representing one
   preconfigured team bundle as a single resource identity. This has atomicity
   only at that composite identity and capacity limit; it does not coordinate
   independently changing member resources and must be described as a bounded
   approximation. Do not synthesize this fallback from multiple independent
   Kairos resources at dispatch time.
5. Eligibility hooks may gate a single resource grant in the pinned engine,
   but are experimental and are not CareOps clinical or jurisdictional policy.
   CareOps supplies only the currently observed eligibility result and retains
   policy ownership. A disabled grant gate is not proof that resource capacity
   or a multi-resource bundle is available.
6. The pinned engine supports preemption only at its resource arbitration event
   for work declared preemptible; it has no intra-work safe-point API. CareOps
   must represent a safe boundary by ending one work segment and submitting the
   next, or declare the entire phase non-preemptible. Queued urgency or priority
   alone never interrupts active work. Resource retention, work progress,
   continuation and resumed work are explicit state. Surgery and other declared
   non-interruptible phases retain their claims until completion.
7. Checkpoint/restart equivalence is not claimed for states involving
   experimental eligibility gates or continuation callbacks until the exact
   serialized subset is independently demonstrated. A3 tests may verify
   same-process deterministic replay; unsupported portable restore remains
   unverified.

## Required A3.2 contract cases

- Two contending staged operations never own the same exclusive resource.
- Failure, timeout-at-grant ties, cancellation, expiry, and partial release
  preserve every acquired claim until release is observed.
- Late grants join rollback; stale request revisions and stale completions have
  no effect on current state.
- Every event conserves claims; rollback order is deterministic and idempotent.
- Urgency alone does not preempt; a non-preemptible segment is never
  interrupted; segment-boundary suspension preserves declared claims and
  progress in-process.
- Starvation policy is explicit and tested; this ADR does not imply a particular
  fairness guarantee from Kairos.
- Any composite-resource fallback declares its membership, capacity bound and
  inability to reflect independent member availability.

## Consequences and limits

CareOps can safely model partial acquisition and compensation without claiming
transactional semantics absent from the pinned engine. If the use case requires
true all-or-none grants across independently contended resources, implementation
must stop and receive a separately reviewed Kairos-owned allocator design,
implementation and exact-head acceptance. This contract does not authorize
changes to the Kairos submodule or parent pin.
