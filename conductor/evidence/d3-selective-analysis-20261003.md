# D3 selective Miri and CodeQL availability — 2026-10-03

## Miri scope

CI now has a changed-scope required Miri job on `ubuntu-24.04`, bounded to 20
minutes and pinned to `nightly-2026-10-02`. It installs the Miri and `rust-src`
components, checks Linux/x86_64 and the actual compiler host triple, resolves
`rustc`/`cargo` from the selected toolchain directory, and captures toolchain,
component and sysroot setup output. The only Miri test target is
`libs/kairos/crates/kairo-ecs-ffi/tests/ffi_integration.rs`, invoked through
`libs/kairos/Cargo.toml`. The changed-scope aggregate requires Miri to succeed;
failure, cancellation, skip, or a missing result fails the check. Setup and test
logs are retained as a seven-day workflow artifact, including on failure.

Coordinator-provided local pointer-fixture evidence is the dated Miri setup and
FFI-target logs at:

- `/private/tmp/careops-d2-main-acceptance-20261003/.artifacts/blocker-resolution/d3-miri-setup.log`
  (SHA-256 `35c37eb5911d8b4dcf657e3059e42c2178ca0c0957ac255ead32e2d3e03f5346`)
- `/private/tmp/careops-d2-main-acceptance-20261003/.artifacts/blocker-resolution/d3-miri-ffi.log`
  (SHA-256 `860ed662bfeb21e5731acbc52ce517fc1f501e43ba6bd7beedcfd67c46b721f3`)

Those logs show 14/14 tests passed on the coordinator host `aarch64-apple-darwin`.
They are local host evidence, not Linux hosted-run proof. The new Linux job has
not yet run on GitHub. This slice does not cover FMI, Wasm, optional unsafe
backends, the entire workspace under Miri, or release safety.

## CodeQL disposition

Coordinator-reported GitHub readback for the current private individual
repository: code-scanning alert listing returned HTTP 403 with a disabled-scanning
response, and `security_and_analysis` was `null`. Therefore code-scanning access is **currently unavailable through this repository
endpoint**, with repository eligibility still unverified; this is not evidence of zero alerts, enabled
Rust extraction tooling being unsupported, or a clean scan. No repository visibility, security setting,
CodeQL action, or workflow configuration was changed in response.

## Status

This packet adds a local CI definition and aggregate guards. Hosted Linux Miri
evidence is still required. D3.2 and full D3 acceptance remain open.
