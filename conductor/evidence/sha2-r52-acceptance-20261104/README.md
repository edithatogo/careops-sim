# SHA2 R52 acceptance

Accepted bounded engineering at the source below; reconciled and archived on5November2026. [Structured receipt](acceptance.json) binds actual gates and review.

## Qualified source and ownership

PR200 source `1d559a632f6e3ebe0c26f2c72317eb72e00269be` and actual GitHub tested merge `e10a3af24af0bd8916b5c9c3deb68109b6332520` have the same complete tree `a6f1e59a7d46dcf6243c1449524cec5705e4fca9`. The merge's parents are base `d12559f77c3572bdf23e448dddab74b02c82777c` and that source. The later archive documentation commit is distinct and is not represented as a hosted source-head pass. The current coordinator retains ownership; no other lane, upstream source or pin changed.

Exact workspace sha20.11.1 with default alloc+oid features replaces only the handwritten ED SHA256 helper. It adds no registry package/version; Cargo.lock is byte-identical. Core, birthing and build-script digest contracts remain. Source-bound R51 receipts preserve actual old artifact readability, changed-model execution refusal, nine unchanged files, 22 independent vectors, process-crash recovery and malformed-part controls. Historical receipts and original recovery software remain preserved.

## Hosted integration and platform evidence

Current CI41787210841 passed 779 default-feature workspace tests and791 native tests on each Linux/macOS host, with zero failed, ignored or skipped native tests. The deliberately failing runner control exits100. Independent B7 and ED IPC/Parquet readers pass. Strict Clippy, format, doctest, Cargo/schema lock, privacy, harness and shared-source security checks are separately recorded in the final receipt.

Linux passes all26 frozen input/canonical-summary pairs,14 invalid-input rejections, `/dev/full`, zero-work/oversized preflight, slow stdout and actual blocked-pipe controls. Original10s/256MiB/8MiB limits remain. Maximum case wall time is0.646s and measured child RSS147271680bytes. Raw output timestamps/hashes are not cross-run parity oracles.

The actual native Linux package run41787210829 passes the out-of-source unpacked consumer. Its ELF SHA256 is `fb6da6fada15335db31e482f78e6ab854e6434983cb0b4206ced012e7fc30d90`, size109583176bytes. Package hash, all9 SPDX entry hashes, resolved lock and explicit project/upstream notices verify. Notice scope is explicitly not a complete binary dependency notice closure. Main-only attestation is an expected conditional skip; this is not a signed stable release.

## Quality, security and controls

Actual fresh coverage is512/578lines (88.5813%), above the80% floor. Mutation accounting is117total:111caught,2existing reviewed equivalents,4known unviable,0timeouts. Both unchanged validators pass. Actual harness suites run825+127tests. Retain the configured80% line floor, complete117-mutant catalogue with only the existing exact reviewed equivalents/unviables and zero timeouts. The library-only catalogue does not cover the ED API wrapper; supplemental exact-current-helper controls separately pass both independent contract tests for the real helper and fail both tests for truncation and a same-length wrong nibble (exit101).

Fresh configured full locked workspace/all-feature cargo-deny advisories/bans/licences/sources and locked fuzz audit pass, along with actionlint, zizmor and Gitleaks. No new exemptions are added. A successful log does not expose the advisory database commit; no hash is invented. Automated PR guard concerns were checked against actual evaluation order, fresh TOML mappings and the pre-existing build edge; no production fix was required.

## Maintenance and cost decision

Removing handwritten crypto reduces production API source by68lines. The current function is7physical lines plus1documentation line: the earlier documented8-line wrapper count includes its doc comment. Tests/support add663lines, including223qualification lines outside the original six prefixes, giving net+595code lines. Manifest adds1line; the CI resource repair adds20YAML lines separately. This is not an overall line-count reduction.

The accepted R50.2 fixed-budget paired primitive measurements are reused only after exact normalized helper-token, library, feature, lock and runtime binding checks. All original latency/build/binary/RSS budgets pass; noisy1KiB measurements are not a portable speedup claim. Actual Linux release ELF growth is9792bytes versus the preserved pre-SHA2 package. Observed release builds560s/554s are descriptive, not paired cold-build or causal speedup evidence.

The net benefit is retiring hand-maintained cryptographic machinery in favour of an already present qualified maintained primitive, with explicit added assurance cost and preserved independent recovery/output oracles.

## Failure and review disposition

The original default-lane SIGBUS linker failure before tests is retained in [the first-failure receipt](default-first-failure.json). Resource pressure remains an unproved hypothesis. The reviewed repair only bounds Cargo link concurrency, incremental/debug payload and logs resources while preserving the original test command, features, assertions, timeout, exit status and zero retries. Fresh repaired default evidence succeeds; cancelled obsolete jobs are not passes.

Independent reviewer `/root/ffi_packet_prepare` accepts all R52 gates without an outstanding blocker. Archive all9task IDs with the unchanged405-task/110-milestone/25-track graph. Preserve other tracks, ordinary hooks and release/clinical/site/operational gates. No PR200/197/198/main merge is implied.
