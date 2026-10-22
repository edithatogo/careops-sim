# Test matrix

Future oracle: Error variant/code/message parity and source-chain tests; FFI mappings unchanged; no sensitive fields exposed. Count net deleted boilerplate and compile-time impact; leave trivial cases if conversion adds no value.

Run focused native tests plus applicable existing CI lanes; record cwd/commit/toolchain/seed/input hashes/exit codes/raw artifacts. Planning: `python3 tools/tasks.py check` and `python3 tools/context.py check`. No runtime acceptance yet.
