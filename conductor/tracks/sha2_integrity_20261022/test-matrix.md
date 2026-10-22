# Test matrix

Future oracle: Independent standard empty/abc/multiblock/boundary-length vectors; Python hashlib or another independent implementation; single-bit corruption, truncated/invalid digest, full-byte coverage including high tick bits; old-format rejection/version tests. Do not treat the old custom hash as a SHA-256 oracle.

Run focused native tests plus applicable existing CI lanes; record cwd/commit/toolchain/seed/input hashes/exit codes/raw artifacts. Planning: `python3 tools/tasks.py check` and `python3 tools/context.py check`. No runtime acceptance yet.
