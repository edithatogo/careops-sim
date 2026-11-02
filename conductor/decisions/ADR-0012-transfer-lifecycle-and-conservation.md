# ADR-0012: Versioned transfer lifecycle and conservation

- Status: accepted for A4.2 implementation after reviewer-agent review
- Date: 2026-10-27
- Scope: CareOps transfer operations and the CC-02 behavioral oracle
- Does not qualify clinical transfer eligibility, local transport policy, remote
  delivery guarantees, cross-process networking, or a Kairos-owned API.

## Context

CC-02 requires a transfer operation identity, acceptance and reservation,
expiry, retry idempotency, authoritative ownership, explicit resource-release
points and compensation. A transfer is not complete when a message is accepted,
a vehicle departs, or the patient arrives. The person may be physically at a
new location while the originating service remains responsible and retains
resource claims until explicit handover.

The existing `contract_negative_tests.rs` uses local test-only stand-ins. Those
checks catch a few invalid state jumps and early release mistakes, but do not
exercise a runtime transfer record, conservation across every event, message
reordering, deadline ties, or compensation. A4.2 must bind executable tests to
the contract below rather than treat the stand-ins as transfer implementation.

## Decision

### Operation identity and envelope

1. Each logical transfer has a stable `operation_id`, `person_id`, `episode_id`,
   origin service, destination service, monotonically increasing operation
   revision, and schema version. Reuse of an operation ID with different person,
   episode, origin, or destination is rejected without state change. A fallback
   or compensation is a new operation with its own ID and an explicit link to
   the operation it compensates; it never creates another person or episode.
2. Each command/acknowledgement has a stable message ID and carries its expected
   operation revision. Applying an exact repeated message ID and payload returns
   the prior logical result without another state or ledger effect. Reusing a
   message ID with different content rejects atomically. A stale revision cannot
   rewind state; a future revision is held pending its missing predecessor and
   has no effect until causal order is restored.
3. The schema version is explicit. An unsupported version or malformed envelope
   fails closed before changing operation state, ownership, location, capacity,
   or resource claims. Time is the simulation's integer virtual tick; no host
   wall clock is read. The scheduler's same-tick sequence is retained for replay.
4. This specifies logical idempotency at the authoritative CareOps ledger. It
   does not promise exactly-once message delivery or network transport.

### Lifecycle state machine

The named states are `Requested`, `Offered`, `Reserved`, `Departing`,
`InTransit`, `Arrived`, `HandoverComplete`, `Rejected`, `Cancelled` and
`Expired`. `Departing` means a transport/pickup has committed at the origin but
physical departure has not yet occurred. `InTransit` begins at actual physical
departure.

| Current state | Allowed next state | Meaning and side effects |
| --- | --- | --- |
| Requested | Offered, Rejected, Cancelled, Expired | No destination capacity is held; request-level expiry leaves origin claims unchanged. |
| Offered | Reserved, Rejected, Cancelled, Expired | An offer is not a reservation. |
| Reserved | Departing, Rejected, Cancelled, Expired | Destination capacity is held exclusively. |
| Departing | InTransit, Cancelled, Expired | The patient remains at origin until actual departure; a pre-departure cancel/expiry releases dispatch and destination claims while origin claims remain. |
| InTransit | Arrived | Convert the existing destination in-flight hold to occupied capacity atomically; origin responsibility and origin claims remain. |
| Arrived | HandoverComplete | Patient is physically at destination; responsibility remains with origin until accepted handover. |
| HandoverComplete, Rejected, Cancelled, Expired | none | Terminal for this operation; only an exact duplicate message is idempotently acknowledged. |

No unlisted transition is inferred. Rejection, cancellation and expiry are
terminal only before physical departure. Expiry is supported from Requested,
Offered, Reserved and pre-departure Departing. At a deadline tick, expiry wins
before reservation or departure (`now >= expires_at` is expired). Once `InTransit`, a
previous offer/reservation deadline cannot free destination capacity or erase
the transfer, including when the deadline and arrival have the same tick.
A4.2's expiry-at-arrival oracle must prove the in-flight hold survives that tie.

A rejection received after physical departure is not a transition to terminal
`Rejected`: the patient and responsibility remain represented by the current
operation. Create a linked compensation operation for diversion/recovery,
record its own reservation and events, and release the failed destination hold
only at an explicit physical/ownership handoff defined by that compensation.
No error, rejection, timeout or cancellation message may make an in-transit
person disappear or silently return to the origin.

### Ownership, location and resources

1. Exactly one service is the authoritative care-responsibility owner for the
   transfer. The origin owns from request through arrival and any pending
   handover; the destination becomes owner only at `HandoverComplete`. Physical
   location is separately recorded as origin, transport, or destination. There
   is exactly one current physical-location record for the person at every
   committed event. An arrived-but-not-handed-over person has destination
   location and origin responsibility.
2. Origin bed/staff/escort claims remain held through `HandoverComplete`; physical
   departure can change an occupied bed to a reserved/held disposition but does
   not release its capacity. At handover, the origin claim is released once by
   its explicit release event, after which site capacity logic may clean or
   reopen the resource. Arrival alone never releases the origin.
3. `Reserved` creates one exclusive destination reservation. `InTransit`
   converts that same claim to an in-flight hold without a free/reacquire gap.
   `Arrived` converts the hold to destination occupied capacity atomically. The
   destination reservation/hold/occupancy is counted once, never as two claims.
   Source and destination resource counts are separate; temporary source hold
   and destination reservation/occupancy may coexist and are not a conservation
   error.
4. Rejection, cancellation, or expiry before departure releases a destination
   reservation if one exists, exactly once, and leaves origin claims in place.
   A failed release remains visible as pending and cannot be reported complete.
   No branch may silently evict another booking or over-allocate capacity.
5. Person, episode, transfer-operation and resource-claim ledgers are distinct.
   One person may have multiple episodes; a transfer does not add/remove a
   person. This contract does not make episode identity equal to operation ID.

## Required executable oracles for A4.2

For every accepted or rejected event, compare the full state before/after:
operation/version/revision, current state, person location, responsible owner,
origin and destination claim identity/disposition, capacity counts, applied
message IDs and compensation links. Invalid input must leave all of them
unchanged. The following cases are mandatory:

1. **Normal path:** Requested → Offered → Reserved → Departing → InTransit →
   Arrived → HandoverComplete. Assert one person, one episode, one operation,
   exactly one location and responsible owner at every step; assert reservation,
   in-flight hold, destination occupancy conversion and the single origin release
   at their specified events.
2. **Offer/rejection:** rejection from Requested/Offered is terminal; no
   destination claim exists and origin claims remain. Rejecting a reserved but
   pre-departure operation releases its destination claim once. Reject every
   invalid state jump without partial mutation.
3. **Cancel after reservation:** cancel Reserved and pre-departure Departing;
   release destination capacity once, retain origin occupancy/hold and ownership,
   and make duplicate cancel idempotent. A cancellation after physical departure
   cannot terminalize or erase the in-flight operation.
4. **Expiry/ties:** at `now == expires_at`, request expiry is applied before an
   offer can be created; it holds/releases no destination claim. Offer expiry
   is applied before reservation; it also holds/releases no destination claim.
   Reservation expiry is applied before dispatch/departure and releases the
   existing destination reservation exactly once while origin claims remain.
   Pre-departure `Departing` expiry likewise releases dispatch and any destination
   claim once. After actual departure, expiry cannot release the in-flight hold;
   expiry at the exact arrival tick converts that hold to occupancy without a
   lost patient or double release.
5. **Retry and ordering:** exact duplicate acknowledgement has one logical
   effect; same message ID with changed payload rejects; stale acknowledgement
   has no effect; a future revision waits for its predecessor then replays in
   causal order. Repeat under both same-tick insertion orders.
6. **Delayed and post-departure rejection:** delayed acknowledgements cannot
   rewind current state. Rejection after departure/arrival retains the person,
   origin responsibility and claims, creates a linked compensation operation,
   and does not double-count person, episode, or destination claims during
   diversion.
7. **Conservation:** after every success, rejection, cancellation, expiry,
   compensation, and failed-release event, person/episode identity is conserved;
   one current location and one responsibility owner exist; resource claims are
   exclusive and all capacity dispositions remain within known inventory.
8. **Horizon and recovery:** a horizon-censored in-transit or handover-pending
   operation remains open with its owner/claims visible and is not counted as
   arrived or complete. A supported CareOps checkpoint/restart preserves
   envelope version, revision, idempotency history, pending messages, deadlines,
   claims, ownership, location and compensation links; continuation matches
   uninterrupted execution without repeating a logical effect. If a required
   state is not serializable by the supported stack, report the gap as blocked /
   unverified rather than dropping it. This does not claim portable Kairos
   checkpoint support.

## Boundaries

A4.1 freezes semantics and regression oracles only. A4.2 implements CareOps
transfer envelopes on the existing single scheduler. It does not add distributed
transport, guarantee network exactly-once delivery, set clinical eligibility,
select providers, assign local crews, prescribe travel distributions, or alter
ED schemas/defaults. The downstream ward/community model and boarding feedback
remain A4.3. Kairos source and its parent pin remain unchanged.
