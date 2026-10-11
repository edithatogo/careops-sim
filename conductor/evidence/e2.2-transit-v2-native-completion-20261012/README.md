# Transit V2 native source acceptance

Accepted commit `6ef40266a6c5adb32362e7ceac136e0612c1b2c1` on Rust/Cargo 1.99.0. Strict Clippy, full ABM tests and formatting passed at identical source hashes: 63 passed, 0 failed, 0 ignored, all ten native proofs.

Four failed attempts are retained with paired receipts and raw logs. The test-only repair stopped at its production-scope boundary. A separately reviewed production packet corrects arrival frontier time after successful action handling; legacy delayed Resume at tick five and real service completion are covered. Historical Ready Acquire restoration retains real completed request evidence.

This accepts native ABM Transit V2 only. Wire/schema2 codec, joined consumers, parent pin, ED and release remain separate. Direct V2 wire is next, parallel to DES direct V4 wire. See [acceptance](acceptance.json) and [manifest](manifest.json).
