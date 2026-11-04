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
