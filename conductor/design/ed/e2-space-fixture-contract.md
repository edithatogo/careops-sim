# E2.1 space, cleaning and diagnostic fixture contract

Coordinator contract reviewed by GPT-6.1-sol and a bounded Luna readback.
Dispatch only after accepted E1.4,
E2.1.staff and E2.1.interrupt. These are synthetic behavior specifications;
all32 cases start native_binding=unbound, targeting E2.2. Rust/Cargo1.99.0.
Reuse the [staff](e2-staff-fixture-contract.md) and
[interruption](e2-interrupt-fixture-contract.md) eligibility/calendar rules and
[E1 terminal semantics](e1-terminal-runtime-contract.md). No public native API
or new engine implementation is frozen by these fixture checks.

## Separate lifecycles

A named ED space has capacity1 and one lifecycle state: ready, reserved,
occupied, dirty_waiting, or cleaning. Aggregate usable space follows the existing
known physical/open/staffed capacity contract. Declared named units must agree
with those counts. Do not represent an invalid static zero-capacity profile as
valid dynamic closure; explicitly configured calendar unavailability can block
work without altering valid static capacity. IDs follow the staff ASCII1..64
rule. Cap1024 named spaces,100000 patients and100000 reservation attempts.
The native aggregate event budget is reviewed in E2.2 before construction.

ED occupancy begins at a real space grant and ends only at physical departure.
An admission decision, offer allocation, staff handover or queued diagnostics
never releases the patient's ED space. Space occupancy/reservation is separate
from a work claim. Each work item grants one unit of one resource, including
one named staff member where configured; no simultaneous staff-plus-space or
staff-plus-equipment atomic request. Preserve claim and waiting history.

On departure, a configured positive-duration cleaning task makes space dirty
immediately. It stays unavailable throughout cleaner wait and cleaning. Select
one eligible cleaner under the staff calendar/zone/skill policy. Cleaning starts
only at the actual cleaner grant; no implicit unstaffed timer. Cleaner work obeys
DrainActive. At cleaning completion, release cleaner claim and make space ready.
A busy/unavailable cleaner is normal queued work, not malformed configuration.
No patient may enter a dirty space while cleaning has not started. Initial spaces
are ready unless the case explicitly supplies a valid initial dirty lifecycle.

For diagnostic cases, reserve one diagnostic room first, then queue one named
staff work claim. The patient retains ED space. Waiting for staff holds no staff
claim; a held diagnostic-room reservation is explicit in output and persists
through that work's completion. On completion release room and staff separately.
Patients waiting for a diagnostic room hold no room claim. This fixed ordering
is the reviewed fixture approximation; no circular or atomic acquisition is
implied. Work, staff wait and room wait remain separate. E2.2 must integrate all
these lifecycles into one authoritative runtime rather than simulate them twice.

## Reservations

A reservation attempt names ID, patient, space, start and expiry with start<expiry;
start is within [0,H] and expiry may exceed H. At its start it succeeds only if
the space is ready and patient-space compatibility holds. Competing attempts
are processed by start tick then reservation ID bytewise. Losers are explicitly
rejected with current-state reason, never queued or silently redirected. An
occupied, dirty, cleaning or already-reserved space cannot be reserved. Overlap
between attempts is valid input with a deterministic rejected attempt outcome.

An accepted reservation blocks all other patients during [start,expiry). Its
named eligible patient may consume it once, converting reserved to occupied.
If unused it expires, returning ready space. After consumption its scheduled
expiry is a stale no-op; it must never release the resulting occupancy. Expiry wins at the same tick as
consumption; the former owner can then compete as an ordinary waiting patient.
At one tick: finish work/physical departures and cleaning, apply calendar changes
and reservation expiries, process reservation attempts, then grant spaces and
eligible work. A departing space requiring cleaning becomes dirty before any new
reservation/grant. Reserved owners consume their own compatible spaces first;
unreserved waiting patients order by arrival tick then patient ID, choosing the
lowest compatible ready space ID. No preemption of an existing space occupant.
A new reservation starting at the previous expiry may win before ordinary grants;
this finite synthetic policy does not establish general starvation freedom.

## Required positive cases (prefix space.)

| Suffix | Independent hand oracle |
| --- | --- |
| cleaning_blocks_reuse | A occupies[1,3); cleaner[3,5); B waiting1 acquires5 |
| cleaner_wait_dirty | A departs3; cleaner holds other work[3,5); dirty wait[3,5), cleaning[5,7), B acquires7, never3 |
| cleaning_calendar | A departs3; cleaner unavailable[3,5); same space/claim intervals as preceding case |
| cleaning_horizon | Cleaner work[3,5) completesH5 and releases cleaner; B occupancy grant5 has observed0 and remains held atH |
| reservation_blocks | B reservation[2,6), B not ready until7; A waiting2 gets space6 |
| reservation_expiry_tie | B reserved[2,5), A waits2, B arrives5: expiry first, A wins ordinary FIFO at5 |
| reservation_consumption | B reservation[2,6), B arrives3, occupies[3,7); no ready-space event at expiry6 |
| reservation_wrong_claimant | A cannot consume B's[2,6) reservation; B arrives3 and alone acquires3 |
| reservation_conflict | IDs rA/rB attempt same space at2; rA reserved until consumed/expired, rB rejected already_reserved with no retry/redirect, independent of input order |
| reservation_occupied | Space occupied[1,5); attempt2 rejected occupied, no eviction or later automatic reservation |
| reservation_dirty | Dirty[3,5), cleaning[5,7); attempts4/6 rejected dirty/cleaning; space ready7 |
| diagnostic_serial | A/B equal-priority requests diag.A/diag.B ready3; one room and worker, A sorts first; A[3,5), B[5,7), B room wait2, ED occupancy retained |
| diagnostic_unavailable | Valid room/staff static profile but staff calendar unavailable throughH; no diagnostic work, no downstream treatment; explicit held room and ED occupancy |
| diagnostic_staff_wait | Room granted3, worker busy until5; staff wait[3,5), work[5,7), room held[3,7), ED space retained |
| boarding_retention | Decisions4, offers4, one transfer worker work2: A[4,6), B[6,8); ED spaces released6/8, not4 |
| boarding_care | Decision4; care[6,8); offer7 cannot allocate during active care, allocates8; transfer[8,10); space held to10 |
| unused_offer | Compatible offer[2,6) with no waiting admission remains unused, expires6, no transfer or occupancy release |
| withdrawn_offer | Offer available4 withdrawn4 is never allocated; boarder remains occupying throughH |
| transfer_horizon | Decision4/offer4, transfer[4,8), H6: observed transfer2, ED occupancy retained, no terminal outcome or completed LOS |
| permutation | Reverse non-conflicting staff/patient/space/offer/attempt arrays; canonical event/claim histories and outcomes unchanged |

Every positive case independently sweeps space states and work claims: capacity1
per named unit, no double occupancy, no staff claim while waiting, observed versus
configured work kept separate, and conserved patients atH. A positive case can
contain explicitly rejected reservation attempts without being invalid input.
Offer withdrawal, consumption, active-care exclusion and physical transfer retain
E1 policy; the fixtures cannot weaken accepted E1 behavior.

## Required negative cases (prefix space.invalid.)

Twelve IDs: duplicate_space_id, count_mismatch, zero_static_capacity,
cleaning_nonpositive_work, unstaffed_cleaning_policy, duplicate_reservation_id,
reservation_nonpositive_interval, reservation_unknown_space,
reservation_unknown_patient, reservation_start_after_horizon,
atomic_staff_space, multi_role_work. Every case has a valid baseline, one
malformed semantic group, specification category/path, and a repair changing
only that group. Unstaffed cleaning means an unsupported unattended-timer mode or missing
work-policy configuration; a configured cleaner with no shifts or no currently
eligible cleaner is valid queued work, not this input error.
Unsupported unstaffed cleaning, atomic acquisition or multi-role
work must reject visibly; never silently substitute an approximation. Busy
cleaners, dynamic unavailability and competing reservation attempts belong to
positive queued/rejected-outcome cases, not input errors. Reuse other staff and
capacity validation without claiming new native error codes.

## Report29 and acceptance boundary

ST001/002: compatible, consumed-once and unused/expired offers. ST003/004: ED
occupancy retained through physical departure and explicit recurrent care claims.
ST005/006: ambulance arrival/triage/handover/offload/crew-release clocks and
receiver-staff sensitivity remain mandatory composed E2.3 work, not delivered
by these space fixtures. ST007/008: multi-person sedation/resuscitation remains
visibly unsupported absent a reviewed dedicated non-overlapping team model.
ST009: finite external synthetic offer streams do not establish causal ward,
fleet or community response effects. Record each mapping/debt explicitly.

One fixture JSON, one focused executable specification checker/test file and one
crosswalk/readme; plus result/logs (five paths), at most three exact checks. All32
native bindings stay unbound. Specification validity is not runtime qualification.
