# CareOps Sim user guide

The [native CLI walkthrough](user-walkthrough.md) is the maintained installation,
scenario-editing, execution, comparison, format conversion and recovery guide.
Its commands describe current source; final R14, hosted and release acceptance
require their own exact-source receipts.

Use Rust 1.99.1 with the committed lockfile. All Python tools/readers require the
verified repo-local Python 3.15.1 interpreter. Preserve active environments.
`careops-ed-cli` builds the `careops-ed` executable; an external `CARGO_TARGET_DIR`
changes its output path.

Runs use `--output-dir FRESH`. Their native datasets live under `FRESH/data`;
`compare` takes two dataset directories and `export` converts one dataset into a
fresh destination using real Parquet/Zstd, Arrow IPC or CSV parts. JSON contains
bounded input/invocation/checkpoint/manifest/comparison controls, not legacy patient
row bundles. There is no overwrite flag. Resume requires the exact matching checkpoint/input/model contract and a capture
root without completion manifest, batch control or data artifacts. Completed
checkpoint replications reuse original captures; incomplete export cannot be
retried in place.

For errors, follow the walkthrough's exit-code table and preserve failed artifacts.
Check the current `--help`, `capabilities`, input schema and artifact integrity
before retrying. Missing files, incompatible contracts, malformed schemas and
output collisions require different remedies. A successful process or local test
is not a release, hosted result or clinical validation.

The nominal/constrained synthetic fixtures retain the controlled 0.4/4.2 tick
mean-wait reference, not minute-based or clinical estimates. No live EHR, autonomous
publication, GPU/Metal, within-run PDES or distributed-network ED capability is
qualified by this guide. Rendering remains a separate consumer.

See the [support matrix](support-matrix.md) for dated platform evidence and current
qualification boundaries, the [delivery contract](../conductor/delivery-contract.md)
for acceptance requirements, and [Polars Conductor](../conductor/archive/polars_data_processing_20261022/index.md)
for outstanding gates. Unsupported schema revisions and unknown fields are rejected;
no stable-v1 or automated legacy migration guarantee is asserted.

Project-owned code uses [Apache-2.0](../LICENSE) with [NOTICE](../NOTICE).
Third-party notices and separately governed data/research rights remain applicable.
