# CareOps Sim support matrix and compatibility boundaries

Updated 4 November 2026. This document separates recorded platform evidence from
qualification of the current native CLI source. A delivery contract or older green
receipt does not certify current-source R14, hosted checks, packaging or release.

## Toolchain and platform evidence

The exact Rust development/CI pin is 1.99.1 (`rust-toolchain.toml`); Cargo declares
`rust-version = "1.99"`. Use the committed lockfile and pinned Kairos/data/Polars
contracts. Python development, readers, tests and helper scripts require the
verified repo-local Python 3.15.1 interpreter. Create `.venv` only if absent and
verify it with `.venv/bin/python tools/python_runtime.py`; preserve in-flight
consumers and existing environments.

| Target | Earlier recorded scope | Current-source boundary |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | Historical Rust 1.99.0 CI/container checks | Requires current-head Linux CI and package/runtime evidence; old checks do not qualify this revision. |
| `aarch64-apple-darwin` | Historical Rust 1.99.0 workstation/harness checks | Later bounded Rust 1.99.1 native checks retain their exact-source scope; final R14/current-head proof remains separate. |

See [Polars Conductor](../conductor/tracks/polars_data_processing_20261022/index.md)
for current task status and receipts. Local Mach-O execution does not establish
Linux executable support, even if an archive uses a Linux-oriented filename.
No additional target, container, OS-version or hardware tier is qualified here.

## Current interfaces

| Interface | Contract |
| --- | --- |
| Scenario input | Root schema 1; proposed capacity/location schema 3. Unsupported versions and unknown fields are rejected. |
| Native execution | Separate `careops-ed-cli` package builds `careops-ed`; `run`/`batch` require a fresh output root. |
| Native dataset | Versioned manifest with checked patients, replications and metrics; real Parquet/Zstd, Arrow IPC or CSV parts. Tick values retain UInt64 high/low limbs and paired nulls. |
| Controls | Bounded JSON input/invocation/checkpoint/manifest/comparison records. Retired RunDocument/descriptor exports are not compatibility aliases. |
| Comparison | Checked native datasets with compatible units, model/runtime/data/Polars contracts and qualified metric estimands. |
| Recovery | Capture-only root without batch control/data/completion artifacts; exact immutable invocation/input/model and matching checkpoint. No overwrite or in-place incomplete-export retry. |

Use the [walkthrough](user-walkthrough.md) for executable commands and error
remediation. Conversion validates existing files and writes a fresh dataset; it
does not rerun simulation, infer movement or convert historical alpha JSON row
bundles automatically. Retain source artifacts and exact hashes when migrating.
Future schema changes need explicit compatibility review and migration evidence;
no stable-v1 promise, two-minor-release window or automated migration tool is
qualified by this document.

## Execution and clinical boundaries

`careops-ed capabilities` describes integrated executable modes rather than
hardware discovery. `run` uses `cpu_serial`; `batch` uses independent local CPU
replications. Explicit unsupported GPU/Metal, within-run PDES, gRPC/MPI or
distributed-network modes fail without silent fallback.

Live EHR/HL7 ingestion, autonomous publication, site-specific calibration,
physical/clinical acceptance and full operational deployment are outside this
synthetic engineering acceptance. Rendering is a separate consumer, not a new
core qualification. Software conservation/determinism checks do not establish
healthcare effectiveness or causal effects of ward policies.

Project-owned code is [Apache-2.0](../LICENSE) with [NOTICE](../NOTICE).
Third-party notices and separate data/research redistribution constraints remain.
