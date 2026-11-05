# Test matrix

Future oracle: Error variant/code/message parity and source-chain tests; FFI mappings unchanged; no sensitive fields exposed. Count net deleted boilerplate and compile-time impact; leave trivial cases if conversion adds no value.

Run focused native tests plus applicable existing CI lanes; record cwd/commit/toolchain/seed/input hashes/exit codes/raw artifacts. Planning: `python3 tools/tasks.py check` and `python3 tools/context.py check`. No runtime acceptance yet.

## R56 executed candidate qualification

[R56 receipt](../../evidence/thiserror-r562-qualification-20261105/qualification.json) records125cases/all87variants, valid message/source-chain negatives, exact registry and cached-source integrity, Rust1.99.0 floor, both rootSyn3.0.6/fuzzSyn3.0.9 closures, three root resolver variants plus separate fuzz projection,20 independent build-identity controls and four fresh configured audit contexts. Frozen latency passes only absolute+34.25ns/error rule;1.60045 ratio is disclosed. No whole-parent/platform or artifact acceptance inferred.
