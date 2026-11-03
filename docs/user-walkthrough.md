# CareOps Sim native CLI walkthrough

Updated 4 November 2026 for the current native CLI source. This describes a
reproducible synthetic workflow; it does not certify final R14, hosted checks,
stable v1 release, clinical validity or site acceptance.

## Build and inspect

Use the pinned Rust 1.99.1 toolchain and committed lockfile. From the repository
root, build the CLI package, whose executable is named `careops-ed`:

```sh
rustc --version
cargo build --locked --release -p careops-ed-cli
./target/release/careops-ed --help
```

If `CARGO_TARGET_DIR` selects an external build target, use its `release/careops-ed`
path in place of `./target/release/careops-ed` throughout. Preserve existing owned
builds and environments. Python helper/reader commands require the repo-local
Python 3.15.1 environment: create `.venv` only if absent with
`uv venv --python 3.15.1 .venv`, verify `.venv/bin/python tools/python_runtime.py`,
and use that interpreter explicitly. Do not replace an in-flight environment.

Current CLI syntax:

```text
Usage: careops-ed capabilities [--require MODE]
  careops-ed validate INPUT.json
  careops-ed run INPUT.json --output-dir FRESH [--seed N] [--horizon T] [--format parquet|ipc|csv] [--stdout] [--backend cpu_serial]
  careops-ed batch INPUT.json --output-dir FRESH [--replications N] [--workers N] [--checkpoint N] [--seed N] [--horizon T] [--format parquet|ipc|csv] [--stdout] [--backend cpu_replications]
  careops-ed batch INPUT.json --output-dir EXISTING --resume EXISTING/capture/checkpoint.json [--workers N]
  careops-ed export DATASET --output-dir FRESH [--format parquet|ipc|csv]
  careops-ed compare LEFT_DATASET RIGHT_DATASET
Default format is real Parquet/Zstd. --stdout requires CSV and streams patients only. Fresh output roots are never overwritten. Resume is capture-only; incomplete export cannot be retried in place.
Software verification of simulation mechanics does NOT constitute clinical domain validation for any healthcare site.
```

## Validate and run two fixtures

Inspect the checked-in [nominal](../crates/careops-ed/examples/nominal_mvp.json)
and [constrained](../crates/careops-ed/examples/constrained_mvp.json) inputs.
Both contain five supplied patient work items; nominal has two acute bays and
constrained has one. These are invented inputs, not observed arrivals or defaults.

```sh
./target/release/careops-ed validate crates/careops-ed/examples/nominal_mvp.json
./target/release/careops-ed validate crates/careops-ed/examples/constrained_mvp.json
./target/release/careops-ed run crates/careops-ed/examples/nominal_mvp.json --output-dir nominal-run
./target/release/careops-ed run crates/careops-ed/examples/constrained_mvp.json --output-dir constrained-run
./target/release/careops-ed compare nominal-run/data constrained-run/data
```

Choose nonexistent output roots; the CLI creates private roots and refuses any
existing destination. Do not pre-create them. To edit a scenario without Rust
changes, copy nominal to a new JSON file, change its `scenario_id` and the
`res.acute` resource bucket's physical/open/staffed counts consistently, validate
it, then run it into another fresh root. Preserve the arrival stream when making
a controlled capacity comparison.

The qualified synthetic reference outcomes are mean waits 0.4 and 4.2 ticks,
respectively, with five completions each. `compare` reads checked native metric
tables and returns five bounded numeric comparisons with units and estimands.
It validates denominator names, sample units and weighting against the metric tables. It requires compatible time units and
model/runtime/data/Polars contracts. It does not reproduce the retired alpha JSON
report or infer empirical superiority. Null estimates and unqualified uncertainty
remain explicit. A tick is not a minute or second in these ED fixtures.

## Inspect artifacts and convert formats

A completed execution root contains original `input.json`, immutable
`invocation.json` and its checksum, captured/checkpoint artifacts under `capture`,
small `batch_run.json` control, completion `manifest.json`, and native dataset
`data/manifest.json` with bounded patient, replication and metric parts.

The invocation binds raw and parsed input hashes, model/build contract, seed,
replications, horizon, format and checkpoint settings. The dataset binds effective
and parsed inputs, model/runtime/data/Polars revisions, table schemas, file hashes,
row counts and part ordering. Read the relevant control's exact fields: the dataset
alone does not claim to store original raw input bytes. Preserve the whole execution
root when retaining invocation provenance.

Default parts are real Parquet with Zstd compression, not DDL. IPC parts are real
Arrow IPC streams. Tick columns preserve unsigned values as UInt64 high/low limbs;
reconstruct a u128 value as `(hi << 64) | lo`, retaining paired nulls. Patient tables
come from original-run capture; export does not rerun the simulator or invent movement.

```sh
./target/release/careops-ed export nominal-run/data --format parquet --output-dir nominal-parquet
./target/release/careops-ed export nominal-run/data --format ipc --output-dir nominal-ipc
./target/release/careops-ed export nominal-run/data --format csv --output-dir nominal-csv
```

Each destination is itself a dataset directory, with its manifest and conversion
receipt. Readers verify controls, schema/metadata, hashes and coverage; an existing
file or a checksum by itself does not prove a complete accepted run. CSV has no
embedded metadata channel and depends on its checked manifest. Keep all parts
with their manifest; do not turn JSON controls into analytical row tables.

To stream patient CSV to stdout while retaining files, select explicit CSV:

```sh
./target/release/careops-ed run crates/careops-ed/examples/nominal_mvp.json --output-dir nominal-stream --format csv --stdout > nominal-patients.csv
```

This stream contains patients only and is bounded to 8 MiB; ordinary commands
return small JSON controls instead. Publication and stdout success are distinct:
a sink failure can occur after files have completed. Inspect the retained artifacts
before deciding recovery; do not infer completion from partial stdout.

## Batch and recovery

```sh
./target/release/careops-ed batch crates/careops-ed/examples/nominal_mvp.json --output-dir nominal-batch --replications 4 --workers 2 --checkpoint 2
```

Resume requires the matching immutable input, invocation and model/build contract,
and a root with no completion manifest, `batch_run.json` or `data` directory. Use
that exact capture root and checkpoint, optionally changing the worker count:

```sh
./target/release/careops-ed batch crates/careops-ed/examples/nominal_mvp.json --output-dir nominal-batch --resume nominal-batch/capture/checkpoint.json --workers 1
```

Do not run this on an already completed batch: completion refuses resume.
A checkpoint may already contain completed replications; their original captures
are reused. Once `batch_run.json` or `data` exists, this resume command refuses an
in-place retry, even if the completion manifest is absent. Preserve the failed attempt and use a fresh export destination
when a valid dataset exists; otherwise inspect the recorded capture/checkpoint
state before recovery. No deletion, overwrite, automatic cleanup or export-time
simulation rerun is implied.

## Troubleshooting

| Exit | Meaning | Next action |
| --- | --- | --- |
| 0 | Operation completed | Retain controls and files for the required verification scope. |
| 2 | Usage error | Check help, command/backend pairing and flags; traversal paths are rejected. |
| 3 | File not found | Check the selected input path and access. |
| 4 | Malformed configuration/artifact | Inspect JSON/schema/identifiers, hashes, metadata and exact diagnostic. Do not repair controls to bypass integrity checks. |
| 5 | Incompatible comparison | Check units and model/runtime/data/Polars contracts, plus metric estimands. |
| 6 | Destination exists | Choose a fresh root; there is no overwrite flag. |
| 7 | Output failure | Check space/permissions or stdout sink; preserve incomplete artifacts. |
| 8 | Simulation failure/unavailable backend | Inspect the diagnostic and declared executable capabilities. |
| 9 | Interrupted/cancelled | Preserve artifacts and apply the capture-only recovery contract above. |

Scenario root schema 1 and proposed capacity/location schema 3 are checked
explicitly; unknown fields are rejected. Do not assume that every zero capacity
is illegal: use the selected model's actual validation rules and conservation
constraints rather than a generic capacity formula.

## Verification and limits

```sh
cargo test --locked -p careops-ed
cargo test --locked -p careops-ed-cli
.venv/bin/python tools/context.py check
.venv/bin/python -m unittest discover -s tests -v
.venv/bin/python -m unittest discover -s tests/ed -v
```

These commands provide their recorded scopes, not automatic release or clinical
acceptance. The [support matrix](support-matrix.md) distinguishes historical
platform evidence from current-source qualification. [Polars Conductor](../conductor/tracks/polars_data_processing_20261022/index.md)
retains open acceptance gates and exact receipts.

The CLI reports `cpu_serial` and independent `cpu_replications` integration via
`capabilities`; this is not hardware detection. Unsupported acceleration,
within-run PDES and network/distributed backends fail explicitly. No live EHR
integration, autonomous publication, physical/site calibration or clinical
endorsement is established. Input/schema changes need explicit compatibility
review; no guaranteed two-release deprecation window or automatic legacy-output
migration is qualified here. Project-owned code is [Apache-2.0](../LICENSE), with
[NOTICE](../NOTICE); third-party and separately governed content rights remain.
