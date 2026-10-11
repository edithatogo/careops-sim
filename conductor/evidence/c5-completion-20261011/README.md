# C5 accepted synthetic implementation — 11 October 2026

C5.1–C5.5, the C-05 recovery portion and C-06 worker/resume equivalence are accepted
at Kairos `6d733f489225da4bf6ceec2673b02369c7918bbe`, using Rust 1.99.0 only.
Three GPT-6-luna workers implemented study/search, CPU workers/recovery and CLI;
the coordinator owned contracts, native Flow integration and acceptance.

## Delivered

- Bounded finite search, integer-tick constraints, disjoint temporal/day splits,
  fixed seed schedules, complete attempt accounting and frozen validation selection.
- Native Flow shadow/free evaluation, winner-only held-out evaluation, known-parameter
  recovery, noisy tolerance and explicit confounded-ridge/MC-indeterminate states.
- Trace verification, replay, calibrate and compare CLI operations with provenance;
  legacy validate/replay/resume-plan exact compatibility remains covered.
- Independent CPU workers, immutable hash-bound shards, deterministic merge,
  cooperative resume, corruption rejection and fresh-process kill/resume equality.

## Evidence

[Acceptance](acceptance.json), [crosswalk](crosswalk.json), [independent reviews](independent-reviews.json)
and [hosted readback](readback.json) bind the exact source and actual outputs.
Local qualification: **712 native tests + 26 CLI tests passed**, six native tests
explicitly ignored, plus format, strict lint and Conductor gates. Hosted C5 on
Linux/macOS: 22 focused native + 26 CLI tests each. **64 PR checks succeeded**;
Q5.2 canonical native regression and Codecov OIDC upload are conditional skips.
The Windows workspace and its new path regressions also passed.

Each worker-count/resume bundle contains 60 byte-identical files (54 initial and
2 final shards plus metadata). The synthetic winner is route2/work3 with zero
validation and final completion error; final2/2 attempts complete. The small
confirmation sample remains **MC-indeterminate**. Summary bytes/task results agree
across native hosts; architecture-specific provenance differs. Prior accepted C3
checkpoint/result bytes remain unchanged. Archives and manifests retain actual
commands, source/toolchain/seed/configuration identities, exits and artifact hashes.

## Scope and retained corrections

Clinical/empirical validity, ED/C6 integration, stable public API, full MVP,
release and merge remain open. Windows recovery covers process termination;
directory-entry power-loss durability and general parallel speedup are not claimed.
The kill/resume bundle retains exit-9 and exact resumed parity; it lacks an
independent pre-kill snapshot of the driver's shard-poll timing.

Historical CI failures and [output-scope deviations](test-output-scope-correction.md)
remain recorded. The original [commit-scope incident](../c5-implementation-20261011/learning/scope-incident.md)
is retained too. Corrected-source qualification supersedes earlier pending snapshots;
it does not recast those historical runs as scope-compliant. Three reviewed advisory
lessons were added to the existing repository learning loop, including an append-only
correction to the earlier output-reservation outcome. This is not unattended authority.

[Draft stacked PR #245](https://github.com/edithatogo/kairos/pull/245) remains open.
Next preparation candidate: **E1.1 generic ED pathway fixtures**. C6 awaits ED E2.
