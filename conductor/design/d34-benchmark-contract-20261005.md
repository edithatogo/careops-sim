# D3.4 bounded native benchmark contract — 5 October 2026

Authority: user requested D3.4; development-readiness specification, ownership
contract and execution model govern this leaf. Base parent 0cc8d10; Kairos
8cd03c8f791ae58b33e5cc61b244071937a839ac. Rust 1.99.0, locked release CLI.
No upstream source, benchmark threshold, schema or dependency change.

## Frozen development budgets (before measurement)

Synthetic E0 FIFO shell: simultaneous tick-1 arrivals, work duration two ticks,
one staffed/open slot, seed 7, padded ascending patient IDs. Sizes 100, 1,000,
10,000, two isolated processes per size. Twenty further 1,000-patient processes
form the bounded process soak. This is ED resource-load representativeness only;
full clinical pathway/ABM representativeness awaits E1/E2 and must be requalified.

Each child: maximum ten seconds wall time, 256 MiB peak resident memory,
eight MiB input and eight MiB captured stdout. Child peak RSS is measured with
POSIX wait4, normalized to bytes for Linux versus macOS; it is not per-patient
memory or a robust tail percentile. Report individual repetitions, no pooling
with Q5.2 or C1.4. Budgets apply to this development fixture, not a release SLA.

Oracle: every result row k has start 1+2k, completion 3+2k, waiting 2k;
all input identities/work/arrival fields and count conservation match. Compare
repeat summaries byte-for-byte after canonical JSON encoding, independent of
input filename. Bind binary and exact input bytes by SHA-256; record actual
source HEAD, host, commands, exits and output hashes. Bad/missing evidence fails.

## Adversarial boundaries

Execute real zero-duration rejection, a delayed stdout consumer, termination
of a blocked child, and Linux /dev/full exhausted-output failure. Harness refuses
oversized inputs before process dispatch. Timeouts kill and reap the child and
remain failed measurements. Cancellation means external process termination;
no cooperative simulation cancellation or checkpoint/resume claim follows.
macOS has no /dev/full: label that check unverified there, require Linux evidence.

The CLI currently reads/serializes without native byte limits. Harness preflight
limits do not harden arbitrary direct CLI or library use. Carry native bounded
input/output and cooperative cancellation into D4.1/E3 qualification; no native
v1 acceptance until those owner gates pass. Scheduler same-time loop protection
is not inferred from rejecting zero-duration scenario input.

## Workers and review

Runner writer: tools/ed_benchmarks.py, tests/test_ed_benchmarks.py,
.artifacts/d34-runner in isolated codex/d34-runner worktree. Coordinator owns
this contract, D3.4 evidence/status and integration; CI writer is separately
reserved. Worker may run focused tests, compile the locked Rust CLI, and run
this contract only. At most three verification commands per packet. Independent
review checks result oracles, bounded IO, kill/reap and RSS accounting. Workers
return ready_for_review; coordinator accepts only actual commands and source.

No changes to strict D3.3 mutation catalog/source binding or Track49/C4 paths.
No release, clinical, GPU, distributed or later visualization qualification.
