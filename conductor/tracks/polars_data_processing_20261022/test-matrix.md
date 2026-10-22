# Test matrix

All runtime rows below are required future tests, not tests already implemented or executed. First-phase tasks must create named native test targets/reader scripts and record their exact commands before implementation acceptance. Never substitute a broad green workspace run for a missing oracle.

| Gate | Cases and required evidence |
|---|---|
| polars-native-contract | Exact pinned features/MSRV; core/Wasm exclude Polars; ID/tick/null schema; overflow and limb-borrow boundaries |
| independent-columnar-interop | PyArrow reads actual IPC/Parquet, Zstd metadata, empty/null/multi-batch values; Python Polars consumes native export |
| deterministic-data-parity | Exact event/seed/ID/counts; canonical ordering; analytical statistical oracle and documented finite/nonfinite comparisons |
| durable-trace-recovery | One simulation execution per replication; bounded capture; disk failure, kill/restart, no missing/duplicate rows, manifest-last |
| data-maintenance-performance | Deleted/retained inventory; large/skew datasets; wall time/RSS/thread/build/binary costs against baseline; existing regression gates retained |
| phase-closeout-check | Conductor review, fixes, clean source, exact receipts; unavailable hardware/hosted checks remain unavailable |

Planning validators: `python3 tools/context.py check`; `python3 tools/tasks.py build` then `python3 tools/tasks.py check`. Runtime: `cargo test --workspace --locked`, targeted ED/birthing tests, `python3 -m unittest discover -s tests -v`, independent-reader and existing performance gates. Execute inherited upstream phase validators only against the accepted initialized submodule; never count missing tools as a pass.
