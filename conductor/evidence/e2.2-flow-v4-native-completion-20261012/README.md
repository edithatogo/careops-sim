# Flow V4 native source acceptance

Accepted commit `a178a46b2399c48b00755c13e8c209ad63dd5685` on Rust/Cargo 1.99.0. Strict Clippy, full DES tests and formatting passed at identical source hashes: 403 passed, 0 failed, 1 ignored, all ten native proofs.

Five failed attempts are retained with paired receipts and raw logs. The original unqualified candidate and separate test repairs are distinguishable. Capture uses the existing runtime/caller registration-key accounting; restore uses serialized image strings. Exact-bound and one-byte-under rejection checks retain positive and zero codec-call controls.

This accepts native DES V4 only. Wire, routing, ABM, consumers, parent pin, ED and release remain separate. Direct V4 wire is next; ABM arrival-frontier repair continues in parallel. See [acceptance](acceptance.json) and [manifest](manifest.json).
