# careops-ed

The Rust library and CLI run synthetic emergency-department fixtures. E0 is a
strict, deterministic skeleton for supplied patient work items. `src/pathway.rs`
implements an experimental pathway runtime for seeded arrivals,
triage/assessment, diagnostics, treatment, and disposition. Inputs are
synthetic. The crate does not assert clinical validity, and it is not the E2
headless MVP or a release-qualified ED simulation.

The workspace toolchain is Rust 1.99.1 (`rust-toolchain.toml` at the repository
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

The declared minimum is Rust 1.99 (`rust-version`); the exact development and
CI toolchain pin is Rust 1.99.1. A compatible `Cargo.lock` is committed so the
pinned toolchain can reproduce dependency resolution.

The one-patient example uses P0's proposed capacity/location schema v3. E0
only runs resource buckets whose open and staffed counts are known. Optional
route distances must use the P0 unit `m`. E0 does not simulate movement.

## Hardened Native v1 Interface & Walkthrough (E4.3 / Delivery Contract Stage 2)

The crate delivers a fully validated, hardened command-line interface (`careops-ed`) and public Rust API for reproducible discrete-event simulation experiments:

```sh
# Validate scenario configuration syntax
cargo run --locked -p careops-ed-cli -- validate crates/careops-ed/examples/nominal_mvp.json

# Execute simulation with atomic output persistence
cargo run --locked -p careops-ed-cli -- run crates/careops-ed/examples/nominal_mvp.json --output nominal_run.json --overwrite

# Compare two simulation scenarios with units and denominators
cargo run --locked -p careops-ed-cli -- compare nominal_run.json constrained_run.json

# Export results to CSV or Arrow/Parquet schema metadata
cargo run --locked -p careops-ed-cli -- export nominal_run.json --format csv --output nominal_patients.csv
```

Detailed guides and specifications:
- [Fresh User Walkthrough & Onboarding Guide](../../docs/user-walkthrough.md): Complete end-to-end user journey with editing, execution, comparison, and troubleshooting.
- [Support Matrix & Schema Migration Policy](../../docs/support-matrix.md): Historical platform results (`x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`), the current exact toolchain pin and explicit out-of-scope boundaries.
- [Delivery Contract Specification](../../conductor/delivery-contract.md): Stage 2 qualification requirements.

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
Native v1 delivers headless CPU simulation. It strictly excludes live EHR/HL7 feeds,
autonomous publishing, GPU/Metal execution, distributed transports, and GUI displays.
Software verification does not constitute domain validation or clinical endorsement
for Cairns Hospital or any physical healthcare facility.

Workspace package metadata declares SPDX `Apache-2.0` via `license.workspace = true`. The repository README section "Licenses, notices, and citation" records that a root `LICENSE` file and a CareOps Sim `CITATION.cff` are still maintainer decisions. Kairos keeps its own `libs/kairos/LICENSE` and `libs/kairos/CITATION.cff`.

