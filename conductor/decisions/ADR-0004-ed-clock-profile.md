# ADR-0004: Generic ED clock profile

**Status:** Accepted for the P0.2 *interface contract* on 2026-09-30 by the
CareOps coordinator after the independent C0 clock supplement. Rust runtime
implementation and calibration trace adapter remain unverified.

## Decision

The CareOps ED profile v1 uses decimal seconds externally and one nanosecond
per Kairos `u128` logical tick. Its ED adapter, rather than Kairos core,
converts values. Fixed duration input is parsed exactly and must scale to a
nonnegative integral nanosecond count within `u128`; trailing zeroes may extend
beyond nine decimal places. Reject binary-float fixed config input, non-finite
spellings, negative values, sub-nanosecond values and overflow.

Sampled finite nonnegative durations are rounded upward to the next tick from
the exact rational sample or exact represented value of a versioned sampler
output. Zero stays zero. Each rounding error is less than one nanosecond; the
positive bias can accumulate. A run records sampler/conversion versions,
precision and rounded-draw count. Floating values never order events.

The generic MVP uses relative schedules with logical origin zero. A
timestamped dataset declares one UTC origin; explicit-offset instants are
normalized before checked subtraction. The original timestamp, offset/zone,
precision and occurrence versus recorded/message lineage are retained by the
surrounding versioned trace event record; the single-instant converter does
not encode an event role. Naive
local timestamps and local recurring schedules are disabled until a separate
IANA-zone adapter defines and tests fold/gap behavior and any gap-shift policy.
No finer observed precision is inferred from nanosecond ticks.

## Evidence and limits

- Kairos core's `SimTime`/`SimDuration` use checked `u128` ticks and support a
  nanosecond precision mode; it does not prescribe a global quantum or
  calendar-time adapter. The C0 supplement recommends this scoped profile:
  [P0.2 C0 clock supplement](../evidence/p0.2-c0-clock-supplement-20260930.md).
- [Reference fixture](../../tools/ed_clock_reference.py) and its tests cover
  fixed exactness/rejection, sampled ceiling/version/error, UTC origin,
  explicit offsets, source precision and rejection of naive local timestamps.
  Multi-field occurrence/recorded/message lineage awaits a separate C0 trace
  record and adapter test.
  `python3 -m unittest discover -s tests -p test_ed_clock_reference.py -v`
  passed 8 tests locally on 2026-09-30. This is a Python contract fixture, not
  proof of Rust adapter or Kairos runtime behavior.
- P0.2 row 90 and other duration/schedule rows must reference this decision.
  E0/C0 mapping, resource bindings and the full P0.2 join still need review;
  this ADR does not accept empirical values or distributions.
