# E1.3 horizon parallel join

Coordinator scheduling contract; independently reviewed by GPT-6.1-sol on
11 October 2026. No dispatch until BOTH E1.3.exits instances (native and
abandonment) are accepted and the combined integrated source passes gates.
Rust/Cargo 1.99.0 only. This schedules the existing
[terminal acceptance](e1-terminal-acceptance-contract.md) and
[abandonment/composition extension](e1-abandonment-extension-contract.md).

## Two independent instances

| Instance | Sole source ownership | Required output |
| --- | --- | --- |
| fixtures | crates/careops-ed/tests/e13_native_fixture_join.rs | all 23 endpoints, 6 E0 extremes, 17 invalid cases with repaired controls |
| composition | crates/careops-ed/tests/e13_arrival_terminal_composition.rs | actual fixed-seed arrivals to full terminal runtime with H censoring |

Use two GPT-6-luna workers, one GPT-6.1-sol reviewer and root integration. Both
read the accepted `crates/careops-ed/tests/support/e13.rs` without modifying it.
Keep any extra adapter/builder local to the owned file. Each packet reserves
one source plus its result, logs and target directory (four paths), with exact
HEAD, frozen source/helper/fixture hashes and accepted prerequisite receipts.
Three commands: full careops-ed package tests with `-- --nocapture`, strict
Clippy all targets, and rustfmt for the owned source. Shared helper coverage is
for the configured Disabled scenarios; do not imply it verifies untested
Abandoned rows. Abandonment-specific source tests and traces provide that proof.

The complete instance_set is [fixtures, composition]. A worker receipt or one
accepted instance never closes E1.3.horizon. Integrate fixtures then composition;
repeat combined gates on clean integrated HEAD and independently reconcile
actual traces before E1.3/E1.4 closeout. Existing milestone and release gates
remain unchanged and autonomous delivery continues beyond them.

## Fixture instance details

Assert exact ID sets and case counts, not just loop success. All 23 endpoints
use `run_terminal`, including historical E1.2-labelled cases. Six extremes use
actual E0 `run_scenario` with their frozen single-stage projection semantics.
Seventeen invalid payloads reach otherwise-valid full terminal configs; retain
fixture classification as metadata and compare actual first native diagnostics.
Do not force an old component diagnostic by changing the intended mutation.

Repeated/disordered stages fail the terminal full-path precheck. For duplicate
identity, begin with valid A/B, mutate only B's PathwayPatient ID to A, leave B's
service/decision bindings intact, and expect the early decision-binding error;
restore only that patient ID for the repaired control. Other malformed groups
are repaired only at their declared semantic boundary. Fixed duration zero is
invalid; never replace it with the legal discrete-law placeholder sentinel.

Derive expected aggregate comparisons from actual native patient intervals;
there is no native aggregate/denominator API claim. Offer wait is distinct from
decision-to-physical-departure boarding time. Assert extra abandoned count zero
for Disabled fixtures. Resource helper assertions must use configured capacities
and preserve actual grant/release versus clipped observation distinctions.

## Composition instance details

Use the accepted seed11 five-patient golden cohort (ticks5,5,6,9,9) with exact IDs,
mode and acuity. Generate via `generate_arrivals`, bind actual values via the
existing two-stage `bind_arrivals` template API, then explicitly append one
Diagnostics and Treatment stage. All four stages take one tick and all clinical
and space capacities are five. H=10. Transfer capacity stays valid but unused.

Hand oracle: two patients terminate9, one terminates10, two start Assessment at
H10 with zero work and no decision/outcome/LOS. Arrived5=discharged3+unfinished2;
transferred=abandoned=0. Assert identity preservation, each clinical interval,
fixed-seed rerun equality and permutation invariance. Do not substitute the
preparation agent's alternative capacity-one 2/3/5 timeline for this contract.

## Actual trace evidence

Each endpoint run and the composition run prints one bounded stable-prefixed
JSON line containing case ID and the serialized ACTUAL native result. No
expected/fabricated trace rows. Console logs are captured in the owned log path;
no extra file side effects. Root consumes those records independent of parallel
test-print ordering to produce the E1.4 manual trace across all acuity/pathways,
including actual clinical, discharge, admission, boarding and transfer timings.
Zero/negative cases retain their real errors. A passing assertion alone is not
a substitute for that phase-level manual reconciliation.
