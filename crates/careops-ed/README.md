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

## Headless MVP Scenarios (E2.3 / Delivery Contract Stage 1)

The crate delivers runnable headless MVP scenarios with configurable bed and
staff capacity, named zones/locations, and fixed-seed repeatability:

```sh
# Run nominal scenario (2 acute bays)
cargo run --locked -p careops-ed-cli -- run crates/careops-ed/examples/nominal_mvp.json

# Run constrained scenario (1 acute bay)
cargo run --locked -p careops-ed-cli -- run crates/careops-ed/examples/constrained_mvp.json

# Test input validation rejection
cargo run --locked -p careops-ed-cli -- run crates/careops-ed/examples/malformed_mvp.json
```

Controlled capacity comparison:
- Nominal (`nominal_mvp.json`): 5 patient arrivals across ATS 1-5 acuity levels;
  2 acute bays provide sufficient capacity with a mean waiting time of 0.4 ticks
  and 100% completion before the horizon.
- Constrained (`constrained_mvp.json`): exact same arrival stream; reducing acute
  capacity to 1 bay induces queueing with a mean waiting time of 4.2 ticks.
- Invariants: patient count conservation (arrivals = completed + unfinished),
  strict non-negative capacity, and zero double allocation hold across both runs.

Boundary specifications (ST001–ST009):
- ST001/ST002: Finite destination-compatible bed offers; explicit offer lifetime
  and withdrawal policies (immediate expiry is not an assumed universal default).
- ST003/ST004: Boarder retained occupancy of treatment bays until physical transfer;
  recurrent care tasks supported without reserving full-time 1:1 staff by default.
- ST005/ST006: Ambulance offload governed by receiving bay and nurse availability.
- ST007: Complex multi-person procedures require dedicated composite teams.
- ST008/ST009: Prohibits causal claims of inpatient ward policies from external
  arrivals alone; no implicit default deterioration hazard.

Limitations:
This functional headless MVP precedes E3 (usable experiment API/Parquet exports),
C3–C6 (empirical calibration and validation), and E4 (hardened native v1). No clinical
operational endorsement or Cairns calibration validity is claimed.

Workspace package metadata declares SPDX `Apache-2.0` via `license.workspace = true`. The repository README section "Licenses, notices, and citation" records that a root `LICENSE` file and a CareOps Sim `CITATION.cff` are still maintainer decisions. Kairos keeps its own `libs/kairos/LICENSE` and `libs/kairos/CITATION.cff`.
