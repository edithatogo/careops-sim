# C2.3 route and public API current-pin verification

This evidence refresh reruns the immutable C20 nine-case route fixture and the external C23 three-case public API smoke against the parent-pinned Kairos source commit `6364a92b12a2b1c186ab1477814bc3dfb8a50aef`, using Rust and Cargo 1.99.0 only. Both suites passed. Exact input hashes, commands, result and log hashes are in `acceptance-rust199.json`; raw logs are preserved beside it.

This closes only the local current-source verification freshness gap for those route/API fixtures. It does not claim full C2.3/C2.4 acceptance, checkpoint/restart acceptance, hosted exact-head checks, release readiness, or clinical acceptance. The C2.4 phase remains unchecked pending its owner decisions and remaining gates.
