# D3 parser fuzz qualification — partial

Coordinator reviewed the isolated fuzz source and corrected worker receipt on
2026-10-03. D3.2 remains unchecked: selective Miri/unsafe and CodeQL applicability
and execution evidence are still required. This does not close D3 or release gates.

## Scope and dependencies

Test-only isolated workspace, publish=false, exact libfuzzer-sys0.4.13 and
serde_json1.0.151; production Cargo manifests/lock unchanged. Parser fuzzing
rejects invalid UTF-8, parses/validates, then asserts serialization/reparse
equality. It never runs arbitrary unbounded scenarios. Root dependency policy
is retained, with an NCSA exception only for libfuzzer-sys=0.4.13. All four
cargo-deny categories passed using0.20.2. The failed initial config-argument
placement and wildcard-tightening experiments remain in the receipt.

## Executed local evidence

macOS ARM64, dated nightly2026-10-02, actual rustc1.101.0-nightly
(c36f145712026-10-01), cargo-fuzz0.13.2. Direct selected toolchain binary paths
and explicit RUSTC/RUSTDOC/PATH prevent Homebrew stable compiler substitution.
17 CI guard tests passed after correcting unsupported direct-cargo +toolchain
syntax. Initial sanitizer build failures and masked-pipeline failure are retained
and are not passes. Final pipeline uses pipefail.

The final corpus began with exactly one synthetic2177-byte fixture, SHA256
ecc460f58de1958a0a3c2ce8cdb69f24fb95a22a6c4c6c4d793c0975afeb1f53.
100000 runs, seed20261003, max120s, maxinput32768bytes, timeout5s, RSS1024MiB.
Completed100000 runs in5s, coverage3109/features5553, peakRSS357MiB, no crash.
Log SHA2566723561b9764a12aa2fc282d80b5fa2cd64ef5a4ee4b490f5c135f5f7a44a28f.
Older mutated corpus and coverage are retained separately and are not pristine
seed evidence. Local logs/receipt: .artifacts/d3-fuzz/result.json in isolated
D3 checkout; clean-start log /tmp/careops-d3-fuzz-fresh.log.

## Hosted gate

Ubuntu24.04 x86_64 lane verifies the compiler host triple, pins nightly/CLI,
requires successful bounded fuzzing and unchanged fuzz lock, and is included in
Required checks. Setup/fuzz logs and bounded crash reproducers are retained on
failure. Missing upload files cannot mask a preceding checkout failure. Hosted
execution remains pending until the exact PR commit runs; macOS proof does not
establish Linux execution. No source coverage percentage is inferred from counters.
