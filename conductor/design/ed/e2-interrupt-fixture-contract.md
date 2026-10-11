# E2.1 interruption and handover fixture contract

Preparation contract independently reviewed by GPT-6.1-sol on 11 October 2026.
Dispatch follows accepted E1.4 and E2.1.staff. This adds
behavior specifications to the [staff/calendar contract](e2-staff-fixture-contract.md).
Rust/Cargo 1.99.0. All cases begin native_binding=unbound; E2.2 must bind them
to actual execution before E2.4. No new public Rust API is frozen here.

## Source constraints and explicit policies

Kairos Flow supports incoming can_preempt plus victim preemptible(Suspend).
It has no resource-level consent flag: the ED adapter must enforce resource
consent and calendar eligibility before exposing a preempt-capable request.
Both clinical task policy and resource policy must permit interruption. Model
these as explicit booleans: incoming consent, incumbent consent, resource consent.
All three are required, and incoming urgency must be strictly greater. Equal
urgency never interrupts. Fixture policy version1 uses Suspend only; Restart and
Abort are distinct engine capabilities, not implicit clinical defaults.

Domain urgency is i32 descending; Flow priority_level is i32 ascending. Freeze
mapping priority_level = !urgency (bitwise complement), avoiding arithmetic
negation overflow at i32::MIN. Assert the two extremes, zero and ordering/ties.
Queue tie policy remains ready_tick ascending then request_id bytewise ascending.
Native integration must explicitly preserve domain ordering; Flow admission
sequence is not automatically the same as a domain request ID ordering.

Calendar phase order stays completion/release, calendar changes, interruption/
handover decisions and grants. No grant or preemption onto unavailable staff.
DrainActive keeps incumbent work running through a break/shift end; eligible
urgent work waits until a staff resource is available. No lower-capacity mutation
may violate Flow's active-claim bound. Completion at the same tick wins over
interruption or handover; do not emit a spurious zero-remaining segment.

A logical episode has one original sampled work amount and sampling lineage.
Suspend checkpoints useful work and resumes the remainder with no new draw.
Each staff capacity is one; every observed work interval must belong to exactly
one active claim. Waiting has no staff claim and is not service work. Preserve
all interruption/assignment history, not just final completion. Useful observed
work plus remaining work equals the original sample; clip observation at H,
while scheduled completion may lie beyond H. Do not double-count paused time.

## Explicit handover

Handover is a configured domain event, not an automatic Flow lease transfer.
For these fixtures it names logical episode, tick, incumbent and replacement.
It is allowed only for an active consenting episode on the named incumbent.
Replacement must satisfy the same zone/skill/calendar rules at the actual grant.
It may be busy or unavailable when the instruction occurs; the continuation waits
without holding a claim until it becomes eligible.
At a handover tick, stop/checkpoint the old segment, release its exact active
claim once, preserve logical episode and remaining work, then enqueue the new
segment for the named replacement. It can wait without holding either staff.
The released incumbent becomes available to other eligible work in that tick.
Do not simultaneously retain an old claim and wait for a replacement claim.

Flow Release checkpoints and terminally releases that request; Cancel also
checkpoints/removes its claim and terminally cancels it. Use Release for an
active handover segment. Never issue Cancel plus Release for the same claim.
A continuation gets a new request/work segment ID; its parent logical episode
and sample lineage persist. Actual API mechanics/notifications are reviewed in
E2.2. A released segment is not a completed patient/episode. No resampling and
no implicit zero-cost extra clinical work or handover overhead is assumed.

For this fixture policy there is at most one explicit handover per episode;
repeated/conflicting handover instructions are rejected. A handover request at
an episode's completion tick is a no-op with an explicit completed-before-handover
observation. If the logical episode has no active segment at that tick, record
not-active without manufacturing a claim. Missing/wrong identifiers remain
configuration errors. Require handover tick in [0,H], IDs from the staff contract,
and positive original work. More elaborate repeated handover is a later reviewed
policy, not inferred by a fixture validator.

## Positive inventory: prefix interrupt.

| Suffix | Independent hand oracle |
| --- | --- |
| suspend_resume | Low work10 starts1; urgent work2 ready4; low [1,4), urgent [4,6), low [6,13); low remaining7 at4, total useful12 |
| resource_consent_absent | Same, resource consent false: low completes11, urgent [11,13) |
| incumbent_consent_absent | Same, incumbent consent false: no interruption, logical work completions11/13 |
| incoming_consent_absent | Same, incoming consent false: no interruption, logical work completions11/13 |
| equal_urgency | Same, equal urgency: incumbent finishes11 then urgent13 |
| completion_tie | Low [0,3), urgent ready3/work2: low completes3, urgent [3,5), no preempt event |
| break_precedence | Low work10 starts1, break[3,5), urgent ready3/work2: low [1,5), urgent [5,7), low [7,13); useful4 at preemption, break overlap2 |
| shift_precedence | Shifts[0,3),[5,20), same timings as break case; low overtime2, urgent cannot grant3 |
| priority_extremes | MIN maps MAX, MAX maps MIN, zero maps -1; strict descending domain ordering survives mapping |
| handover_immediate | Sample5, A [1,3), B [3,6); one logical completion6, segment work2+3, one sample lineage |
| handover_wait | Sample5, A [1,3), B occupied [0,5), continuation B [5,8); explicit wait[3,5), no A claim after3 |
| handover_completion_tie | Work[1,3), handover3: completion wins, no replacement segment/claim |
| handover_not_active | Episode ready5, instruction3: explicit not-active, ordinary eligible work begins5 |
| horizon_suspended | Low starts1/work10, urgent ready4/work5, H6: low observed3 remaining7; urgent observed2 remaining3; no completion |
| horizon_handover | A sample5 starts1, handover3 to B, H3: A observed2; B grant3 observed0 remaining3; one active B claim at H |
| permutation | Reverse input staff/requests/events for a non-conflicting mixed case; canonical IDs, intervals and endpoints unchanged |

All cases must record work, wait, interruption, handover and calendar observations
separately. Independently sweep expected staff intervals for exclusivity and
logical episode segment sums for conservation. Fixed fixtures establish hand
oracles only; actual seeded no-extra-draw proof remains a native binding gate.

## Negative inventory: prefix interrupt.invalid.

Eight cases, each with otherwise-valid baseline and one targeted repair:
unknown_episode, unknown_incumbent, unknown_replacement, wrong_incumbent,
duplicate_handover, handover_after_horizon, unsupported_strategy,
handover_consent_absent. Wrong incumbent means the named source does not match
the episode's configured initial assignment. Handover is an explicit instruction,
so absent required consent rejects rather than silently transferring ownership.
Unsupported strategy is any version1 policy other than Suspend. Repair only the
malformed semantic group. Reuse all staff input validation and cardinality bounds;
cap handover records by episode count before allocation. Exact aggregate runtime
event budget and native diagnostic types are frozen in E2.2, not fabricated here.

## Bounded output and acceptance

One JSON family with exactly16 positive and8 negative cases, focused executable
specification checks, and a native-binding crosswalk/readme. Maximum five write
paths including result and logs; no production/engine edits. Validate exact IDs,
hand calculations and24 unbound native rows, preserve expected rejection and
repair controls. Coordinator plus independent reviewer accepts the fixture leaf;
only actual runtime execution later can close the native-binding debt.
