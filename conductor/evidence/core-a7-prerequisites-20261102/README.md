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
