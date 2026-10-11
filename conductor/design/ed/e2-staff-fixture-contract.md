# E2.1 staff and calendar behavior fixture contract

Status: coordinator preparation contract independently reviewed by GPT-6.1-sol
on 11 October 2026. E1.4 closeout required before any E2.1 worker dispatch. Rust/Cargo1.99.0. This freezes synthetic
behavior expectations, not a public Rust API or a claim that staff calendars
already exist. Existing E2 staff -> interrupt -> space recipe order remains.

## Evidence boundary

E2.1 writes executable behavior specifications and negative/repair cases. Their
schema/hand-oracle checks do not establish native staff execution. Every case
starts native_binding=unbound, with target E2.2; all must bind to actual shared
native execution before E2.4. E2.3 additionally requires the composed ED, Macro/
Micro, interruption/recovery and config-only nominal/constrained MVP recipe.
No E1 or E2 requirement is removed by these fixture boundaries. E2.1 native
API contracts will be reviewed before implementation and any Kairos changes
retain their upstream owners. No duplicate private engine is authorised.

Existing source: Kairos `staff_dispatch.rs` and `staff_flow_lifecycle.rs` in
kairo-ecs-calibration are private. The selector demonstrates explicit priority,
FIFO, stable IDs and exact zone/skill availability but is not callable by CareOps.
Flow `set_capacity` rejects shrink below active claims transactionally; it does
not provide a shift calendar or transfer claim ownership. Cancellation/release
must be inspected for handover; no assumed lease transfer. New staff behavior
belongs to the parent domain adapter unless reusable engine changes are reviewed.

## Model and fixture representation

Version1, synthetic provenance with explicit notes, integer ticks and fixed
positive work for these fixtures. IDs ASCII1..64, case-sensitive, no trimming or
normalisation. Apply the ID bound to staff, request, patient, zone and skill
identifiers. Cap32 skills per staff and32 required skills per request before
allocating lookup state. Staff: staff_id, assigned_zone, sorted unique skill IDs, shifts
and breaks as [start_tick,end_tick). Requests: request_id, patient_id, ready_tick,
assigned_zone, required_skills, urgency:i32, work_ticks:u64, required_staff_count=1.
Urgency is configured synthetic request priority, never an inferred clinical
acuity rule. No geographic movement or multi-zone permission is inferred.
Horizon H positive, requests ready before H; completions at H are observed and
new grants at H have zero observed work. Use u128 for sums/completion arithmetic.

Shifts must have positive length and not overlap for one staff member; adjacent
periods are allowed. Breaks must have positive length, not overlap, and be fully
contained within one shift. Declared intervals may end beyond H; observation clips
at H, configuration validity does not silently truncate them. A staff member may
have no shifts, meaning never available. Cap100000 requests,1024staff,1024calendar
intervals per staff and bound total event construction before allocating state.
Exact aggregate event budget is part of the later native interface contract;
fixtures include early cardinality rejection, not an invented exposed counter.

Each case records inputs, expected assignments/work intervals, unassigned/censored
rows, calendar-boundary observations and resource conservation. Negative cases
record diagnostic category and offending path, plus a repair group and an otherwise
identical valid control. Expected outcomes must be independently hand-derived;
validators checking file consistency are not the native simulator. Store a complete
case ID inventory and an explicit native-binding crosswalk.

## Dispatch and calendar semantics

At each relevant tick: complete work/release claims, apply calendar changes, then
grant new work. A request is eligible after ready_tick. Visit waiting requests by
urgency descending, ready_tick ascending, request_id bytewise ascending. For each,
choose the lowest staff_id among idle staff whose assigned_zone exactly matches,
whose skills include every required skill, and who are inside a shift and outside
all breaks. A higher-priority request with no eligible staff does not block a
lower-priority eligible one. Repeat until no eligible pair exists. Staff may hold
at most one active work assignment; waiting consumes no staff claim.

Default fixture in-flight policy is explicit version1 DrainActive: calendar
unavailability forbids new grants but active work continues to its actual end.
Do not shrink a Flow resource below active claims or manufacture a failed grant.
At a break or shift end, record active work overlapping that interval. Distinguish
break_overlap_ticks from work outside all shifts (shift_overtime_ticks). Their
union is scheduled_unavailable_work_ticks; disjoint shift/break rules prevent
double-counting. Do not invent a postponed break or pretend the policy proves
roster/clinical compliance. This is an explicit model approximation.

At an interval endpoint, half-open rules apply: no new grant at shift end/break
start; grants may start at shift start/break end. Work completing at such a boundary
releases normally; a replacement can start only if the resulting calendar permits.
A boundary at H still affects state/grants but contributes no time beyond H.
Handover and urgency preemption are mandatory separate E2.1.interrupt behaviors;
DrainActive here does not waive them. Cleaning/reservation is E2.1.space.

## Required positive case IDs and hand oracles

Use prefix staff. and the following20 IDs. Inputs can be minimal, but assertions
must isolate the intended rule and retain all rows, including unassigned work.

| Suffix | Required oracle |
| --- | --- |
| empty | no requests/staff produces empty assignments and zero busy time |
| single_work | ready1, work3, shift[0,10), H10 -> start1/end4/work3 |
| urgency | equal ready, urgency2 beats1 regardless input order |
| fifo | equal urgency, earlier ready wins even with lexically later ID |
| request_id_tie | equal urgency/ready chooses lexical request ID |
| staff_id_tie | equally eligible idle staff choose lexical staff ID |
| skill_subset | all required skills necessary; extra skills allowed |
| zone_exact | other zone never matches; matching staff selected |
| ineligible_high_skip | unavailable high-priority request stays queued while eligible lower request starts |
| shift_start_inclusive | shift[3,8), ready1 -> start3 |
| shift_end_exclusive | ready3, shift[0,3) -> no start |
| break_start_exclusive | break[3,5), ready3 -> start5 |
| break_end_inclusive | break[3,5), ready5 -> start5 |
| drain_shift_end | ready1/work5/shift[0,3) -> work[1,6), overtime3; queued request cannot start3 |
| drain_break | ready1/work5/break[3,5) -> work[1,6), break overlap2; no invented later break |
| completion_at_boundary | work ends3 when break begins; next ready request starts5, not3 |
| horizon_active | start1/work5/H4 -> observed3, no completion/LOS invented |
| horizon_grant | shift[10,12), ready9/work2/H10 -> start10, observed0, no completion, active at H |
| calendar_gap | shifts[0,2),[5,9), ready3 -> start5 |
| permutation | reverse staff/request/calendar input arrays, canonical result unchanged |

Positive cases must independently sweep actual expected busy intervals against
capacity1 per staff and assignment exclusivity. Eventual native binding compares
actual native rows to those same frozen expectations, including zero-work Hgrants.
Do not count overlapping work twice or hide queue delay inside sampled service.

## Required negative case IDs

Use prefix staff.invalid. and12 suffixes: duplicate_staff_id,
duplicate_request_id, malformed_id, nonpositive_shift, overlapping_shifts,
break_outside_shift, overlapping_breaks, zero_work, ready_at_horizon,
zero_horizon, count_limit, multi_person_unsupported. For the last case,
required_staff_count=2 must fail visibly before any assignment; no implicit
atomic staff-plus-space or multi-person resource acquisition. Repair it to1.
Each of the12 gets a positive repair control changing only its malformed group.
No engine event-count or typed native error-code claim exists until actual APIs
are frozen and bound; specification categories remain explicit metadata.

## Bounded worker proposal after E1.4

One E2.1.staff instance: own a staff fixture JSON, focused Python specification
validator/tests and a small readme/crosswalk only after coordinator chooses actual
paths and exact commands. Keep packet <=5write reservations, including result/logs;
no Rust compilation target is needed for a pure fixture-spec packet. Native test
work follows the reviewed E2.2 interface and cannot be marked proven here.
Independent review must check all32 IDs, hand timing and12repairs; task acceptance
retains native-binding debt. Shared manifests, source modules and policy changes
are outside that worker's ownership.
