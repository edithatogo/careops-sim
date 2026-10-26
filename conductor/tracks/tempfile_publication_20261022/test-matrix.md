# Test matrix

Future oracle: Concurrent directory creation/isolation; cleanup on normal/error paths; intentional retained diagnostics; same-filesystem publication, failure before/after persist, no-clobber and old-output preservation. End-to-end process-kill/disk-full/manifest recovery remains mandatory in Polars and is not certified by this supporting track.

Run focused native tests plus applicable existing CI lanes; record cwd/commit/toolchain/seed/input hashes/exit codes/raw artifacts. Planning: `python3 tools/tasks.py check` and `python3 tools/context.py check`. No runtime acceptance yet.
