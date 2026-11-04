# Test matrix

Future oracle: Independent standard empty/abc/multiblock/boundary-length vectors; Python hashlib or another independent implementation; single-bit corruption, truncated/invalid digest, full-byte coverage including high tick bits; old-format rejection/version tests. Do not use the historical pseudo-hash or the candidate implementation itself as an independent SHA-256 oracle.

Run focused native tests plus applicable existing CI lanes; record cwd/commit/toolchain/seed/input hashes/exit codes/raw artifacts. Planning: `python3 tools/tasks.py check` and `python3 tools/context.py check`. No runtime acceptance yet.

## R50.2 executed scope — 4 November 2026

[Source-bound qualification](../../evidence/sha2-r502-qualification-20261104/README.md):
22 vectors compare the extracted current Rust helper and exact0.11.1 candidate
to Python hashlib; six probe-comparator negative controls;42 paired local
measurements meet frozen budgets; actual build.rs and independent transcript
agree on eight relocation/source/manifest/flags/features cases. Cargo-deny
advisories/bans/licences/sources pass on the exact isolated closure. Local
Rust1.99.1/Python3.15.1 verified; no fresh Linux/x86 runtime/MSRV1.88 claim.
R50.3 review and parent R51 contracts/recovery/platform tests remain unaccepted.

## R51 executed scope

Tests-first c6bccc5 precedes implementatione44946b. Actual default parent tests:
2 digest contracts (22 independent vectors),10 recovery tests including24
process kills and4 invalid-part cases,5 C63 tests with100ms unchanged. Minimal
normal graph adds2 executed digest contracts. Explicit original/actual successor
artifact proof passes with different model IDs and9 original files unchanged.
Current CLI26 frozen pairs and14 negatives/zero/oversized/slow/blocked stdout
pass; strict affected Clippy,28 workflow-policy tests and canonical tool/metadata
policy pass. macOS `/dev/full` remains unverified for the new-source Linux R52
check. Closed successor guards retain all old source/receipt/lock/pin negatives
and add digest/build/test source and receipt co-drift controls. No new full
hosted/workspace/coverage/mutation or release pass is claimed.

## R52 final scope

[Final source evidence](../../evidence/sha2-r52-acceptance-20261104/README.md) closes the earlier platform/hosted/quality gaps: Linux `/dev/full` passes; all7workflows and required aggregate pass. Both native791test suites, default779, Python825+127,512/578line coverage and117mutants complete, plus supplemental exact-helper fault controls. Reused R51 recovery and R50.2 paired costs remain exact-source bound; no broad local rebuild or portable speedup claim.
