# Review Report: Polars migration planning

## Summary
The planning package reflects the user-approved clean break and library-replacement objective; runtime implementation remains pending.

## Verification Checks
- [x] Plan Compliance: both repositories, upstream-first gate, Rust-side processing, Parquet default, bounded floating-point comparisons.
- [x] Style Compliance: experimental maturity and explicit evidence boundaries.
- [x] New Tests: runtime test requirements specified; no runtime tests claimed implemented.
- [x] Test Coverage: planning registry/DAG/context checks only; runtime coverage not assessed.
- [x] Test Results: planning validators passed as recorded below.

## Refinements applied
Explicit tick-borrow arithmetic, duplicate-key conservation, FFI buffer ownership, effective-input provenance, trace-once capture and restart watermarks, memory/index/thread limits, manifest publication durability and code-deletion evidence. No DataFusion or duplicate backend introduced. External upstream gate remains manual because the parent task DAG cannot validate another repository's acceptance.

## Validation receipt
Date: 2026-10-22 (Australia/Brisbane). Working directory: this isolated repository worktree. Source base: 6bcbc0b947ab82b2d9e4b4e54ea51e662df8a53d. Deterministic seed/input hash: not applicable to Markdown/registry checks; runtime checks skipped because this increment changes planning only.
- `python3 tools/tasks.py build`
- `python3 tools/tasks.py check`
- `python3 tools/context.py check`
- `git diff --check`
Exit status: 0 for each listed planning command. Artifacts: this report and the generated parent task catalogue where applicable. Source commit is the containing planning commit; no hosted CI, merge, release or runtime acceptance claim. Pushed ref: none.

## Review disposition
No unresolved planning blocker. Exact dependency/feature qualification is a first-phase deliverable and a stop gate, not assumed successful. All implementation checkboxes remain unchecked.

Initial context validation failed because this new worktree had uninitialized submodules. Initialized exact recorded pins and reran successfully; no pin changed.

## Harness exception and corrective audit

The original precommit harness check returned exit 1: input hash drift for conductor/index.md. That file had been intentionally edited within the claimed conductor scope, but was mistakenly also registered as an immutable input. Commit commands nevertheless ran; the original precommit gate therefore did not pass. A postcommit diff-tree audit verified every changed path is inside conductor/. Released the original lease and acquired a fresh clean-tree claim with only unchanged workflow as immutable input, then recorded this exception and reran the scope check before the corrective commit. This correction does not retroactively turn the original failed gate into a pass.

## R10 phase review and acceptance — 30 October 2026

### Summary

R10 is accepted for the bounded remote ED/core/CLI baseline at `e65570b7a1046dafb15511ab5118b46b587f2f5d`, with Kairos `07410266bf54b6f73c59db2f8cbfbc5697c53734`. Runtime Polars replacement is not accepted. The earlier planning review above remains historical.

### Verification checks

- [x] Plan compliance: R3 entry prerequisite accepted; R10.1 Track 70 receipt verified, R10.2 inventory and baseline retained, R10.3 reference tests and compatible pin integrated.
- [x] Style compliance: existing architecture, RNG and scientific semantics preserved; no additional code-style directory exists in this checkout.
- [x] New tests: R10.3 statistical and row-reference tests retained; this phase closeout changes documentation/metadata only.
- [x] Test coverage: exact prior full local and hosted evidence reused after verifying unchanged source, lockfiles, configuration, schemas, fixtures and pin. Runtime migration oracles remain R11–R14.
- [x] Test results: accepted source checks pass; current phase/context/DAG/pin/support checks recorded below. No fresh performance, clinical or hardware claim.

### Evidence and review disposition

[Code PR #188](https://github.com/edithatogo/careops-sim/pull/188) merged as `0db23f1d5746314b33db315d1020ff47b3238f1f`, tree-identical to tested `ef1f0b0e38cdd5aae13eac562e543888db9b0ab2`. [Acceptance PR #189](https://github.com/edithatogo/careops-sim/pull/189) merged as the reviewed base `e65570b`, tree-identical to `1bca5fcd4a6bf5e9b6226beb6f47433e6034b3f5`. Provider readback on 30 October Brisbane reconfirmed #189 merged with all 14 reported checks successful, including required aggregate, both native hosts, privacy/security and session guards. [Its hosted run](https://github.com/edithatogo/careops-sim/actions/runs/40887777304) reran the full gates successfully.

The [R10.3 evidence packet](../../evidence/r103-remote-20261029/README.md), [local command hashes](../../evidence/r103-remote-20261029/local-verification.json), [26-case seed-7 parity](../../evidence/r103-remote-20261029/parity.json), [prerequisite inventory](../../evidence/r103-remote-20261029/prerequisites.md) and active `conductor/evidence/d2.4-kairos-contract.json` bind the exact inputs/source. Prior results: 358 parent Rust tests, 728 Python tests, full Clippy, 116 mutation outcomes (111 caught, two reviewed equivalent misses, three unviable, zero timeout), ED line coverage 500/567 (88.18%), and 588 Kairos tests on each native owner host. These are reused results, not newly executed tests in R10.4.

Independent reviewer `/root/acceptance_review` reviewed clean base `e65570b`, using the prior source reviews and exact receipts. No new runtime blocker. Accepted findings: reconcile stale proposed/unstarted metadata, unresolved external gate, planning-only handoff/index and old 44261eb retention instructions; record the bounded acceptance and explicit next-phase decision. Those corrections are applied here. Preserve the other lane's global A1/A2 dispatch; scoped Polars resume metadata is updated separately.

Rejected/cross-owner work: broad modern Kairos adoption and RNG changes were rejected in favor of the reviewed bridge; no golden updates or relaxed tolerances. The bridge is not for merging into modern Kairos main. Advanced A2–A6/identity/birthing source remains separately owned and unintegrated by this packet. Historical fixture retention proves exact bytes only, not current physical-schema behavior. The unavailable macOS disk-full probe, hardware, release and clinical acceptance remain unverified. Real trace capture, persisted columnar interoperability and restart durability remain required future work.

### Current verification receipt

Owner: chat `01a1876f-9b26-71e0-b55c-daeb9d9a1335`; managed worktree `careops-r103-remote/careops-sim`; exact review base `e65570b`; no native service writes. Parent commands run from the worktree root with the verified repo-local CPython 3.15.1. Upstream commands run in its initialized `libs/kairos` checkout at exact pin 0741026. Rust 1.99.1/seed 7/runtime hashes are inherited only from the linked immutable R10.3 receipts. This metadata-only closeout has no new simulation seed or input.

- `.venv/bin/python tools/python_runtime.py`: verified actual 3.15.1 interpreter.
- `.venv/bin/python tools/context.py check`, `tools/tasks.py build`, `tools/tasks.py check`, `tools/mvp.py check`, `tools/check_kairos_pin.py`: exit 0 after closeout edits.
- `.venv/bin/python tools/quality_support.py --root .`: exit 0; semver comparison remains explicitly unavailable without an owner-approved stable API baseline. An initial invocation lacking `--root` exited 2 and is not counted as a pass.
- `pwsh -NoProfile -File scripts/validate_conductor_phase_gates.ps1`: exit 0, no errors/warnings.
- `pwsh -NoProfile -File scripts/validate_conductor_dag.ps1`: exit 0, 106 tracks/47 agents, no errors/warnings.
- `pwsh -NoProfile -File scripts/validate_conductor_git_closeout.ps1 -LedgerPath ../../conductor/evidence/r103-remote-20261029/track70-closeout.yaml -RequireCleanWorkingTree`: exit 0 against the exact canonical Track 70 receipt retained from f9a19d0. Initial default-ledger runs failed for missing historical objects and then the deleted `origin/feat/tracks-70-93-94-foundation` ref in Track 70/93/94 entries. Exact objects were fetched; no source/pin/ledger or fabricated ref was changed. The default whole-ledger check remains failed for that historical cross-owner ref, not reported as passing. The corrected accepted prerequisite receipt uses 5692e6a on real origin/main and passes the unchanged validator.
- Parent harness scope check and `git diff --check`: required before the closeout commit; clean committed source required after it. The containing commit is the phase-record commit; its parent is the exact reviewed source above. Remote integration is reported separately from local acceptance.

### Next-phase decision

Permit **R11.1 preparation only** after R10.4 acceptance. Bind a fresh exact-base packet, context hashes, owned paths and claim before source edits. Name and execute failing oracles for one invocation per replication, bounded capture/backpressure, disk failure and checkpoint-resume duplicate/loss behavior before implementing capture. Retained dependency work gates only its demonstrated consumer/oracle; no blanket sha2/tempfile/proptest adoption gate. No R11 task is checked complete or dispatched by this review. Keep this track active through R14; do not archive it.


## Review Report: R11.3 local engineering — 3 November 2026

### Summary

Ready at244907e for local R11.3 engineering acceptance; full R11phase hosted acceptance remains open.

### Verification Checks

- [x] Plan compliance: durable control publication, preserved attempts, model/envelope binding and completed-manifest recovery are implemented.
- [x] Style compliance: ED all-target Clippy with warnings denied and formatting pass.
- [x] New tests: actual child-process kills at8windows with1/2/4worker recovery; sync faults, unknown versions and model binding.
- [x] Integration tests:646Rust workspace and774Python tests pass; actual CLI/PyArrow typed Parquet/Zstd/hash/model/cross-worker checks pass.
- [ ] Hosted phase gates: pending; no R11.4 or R12 acceptance.

### Findings and disposition

Reviewer ofcfe6151 found P2: nested BatchConfig schema_version was not checked. A new actual export regression failed on that source. Commit244907e validates before export/resume binding and before new output creation;24process-window probes also require refusal before mutation. Independent exact-commit re-review reports the finding fully addressed, ready, no new delta findings. No rejected findings or cross-owner changes.

### Remaining acceptance boundary

R11.1–R11.3 checkboxes are local engineering only. The normal privacy/history hook passed publication of the draft branch; actual hosted checks and clean committed phase records still gate R11.4. R12 processing, R13 consumer migration and R14 performance/removal remain open. Hardware power-loss, clinical and broader A7 acceptance are not established.

## Review Report: R11 phase — 3 November 2026

### Summary

R11 is ready for bounded engineering acceptance at2796f44, simulation
sourcef41d938, with complete actual hosted evidence; R12–R14 remain open.

### Verification Checks

- [x] Plan compliance: original-run bounded capture, durable acknowledgments,
  model/envelope validation, preserved attempts and manifest-last publication.
- [x] Style compliance: formatting and warnings-denied Clippy pass.
- [x] New tests: actual cancellation/progress corruption and UInt64 overflow
  controls join real process-kill, sync-failure and schema controls.
- [x] Test coverage:648Rust/774Python,36focused and24kill-window local controls;
  actual critical line coverage511/578 (88.408%) and complete117mutant catalog.
- [x] Test results: all17hosted checks pass, with expected main-only skip;
  111caught,2reviewed equivalents,4unviable,zero viable survivors/timeouts.
  Both native hosts and26benchmarks pass. See the
  [exact receipt](../../evidence/polars-integration-20261103/R11-hosted.json).

### Findings and disposition

Partial progress incorrectly reported zero after two durable acknowledgments;
corrupt journal state could publish a misleading marker. f41d938 fixes both,
with actual failing controls retained. A valid completion atUInt64::MAX exposed
legacy spatial tick-addition overflow, now explicitly rejected. A defensive
checked subtraction is present; no reachable prior subtraction panic was claimed.
Manufactured spatial movements remain the reviewed R12/R13 removal, rather than
silently changing the legacy export in R11.

The75minute outer timeout left113/117mutations incomplete and remains failed
evidence. A finite90minute ceiling permits completion without changing any
per-mutant limit, catalog, compiler proof, coverage floor or survivor rule.
The second stale75minute test assertion was corrected at2796f44;27affected tests
pass and exact independent review is ready. No rejected/cross-owner changes.

### Next-phase decision

Accept R11.4 and permit R12.1 under a fresh reviewed packet and advisory claim.
This closeout records source2796f44 evidence, not hosted results for its containing
documentation commit. Preserve R12–R14, physical, clinical, site, operational,
release and broader A7 gates. Keep the track active; do not archive it.


## Review Report: R12 local engineering — 3 November 2026

### Summary

R12 is ready for scoped local engineering acceptance at65731f0; independent
review reused exact slice reviews and verified actual coherent milestone logs.
Closeout69df957 records exact legacy paths/callers/owner/R13 dispositions.

### Verification Checks

- [x] Plan compliance: native ED statistics, validated fact/dimension joins,
  nullable synthetic calibration provenance and five captured B7 theatre views.
- [x] Style compliance: affected all-target warnings-denied Clippy and fmt pass.
- [x] New tests: independent frozen numerical/key controls plus actual native
  error, policy ambiguity, same-tick reassessment, callback and IO controls.
- [x] Test results:739native including doctests,0failed/ignored;774top-level
  Python plus separately selected8EDmirror tests pass at exact65731f0.
- [x] Scope: generic core/model progression retained; no speculative backend.

### Findings and disposition

Applied reviewed duplicate/null/endpoint, neutral interval naming, callback/
case timing, receipt binding, stale same-tick policy and native-budget fixes.
Actual red controls and final receipts remain in the R12 evidence. Legacy
comparison standard errors and JSON/CSV/DDL are explicitly R13 migration/removal,
not newly qualified statistics. No cross-owner implementation was attempted.

### Next-phase decision

Accept R12.2/R12.3/R12.4 as local engineering and proceed R13 output/consumer
work. Hosted2e2864f is older source; final-head hosted proof remains mandatory.
Generic populated domain C Data needs the Track70 owner followup; serialization
or event-log-only FFI receipts cannot substitute. R13/R14 removal, performance,
complete consumer, release, clinical/site/operational gates remain open.

## R13.1 local acceptance and upstream checkpoint — 3 November2026

Sole coordinator chat01a19b9f retains careops-pr194-checks/careops-sim ownership.
Accept bounded independent file/provenance/test authoring and CI wiring at
b1b91ca, following reviewer /root/review_pr194 and applied metadata/oracle/log
retention findings. Six native format controls, two exact independent readers,
26workflow tests and actual isolated Python3.15.1/PyArrow26/Polars2.1/runtime32
profile pass. No production consumer migration or whole R13 acceptance follows.
Parent043edab hosted17successes plus expectedskip is older R12 source.

Upstream f34750533c07e9e868481606e5fe5c89ec9b76be draft367 is mergeable, behind
main and passes actual hosted Linux/macOS/Windows native Arrow/FFI, committed
Git-source closure, policy/core/binding/package gates; configured publication
skips remain. Prior conflicting7c and failed0f receipts remain historical.
Git-source acceptance is distinct from unpublished registry Arrow/FFI packages.
Parent runtime074/data4134/Polars964 unchanged. Separate reviewed exact parent
adoption/interop proof still required. Continue R13.2/R13.3; pull measured
capture encoding cost prerequisite forward under bounded review, preserving
row/byte bounds, durability, fault/recovery and final R14 performance gates.

## Data-source prerequisite closeout — 3 November2026

Source43511e665706cf31c434758675ce291d8c12547b records the reviewed data-only
adoption of upstreamf34750533c07e9e868481606e5fe5c89ec9b76be. Runtime074,
Polars964 and all registry/version edges remain unchanged. Actual57EDnative/
39B7native,2file/1B7independent reader,13compatibility guard/16composition
tests,26frozen current CLI summary cases and strict policy/Clippy pass.
Historical join/source-compatibility/reference/driver bytes are unchanged.
A7 boundary finding was fixed with explicit separately bound data-only parity
evidence; dirty tested source was labelled as a base plus hashes. This committed
identity closes that source binding. Parent hosted exact newhead remains pending.

Scoped sizing source1cb4530 is locally reviewed with33capture/recovery tests,
including24process kills. Large-case elapsed cost improves, but four-worker
legacy full JSON bundle278,331,392bytes exceeds256MiB; final performance is not
accepted. R13.2/R13.3 must remove full-table JSON/DDL/manualCSV and migrate
production consumers, followed by whole R13 review and R14 full native/replay/
interop/streaming/performance/coverage/mutation/security/clean-package gates.
Same sole chat01a19b9f owns careops-pr194-checks/careops-sim; upstream standard
worktree retains its separate history, no active upstream reservation. No merge,
registry publication or clinical/site/operational acceptance follows.

## Native output and callable CLI checkpoints — 3 November2026

Accept scoped local library outputf6297fd and requiredreader wiringae20548,
then callable CLI execution/recovery coreacf217bf06b9a5325c12d6845fa50b1ad742b301.
The latest core has10native CLI plus42output/capture tests, including24process
kills,13compatibility guard tests and strict Clippy/fmt/integrity success.
Prior independent9native/3standardreader output proof and26workflow tests retain
their exact-source scope. Reviews by /root/review_pr194 and /root/polars_ci_review
found no remaining actionable issue; false readback wording and pre-cancel test
assumptions were corrected. Exact source/commands/failedattempts/hashes are R13
evidence and private receipts. The ownership expiry rejected writes, then
same-owner stopped-phase/process audit and exact preservation/re-reservation
recovered safely; no other-owner takeover, hook bypass or native retry occurred.

Actual hosted92d8daa now has17successes plus the expected main-only skip. It
qualifies the earlier data adoption/capture-sizing checkpoint, not later native
output/callableCLI source. Runtime074/dataf347/Polars964 remain fixed. Main is
still legacy until a coherent strict native benchmark/all-consumer cutover.
Continue typed native read/export/compare, then binary/consumer migration, B7/
calibration output migration and legacy removal; R13.2/R13.3 and whole R13/R14
remain open. Large legacy JSON four-worker RSS still fails256MiB and must not be
waived. Populated direct parent FFI and final full native/replay/performance/
streaming/clean-package/coverage/mutation/security/hosted gates remain mandatory.
Same sole chat01a19b9f owns careops-pr194-checks/careops-sim; no new runtime,
merge, publication or clinical/site/operational acceptance is implied.


## R13 continued local cutover checkpoint — 3 November 2026

Reviewed implementation is integrated through9ac72c2; R13.2/R13.3 and R14 are
still open. This dated section supersedes earlier next-action prose without
rewriting immutable source/reader receipts or earlier failed benchmark attempts.

- b0ddd0c: real CLI entrypoint/export/compare and separately versioned benchmark.
  Release26case parity and14rehashed-file negatives pass. Slow10k-patient stdout
  takes1.584seconds/74,645,504bytes peak RSS; unread live pipe terminates via TERM
  in1.558seconds with one reap and no KILL. Existing10second/256MiB/8MiB budgets
  are unchanged. Source bindings still match through9ac72c2. CLI13, guard20,
  benchmark4 and strict CLI Clippy passed. The build source was dirty atd7c188f;
  receipt binds exact hashes, not a false same-commit test claim.
- cb7362d/d3e0cf8/5a62834: reviewed CI and Python consumer cutovers, actual native
  typed readers, legacy example-runner removal and cold-CI release-build ordering.
  Existing analytical, conservation, warm-up and timing thresholds are retained.
  P5.2 six scenarios repeat with equal typed data/logical provenance; file byte
  equality is not required. Supporting22+2+27 tests pass.
- 9d126f3: E31(9), E33(8), E43(4), CLIconsumer(3) and nativeCLI(13) pass; real
  cancellation/resume, worker equality and calibration/domain oracles retained.
- 441e25a: capture/recovery migration reviewed independently by
  /root/legacy_removal_review at exactdiff0ada5d922d076a76aea31b9dcebf708e7ba4f4820107cbf4427f93198ea0a161.
  R11213/R1138/R1114 and minimal observer1 pass using the external target.
  Actual native publication/readback remains under the original invocation
  observer; full typed-frame equality, crash/restart and corruption checks remain.
- 9ac72c2: unused dashboard fake JSON/CSV/Arrow-description exports removed.
  Strict TypeScript5.7.2 typecheck/build, actual Node4 and Python8 checks pass.
  Separate review found and fixed progress-heartbeat interference with delta
  base ticks; accepted source has separate applied-state and progress clocks.
  Existing configuration, state/spatial/replay/live exports are retained.

Canonical Cargo Deny advisories/bans/licenses/sources passed at9d126f3 with
unchanged dependency inputs; no exemptions were added. Twenty source-compatibility
tests passed again at441e25a. Private command/result/hash receipts are retained
as polars-release-final-root-01.json, polars-capture-consumer-result.json and
polars-dashboard-source-result.json. Temporary isolated dashboard tools are
verification inputs, not a new adopted repository dependency lock.

Remaining work: reference/fixture/domain/C63/capability Rust consumers, production
legacy API/serializer removal, B7 metadata-bearing native boundary, populated
parent FFI and all final R14 source/package/performance/coverage/mutation/security/
hosted checks. The known whole-workspace rustfmt reader wrapping drift is open.
Runtime/data/Polars pins and genuine acceptance gates are unchanged. All completed
packet claims were released after their normal-hook commits; historical frozen
worktrees remain preserved. No archive or whole-layer completion follows from
this checkpoint. Coordinator chat01a19b9f retains the registered Polars worktree
and shared track surfaces; native scope is local synthetic CLI only.


## Native boundary and consumer continuation — 4 November 2026

Local source is integrated through63a30b8 with independent bounded review. This
checkpoint does not close R13.2/R13.3 or R14 and does not overwrite earlier
source-bound receipts. The eight coordination paths form one checkpoint join;
no source implementation or plan checkbox is changed by this record.

- 4357206: reference/analytical/capacity/calibration consumers use actual native
  tables. Native48/minimal35 tests pass. The retired censor_status DTO assertion
  was corrected to real completed status, exact completion limbs and zero
  unfinished count; first failures remain in private receipts.
- b215047: the native release workflow now exercises fresh native output roots
  from the unpacked compiled binary, qualified PyArrow, file/control hashes,
  actual5patient/1replication/5metric tables, logical repeat equality and five
  zero comparison deltas. Synthetic/nonclinical assertions, build/package,
  inventory and attestation gates remain. Exact extracted consumer script passed
  outside source on macOS at compiled4357206;32support tests pass. This is not
  Linux hosted or attestation proof. Binary SHA-256 is
  d6efba2e2f0d972667b55b8f70b86a467c2935c81274606d183c67b974e8a6d0.
- 443f506: C63 lifecycle/capability gates moved to the CLI crate without an ED
  dependency cycle; nativeCLI24/EDnative5/EDminimal5 and a final focused identity
  assertion pass. Real1000patient comparison takes31.924ms within the unchanged
  100ms gate. Export227.317ms is a measurement; the original test had no export
  threshold. Historical fixture/source oracles and every typed replay table remain.
- 95fe28c: reviewed Birthing v2 source4d6f97b is preserved byte-for-byte across
  four owned files. Exact-range preflight is clean; shared source-track Conductor
  changes were not copied. Batch8/theatre27 plus actual PyArrow readback pass.
- 63a30b8: B7 persists and checks11exact metadata fields in real Parquet/IPC.
  Batch10/theatre27, actual2x148row native/PyArrow readback and26independently
  rehashed metadata negatives pass. Shared checked rows retain original sequence,
  canonical digest/identity and8MiB payload bound; only the second JSON parse is
  removed. The single raw-domain decode is retained canonical-control validation.
  Existing five native views suffice; persisting five new derived formats would
  be extra scope. Uncertainty, policy, entity/null/zero, callbacks and recovery
  remain unchanged. Independent review accepted final1490e3bf2032dbe46ad0f6f64a76cc238964e37b9e2c25fac02625efbe41310b.

Normal hooks passed for every commit and claims were released immediately.
The supported Cargo dev-profile clean retired only this lane's ED/CLI debug
artifacts after all processes stopped; actual6.3GiB freed, release bytes verified
unchanged, packages and receipts preserved. No environment or third-party cache
was replaced. Commands/results/source hashes are in the private external
polars-reference-consumer-result.json, polars-native-release-consumer-qualification.json,
polars-c63-consumer-result.json and polars-b7-native-metadata-result.json.

Remaining: all-patient fixture view and C62/E42-heldout/C64 consumers, production
legacy serializers/reductions, active documentation, populated parent FFI, reader
formatting repair with exact evidence, maintained-code audit and all final R14
native/Python/performance/package/coverage/mutation/security/policy/hosted gates.
Optional GPU/distributed/clinical/site/stable-release capability is not invented
as an engineering closeout blocker. Runtime074/dataf347/Polars964 remain fixed.

## R13 local engineering phase closeout — 4 November 2026

Source f7563a8 completes the known consumer and legacy cutover. The retained
[acceptance record](../../evidence/polars-r13-acceptance-20261104/README.md) binds
all scoped commits, actual776-test coherent workspace pass (zero ignored/filtered),
796 top-level Python passes and all127 ED cases accounted with the corrected
exact E61 receipt test. Original failed runs remain private evidence. Independent
packet reviews accepted fixes without serial test harnesses, skipped oracles,
new tolerances, dependency exemptions or hook bypasses.

Release binary0c2e8487d3fc8a6fb7128dde24a78c7dfacb903b1d6ca0f1a331c1f58d9b9b88
passed all26 frozen cases plus unchanged stdout/RSS/time gates after the exact
reader formatting repair at7b9892b. Linux /dev/full remains locally unverified.
Earlier package and hosted receipts retain their old source scope.

The actual root FFI probe passed3 domain projections but its graph failed
unexempted policy for duplicate Kairos core/rng sources. The same stopped upstream
owner has a fresh bounded Track70 reservation for a shared trusted-import crate,
with architecture/security review, unchanged canonical importer and old engine
contracts. Arrow's unsafe prohibition is preserved. Its native/source/policy/
Git-consumer/hosted qualification is pending; parent pin/profile adoption has not
started. Root runtime074/dataf347/Polars964 remain fixed. R14 and archival remain
open; optional clinical/site/GPU/distributed/stable-release approval is separate.
