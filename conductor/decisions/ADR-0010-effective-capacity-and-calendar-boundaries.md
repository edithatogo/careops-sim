# ADR-0010: Effective capacity and calendar boundaries

- Status: accepted for A2 implementation
- Date: 2026-10-26
- Scope: CareOps-neutral capacity snapshots and UTC calendar intervals
- Does not qualify clinical staffing policy, local time-zone rules, or allocation
  semantics owned by Kairos.

## Context

CC-05 requires physical, equipped, staffed, open, reserved, occupied, cleaning
and unavailable capacity to remain distinguishable. `ResourceBucket` in the ED
profile already has `pool_group_id`, but the current pathway runner counts each
bucket independently. The core abstraction must therefore define whether those
fields are additive capacity, availability constraints, or dispositions before
an ED adapter can safely consume them.

## Decision

### Snapshot accounting

All counts in a `CapacitySnapshot` describe the same instant and the same
resource unit. `physical`, `equipped`, `open` and `staffed` are independent
capacity ceilings. Physical and equipped counts are inventory ceilings;
open and staffed are gross concurrent-capacity ceilings before the disposition
counts below are subtracted. For example, an open ceiling of 4 with 1 occupied
and 1 unavailable unit permits at most 2 new assignments, subject to the other
ceilings. An adapter with a net-open count must convert it to gross semantics
or leave it unknown; passing the net count through would subtract dispositions
twice. A missing count is
unknown; `Some(0)` is known zero. Unknown must not be silently converted to zero
or to an unrestricted value. A known zero physical, equipped, open or staffed
ceiling establishes zero assignable capacity even if another ceiling is
unknown. Demand is not part of capacity and zero demand is represented
separately by the requesting workload.

`reserved`, `occupied`, `cleaning` and `unavailable` are mutually exclusive
dispositions of equipped units. The accounting sum uses checked arithmetic and
must not exceed the minimum known physical/equipped inventory ceiling. A known
equipped count must not exceed a known physical count. Open and staffed ceilings constrain only new assignments:
existing reservations, occupied work and cleaning are retained if a closure or
staffing change lowers a ceiling. The number assignable to new work is
`max(0, min(known physical/equipped/open/staffed ceilings) - reserved - occupied - cleaning - unavailable)`
when all relevant ceilings are known; a known zero ceiling yields zero even if
another ceiling is unknown. Otherwise the amount is unknown. This saturation is
not permission to cancel active work.

`open` must not exceed a known physical count. `equipped` greater than
`physical` is invalid. `staffed` may exceed physical capacity because it can
represent surplus workforce; the minimum ceiling still bounds assignments.
Ceilings may be lower than active dispositions after closure or staffing
changes; this is valid and yields zero capacity for new work where existing
dispositions consume the remaining ceiling.

Staffing counts in this phase are availability ceilings only. Named staff,
shared staff identity, calendars, handover and call-in latency remain A3.3; a
staff count must not be summed across resource buckets without a distinct pool
identity and accepted allocation semantics.

### Shared pools and capability matching

A pool identifier aliases one physical capacity snapshot; it does not create
additional units. All declarations for the same pool at the same instant, unit
and snapshot revision must agree on the complete snapshot or validation fails
closed. A resource member may expose capabilities and location metadata, but
the pool's capacity is counted once when aggregating members. This is an
accounting rule only; grant and contended allocation enforcement is deferred to
A3.

Capabilities are opaque neutral identifiers. A request matches a resource when
the resource's capability set contains every required capability. Input order
and duplicate capability IDs do not change matching. Matching does not encode
clinical eligibility, jurisdictional permission, priority, or dispatch policy.

### Time and calendar mapping

Core calendar intervals use signed UTC Unix seconds and half-open bounds
`[start, end)`. Construction rejects `end <= start`; adjacent windows are
continuous without overlap, the start is included and the end excluded. A
calendar rejects overlapping windows rather than resolving precedence
implicitly. UTC-to-simulation-tick conversion is an adapter responsibility and
must declare its epoch, unit and checked precision/overflow behavior.

Core accepts resolved UTC instants only. A local-time adapter must supply the
zone-rule result/offset and an explicit ambiguity or gap decision. For a
repeated fall-back wall time, each supplied offset maps to its own UTC instant.
For a nonexistent spring-forward wall time, the adapter must explicitly reject
or skip it; core does not infer a time zone, choose an offset, or bundle an
implicit TZDB. A 23-hour or 25-hour local day is represented by the resulting
UTC interval length, not by assuming every local day is 24 hours.

## Required contract cases

1. Reject equipped greater than physical, open greater than known physical, and
   dispositions greater than the minimum known physical/equipped inventory;
   accept equality and checked-sum boundaries, reject arithmetic overflow.
2. Preserve unknown and known-zero ceilings distinctly. Unknown availability
   yields unknown assignable capacity unless a known effective ceiling is zero;
   known zero yields `Some(0)` and is not confused with zero demand.
3. Lower open or staffed ceilings while reserved/occupied/cleaning work exists:
   retain every disposition and expose zero new assignable capacity where
   exhausted; never evict active work as a side effect.
4. Shared-pool aliases with equal unit, revision, instant and snapshot share
   one counted ceiling; conflicting declarations reject; aggregation counts
   the pool once. Allocation arbitration is not claimed by A2.
5. Capability matching is set-based, duplicate/order invariant, and fails if any
   required capability is absent. A capability match does not multiply pool
   capacity.
6. Calendar start is included and end excluded; adjacent intervals have no gap
   or double count; zero/reversed and overlapping intervals reject.
7. Two explicit offsets for a repeated fall-back local time yield distinct UTC
   instants. A spring-forward nonexistent time requires explicit reject/skip.
   UTC intervals mapped from local days may be 23 or 25 hours.
8. A pure explicit-offset-to-UTC conversion rejects arithmetic overflow. Any
   UTC-to-simulation-tick adapter rejects overflow and precision loss when that
   adapter is introduced; core consults no host-local clock or implicit zone
   database.

## Consequences and boundaries

This is a snapshot/accounting contract, not an allocator. The new core types
must not replace ED schema fields or change ED admission defaults in A2. A later
compatibility adapter may map existing ED profiles after parity tests. Calendar
recurrence expansion, jurisdictional staffing policy, Kairos leases, safe
preemption and clinical acceptability remain outside this decision.
