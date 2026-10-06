# C2.2 stable seed and bound-retention leaf — 2026-10-06

This evidence accepts one bounded local `C2.2.seeds` instance at Kairos pin
`3d6efca658ba540c96f71f8f72c50214a4ef0eb2`, using stable Rust 1.99.0 only.

The implementation adds assertions to the actual calibration-provider/Flow
fixture around a nonzero Micro route. It checks that the frozen fidelity mode,
original intrinsic duration, bound Service identity and Service draw position
remain unchanged through actual in-memory transit pause/resume and arrival. The
seed-map contract remains explicit about stable task/purpose keys and excludes
thread or wall-clock identity. A supporting package run exercised those
seed-map tests; its runner result (80 passed, 0 failed, 2 unrelated ignored) is
preserved as an oracle mismatch, not a passing runner result. The final exact
runner passes the named retention fixture: 1 passed, 0 failed, ignored or
measured, and 81 filtered. Formatting and strict Clippy also pass at Rust
1.99.0.

This leaf does not establish portable checkpointing, process restart or stream
restoration. The C2.2 and C2.3 parent joins remain open. Track03 owner review,
C2.1 runtime acceptance, hosted checks, release readiness and clinical
acceptance remain open. Full commands, source hashes and artifact hashes are in
`acceptance-rust199.json` and `sha256sums.json`.
