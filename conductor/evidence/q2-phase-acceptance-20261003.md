# Q2 non-preemptive acceptance and C1 partial



Reviewed source through 6b5176b. Priority ordering uses signed resource priority,
then original committed admission sequence. Checked buffered cancellation/rekey
supports pending, queued and active requests. Grant clears the waiting deadline;
terminal claims retain identity and resubmission uses a new request.
Deadline <= dispatch time expires before resource arbitration, in either token
insertion order. Only command-target resources participate in boundary expiry;
owner despawn targets all its nonterminal claim resources canonically. Independent
timeout rows survive rejected explicit rekey. Full arithmetic/preflight errors
still preserve staged ECS state. Deadline tokens use reserved 4002; commands4000.

Independent review found and corrected wrong timeout event kind and unrelated
resource expiry. No further non-preemptive source blocker was found. Generated
qualification uses 32 LCG seeds x100 mixed priority/deadline/cancel/rekey/capacity/
release operations, checking capacity, terminal uniqueness, queue/active membership
and original sequence. Explicit tests cover both release/growth deadline insertion
orders, reverse equal-priority commands, scheduler priority override, active lease
rekey, pending cancel/stale events, unrelated resource causality, and two-token
admission budget preflight without leaked entity/work association.

Executed in the active Kairos checkout: 43 DES tests pass on current Homebrew
Rust/Cargo 1.99.0; pinned rustup1.98.1 complete DES+Arrow regressions also pass.
Release and Rust1.76 DES suites passed before the final growth-only regression;
the added growth test passed separately in release and Rust1.76. Clippy all-targets
with -D warnings passed after the final regression. Logs: parent ignored
.artifacts/blocker-resolution/q2-native.log, q2-release.log, q2-msrv.log,
q2-growth-release.log, q2-growth-msrv.log, q2-clippy-final.log and
q2-c1-pinned-native.log. No empirical inputs/new engine RNG draws.
Timed completion, Suspend/Abort/Restart, notifications and persistent same-tick
budgets remain Q3-Q4. World staging performance remains Q5; no performance claim.
Parent Q2 acceptance still awaits exact final commit native owner CI.

Parallel C1 partial: Luna implemented pure already-resolved UTC arithmetic and
semantic exclusions; independent review corrected conflated missing timezone,
DST ambiguity and DST-gap reasons. Public errors use already-reviewed
thiserror2.0.20. Sixteen Arrow package tests pass with legacy smoke API unchanged.
No timezone parser, IPC/Parquet, empirical mapping or full C1 acceptance follows.
Worker receipt remains .artifacts/c1-temporal/result.json; coordinator fixes and
pinned verification supersede its initial output hashes. Manifests/lock remained
coordinator-owned. Full C1 tasks remain pending.

Final Kairos 5f5a8d9cf5a1ce3ece05312f41f489f8554a115a passed exact native owner CI on Linux/macOS: https://github.com/edithatogo/kairos/actions/runs/37086360809.
Phase gate and strictclean-tree Git closeout passed. Q2.1-Q2.4 accepted. FullC1 and timed/preemptiveQ3 remain pending.
