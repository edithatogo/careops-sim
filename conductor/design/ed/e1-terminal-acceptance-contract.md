# E1.3 terminal acceptance contract

Status: coordinator contract reviewed by GPT-6.1-sol on 11 October 2026;
no dispatch or runtime acceptance follows from this document.
Rust/Cargo 1.99.0 only. The frozen [runtime contract](e1-terminal-runtime-contract.md)
owns semantics. Existing endpoint/extreme/invalid fixture payloads stay unchanged.

## Sequential ownership and join

After accepted boarding, E1.3.exits owns only
`crates/careops-ed/tests/e13_terminal_runtime.rs` and
`crates/careops-ed/tests/support/e13.rs`. The helper builds canonical valid terminal
configs, loads fixtures and supplies independent interval/reduction assertions.
After accepted exits, E1.3.horizon owns only
`crates/careops-ed/tests/e13_native_fixture_join.rs`, using that accepted helper
read-only. Missing helper functionality returns to the coordinator for a separate
claim; no simultaneous helper writer. Production defects return to the runtime
owner in a new bounded packet, followed by both affected test suites.

Each dispatch binds exact HEAD, contract/source hashes, accepted prerequisite
receipts and three actual commands: careops-ed integration/full tests, strict
Clippy all targets, and rustfmt for owned sources. Each packet also owns its
result file, logs and Cargo target directory; exits has five write reservations,
horizon four. Preparation is read-only until the packet is reviewed and claimed.

## Exits lifecycle oracles

Cover immediate and delayed discharge, including pending departure beyond H;
Disabled abandonment with no offer retains the boarding patient; transfer at H14
is unfinished and at H15 terminal for the frozen 11/13/15 anchor. Exercise finite
offer incompatibility and expiry, consumption surviving withdrawal, transfer
capacity contention and ED-space retention until physical departure. An outside
patient cannot start a clinical stage before space acquisition.

Exercise care due coincident with allocation (canceled without acquired resource
or observed work), active care blocking allocation until completion, expired
offers while care is active, recurrent care without accumulated duplicate tasks,
and care sharing a clinical pool under the frozen FCFS ordering. Include
simultaneous zero-delay departure, offer and care-due events; space release/grant
at exactly H must allow zero-work starts without falsely completing them.
Include input permutations, u64::MAX time arithmetic and malformed policies.

Independently sweep half-open clinical/care/transfer/space intervals to establish
busy_ticks, capacity_ticks, peak and capacity conservation. Track grants at H
separately for busy_at_horizon. Canceled queued care contributes neither busy nor
queued-at-H counts and never releases an unacquired resource. Derive conservation,
terminal iff no unfinished reason, completed LOS and clipped observed elapsed
from actual rows, avoiding assertions solely between runtime-produced totals.

## Horizon: all 46 frozen cases

Load exactly [23 endpoints](e1-endpoints.json), [6 extremes](e1-extremes.json),
and [17 invalid cases](e1-invalid.json). Assert exact case counts and IDs so a
new, skipped or duplicated case cannot silently pass.

All 23 endpoints run through `terminal::run_terminal`, including the 12 historical
E1.2-labelled discharge/empty cases. Compare actual patient IDs, clinical stages,
knowledge/decision observations, realized outcome, terminal tick, completed LOS,
observed elapsed, endpoint state/reason, boarding, pending transfer, transfer,
space and offer observations to fixture expectations and frozen runtime anchors.
The fixture boarding_interval measures decision-to-allocation offer wait; it is
separate from boarding_time_ticks, which ends at physical departure. Compare
fixture aggregate expectations with an independent reducer of actual native rows.
This is not a claim of a public runtime aggregate/denominator API.

The six extremes are frozen single-stage projections and use actual E0
`careops_ed::run_scenario`. Do not append artificial clinical stages. Saturated
capacity-one H7: starts 1,3,5,7; completions 3,5,7,None; three completed, one
unfinished. H2: first patient work1, others not started. Duration8 at arrival1:
H7 observed work6 unfinished; H9 completes exactly. Zero duration rejects.
Compare configured work, actual start/completion/status/wait and completed LOS;
observed work and censored elapsed are explicitly test-derived interval values.
Verify permuted input equality.

All 17 invalid cases reach an otherwise valid full-treatment TerminalConfig.
Preserve the malformed fixture value and its offending path/category as fixture
metadata. Assert native InvalidInput and an established diagnostic substring;
do not invent typed categories or event-count fields. Each invalid case has a
positive control repairing only its targeted malformed group. Full-path extension
must preserve repeated/disordered stage mutations using otherwise legal distinct
resource slots. The current terminal prevalidator may reject before the older
pathway validator: bind actual earliest diagnostics at dispatch, preserving the
malformed payload rather than forcing a historical message. Compare fields absent
from frozen fixtures against explicit runtime hand anchors. Review validation ordering to
prove rejection before Scheduler construction; Err alone is not observed evidence
of an exposed event counter. Do not rewrite historical runtime_status labels.

## Evidence and limits

Record actual argv/cwd/base/toolchain/source and fixture hashes, exit status,
counts and raw logs. Independently review the integrated source and repeat gates
on the integrated clean commit. E1.3 closes only after offers, boarding, exits and
horizon are accepted; E1.4 additionally reconciles the whole phase. These synthetic
checks do not validate Cairns, ward-policy causality, ambulance-fleet effects,
clinical fitness, external runtime profiles or release readiness.
