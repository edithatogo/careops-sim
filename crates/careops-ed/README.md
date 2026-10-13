# careops-ed

The Rust library and CLI run synthetic emergency-department fixtures. E0 is a
strict, deterministic skeleton for supplied patient work items. `src/pathway.rs`
implements an experimental pathway runtime for seeded arrivals,
triage/assessment, diagnostics, treatment, and disposition. Inputs are
synthetic. The crate does not assert clinical validity, and it is not the E2
headless MVP or a release-qualified ED simulation.

The workspace toolchain is Rust 1.99.0 (`rust-toolchain.toml` at the repository
root; `Cargo.toml` `rust-version` is `1.99`). From the repository root:

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

The workspace MSRV is Rust 1.99, matching `rust-version` and the 1.99.0
toolchain pin. A compatible `Cargo.lock` is committed so that toolchain can
reproduce dependency resolution.

The one-patient example uses P0's proposed capacity/location schema v3. E0
only runs resource buckets whose open and staffed counts are known. Optional
route distances must use the P0 unit `m`. E0 does not simulate movement.
Staffing, spatial behavior, and the remaining operational constraints belong
to E2 and are not this crate's headless MVP. Later calibration, exports, and
release qualification remain separate gates.

Workspace package metadata declares SPDX `Apache-2.0` via `license.workspace = true`. The repository README section "Licenses, notices, and citation" records that a root `LICENSE` file and a CareOps Sim `CITATION.cff` are still maintainer decisions. Kairos keeps its own `libs/kairos/LICENSE` and `libs/kairos/CITATION.cff`.
