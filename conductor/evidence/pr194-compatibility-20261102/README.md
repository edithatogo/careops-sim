# PR194 exact-pin compatibility repair — 2 November 2026

Base: `03207671e632a3ea90f06b785407de3abdadbb92`. Kairos remains
`07410266bf54b6f73c59db2f8cbfbc5697c53734`; no upstream source or pin changes.

The conformance adapter now uses the pin's purpose-specific
`StagedResourceRuntime`, `StagedRequestId`, `StagedResourceId` and
`StagedRequestState`, its checked constructor, direct submission and capacity
snapshot. The capacity-one contention, observed lease revision, queued grant,
release readback and CareOps acknowledgement assertions are retained. This
qualifies the bounded single-resource contract only; it does not restore the
removed general-purpose Flow API or qualify atomic multi-resource allocation.

The only ED lib source change in the integration packet adds `pub mod identity;`.
It shifts existing mutation locations by one line. The validator binds the new
source/catalog hashes and shifted identities; all 116 rows, the same two exact
equivalent misses, three unviables, command identity, baseline and zero-timeout
requirements remain enforced. The historical source/catalog rejection controls
remain active. The canonical hosted mutation experiment below satisfies this source-bound gate.

The fuzz lock adds the eight transitive packages required by the core lane's
sha2 dependency. Existing locked package versions remain unchanged. Parent
`cargo update --workspace` leaves its lock unchanged.

## Verification and acceptance

Executed in the isolated repair worktree with Rust/Cargo 1.99.1 and repo-local
Python 3.15.1. Logs are private under `<PR194_EVIDENCE_ROOT>`.

- Allocation conformance: 1 passed (the original target failed on missing imports).
- Mutation validator: 20 passed (originally 3 failures and 3 errors).
- Locked workspace tests: 554 passed across 77 targets.
- All-target/all-feature Clippy with warnings denied: passed.
- Parent and fuzz cargo-deny: advisories, bans, licences and sources passed.
- Hash-locked Python: all 728 cases executed; 724 passed and four trust-boundary
  cases could not find their hard-coded local runner path with the external
  target directory. All six cases in that affected file then passed with the
  expected local build layout. Hosted full Python suite also passed its step.
- Independent read-only reviewer `review_pr194` examined exact commit
  `c786fb3ef6b5a2075e74d0e5a1b823aa87cc8450`: no actionable findings.
- Ordinary pre-commit and pre-push hooks: zero findings; no historical
  disposition additions. The repair uses a strict additive checkout registration,
  preserving other entries and all hook implementation bytes.
- Exact source `c786fb3`: 16 successful hosted checks, zero failures, one
  conditional main-only attestation skip. Run `41423996966` passed the required
  aggregate, Linux/macOS native tests, Clippy, default features, formatting,
  doctests, full 728-case Python suite, policy, 88.1834% ED line coverage and
  the canonical source-bound mutation gate. Lock run `41423997029` passed.
- Canonical hosted mutation result: 116 rows, 111 caught, the same two exact
  equivalent misses, three unviables, zero timeouts, successful baseline. Source
  SHA-256 `36c306bbff204267a5d3c1c68205e29f411cedf7442e8f741d2ab469ea6adb67`;
  catalogue SHA-256 `cfaa768ded199e7a3e5af5b20d03c5f2500ce456396bc60bf1ed4166a6597a3e`.
  The hosted command used the exact resolved Rust 1.99.1 Cargo executable.
- This closeout changes documentation only. Its runtime/source/lock blobs are
  identical to reviewed and hosted-qualified `c786fb3`. Required checks on every
  subsequently published revision remain mandatory before merge; the PR remains
  unmerged and no phase or operational acceptance is added.

An initial local workspace run exhausted the system disk; build output moved to
a dedicated external directory. An interrupted mutation run shared a build
directory with another validation process; its successor's baseline detected
contaminated compiled output and failed. That result is rejected. The replacement
experiment uses cargo-mutants' isolated build directories. It completed with
111 caught, two existing equivalent misses and three unviables, but its command
record used a Homebrew Cargo path. The strict validator rejected that path; the
local experiment is corroborative, not accepted mutation-gate evidence. The
hosted canonical toolchain experiment is the accepted gate evidence; no identity
rule is waived. An initial Python
suite lacked the locked schema dependencies; it is not acceptance evidence.
Historical receipts are retained as historical evidence, not fresh qualification.
A7.1 still requires birthing B7.4; clinical/operational and hardware gates remain.

## In-progress tracks at the repaired PR source

The registry has three in-progress tracks. Across all 25 registered plans, 109
phases contain 401 tasks: 207 checked, zero marked `[~]`, 194 pending (51.6%
checked). These are scoped plan counts, including historical acceptance, not
clinical, hardware or operational completion. PR194 remains unmerged.

| Track | Remaining phases/tasks | Blocker or next gate |
| --- | --- | --- |
| Multi-domain core | A7.1 join both consumers; A7.2 observation/calibration; A7.3 activity/resource/cost/funding/outcome/equity ledgers and fiscal exchange; A7.4 phase review | A7.1 requires birthing B7.4. B0.1 evidence intake precedes the unaccepted birthing programme. |
| Generic ED | Historical E0–E8 task checkboxes remain checked; E6 Metal and E8 distributed capability reconciliation remains open | Actual device and multi-host receipts remain external qualification gates. Historical checkboxes do not satisfy them. |
| Whole-layer Polars | R11.1 invocation/buffer/error/resume tests; R11.2 original-run capture; R11.3 watermarks/manifest-last recovery; R11.4 review | R10 accepted; R11 and later phases remain unaccepted. Separate draft PR191 has a failed required aggregate because its context job was cancelled. No rerun or source work in that lane is performed here. |
| Whole-layer Polars | R12.1 analytical oracles; R12.2 Polars transformations; R12.3 demonstrated shared helpers; R12.4 review | Accepted capture/recovery and exact consumer/dependency evidence required before replacement. |
| Whole-layer Polars | R13.1 independent-reader tests; R13.2 Parquet/provenance and consumers; R13.3 fixtures/retirement; R13.4 review | Interchange and all known consumer parity remain required. |
| Whole-layer Polars | R14.1 native/replay/restart/numerical/performance gates; R14.2 code-removal audit; R14.3 commit/receipt/rollback closeout; R14.4 review | Full consumer validation and phase closeouts remain required. |

Hosted log SHA-256: `58a9312cb508fff761c9a7860253b8a36ebaa8d8c2b7b3756369782a9b55eb33`.
