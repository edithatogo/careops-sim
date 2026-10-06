# C2.2 seed snapshot continuation — 2026-10-06

This evidence bundle records a bounded `C2.2.seeds` instance at Kairos commit
`9b360f433210d38868e1e5f102473fd6f148fccf`. The change adds one unit test for
the existing opaque in-memory `CalibrationStreamSnapshot` contract; production
logic and API do not change.

Before the edit, the exact test filter compiled the real crate test target and
selected zero tests (`running 0 tests`). This is a test-presence baseline, not
evidence that restore behavior was broken. After the edit, the exact named test
passed once and compared a restored stream's key, draw position, next `u64`,
and following `u32` against an uninterrupted same-identity control. Formatting
passed on stable Rust 1.99.0.

This proves an in-memory RNG continuation test case only. It does not add or
prove Flow bridge snapshotting, process restart, serialization, portable
checkpointing, full C2.2 parent acceptance, C2.1 runtime acceptance, hosted
checks, release readiness or clinical acceptance. Those remain separate gates.
