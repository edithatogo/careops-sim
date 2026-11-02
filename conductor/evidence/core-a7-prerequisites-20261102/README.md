# Core A7 prerequisite reconciliation

Base: main merge `672d3e8f78f5e502a760b9d41200fdbaf4d9d144`.
Retained source: `f5689090187e76b738ceedfeb7d0a06270317e04`; its checkout is
unchanged. B0/B1 scoped local acceptance was already recorded in that lineage;
this integration does not replace clinical/site/specification approval.
The sole portability edit replaces an old absolute workstation path in the B0.1
receipt with `<B0_SOURCE_WORKTREE>`; original bytes remain in the source commit.

Workspace addition uses current main dependencies and Kairos `07410266`, adding
only the `careops-birthing` package to the lock. No registry version or engine pin
is advanced. Historical packet files remain historical, not new dispatch authority.

Executed focused checks on the integrated candidate (Python 3.15.1, Rust 1.99.1):

- Python `test_birthing_*.py`: 19 passed.
- Birthing `runtime_conformance`: 7 passed, synthetic seeds 41/77/4 and bounded
  multi-seed trajectories from the retained tests.
- Core `pathway_failing_cases_tests`: 13 passed; `policy_classification_tests`: 6 passed.
- `tools/tasks.py check`: 401 records, valid coverage/prerequisites/DAG.
- `tools/context.py check`: 25 tracks, 109 milestones, no broken links.
- `tools/mvp.py check`: 82 parents, 258 leaves, valid joins/hashes.
- `cargo fmt --all -- --check`: passed.

First attempts retained: offline Cargo lacked cached libc; one command used the
nonexistent selector `policy_contract`; no tests ran in either attempt. After
fetching locked dependencies and using `policy_classification_tests`, all selected
tests executed successfully. Initial context check found eight missing historical
packet links; importing the ten original B0/B1 packets corrected them.

A7 and B2–B7 remain incomplete. No shared cross-domain scheduler, atomic bundle,
clinical calibration, operational acceptance or independent Arrow/Parquet proof
is inferred from these checks.

Integration review: reviewer `/root/review_pr194` compared the imported files
with frozen source and confirmed consumed core interfaces match main. No blocking
findings. Its minor status finding (summary omitted accepted B1.4) is corrected.
Historical candidate receipt plus acceptance commit `409e0b9` retain chronology.
Full core/birthing package milestone test command passed; see private native log.

## B7 native-data dependency candidate

Disposition: candidate; manifests/lock remain unchanged at `fd9ee4c`.
Read-only registry refresh completed with33 successful observations on
2026-11-02T15:53Z. Arrow60.1.0, nextest0.9.150, CodeQL4.39.1,
Renovate44.202.1 and Codex0.166.0 are newer candidates, not adopted changes.
Rust remains canonical1.99.1; no toolchain or broad registry upgrade is proposed.

B7 needs actual IPC/Parquet, absent from the runtime074 data descriptors.
Reuse separately pinned `kairo-ecs-arrow` data commit
`4134a3733ba7e833a8c520621ec2af5a4bd4b31a` with `polars-native`, Polars
`96439685eb2f95108f51087d2b76091917f2593c` (0.55.1) and its exact hashbrown
patch `d76a6f81abd8f388676aff15068236a8d21c7e1a`.
This is the already qualified Track70/PR191 data component. Runtime074,
submodule pin, ED/core source and original owners stay separate. Do not merge
PR191 or promote its unmerged runtime/capture work as a side effect.

Only a new native birthing CLI consumes Polars. Core and browser/Wasm graphs
must remain free of that dependency. Data-component EventId/EntityId types
come from4134; bridge explicit primitive values rather than crossing revision-
distinct Rust types. No generic data framework or old-format compatibility layer.

Seed the existing qualified data closure, review actual lock differences and
retain exact source restrictions, named/versioned upstream licence and duplicate
version decisions. No advisory exemption, blanket source or licence allowance,
skipped platform or weakened per-mutant/coverage oracle is proposed.
Current cargo-deny must prove the actual new graph before acceptance.

Qualification: affected birthing CLI/native composition/independent IPC+Parquet
reader tests, default/minimal core graph check, Rust1.99.1 locked builds and
warnings-denied Clippy, deterministic1/2/4worker/recovery controls, local and hosted
Linux/macOS checks. Final combined-head ED coverage/mutations and all other
required gates remain mandatory; dependency/config drift invalidates earlier
narrow receipt reuse. The measured PR191 outer context75minute/debug0 build
profile may be reused for the same heavy graph; per-mutant180seconds stays fixed.
Rollback is removal of the new CLI edge/package and exact lock/policy changes.
No new package publication, clinical acceptance or full-spec approval follows.
