# Transit V2 direct wire acceptance

Accepted at clean Kairos commit `92735f04573a2d66dee8a789827ac235f7f41034`, following integration of the accepted DES context schema prerequisite. ABM source remains exactly the preserved candidate. Rust/Cargo 1.99.0 strict Clippy, full ABM tests and formatting passed at identical hashes: 84 tests passed, none failed or ignored, across nine suites; all ten required proofs passed.

Root and GPT-6.1-sol independently reviewed source and paired raw evidence. Both original failed executions remain recorded; this rebound added no failures. The source-only malformed witness fixture correction is now validated. Schema2 registration is used without fallback.

This accepts direct wire only. DES routing remains active; accepted routing and wire then join for parallel consumer work and generic Handoff process proofs before any parent pin. ED production provenance and native fixture bindings remain separate. See [acceptance](acceptance.json) and [manifest](manifest.json).
