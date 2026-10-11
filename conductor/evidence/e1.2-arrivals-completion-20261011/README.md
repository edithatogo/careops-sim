# E1.2.arrivals acceptance — 11 October 2026

Accepted at integrated source `c27588e46921e8436a67034a54418abe1a188e5f` on Rust/Cargo 1.99.0. See [acceptance](acceptance.json) and command receipts alongside it.

36 native tests (8 arrivals and 28 existing E0), 12 fixture specification tests, strict Clippy and rustfmt passed. Independent source review matched the accepted hash; golden vectors were calculated independently. Parent minimum Rust is 1.99; Kairos pin is unchanged.

The generator validates bounded inputs, preserves count/acuity partitions, uses a dedicated deterministic stream with unbiased bounded rejection, and assigns IDs before sorting. Within-bin uniform placement is synthetic. P4 loading, triage/pathways and clinical acceptance remain outside this leaf. All 46 composite E1 fixture runtime bindings remain outstanding; E1.2 and E1.4 remain open.

## Evidence and corrections

Logs are gzip archives of exact raw bytes; decompress before comparing receipt log hashes. Worker context is plain text. Only explicitly selected evidence was copied; private leases are excluded.

Worker failures are retained: the initial tests-first wrapper did not capture a reliable exit status. Initial Clippy selected an incompatible Homebrew component because PATH was omitted from its packet. Corrected PATH revealed an existing map_or lint; the coordinator used equivalent Rust 1.99 is_none_or. Final integrated receipts qualify acceptance without erasing failures. The qualification claim was checked during the first native command rather than immediately before; subsequent checks preceded execution, and no scope violations were found. An initial archive attempt used an incorrect relative worker path and stopped before acceptance creation; it was corrected to an absolute path.

The guarded E1.1 handoff actually dispatched this leaf. Retire the bounded heartbeat after independent acceptance readback. Next: prepare E1.2.triage with one module writer and separate contract/review work; no later leaf dispatched.
