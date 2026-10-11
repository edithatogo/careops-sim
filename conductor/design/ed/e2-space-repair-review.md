# E2.1 space specification repair review

Status: repair in progress, not acceptance. Root and GPT-6.1-sol independently
reviewed the full 32-case draft after the worker reached its correction limit.
The three failing assertions understated the semantic defects. Preserve every
original attempt log and the immutable dispatched packet and recorder.

## Findings and bounded recovery

- Cleaning/reservations: missing cleaning claim space references, ignored ready
  cleaning requests without departures, delayed reservation consumption despite
  an eligible owner, missing ordinary grants after rejection, and dirty/cleaning
  rejection times inconsistent with the declared lifecycle.
- Diagnostics: a worker busy only until 3 could not explain a wait until 5.
  Correct the actual configured work and claim, retain the diagnostic room and
  ED space, and represent room and staff waits separately.
- Boarding: undeclared staff, a reused consumable offer, missing admission inputs,
  duplicate occupancy rows, skipped cleaning after departure, and transfers
  misclassified as discharges.
- Permutation: room state and horizon counts contradicted actual occupancy.
  Comparing a supplied invariant flag does not prove canonical input equality.
- Checker: partial hand witnesses and incomplete reference, calendar, lifecycle,
  conservation and mutation-isolation checks could permit false acceptance.

Root authorized JSON-only checkpoints: first 11 cleaning/reservation cases,
then three diagnostics, then six boarding/permutation cases. The worker pauses
for coordinator readback between groups. Checker, negative controls and crosswalk
follow only after those input/oracle repairs. Do not build a second simulator;
use independent literal witnesses and narrow interval/reference invariants.

Checkpoint A SHA256 de223c8c88e2f77ec5f2fb6d53b2c6cb4f7ee54e88f4c510e8b02b4660cd31fc
corrected reservation timelines, but root found remaining null cleaning-space
references and omitted waits. The next checkpoint corrected those and diagnostic
work timing. Checkpoint B SHA256
bd2845cccfb5ee9d5e17cd5b01fc546ac7a0a57e175475deba8174376911ce1c
was read back before authorizing the final positive group. These hashes describe
intermediate drafts, not frozen final sources or accepted evidence.

## Remaining acceptance

All 20 positives must reconcile configured inputs, complete literal histories,
claims, waits and horizon conservation. All 12 negatives must start with a valid
complete baseline, alter one semantic group, reject through the same validator,
and repair exactly to baseline. Check raw case count and unique identities.
Record Report29 ST001-009 delivery versus E2.3 debt explicitly. Then run the
three immutable wrapper gates, retain every attempt, independently review frozen
source hashes, commit under the worker claim, integrate and qualify the join.

All native bindings remain unbound until E2.2. Rust/Cargo 1.99.0 remains required.
Parallel preparation continues for the pure parent dispatch policy and Kairos
resource grant gate; neither implementation is dispatched by this review.
