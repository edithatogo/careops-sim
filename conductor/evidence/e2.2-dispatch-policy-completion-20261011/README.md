# E2.2 native pure dispatch policy

Accepted narrowly after root and GPT-6.1-sol source/oracle review and integrated
Rust/Cargo 1.99.0 qualification: 143 native tests, including 25 policy tests and
one compile-fail constructor-boundary doctest; strict Clippy and formatting pass.

Worker source fd8f7ea was integrated as 1dd97ec. The coordinator committed the
identical approved source after confirming the worker stopped and recovering its
expired lease; a fresh claim covered the commit. See acceptance.json and
coordinator-takeover.json for exact hashes and evidence-history.json for limits.

The archive preserves 21 wrapper attempts (6 failed,15 passed), two prior frozen
results, immutable dispatch packet/context/binding and three integrated gate
receipts. Compressed raw logs use deterministic gzip; receipt hashes refer to
uncompressed bytes. Private lease/harness outputs and build caches are excluded.
Manifest hashes cover stored bytes and exclude the manifest itself.

This proves pure validated selection only. Engine gates, checkpoint/consumer join,
single-runtime migration and all88 E2 native fixture bindings remain open. No
phase, MVP, empirical, clinical, hosted or release completion is claimed.
