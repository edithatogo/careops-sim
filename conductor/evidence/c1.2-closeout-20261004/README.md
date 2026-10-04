# C1.2 local and native qualification acceptance — 4 October 2026

Accepted development source: Kairos `f18aba1f1930aec228e830c9e58c67e4081ac4bb`,
[PR #211](https://github.com/edithatogo/kairos/pull/211), stacked on C1.1 PR #209.
The parent pin preserves Q5.1 `bcb11cd61bc38b4813815f574a6a051f3b9e8faf`
as an ancestor. Q5.2 and its benchmark results are unchanged.

## Executed evidence

- Independently declared physical-v2 Rust schemas match actual IPC file, IPC
  stream and Parquet fixtures for all three tables, including nested types,
  nullability and field/global metadata. Schema mutations and truncation reject.
- Actual typed reader tests pass with IPC alone, Parquet alone and both on Rust
  1.88.0; both also pass on 1.99.0. Byte, row, batch, column and output limits
  are exercised, with Parquet decoder chunking distinguished from IPC batch caps.
- Eight locked, offline builds cover none/IPC/Parquet/both on 1.88.0 and 1.99.0.
  Default IO, smoke-format, core and DES dependency trees exclude Arrow/Parquet.
- PyArrow 25.0.1 independently reads nine Rust outputs at each qualified floor
  and current run. Full declarations do not derive schemas from input fixtures.
- Default legacy suites passed 23 tests on each of Rust 1.76.0 and 1.99.0.
  An additional frozen-byte guard passed on each: the 296-byte pre-feature smoke
  fixture retains SHA-256 `3928adcb2a84bfb0521c267cc834534693ee2acaa8ab7a080523dc20491163ac`,
  custom header and trailing newline, and is explicitly not IPC or Parquet.
- After the final Rust-2021 import-order correction, formatting, two typed tests
  and nine-output independent readback passed again. Hosted native-owner run
  [37202755655](https://github.com/edithatogo/kairos/actions/runs/37202755655)
  passed all eleven Linux/macOS feature/MSRV/current jobs at this exact PR head.
  API readback is retained in `hosted-native.json`.

Detailed executed commands, source/tool/input/output hashes, logs, actual
archives and failed attempts are retained at the pinned child path
`conductor/evidence/c1.2-typed-qualification-20261004/`, with its inventory and
qualification receipt. No planned check is counted as a pass.

## Remaining boundaries

C1.2 implementation and qualification are accepted for development. PR #211
remains unmerged: broader Kairos validation, documentation, package evidence
and stable quality checks fail. The current stable failure is a predecessor
`interop_v1.rs` needless borrow; validation lacks the native-owner workflow
inventory; documentation references a missing parent evidence file; package
metadata omits the new workspace members. These are not waived by native success.

C1.3 normalization/sorting/layout invariance and C1.4 phase review remain open.
This does not close all C1, approve a stable public schema, provide hostile-decoder
peak-memory isolation, qualify a real clinical feed or establish release readiness.
No whole-workspace Rust 1.76 support is claimed.
