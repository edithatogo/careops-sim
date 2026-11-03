# careops-ed

The Rust library and separate `careops-ed-cli` package run synthetic emergency
department fixtures using Kairos. The E0 supplied-work fixture and experimental
pathway runtime have different scopes; neither establishes clinical validity or
release qualification. Inputs are synthetic, and ticks have no conversion to
minutes or seconds unless an explicit input contract supplies one.

The exact workspace toolchain is Rust 1.99.1; Cargo's declared minimum is 1.99.
From the repository root, use a fresh, nonexistent output directory:

```sh
cargo run --locked -p careops-ed-cli -- validate crates/careops-ed/examples/nominal_mvp.json
cargo run --locked -p careops-ed-cli -- run crates/careops-ed/examples/nominal_mvp.json --output-dir nominal-run
cargo run --locked -p careops-ed-cli -- run crates/careops-ed/examples/constrained_mvp.json --output-dir constrained-run
cargo run --locked -p careops-ed-cli -- compare nominal-run/data constrained-run/data
cargo run --locked -p careops-ed-cli -- export nominal-run/data --format ipc --output-dir nominal-ipc
```

The default is real Parquet/Zstd. `ipc` writes Arrow IPC streams and `csv` writes
CSV parts; each dataset includes patients, replications and metrics plus a checked
manifest. JSON is reserved for input and bounded invocation/checkpoint/manifest/
comparison controls. The stdout completion report is not a patient table.
`run` and `batch` publish datasets under `OUTPUT/data`; `export` reads that dataset
and writes a converted dataset directly to its fresh destination, without rerunning
the simulation. There is no overwrite flag or schema-descriptor export mode.

Original-run capture, input/model hashes, typed tick limbs and recovery boundaries
are explained in the [walkthrough](../../docs/user-walkthrough.md). Source
behavior and evidence are distinct: current-source R14 and hosted qualification
remain tracked in [Polars Conductor](../../conductor/archive/polars_data_processing_20261022/index.md).
The [support matrix](../../docs/support-matrix.md) separates earlier platform
receipts from current-source checks; the [delivery contract](../../conductor/delivery-contract.md)
sets acceptance requirements, rather than certifying this checkout.

## Controlled capacity comparison

The nominal fixture has five arrivals across ATS 1–5 and two acute bays, with
mean wait 0.4 ticks and all five completed before its horizon. The constrained
fixture preserves those arrivals and reduces acute capacity to one bay, giving
mean wait 4.2 ticks. Patient conservation, non-negative capacity and no double
allocation remain test oracles. These numbers describe invented work items,
not observed ED performance, clinical defaults or minute-based estimates.

The one-patient E0 example uses the proposed capacity/location schema v3 and
resource buckets with known open/staffed counts. Optional distances use `m`;
E0 does not simulate movement. Its seed is recorded, but that deterministic
supplied-work fixture contains no random draws.

## Domain boundaries and checks

ST001/ST002 retain finite destination-compatible offers and explicit lifetime/
withdrawal policy. ST003/ST004 retain boarder occupancy until physical transfer
and recurrent care without an assumed full-time 1:1 reservation. ST005/ST006
gate ambulance offload on receiving space and nursing. ST007 requires actual
composite teams. ST008/ST009 reject ward-policy causal claims from external
arrivals alone and implicit deterioration defaults.

```sh
cargo test --locked -p careops-ed
cargo test --locked -p careops-ed-cli
```

`careops-ed capabilities` reports executable integration, not hardware discovery.
`run` uses `cpu_serial`; `batch` uses independent `cpu_replications`. Unsupported
GPU/Metal, within-run PDES, gRPC/MPI and distributed ED requests fail explicitly.
Live EHR/HL7 feeds, autonomous publishing and clinical/site/operational acceptance
are outside this engineering fixture's scope. Rendering is a separate consumer.

Project-owned code is Apache-2.0: see root [LICENSE](../../LICENSE) and
[NOTICE](../../NOTICE). Upstream notices and separately governed data/research
rights remain applicable; this code licence does not clear their redistribution.
