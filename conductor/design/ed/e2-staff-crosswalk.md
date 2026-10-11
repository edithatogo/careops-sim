# E2.1 staff fixture crosswalk

This file maps the 32 frozen specification rows in `e2-staff.json` to their
independent checks in `tests/test_e21_staff.py`. All rows are synthetic and
`native_binding=unbound` with target E2.2. The Python hand sweep checks fixture
consistency and timing only; it is not native execution or acceptance.

| Case IDs | Frozen behavior / validation | Independent check |
| --- | --- | --- |
| `staff.empty`, `staff.single_work` | Empty accounting; `[1,4)` for ready 1 and work 3 | Exact inventory, reference sweep, observed work conservation |
| `staff.urgency`, `staff.fifo`, `staff.request_id_tie` | Priority descending, then ready time, then bytewise request ID while both requests wait for shift start 3 | Explicit first-grant witness plus reference sweep |
| `staff.staff_id_tie`, `staff.skill_subset`, `staff.zone_exact` | Lowest eligible staff ID; skill subset; exact zone | Explicit staff selection witnesses; reference sweep |
| `staff.ineligible_high_skip` | Ineligible urgent request remains queued; eligible lower request may start | Explicit low-priority grant witness; retained unassigned row |
| `staff.shift_start_inclusive`, `staff.shift_end_exclusive` | Half-open shift boundary | Explicit start and no-grant witnesses |
| `staff.break_start_exclusive`, `staff.break_end_inclusive` | Half-open break boundary | Explicit grant-at-break-end witness and reference sweep |
| `staff.drain_shift_end`, `staff.drain_break` | DrainActive; actual end retained; overtime and break overlap separate | Explicit accounting checks and clipped interval sweep |
| `staff.completion_at_boundary` | Completion/release at tick 3 precedes calendar grant eligibility; next grant at 5 | Explicit ordered assignment witness |
| `staff.horizon_active`, `staff.horizon_grant` | Observed work, completion, active status, and remaining configured work are distinct | Explicit H=4 and H=10 accounting checks |
| `staff.calendar_gap` | No grant in gap; next grant at second shift start | Explicit start-at-5 witness |
| `staff.permutation` | Reversed staff/request/shift inputs preserve canonical output | Reference sweep compared on both stored input orders |
| `staff.invalid.duplicate_staff_id`, `staff.invalid.duplicate_request_id` | Duplicate identifiers | Valid baseline, single identifier mutation, typed diagnostic path, repaired baseline |
| `staff.invalid.malformed_id` | ASCII identifiers of any content and length 1..64 (patient identifiers may repeat) | Valid baseline, single identifier mutation, typed diagnostic path, repaired baseline |
| `staff.invalid.nonpositive_shift`, `staff.invalid.overlapping_shifts` | Positive, nonoverlapping shift intervals | Valid baseline, interval mutation, typed diagnostic path, repaired baseline |
| `staff.invalid.break_outside_shift`, `staff.invalid.overlapping_breaks` | Break containment and nonoverlap | Valid baseline, interval mutation, typed diagnostic path, repaired baseline |
| `staff.invalid.zero_work`, `staff.invalid.ready_at_horizon`, `staff.invalid.zero_horizon` | Work and horizon eligibility bounds | Valid baseline, single scalar mutation, typed diagnostic path, repaired baseline |
| `staff.invalid.count_limit` | Early cap rejection uses 1025 distinct staff records | Asserts payload length and identifier uniqueness; repair is the valid 1024-record baseline |
| `staff.invalid.multi_person_unsupported` | Count 2 rejected before assignment; repair count is 1 | Explicit invalid and repaired count values; baseline/diagnostic checks |

The negative rows each retain a valid baseline, one declared mutation group,
its diagnostic category and offending path, and a positive repair control. The
count-limit fixture contains the actual 1025 distinct staff records. No engine
event count or native typed error is asserted. Acceptance and shared native
binding remain coordinator-owned follow-up work.
