# E2 resource grant-gate runtime qualification

Accepted narrowly after root and GPT-6.1-sol source/oracle review and fresh
qualification on9cd9361: 307 native DES tests, strict all-target Clippy and
owned-path formatting under Rust/Cargo1.99.0. Runtime source is382b333; reviewed
V2 contract follows as9cd9361. Parent Kairos pin remains6d733f4.

The native gate blocks new grants, preemption and resume while active work
drains; same-tick closure, enable ordering, horizon and admission/budget atomicity
are directly exercised. V1 DTO/wire stays unchanged and rejects disabled/pending
gate state. V2 and consumers must qualify before composed parent binding.

Archive preserves22worker wrapper attempts (8failed,14passed), one prior frozen
result, original packet/context/driver, reviewed contracts and three fresh
qualification receipts. Compressed rawlog hashes in receipts refer to uncompressed
bytes. Stored-byte manifest excludes itself. Private leases, harnessstdout and
build caches are excluded; workflow limitations remain in evidence-history.json.

No E2phase/MVP, fulljustci, hosted/platform, release, empirical or clinical
acceptance follows. Continue checkpoint native DTO/capture/restore, then V2 wire,
then calibration consumer joins, before parent pin and single-Flow binding.
