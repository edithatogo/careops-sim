# E0 native validation receipt — 2026-10-02

Target commit: `c70e26c` (E0.2 skeleton reconciled on the P3 acceptance branch).
Working directory: `/private/tmp/careops-sim-p33`.

## Environment and pinned inputs

- Host: `aarch64-apple-darwin`.
- Rust/Cargo toolchains executed via rustup: 1.76.0 and 1.98.1.
- Kairos submodule: `a71adfd48f42d7c4d04bcb034aad09295c004f40`.
- Conductor submodule: `7a5c560a4fdf5297be58594cc37527eb12790272`.
- `Cargo.toml` SHA-256: `fc08ae10213daaa7772d607b165508fdfe71d25b293a1d0b7b1244902590ff3d`.
- `Cargo.lock` SHA-256: `e7ed84c382561bedf9d26ac3ecc2a1649e8cfe8ab7c7af763f10ec047094d0a3`.
- Example input SHA-256: `ecc460f58de1958a0a3c2ce8cdb69f24fb95a22a6c4c6c4d793c0975afeb1f53`.

## Executed commands

| Command | Exit | Result/log |
| --- | ---: | --- |
| `rustup run 1.76.0 cargo fmt --all -- --check` | 0 | pass; `.artifacts/e0/fmt-rust-1.76.log` (empty output) |
| `rustup run 1.98.1 cargo fmt --all -- --check` | 0 | pass; `.artifacts/e0/fmt-rust-1.98.log` (empty output) |
| `rustup run 1.76.0 cargo test --locked --workspace` | 0 | 55 tests passed; `.artifacts/e0/cargo-test-rust-1.76.log` |
| `rustup run 1.98.1 cargo test --locked --workspace` | 0 | 55 tests passed; `.artifacts/e0/cargo-test-rust-1.98.log` |
| `rustup run 1.76.0 cargo run --locked -p careops-ed-cli -- run crates/careops-ed/examples/one_patient.json` | 0 | output `.artifacts/e0/e0-one-patient-rust-1.76.json` |
| `cargo run --locked -p careops-ed-cli -- run crates/careops-ed/examples/one_patient.json` (Rust/Cargo 1.98.1) | 0 | output `.artifacts/e0/e0-one-patient-rust-1.98.json` |
| `cmp .artifacts/e0/e0-one-patient-rust-1.76.json .artifacts/e0/e0-one-patient-rust-1.98.json` | 0 | byte-identical outputs |
| `python3 -m unittest discover -s tests -v` | 0 | 296 tests passed; `.artifacts/e0/python-tests-20261002.log` |

Both CLI outputs have SHA-256 `bd01bb5e23c96d60ebd9a646f5e704b7987213f0eb3eaa38d5ec6301f58678a5`.
The hand-computable patient result is one arrival at tick 1, start at tick 1,
completion at tick 3, zero wait, two supplied work ticks, and one completed / zero
unfinished. The run seed is recorded but the E0 runner performs no random draws.

Log SHA-256 values:

- Rust 1.76 formatting: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- Rust 1.98 formatting: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- Rust 1.76 tests: `c8d1e0929dce3f2eb53bae8aa97ca2d90d99beb0380f8212df1962b3a25fd31a`.
- Rust 1.98 tests: `0537fc485b427379ad3661917f411b003364881862ac97569296872c575a98c8`.
- Python harness: `26e5d13a72636b9c8df23e6f134b67509005f12e62860fd7f30bd193d951840c`.

These are local checks on the stated host. They do not establish hosted CI,
release support on other systems, remote submodule availability, or clinical or
Cairns validity.
