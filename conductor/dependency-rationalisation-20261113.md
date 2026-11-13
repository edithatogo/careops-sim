# Rust dependency rationalisation — careops-sim (2026-11-13)

**Status:** scoped Conductor adoption contract; code substitutions not yet implemented or certified by this file.

## Repository-specific decisions

- Reconcile existing dependency-sequencing and `conductor/library-adoption-review.md`; preserve exact Kairos runtime pins and upstream/downstream ownership.
- **Verified:** Serde and Kairos workspace crates are used. CLI delegates raw arguments to `careops_ed::disposition::api_runner::execute_cli_args`.
- **Evaluate:** Clap at CLI boundary, first golden-testing all argument/flag/output and exit-code contracts; retain the direct library argument API.
- **Evaluate:** Tracing for bounded patient-free scenario diagnostics; exclude high-cardinality per-event hot-loop spans.
- **Extend:** Proptest for queue conservation, capacity, occupancy, time bounds, error handling and seeded replay, including 1/2/4 worker equivalence.
- **Benchmark:** Criterion and scenario-level CPU/memory benchmarks. Rayon and Polars only through existing upstream adoption tracks and deterministic Arrow/Parquet parity.

## Delivery and acceptance contract
- Reconcile existing Conductor tracks and ownership before editing code; do not duplicate completed tasks.
- Write a failing behavioural/regression test first for each runtime change; preserve APIs, schema versions, stdout/stderr, exit codes, error codes and MSRV or explicitly version breaking changes.
- Run the existing repo gates: targeted tests, full workspace tests, fmt, clippy and contract/interop checks. Record actual passing CI SHA; this document is planning, not validation.
- Inventory duplicate crate versions/features with `cargo tree -d` and `cargo tree -e features`; review unused dependencies, `cargo audit`, `cargo deny`, licence, MSRV, SBOM and release implications where applicable.
- Benchmark performance proposals with fixed fixtures and baseline/candidate receipts, including correctness, memory, CPU and determinism.
- Decision for each candidate must be recorded as adopt, retain, reject or defer with evidence, risks and rollback.
- Do not expose identifiable, secret, copyrighted or otherwise restricted raw data in tracing, snapshots, benchmarks or CI artifacts.
