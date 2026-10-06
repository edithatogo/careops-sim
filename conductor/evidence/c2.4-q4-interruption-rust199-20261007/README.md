# C2.4 Q4 urgent interruption integration — Rust 1.99.0

**Local leaf disposition: accepted at the exact integrated Kairos pin. C2.4 phase acceptance remains withheld.** This evidence closes the bounded C2.4 Q4 urgent-interruption runtime leaf only; it does not close Track03 owner/phase review, Track22 portable checkpoint/restart, hosted checks for this exact head, release, clinical acceptance, or the C2.4 Conductor phase review.

## Implemented behavior

At Kairos commit `6053d8dd2f8057c423decc929c90a8bcd10837f9`, a test-only calibration bridge fixture runs Macro and routed nonzero-transit Micro through the actual Flow runtime. A capacity-one blocker holds until tick 5. The seeded service provider samples 20 ticks; an urgent two-tick request at tick 8 suspends the same service work with 3 useful ticks elapsed and 17 remaining. The urgent request completes at tick 10, the original work resumes, and it completes at tick 27.

The test observes the Micro route's two progress callbacks and one accepted arrival, then submits the original service request once. It asserts the bridge WorkId maps only to its one RequestId and records exactly one queued, granted, preempted, resumed, and completed transition. While suspended, the request remains queued behind the urgent allocation; after completion, the resource is fully available with no active or queued requests. The same service key, derived seed, sampled duration, and draw position are preserved across the paired fixture runs.

Elapsed time is read from runtime state and callbacks, not just arithmetic: Macro is queue 5 + useful service 20 + interruption 2 = 27 ticks; Micro is transit 2 + queue 3 + useful service 20 + interruption 2 = 27 ticks. Existing accepted C2 evidence covers Macro's no-transit behavior and the TransitContext's lack of a Transit RNG stream; this leaf adds no RNG or production API.

## Verification

- Worker checks, at the clean isolated worktree and exact source commit, used only `rustup run 1.99.0`: formatting passed; all calibration crate tests passed (164 passed, 0 failed, 4 ignored); strict Clippy passed with warnings denied.
- Coordinator exact-pin rerun at the parent submodule pin used only Rust 1.99.0: DES, ABM, and calibration packages passed 447 tests, 0 failed, 5 ignored across 43 test summaries; strict Clippy and formatting passed.
- An independent read-only source review returned READY against the bound packet. The parent receipt records the packet, worker result, source, and log hashes.
- The accepted Q4.5 development commit `1125b5268bb349a5befe12f5789045042faab3e3` is an ancestor of the integrated source commit. Q4's prior hosted result is not a hosted check for this new C2 head.

The retained worker packet and results, bounded worker context, correction logs, and final verification logs are in this directory. Earlier failed test attempts are preserved as diagnostic history and are not counted as passing evidence.

## Open gates

Track03 owner/phase review; Track22 portable checkpoint/restart; hosted exact-head checks; release acceptance; clinical acceptance; and C2.4 Conductor phase review remain open. No C2 phase checkbox is changed by this local leaf.
