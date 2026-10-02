# careops-ed

The E0 Rust library and CLI provide a strict, deterministic skeleton for
synthetic supplied patient work items. It does not implement ED pathways or
assert clinical validity.

From the repository root:

```sh
cargo run --locked -p careops-ed-cli -- run crates/careops-ed/examples/one_patient.json
```

The CLI validates the configuration, schedules arrival and work-completion
events with Kairos, and writes a JSON summary plus an SHA-256 input manifest to
stdout. The example's IDs, tick values and capacity/staff counts are invented
test inputs, not observations or defaults. A tick has no conversion to minutes
or seconds. The recorded seed is not consumed because this E0 fixture contains
no random draws.

Schema and runtime checks:

```sh
cargo test --locked -p careops-ed
cargo test --locked -p careops-ed-cli
```

The workspace MSRV is Rust 1.76.0; a compatible `Cargo.lock` is committed so
the declared minimum toolchain can reproduce dependency resolution. Run with
`cargo +1.76.0 ...` when using rustup's toolchain aliases.

The input uses P0's proposed capacity/location schema v3. E0 only runs resource
buckets whose open and staffed counts are known. Optional route distances must
use the P0 unit `m`, but E0 does not simulate movement. E1/E2 own patient
pathways and operational constraints; later calibration, exports and release
qualification remain separate gates.
