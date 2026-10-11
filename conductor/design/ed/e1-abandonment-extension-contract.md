# E1.3 optional abandonment extension

Status: coordinator contract independently reviewed by GPT-6.1-sol on
11 October 2026. Rust/Cargo 1.99.0.
This adds the enabled optional policy required by the E1.3 plan; it does not
reopen accepted Disabled behavior or modify an active worker's frozen contract.
The [terminal contract](e1-terminal-runtime-contract.md) otherwise applies.
All inputs and assumptions are synthetic; no inferred patient behavior or clinical
validity is claimed. Disabled remains the explicit existing policy choice; no implicit serde default
is introduced. Adding abandoned to counts changes the serialized shape; existing
Disabled behavior and prior field values remain unchanged.

## Bounded API

Extend AbandonmentPolicy with BeforeFirstCare { patience_ticks:u64 }, serialized
as the existing serde snake_case enum convention. Patience must be positive;
zero is InvalidInput before cloning or scheduling. Keep Disabled unchanged.
Extend RealizedOutcome with Abandoned and TerminalCounts with abandoned:u64.
For Disabled runs abandoned=0 and all existing rows, times, RNG draws and prior
counts are unchanged. Conservation becomes arrived=discharged+transferred+
abandoned+unfinished. All terminal outcomes obey the existing terminal/LOS rule.
Do not add automatic enabled defaults, acuity-dependent hazards, abandonment
while receiving clinical work, or abandonment after first care begins.

A patient's deadline is arrival_tick + patience_ticks computed in u128. Enqueue
one deadline event only if deadline<=H, otherwise leave the patient censored by
the usual rules. Positive patience prevents arrival/deadline coincidence.
Before scheduler construction, add patient_count to the checked conservative
4,400,000 event bound when enabled; retain the actual scheduled-event counter.
No saturating/wrapping arithmetic and no fabricated event after the horizon.

## Eligibility, ties and physical state

Eligible at deadline only when triage has never started and the patient is still
waiting outside for space or queued for triage. A triage start at the deadline
wins over abandonment. Clinical active, later-stage queued/active, boarded,
pending transfer, transfer active and terminal patients are ineligible.

For each tick preserve the existing batch of completions, arrivals, decisions,
care notifications, space grants, once-only offer allocation and work grants.
Deadline events collect candidates; evaluate them AFTER the first work-grant
phase, so first care at the same tick saves the patient. Determine the complete
eligible candidate snapshot before cancelling anyone. Then cancel all remaining
eligible candidates, remove their actual waiting-space/triage queue entries,
release held ED space once and realize Abandoned at the deadline. Regrant freed
space and work at this same tick using a shared canonical grant helper. Never
run OfferBook.allocate twice at one tick. New positive work cannot create another
same-tick clinical decision. Simultaneously expired patients cannot rescue one
another merely because another cancellation releases space.

Preserve an already created but unstarted triage StageEpisode as historical
queue-entry data: start/completion remain None and observed work is zero.
Terminal outcome Abandoned plus terminal_tick explicitly identifies its
cancellation time. Do not fabricate a clinical completion or mutate sampled
service work into wait. PathwayState may retain its queued clinical-history
state; Terminal endpoint disambiguates the realized outcome. Removed queues
must not contribute to queued_at_horizon, and unacquired work never releases
resource capacity. A patient abandoning outside has no acquired/released space.
A patient abandoning after space acquisition releases exactly at deadline;
occupancy is the actual half-open interval. LOS=deadline-arrival; no boarding,
allocation, transfer or decision appears before treatment completion.

## Implementation ownership and phase join

One separately bound E1.3.exits instance named abandonment owns only
`crates/careops-ed/src/pathway.rs` and `crates/careops-ed/src/terminal.rs` plus
its declared result/log/target paths. Preserve one shared scheduler; extract
and reuse grant logic instead of copying whole loops. Existing native exit
checks remain a separate instance. Both instances must be accepted before the
E1.3.exits leaf closes and horizon dispatch starts. Earlier native packet's
one-instance declaration was bound before this phase-scope gap was identified;
coordinator join records the additional required instance without rewriting it.

Tests belong in the owned source files. Run full careops-ed package tests,
strict Clippy all targets, and rustfmt with the exact Rust1.99 toolchain.
A production defect in a different owned source or required test-helper change
escalates to coordinator for a separate claim; no silent reservation expansion.

## Required independent hand oracles

1. Disabled baseline unchanged and abandoned count zero. Positive-patience
   serialization roundtrip and zero-patience pre-run rejection.
2. Space capacity one, patient A remains in clinical work past B's deadline:
   B abandons outside, never acquires space, no clinical start, zero occupancy;
   removed space queue has no horizon entry and no capacity release.
3. Space available but triage worker busy: B acquires space then abandons while
   triage queued; queue removed, occupancy ends at deadline, no staff release.
   Later arrivals can use released space without an extra artificial tick.
4. Triage becomes available exactly at deadline: B starts triage, is not
   abandoned, and consumes zero observed work when that tick is H.
5. Multiple expired waiting patients at one tick all abandon according to the
   pre-cancellation eligible snapshot; released space cannot save another
   already-expired candidate. Input permutations preserve byte-equivalent rows.
6. A patient whose first care started before deadline never abandons while
   waiting for assessment, diagnostics, treatment, offers, care or transfer.
7. Deadline==H realizes abandonment and releases occupancy; deadline>H remains
   unfinished. arrival+patience>u64::MAX does not wrap or produce a false event.
8. Independently reconcile abandoned/discharged/transferred/unfinished counts,
   actual intervals, busy/queued/peak capacity and terminal reason/time/LOS.
   Completed service and deadline ties must not underflow busy or double release.

## Following phase audit

The horizon join still binds all 46 existing frozen fixtures without changing
payloads. Add a separate real generated-arrivals-to-terminal integration check
before E1.4: accepted seed11 golden cohort arrivals5,5,6,9,9; capacity5 and fixed
four-stage duration1 yields terminal9,9,10 and two assessment starts atH10 with
zero work. Use actual generate_arrivals and bind_arrivals (two-stage templates),
then explicitly append diagnostics/treatment. Assert exact IDs/identity, stages,
repeatability/permutations, conservation5=3discharged+2unfinished and absence of
false terminal/LOS. This is distinct from the historical 46-case fixture join.
Manual E1.4 traces must reconcile every acuity/pathway and actual row times;
passing individual components alone does not establish composition or phase
acceptance. No milestone ends the continuing user-authorized delivery goal.
