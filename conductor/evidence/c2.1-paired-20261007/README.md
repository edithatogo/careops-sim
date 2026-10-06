# C2.1.paired — Rust 1.99.0 local leaf acceptance

Kairos change `bfbefa87ba15488837b01020f343685907ae09ac` extends the existing actual-Flow pair. Macro with Route intent and explicit-zero Micro with Zero intent use the same Service identity, derived seed and draw position. The fixture compares matching requests, sampled work, Flow dispatch trajectory, terminal progress and request/work states, completion time, and scheduled event counts. Two independent packet reviewers and two independent source reviewers returned READY.

The fixture has one matching timed-work trajectory and no transit event. Its typed deterministic TransitContext has no RNG field, so there is no Transit-purpose stream in this context to consume; this is a fixture-scoped statement, not a global RNG claim. The worker’s initial time-expectation and formatting failures, corrections, and latest successful outputs remain in the logs.

The parent exact-pin join at this commit reports 442 passed, 0 failed, and 5 ignored across 43 result summaries; strict Clippy and formatting pass. See the parent receipt at `conductor/evidence/c2.1-parent-rerun-rust199-20261007/acceptance-rust199.json`. Track03 owner/phase review, C2.4 phase review, portable restart, hosted checks, release, and clinical acceptance remain open.
