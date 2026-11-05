# Tempfile R55 acceptance

Accepted bounded engineering on 5 November 2026. [Structured receipt](acceptance.json) binds the actual gates and independent review.

PR201 source `e86c9926efa5f1af13d189983be802717deb5ebf` and tested merge `efcc6900b7afd4b4132714502761041a111ba15f` share complete tree `43a0549466d0185d2b40cf9cc2a70dd998d48990`. Later Conductor documentation is a separate closeout commit, not a hosted qualification of that later commit. Existing coordinator ownership and all upstream pins remain unchanged.

The bounded replacement fixes observed concurrent-writer collision, unrelated PID-staging clobbering and partial-write scratch leaks. Ten manual Rust test-directory helpers now retain RAII owners. Exact tempfile3.27.0/default getrandom preserves overwrite-only API, OutputWriteFailed, Unix0666&umask and the nonsync core contract. ED durable publishers, four caller ignored-write policies, simulation RNG and profiles remain unchanged. R54 records actual old-artifact reading, foreign-model resume refusal and the exact successor guard; original Polars/SHA2/R53/R54 receipts remain immutable.

## Hosted gates

All six applicable workflows and Required checks succeed at the qualified source. Unchanged schema workflow path filters do not select two schema workflows; this is documented with exact unchanged trigger hashes, not a waiver.

Default-feature workspace tests pass787 across108 groups. Linux and macOS native jobs each pass797 with zero failed/ignored/skipped and deliberate red control exit100; independent IPC/Parquet readers pass. Hosted Python suites pass834+127. Strict format, Clippy, doctests, Cargo lock, privacy, harness and shared-source security checks pass.

Linux passes all26 frozen input/canonical-summary pairs,14 invalid-input controls, `/dev/full`, zero-work/oversized preflight and slow/blocked stdout controls. Unchanged10s/256MiB/8MiB budgets pass: maximum case wall1.673s and child RSS144793600bytes. Raw timestamps/output hashes are not cross-run parity oracles.

Fresh coverage is512/578lines (88.5813%), above80%. Actual catalogue45357afa has117 mutants:111caught,2existing reviewed equivalents,4known unviable,0timeouts. Review corrected a receipt extractor that had selected an earlier synthetic unittest record; actual terminal catalogue/artifact/source bindings now match. No rerun or policy relaxation was needed.

Fresh configured locked root/all-feature and fuzz cargo-deny advisories/bans/licences/sources, actionlint, zizmor and Gitleaks pass without new exemptions. The first candidate41d90e9 correctly failed the separate fuzz lock. [Repair receipt](fuzz-lock-repair.json) preserves that failure:35→44 fuzz packages, nine additions matching qualified root tuples/checksums, no existing registry changes or policy edits. The initial offline cache miss is not a pass; subsequent locked metadata/audit pass.

## Package and cost

The actual Linux package/unpacked consumer passes. Its ELF is109598312bytes, SHA256 `80d22573df46c504cf62284b8db21b3d2d2aecff3f9ef6ededd8ee4b142a14ab`, +15136bytes versus SHA2. Package checksum, all9SPDX entry hashes, root lock and explicit project/upstream notices verify. Notice scope is not a complete binary dependency closure. Main-only attestation is an expected conditional skip; this is not a signed stable release.

Observed release builds545s versus554s are descriptive runner observations, not paired cold-build or causal speedup evidence. Reviewed reuse of the same R53.2 helper operations/exact dependency closure retains +6.2%/+13.8% page-cache write overhead within frozen budgets. The formatted helper grows7lines; Rust/Python/Cargo source/support grows367lines, with76fuzz lock lines and documentation/evidence costs separate. No overall code reduction or portable speedup is claimed. Retiring manual staging/cleanup removes the three demonstrated defects while retaining the independent output/recovery oracles.

Independent reviewer `/root/tempfile_phase_review` gives final R55 GO, no remaining findings. Tempfile R53–R55 is complete. PR201 remains draft/frozen and unmerged; no release or pin promotion. Track70 retains end-to-end durable publication responsibilities; this supporting track does not certify physical crash durability, clinical or site acceptance.
