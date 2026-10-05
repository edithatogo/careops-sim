# Rust 1.99.0 sole-toolchain policy evidence

## Scope

This package records bounded local evidence for adopting Rust 1.99.0 as the sole current Rust toolchain in the parent repository. It is not hosted CI, release acceptance, or complete security qualification. All raw receipts and logs below are exact byte copies of the identified ignored artifacts; see `acceptance.json` for the path/digest export index.

## Source and execution lineage

- **Foundation source state:** `f874f113763f35708eeb2f39ec61f303b18b0eae`. The recorded Rust 1.99.0 local checks passed 21 quality policy tests, the quality policy check, and 80 Rust workspace tests.
- **CI policy source state:** `f4b5dbb4471005714565852b9237062f337b6592`. The recorded local checks passed 22 workflow tests and 80 Rust 1.99.0 default-feature workspace tests. Its first full 497-test Python harness run had one failure because a test expected the old `cargo +1.99.0` spelling. That failure is retained as historical run evidence and was addressed in the next test-only change.
- **D3.4 assertion source:** committed head `b625d159971616af76db0fd958cf08d31f0fc4b8`. The focused D3.4 test and full 497-test Python harness passed with Rust 1.99.0 environment variables. Those commands ran on execution base `f4b5dbb4471005714565852b9237062f337b6592` with a working-tree version of `tests/test_d34_ci.py`; its SHA-256 is identical to the file committed at `b625d159971616af76db0fd958cf08d31f0fc4b8`. This records tested source content separately from the then-current committed HEAD.

## Scope limits

Miri and libFuzzer AddressSanitizer runtime qualification were not performed. Those runtime lanes require nightly and are unavailable under the current sole-Rust-1.99.0 owner policy. Their absence does not waive later release obligations for FFI/unsafe review, malformed-input and resource-exhaustion assurance, sanitizer/Miri or fuzz evidence, or an equivalent owner-approved method. No hosted workflow run, hosted required-check result, release acceptance, or complete D3/security qualification is claimed.

Two process limitations are retained as process history, not test outcomes: one precommit workflow-test log was created before its artifact-only claim and is kept under `attempt-precommit`; an ephemeral cooperative lease token appeared in a tool response and was subsequently released. No token value is included in this package.

## Files

Raw receipts and logs are stored under commit-labeled `runs/` directories. Historical failure evidence remains beside later passing evidence, with each run's commit/source distinction recorded in `acceptance.json`.
