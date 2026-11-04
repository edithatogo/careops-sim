# R13 local engineering acceptance — 4 November 2026

Accepted source: f7563a8bfaf3a8da623b2373cc369883aca609f6 on the existing
codex/polars-migration-integration lane. This is local engineering acceptance,
not current-head hosted, merge, stable release or clinical/site acceptance.

## Integrated source and review

- a44a938: typed all-patient fixture view and C62/E42-heldout/C64 consumers.
  Unique keys bind E42 display labels to their existing true numeric case IDs;
  online scheduling/work/routes remain unchanged. No string-key parity is claimed.
- 2a4dbec: removes2382 obsolete production lines. Validation/path/seed/cancellation,
  SHA integrity, domain statistics, trajectory APIs and online model projections
  are retained byte-for-byte. No dependency or runtime pin changes.
- 28c2dd1/e494e44: remaining Python/native consumers, real-capability feature
  guards, current walkthrough/support/licensing and explicit ED CI discovery.
- 7b9892b: exact rustfmt-only reader successor plus new26-case receipt, preserving
  every immutable earlier receipt. Unknown sources still refuse; float/Boolean
  case aliases are rejected.
- 66abaea/1d1e5a4: all ED Cargo calls retain locked graphs; actual passing records
  replace nonexistent upstream labels and require all6 parent numeric cases.
  Python catalogue/tolerances are unchanged; no GPU/native-catalogue claim follows.
- 9b406d3/f7563a8: native mutation fixtures preserve original actual file metadata;
  concurrent scratch roots use the existing tempfile allocator. All original
  schema/null/lifetime/numeric assertions remain; no retries or serial execution.

Each packet received independent reviewer-agent source/evidence review. The
coordinator applied review fixes and committed with ordinary privacy hooks enabled;
all commits passed with zero findings and claims released immediately. Other
worktrees and the separate Birthing source lane remain preserved.

## Actual commands and receipts

Working lane: careops-pr194-checks/careops-sim. Python is the verified repo-local
3.15.1 interpreter. Native builds use the existing external target, jobs1,
incremental0, dev/test debug0 and POLARS_MAX_THREADS=1. Test compiler is actual
Homebrew Rust1.99.1/LLVM23.1.3; policy uses canonical Rust1.99.1 tools. No compiler
identity equivalence is inferred.

- `cargo test --locked --workspace`:776 passed across115 groups including
  doctests,0 ignored/filtered. Private log polars-r13-coherent-workspace.log.
- `.venv/bin/python -m unittest discover -s tests -v`:796 passed. Private log
  polars-current-top-level-python.log. Later changes are confined to ED test
  receipts/temporary helpers; no unchanged top-level result was rerun or relabelled.
- Full ED discovery passed126 of127; the invalid E61 label expectation was fixed
  and its exact test passed, executing actual upstream cases and all6 parent
  numeric cases. All127 are accounted; no fresh complete discovery pass is claimed.
- Native consumer22 tests and minimal3-target checks pass after scratch isolation.
  Metadata mutation tests4 pass. Original full-suite failures are retained.
- `cargo fmt --all --check` and23 source-compatibility guard tests pass. Parent
  canonical `cargo deny --workspace --locked --offline check` passes all4 groups.
- Actual external locked release rebuild and `tools/ed_native_benchmarks.py`
  pass26 frozen cases,14 rehashed-file negatives, zero-work/oversize controls,
  slow10k-patient CSV and live unread-pipe termination within unchanged10second,
  256MiB RSS and8MiB stdout budgets. Binary SHA256:
  0c2e8487d3fc8a6fb7128dde24a78c7dfacb903b1d6ca0f1a331c1f58d9b9b88.
  [Exact source/26-case receipt](../polars-integration-20261103/native-reader-format-source-compatibility.json).
  Raw reports/logs remain private external evidence. Linux /dev/full is unverified
  locally and remains a required hosted gate. The opt-in historical conversion
  test returning without environment variables is not conversion proof.

## R14 boundary and ownership

Private populated C Data/Stream tests passed3 real ED/calibration/B7 projections.
Their proposed engine-FFI dependency graph failed policy for duplicate core/rng
sources; it was not adopted and no exemption was added. A reviewed upstream
Track70 packet extracts the unchanged trusted importer into one narrow shared
crate for the old FFI and parent consumers. Arrow's unsafe prohibition and old
engine/default/no-default contracts remain. Same original stopped owner, separate
standard Git worktree and fresh exact reservation are recorded in upstream
handoff; private evidence retains actual paths/tokens. No parent pin has advanced.

R14 requires accepted upstream source/hosted/Git closure, closed parent default-off
FFI/data-source adoption, actual historical read/conversion, final small/large
ensemble/contention/output/build/code-cost, package, coverage/mutation/security/
policy and current-head hosted proof. Earlier4357206 package and92d8daa hosted
receipts keep their exact source scope. Runtime074/dataf347/Polars964 are unchanged.
No archival, merge, registry publication or clinical/site/stable-release approval
is implied by this R13 closeout.
