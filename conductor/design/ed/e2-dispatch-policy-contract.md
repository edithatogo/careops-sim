# E2.2 pure dispatch policy contract

Status: coordinator contract independently reviewed by GPT-6.1-sol.
E2.1.space acceptance and a reviewed bound packet are required before dispatch.
Rust/Cargo 1.99.0 only. Parent ED ownership; no Kairos API or manifest changes.

This leaf implements ordinary one-staff task selection. It borrows an immutable
snapshot, validates the complete snapshot, and returns one request/staff identity
pair. It does not advance time, create events, draw randomness, mutate claims or
execute a second simulation. Preemption, Suspend, handover and actual allocation
remain the later authoritative FlowRuntime adapter's responsibility.

## Exact Rust surface

Experimental `#[doc(hidden)] pub mod dispatch` in careops-ed's lib.rs.
Use standard-library types only, no serialization interface in this leaf.

```rust
pub struct CalendarInterval { pub start_tick: u64, pub end_tick: u64 }
pub struct DispatchRequestInput {
    pub request_id: String,
    pub ready_tick: u64,
    pub assigned_zone: String,
    pub required_skills: Vec<String>,
    pub urgency: i32,
    pub required_staff_count: u32,
}
pub struct StaffSnapshotInput {
    pub staff_id: String,
    pub assigned_zone: String,
    pub skills: Vec<String>,
    pub shifts: Vec<CalendarInterval>,
    pub breaks: Vec<CalendarInterval>,
    pub idle: bool,
}
pub struct ValidatedDispatchSnapshot<'a> { /* private fields only */ }
pub struct DispatchSelection<'a> {
    pub request_id: &'a str,
    pub staff_id: &'a str,
}
pub enum DispatchValidationCategory {
    Cardinality, Horizon, CurrentTime, Identifier, DuplicateIdentifier,
    ReadyTime, Skills, UnsupportedStaffCount, CalendarInterval,
    CalendarOverlap, BreakOutsideShift,
}
pub struct DispatchValidationError {
    pub category: DispatchValidationCategory,
    pub path: String,
}
pub fn validate_dispatch_snapshot<'a>(
    now: u64, horizon: u64,
    requests: &'a [DispatchRequestInput],
    staff: &'a [StaffSnapshotInput],
) -> Result<ValidatedDispatchSnapshot<'a>, DispatchValidationError>;
pub fn select_dispatch<'a>(
    snapshot: &ValidatedDispatchSnapshot<'a>,
) -> Option<DispatchSelection<'a>>;
```

Input/value/error structs support Debug, Clone, PartialEq and Eq; selection also
Copy. The validated wrapper exposes no unchecked constructor, mutable input
accessor or public fields. It may own derived indices while borrowing inputs.
No interior-mutability containers. Selection IDs borrow original strings, with
no trimming, case folding or normalization. Subject, patient/space, work duration,
episode and runtime validation remain the enclosing adapter's responsibility;
this allows patient, space and non-patient tasks to share this policy.

## Validation and deterministic diagnostics

Validate all rows before selection, including currently ineligible/future rows.
No invalid high-priority request is silently ignored. Use array-index paths such
as `requests[2].ready_tick`, `staff[0].breaks[1]`, `horizon`, `now`.

First-error order: request cardinality, staff cardinality, horizon, now; request
identities in input order; staff identities in input order; remaining request
fields in input order; remaining staff fields and calendars in input order.
Within request fields: ready_tick, assigned_zone, required_skills,
required_staff_count. Within staff: assigned_zone, skills, combined calendar
cardinality, shifts, breaks, then break containment. Invalid identity precedes
its duplicate check. Separate request and staff identity namespaces.

- At most 100000 requests and 1024 staff; check caps before derived allocation.
- H > 0 and now <= H. Every original ready_tick < H. Future ready times before H
  are valid and ignored by selection until ready_tick <= now.
- Every identity, zone and skill is ASCII, 1..64 bytes. At most 32 skills per list,
  sorted strictly ascending, unique. Empty skill lists are valid.
- required_staff_count must equal 1; reject other values visibly.
- At most 1024 combined shift/break intervals per staff. Every interval has
  start < end. Both arrays may arrive in any order; normalize derived views,
  never mutate the inputs. Within each array intervals cannot overlap; adjacency
  is valid. Every break is contained in one shift. Empty shifts are valid and
  mean unavailable. Intervals may extend beyond H without truncation.
- Rust integer and bool types define scalar representation; no JSON coercion is
  introduced by this leaf. Urgency uses all i32 values without negation overflow.

Use Cardinality for outer/combined calendar caps, Skills for malformed skill
lists, Identifier for identity/zone values, and the other named categories for
corresponding rules. A skill's malformed identity reports Skills at that list's
path. CalendarOverlap reports the calendar list path; BreakOutsideShift reports
its original break index. Outer cardinality paths are `requests` and `staff`; the combined calendar cap
uses `staff[i]`. Skill errors use `requests[i].required_skills` or
`staff[i].skills`. Calendar validation order is: inspect original shift intervals,
then normalized shift overlap; inspect original break intervals, then normalized
break overlap; finally break containment in original input order. When multiple
breaks fail containment, report the first original array index. This clarification
was reviewed by the coordinator and GPT-6.1-sol during native implementation;
the dispatched packet keeps its original immutable contract hash.
An invalid interval reports its
original `staff[i].shifts[j]` or `staff[i].breaks[j]` path.

## Selection

At explicit now, staff eligibility means idle, exact zone match, all required
skills present, inside a half-open shift, and outside every half-open break.
A busy staff member remains unavailable even if DrainActive crosses a boundary.
Select ready requests by urgency descending, ready_tick ascending, request ID
ASCII ascending. Skip a request with no eligible staff. For the first eligible
request, choose the lowest ASCII staff ID. Return None if no pair exists.
A caller must rebuild the snapshot after an actual claim before selecting again;
repeated selection over an unchanged snapshot returns the same pair.

Avoid repeatedly scanning all staff for every skill of every request. Sort the
bounded staff list by ID once; derive available per-zone and per-skill bitsets
at now. Intersect at most 33 masks per ready request and compare its request key
against the current best, without globally sorting all requests. With at most
1024 staff, each mask uses at most 16 u64 words. Index only staff zones/skills,
not arbitrary request-only skills. Missing zone/skill short-circuits to no match.
BTreeMap borrowed keys provide deterministic lookup; unused high bits stay zero.
This bounds selection to O(R*(K+1)*ceil(S/64)) word operations plus map lookups.
Validation can use ordered identity sets and normalized calendar copies. No
performance, speedup or model-qualification claim follows without measurement.

## Native qualification and bounded ownership

Source paths: `crates/careops-ed/src/dispatch.rs`, single module declaration in
`crates/careops-ed/src/lib.rs`, `crates/careops-ed/tests/e22_dispatch_policy.rs`.
Reserve lib.rs exclusively. Result and logs complete the five owned paths.
Exact commands are resolved in the later reviewed packet: focused native tests,
strict Clippy, rustfmt, all under Rust/Cargo 1.99.0 with preserved attempt logs.

Oracles: empty/no eligible/one pair; urgency vs FIFO; genuine already-waiting
ready1/2 at available3 with lexically reversed IDs; request/staff ID ties;
exact zone case; all skills/extra allowed; ineligible high request skipped;
busy/unavailable; shift/break start and end; unordered and adjacent calendars;
now==H selection with original ready<H and shift startingH; future ready ignored;
reversed arrays produce the same pair; i32 MIN/MAX/zero; repeatability and inputs
unchanged. Include malformed IDs, duplicates, 32/33 skills, 1024/1025 calendars,
overlap/containment, H0, now>H, ready==H and unsupported count. Check both error
category and deterministic path, plus private-wrapper construction boundary.
Bitset oracles must place the only eligible staff at sorted indices 63, 64 and
1023; cover skills that each exist but have an empty joint intersection, and
zone/skill intersection excluding otherwise matching staff. These are behavior
checks, not timing benchmarks or a performance claim.

Acceptance proves only native pure policy. The 32 staff, 24 interruption and
32 space runtime bindings, single-runtime migration, E2.2 phase, E2.3 composed
MVP, clinical and release gates remain open. Once space is accepted, this parent
leaf can run alongside the separately reviewed Track03 grant-gate packet.
