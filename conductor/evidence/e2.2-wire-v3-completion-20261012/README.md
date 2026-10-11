# Explicit Flow checkpoint V3 wire acceptance

Accepted local joined commit: `81f889f66969c27048a8d23a58b47c5a75132807` on Rust/Cargo 1.99.0. Core wire commit: `3eae7b741d9957d651883bda646aead18961c220`.

Seven private wire proof families and five public runtime fixture families passed. The final four-package run passed 829 tests, failed zero, and retained six ignored tests across 67 suites; strict Clippy and owned-file formatting passed. Root verified all 22 process input, exit, ASCII-hex and decoded continuation hashes. The unchanged process source retains 19 legacy envelope goldens.

The public fixture source is byte-identical to the independently reviewed draft. The core and fixture source hashes match their qualification receipts and scoped commits. All failed development/format attempts are retained; no final failure was relabelled. Receipts are preserved verbatim, with original absolute provenance paths and this archive's mapping in artifact-manifest.json. Logs, process payloads, source snapshots and contexts are losslessly compressed. No private harness/token/build-cache material is included.

This accepts explicit V3 wire/native restore support only. Routing, actual V3 preview/coherent/portable consumers, the parent Kairos pin, ED bindings and subsequent delivery gates remain open. The next contract is being prepared in parallel; source dispatch waits for its reviewed acceptance and cross-track handoff.
