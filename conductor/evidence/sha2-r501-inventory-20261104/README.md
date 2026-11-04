# SHA-2 R50.1 prerequisite and consumer inventory

R50.1 inventory and historical prerequisite verification are accepted after independent review. R50.2/R50.3 and every runtime/replacement/acceptance task remain pending. No dependency, runtime source, pin or public digest changed.

## Source and prerequisite

Parent snapshot `ecbf0ffc9b11f152a36b513cb9ff19bc48fffe40`; runtime `07410266bf54b6f73c59db2f8cbfbc5697c53734`; data source `473ff2c0e640a7e457f500593f6ea1f06602ab35`.
Kairos Track93 Phase2 is complete in merged PR304 at `5692e6a9ee866131262d60103a84e2544dc51e4a`. That commit is an ancestor of the runtime pin. Five receipt/plan/test files are byte-identical to that accepted source, with hashes in the companion inventory JSON.
Historical test/closeout evidence is reused; no fresh library/runtime/host pass is inferred. Source review found a limit: Rust tests assert known empty/abc/56-byte vectors, but the multiblock test checks shape only; the Python “polyglot parity” test hashes Python smoke bytes and does not compare a Rust digest. Therefore the historical receipt is not direct Rust/Python full-byte parity proof. Do not propagate that stronger claim. Actual current boundary/high-bit/multiblock cross-implementation proof remains with the existing R50.2/R51 qualification gates. The old receipt names sha2 0.10.8/0.10.9; current parent declares/locks 0.11.1. Current exact version/features/licence/MSRV/security and value remain R50.2.

## Current scope

- Replace-candidate only: `crates/careops-ed/src/api_runner.rs::compute_sha256_hex`, 74 function lines / 3118 bytes. It implements standard FIPS SHA-256; it is not sha256_simple. Preserve lowercase64-hex and exact full-byte digests across all current artifact contracts.
- Its production consumers are ED CLI invocation/input/manifest/conversion receipts, original capture/recovery, native writer/reader integrity and native-table model metadata.
- Retain existing standard sha2 in core identity, birthing CLI output and ED/birthing build scripts; retain canonical byte layouts, identity hashing contracts and RNG stream assignments.
- Parent sha256_simple is absent. Obsolete pseudo-Parquet production serialization was removed under Polars, so it is not a new SHA2 implementation target. The read-only runtime submodule history is not parent deletion authority.
- Python hashlib is retained as an independent oracle; do not replace it with the candidate helper or count candidate-computed expected values as independent evidence.

## Source inventory

| File | Classification | Matched source references |
| --- | --- | --- |
| `crates/careops-birthing-cli/build.rs` | production/build | 2 |
| `crates/careops-birthing-cli/src/lib.rs` | production/build | 2 |
| `crates/careops-core/src/identity.rs` | production/build | 3 |
| `crates/careops-core/tests/identity_contract.rs` | test/support | 2 |
| `crates/careops-ed-cli/src/lib.rs` | production/build | 12 |
| `crates/careops-ed-cli/tests/c63_native_contract.rs` | test/support | 3 |
| `crates/careops-ed-cli/tests/native_cli.rs` | test/support | 7 |
| `crates/careops-ed/build.rs` | production/build | 2 |
| `crates/careops-ed/src/api_runner.rs` | production/build | 1 |
| `crates/careops-ed/src/capture.rs` | production/build | 5 |
| `crates/careops-ed/src/native_output.rs` | production/build | 2 |
| `crates/careops-ed/src/native_reader.rs` | production/build | 2 |
| `crates/careops-ed/src/native_tables.rs` | production/build | 1 |
| `crates/careops-ed/tests/c63_compatibility_replay_performance.rs` | test/support | 10 |
| `crates/careops-ed/tests/c64_handoff_contracts.rs` | test/support | 3 |
| `crates/careops-ed/tests/e31_public_api_tests.rs` | test/support | 4 |
| `crates/careops-ed/tests/e42_sensitivity_identifiability_warmup.rs` | test/support | 2 |
| `crates/careops-ed/tests/r111_capture_boundaries.rs` | test/support | 5 |
| `crates/careops-ed/tests/r112_production_capture.rs` | test/support | 2 |
| `crates/careops-ed/tests/r113_recovery.rs` | test/support | 1 |
| `crates/careops-ed/tests/r131_native_files.rs` | test/support | 4 |
| `crates/careops-ed/tests/r132_native_reader.rs` | test/support | 3 |

Counts are matched definitions/imports/uses, not claimed function-call counts. Exact lines/source hashes are in the inventory JSON.

## Boundaries and next task

R50.1 verifies an accepted historical prerequisite and inventories current consumers. It does not accept a new sha2 version or authorize R51 replacement. R50.2 must assess the already-present locked0.11.1 dependency, minimal/native/optional graphs and measured maintenance/build/runtime benefit. ED currently declares sha2 only as a build-dependency; R50.2 must specifically assess a normal ED runtime edge to the already-present workspace0.11.1, including features/minimal graph and cost. No package installation or version upgrade is needed to start that decision.

Before replacement, retain independent empty/abc/55/56/63/64/65/multiblock/binary-high-bit vectors and Python hashlib parity, exact old manifest/replay/input hashes, corruption/truncation/invalid-hex rejection, and current full-byte field coverage. An unchanged standard digest needs byte parity rather than an invented format migration; actual changed contracts require explicit versioning. Existing R51 failing-oracle and R52 recovery/performance gates remain.

Existing path owners retain scientific/public-output semantics. Bound the future writer to the ED helper, exact dependency edge, independent tests and consumer receipts; no libs/kairos writes or generic hash framework.

## Recorded completion boundary

R50.1 is checked for inventory/prerequisite verification with the historical source/date distinction; R50.2/R50.3 and all R51/R52 tasks remain pending. Preserve all other dependency rows, phases and receipts. The PR199 integration gate stays separate until fresh hosted success; no source or main merge is implied by this checkpoint.

Independent reviewer `/root/ffi_packet_prepare` approves R50.1 inventory/historical-prerequisite completion only. The active spec reconciles its absent sha256_simple target to the current standard handwritten helper; preserve the upstream historical pseudo-hash record. No R50phase/R50.2/R51/R52 acceptance is implied.

The proposed ED normal-dependency edit also touches a source currently bound by `tools/ed_source_compatibility.py::SHARED_SOURCES`. R50.2/R51 must account for a reviewed successor qualification that preserves the existing immutable receipts and rejects unrelated drift. This inventory does not alter that guard or silently regenerate a historical receipt.

[Source and consumer bindings](inventory.json) record the exact hashes and historical evidence limits. This checkpoint has no source/runtime adoption claim.

## Checkpoint verification

Repo-local Python3.15.1 was verified. Focused38 task/16 context/9 dependency-handoff tests pass; context/tasks/MVP checks pass at25 tracks,110 milestones,405 task records and82/258 MVP parents/leaves. Only R50.1’s accepted flag/objective changes; the other404 task records and all other dependency rows remain unchanged. This is inventory/integrity evidence, not a native/runtime or current-library pass.
