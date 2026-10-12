# Flow V4 routing acceptance

Accepted source `81eb3a7703e1524ccf19c63981a44e237691ab02` using Rust/Cargo 1.99.0. Strict Clippy, full DES tests and formatting passed: 431 passed, 0 failed, 1 ignored across 44 suites. All eight routing proofs and the retained legacy lifetime proof passed. Three historical failed executions and all reviewed compatibility corrections are preserved.

Next: join accepted ABM source and run the preview, coherent Acquire and portable Handoff consumers in parallel. Parent pin and ED binding remain pending. See [acceptance](acceptance.json) and [manifest](manifest.json).
