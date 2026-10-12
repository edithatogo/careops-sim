# Context codec schema version prerequisite acceptance

Accepted source commit `34ff8fc54ccbbf976d1c8e3d9158d3456c26ad9b` on Rust/Cargo 1.99.0. Strict Clippy, full DES tests and formatting passed at identical source hashes: 422 passed, 0 failed, 1 ignored across 43 suites; all five required proofs passed. Root and GPT-6.1-sol reviewed the one-line production guard change and substantive runtime/negative controls. All 0 failed executions are retained.

Caller context schemas now accept nonzero u32 values; zero, duplicate registrations, exact manifest matching and restart-template restrictions remain enforced. Existing native and wire layouts are unchanged.

This accepts the prerequisite only. ABM wire qualification and DES routing follow in parallel, then joined consumer proofs before any parent pin or ED integration. See [acceptance](acceptance.json) and [manifest](manifest.json).
