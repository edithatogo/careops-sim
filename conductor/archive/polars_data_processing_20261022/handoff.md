# Handoff

## Summary
R10 entry/reference phase accepted on 30 October 2026 for merged remote source e65570b and Kairos 0741026. Current R11.1/R11.2 bounded local engineering acceptance is recorded below; R11.3 durability work is next, with full R11phase/hosted gates still pending. See [phase review](review-report.md#r10-phase-review-and-acceptance--30-october-2026).

## Follow-up issues
Bind and qualify the exact source/dependency/feature matrix during each affected R11/R12 packet. Any unsupported interchange or memory requirement blocks affected replacement; no invented fallback engine.

## Integration notes
Kairos delivery precedes parent CareOps pin and processing migration. Both planning changes must be integrated into their repositories; worktree commits are not main-branch integration. No release/push authority inferred.

## Phase closeout evidence
R10 is accepted as scoped in the dated phase review. Remaining phases must record `$conductor-review`, accepted fixes, rejected findings, command/cwd/toolchain/seed/input hashes, exit statuses and artifacts, commit SHA, pushed ref or explicit blocker, and next-phase decision. Run `pwsh -NoProfile -File scripts/validate_conductor_git_closeout.ps1 -RequireCleanWorkingTree` in Kairos; use parent harness and clean-tree verification for CareOps. Planning validation is not runtime proof.

Planning-only validation and refinements: [review report](review-report.md). R11–R14 remain pending; the original planning review is historical.

## Sequencing revision — 23 October 2026

User-authorized planning change: inventory-first Polars; supporting dependencies gate only demonstrated affected tasks. No runtime paths, FMI work, pins, acceptance statuses or phase closeouts changed. See dependency-sequencing.md at the Conductor root for the decision contract.

## R10 terminal handoff — 30 October 2026

Owner chat `01a1876f-9b26-71e0-b55c-daeb9d9a1335`; managed worktree `careops-r103-remote/careops-sim`; owned paths: this track and affected Conductor index/registry/resume/catalog records; no native service scope. Code PR #188 and acceptance PR #189 are merged. R10.4 reconciles the phase records against those receipts. Next operation: prepare the bounded R11.1 packet against the then-current source, preserving the separate advanced A2–A6/birthing lane and global A1/A2 dispatch. No R11 source work is dispatched by this closeout.

## Current integration writer — 3 November 2026

Chat01a19b9f-69e7-7353-9b75-445d5653feb4 owns the bounded Polars integration in
`${PRIVATE_HOME}/.codex/worktrees/careops-pr194-checks/careops-sim`, branch
`codex/polars-migration-integration`, base19fe920, sourcePR1914b0a264.
Original owner worktree96e8cc8 remains clean/unchanged; its prior terminal source
claim was released and no native operation is in flight. Earlier assignments and
receipts remain history. No tenant/authentication scope and no Kairos writes.

The retained source is applied with ordinary three-way Git patch integration;
runtime074, dataGit4134/Polars964, core/B7/E9 source, CI and policy gates are
preserved. Workspace policy/pin/support checks and69targeted native tests pass.
[Integration receipt](../../evidence/polars-integration-20261103/integration.json).
Next: reviewer-confirmed R11.3 durability/model/envelope/process-window controls,
then full combined phase gates. No R11phase acceptance is claimed yet.

## R11.1/R11.2 local task acceptance — 3 November 2026

Source3ee7ac1:638Rust workspace tests pass;69focused capture/library/E9 controls
pass;774Python tests pass after retaining a frozen A7 source join and requiring a
separate exact26case observer/capture compatibility receipt. Actual frozen154ba1a
and current3ee7ac1 binaries match all26seed7 input/summary hashes and the retained
analytical reference. Eight negative compatibility controls reject stale/missing
source/runtime/driver/case evidence; historical A7 bytes remain unchanged and no
renewed whole-A7 acceptance is asserted. Fuzz isolates native features while the
production native-data default remains required. Workspace policy/pin/support pass.

Independent integration and guard reviews are ready, with no actionable findings.
The first full Python run retained two stale-guard failures and a build failure
during private preparation preservation; original build.rs was restored exactly.
The six trust-boundary tests pass after restoration; final774Python run is green.
No unrelated source/checkouts or environments were reset.

R11.1/R11.2 checkboxes record local task acceptance only. R11.3 needs real durable
control/model/attempt/crash-window fixes; R11.4 still requires full combined source
review and hosted gates before phase acceptance/integration. R12–R14 remain open.

## R11.3 reviewed local handoff — 3 November 2026

Exact source244907e on this chat's existing integration branch completes the
durable-control/model/envelope/attempt work. Independent review found one nested
BatchConfig version omission; the real red regression and24kill-window refusal
probes now pass after the committed fix, and exact re-review is ready.
646Rust/774Python tests,34focused controls, Clippy, policy/pin/support and an actual
CLI/PyArrow1/4worker round trip pass. [Detailed evidence](../../evidence/polars-integration-20261103/R11.md).
The disk-full linker attempt remains failed evidence; the subsequent full retry
passes after cleaning only terminal, owned ED build artifacts.

Normal hooks accepted the branch push with0new findings. No source/environment,
native operation, private binding or owner worktree was transferred. This writer
continues as coordinator for R11.4 hosted phase qualification; original assignments
remain history. R12–R14 stay open behind that phase decision. No merge/release or
clinical acceptance is inferred from the local handoff.

## R11.4 terminal evidence handoff — 3 November 2026

Existing chat/worktree ownership and runtime074/data4134/Polars964 remain
unchanged. Head2796f44, simulation sourcef41d938, passes all17hosted checks
plus expected main-only attestation skip in run41537272425. All117mutations
are accounted:111caught,2exact reviewed equivalents,4compiler-unviable,
zero viable survivors/timeouts. Critical line coverage88.408%;26benchmarks pass.
The [R11 hosted receipt](../../evidence/polars-integration-20261103/R11-hosted.json)
binds exact source/tree/artifacts and separates local from hosted evidence.

R11 is accepted for bounded engineering. Next: this same coordinator prepares
and reviews R12.1 statistical/join oracles, then claims only selected ED/birthing
processing paths. Preparation identifies existing calibration fixture projections,
checked integer reduction and cardinality validation. No R12 implementation or
acceptance is inferred from those probes. No native service operation or owner
transfer is in flight. Prior assignments, frozen R103/A7 references and failed
attempts remain history. PR199 remains draft/unmerged; R12–R14 and broader gates
remain open.

## R12.1 authoring handoff — 3 November 2026

Existing integration chat/worktree remains the sole writer. Exactb143bd1 has
reviewed statistical contract, frozen independent vectors and14compiling native
baseline controls (5pass,9actual method failures). Exact independent review is
ready. Test-authoring completion permits R12.2; it is not green runtime acceptance.
No production source change, native service scope, owner transfer or new pin.
The intentionally red authoring commit remains local until a coherent implemented
green milestone. Next: fresh reviewed R12.2 processing packet/claim, preserving
the frozen expected vectors and all subsequent phase gates.


## R12 engineering handoff — 3 November 2026

Sole parent integration chat/worktree/branch remains unchanged. Source65731f0
passed739native including doctests,774top-level Python and8explicitEDmirror;
independent slice/phase review is ready. Closeout69df957 records exact R13
legacy dispositions. Accept R12 local engineering and prepare R13.1 output/reader
controls, then bounded consumer migration/removal. No native service scope,
owner transfer or pin change. Generic arbitrary-domain C Data requires a
separate Track70 owner packet; it blocks affected FFI/final acceptance only.
Current-source hosted, R13/R14 performance/removal/consumer and broader gates
remain open. PR199 stays draft/unmerged; publish this coherent checkpoint next.


## R13 ED provenance and upstream handoff — 3 November 2026

Same sole parent chat/worktree remains. ED provenance source232ee91 has5new
native controls, reused21capture/recovery tests and final independent20-row/four
replication PyArrow26 proof; reviewed local slice only. Whole R13 remains open.

The upstream standard Git worktree kairos-polars-domain-ffi-20261103 produced
reviewed scoped commitsd835c525 (trusted declared-domain C Data) and0f981f72
(metadata-aware original publisher). Draft Kairos PR367 is published at0f981f72.
Upstream133native/29default C Data package proof and100Arrow metadata native
passes, independent real Parquet/IPC metadata/data/receipt proof, strict Clippy/
docs and reviews are local receipts, not parent or hosted acceptance. Locks and
compiler identities remain distinct. No upstream active source lease is retained.

Parent runtime074/data4134/Polars964 are unchanged. Actual upstream hosted/
package acceptance gates new API/pin promotion. Independent ED R13 output/
consumer work can proceed; affected B7 metadata and populated direct FFI proof
wait on that exact source. Parent FFI import tests belong behind a default-off
root feature with an explicit required-features target/CI invocation, preserving
production ED/core/birthing and minimal/Wasm/fuzz graphs. No engine handle or
new-runtime domain type crosses this boundary. Keep PR199 draft/unmerged and
retain all consumer/removal/performance/final-head/clinical release gates.

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


## Native reader qualification checkpoint — 3 November 2026

Source1b6ca2deaead85f1bc0da50c1d6aa00df54e2cba is committed with normal privacy
hooks passing zero findings. Original chat01a19b9f-69e7-7353-9b75-445d5653feb4
retains coordinator scope in `<managed-worktrees>/careops-pr194-checks/careops-sim`,
branch `codex/polars-migration-integration`; no native/authentication scope.
Birthing/Generic ED reconciliation commits/worktree remain separate and unchanged.

Final5profile/9output/5reader tests pass; explicit historical conversion1 plus
independent3tests pass, including6datasets/18tables. All26actual frozen summary
cases and16strict compatibility tests pass. Final Clippy/fmt, canonical dependency
policy, minimal graphs and Conductor integrity pass. Independent final review is
ready with no remaining findings. Original historical receipts are unchanged.
[Detailed scope and failure history](../../evidence/polars-integration-20261103/R13.md).

The existing external build target is qualified for continued work; internal
storage link failures are retained as failed evidence. No new hosted run, push,
whole R13/R14 acceptance or archive is claimed. Next: native CLI export/compare,
main/default and all-known-consumer cutover, then removal and existing R14 gates.
These are concrete unfinished tasks; the track remains in progress.


## Integrator checkpoint — 4 November 2026

Coordinator chat01a19b9f-69e7-7353-9b75-445d5653feb4 retains
careops-pr194-checks/careops-sim on codex/polars-migration-integration.
Reviewed source is63a30b8; packet claims are released, historical frozen worker
worktrees and source4d6f97b remain preserved. Native scope is local synthetic CLI
artifacts only. Next operation is the bounded native fixture projection and
C62/E42-heldout/C64 migration, then production legacy removal and final R14.
The [dated review](review-report.md#native-boundary-and-consumer-continuation--4-november-2026)
records exact acceptance scopes and remaining gates. No whole-layer completion,
archive, upstream source/pin mutation or merge is inferred.

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

## R14 source qualification checkpoint — 4 November 2026

Coordinator chat01a19b9f-69e7-7353-9b75-445d5653feb4 retains the existing
careops-pr194-checks/careops-sim lane and local synthetic-artifact scope. No prior
assignment or immutable receipt is replaced. Parent source86dcc7c follows actual
adoption a61f259 of reviewed data473ff2c and its engine-free shared importer;
runtime074 and Polars964 remain fixed. Upstream exact-source hosted run41667625899
passed all three platforms. Parent source was pushed with normal privacy/history
checks; final hosted acceptance remains a separate readback.

[The R14 checkpoint](../../evidence/polars-r14-qualification-20261104/README.md)
records populated FFI3, guard30, support68, actual historical conversion1+3 with
all60 original files unchanged, release26/14-negative/stdout budgets,12 measured
commands with8 conversions and2 worker-equivalence checks, and the current local
unpacked-package consumer. Measurement report source is a61f259; guard-only86dcc7c
retains the exact critical source/binary, rather than inventing a new producer run.
The maintained audit is net+859 production/runtime and+2564 tests/support,
aggregate+3423 within its six-prefix scope; no overall code reduction is claimed.

Python discovery ran806 cases:801 passed,5 selected stale in-tree binary checks
failed, then their corrected exact5 passed against the hash-verified external
release. No fresh full806 pass is claimed. The all-features workspace attempt
failed during build for lack of disk, before tests ran. The owned operation stopped;
supported narrow ED/CLI dev-cache cleanup recovered8.2GiB and preserved release
hash. Strict Clippy passed the locked workspace with all features and targets. Final
coherent native, coverage/mutation and exact-head hosted CI/release readbacks
remain open. All R14 tasks stay unchecked; archive only after actual acceptance.

Hosted Linux release run41678403333 passed at86dcc7c; main-only attestation was
an expected skip. Main CI context first failed before compilation/tests on a
preinstalled partial Rustup toolchain conflict (`bin/cargo-clippy`), while other
jobs continued. Preserve that log and use one bounded targeted fresh-runner retry
after the run ends; coverage/mutation/full-native/aggregate remain required.

C63 repair: first hosted Nextest787 ran786 pass/1 timed comparison failure at
114.640051ms against100ms. Preserve the failure. The default Cargo lane and
exact local isolated30.814334ms check pass. Reserve the exact timed case
with Nextest0.9.146 `num-test-threads`; all787, validation/oracles,100ms and
retries0 remain. Actual two-process control proves scheduler exclusion; full
fresh hosted qualification remains required. No production source changed.


## Final-source context deadline — 4 November 2026

Source40efa3b CI41683463668 passed both787-test native suites (zero ignored or
skipped), default workspace, doctests, Clippy, policy and the exact Linux package.
Context passed815 top-level and127 ED Python tests, the26-case benchmark and
coverage. It then hit the90-minute whole-job deadline at18:59:18UTC, after115 of
117 mutations:109 caught,2 existing reviewed equivalent misses,4 unviable,
zero per-mutant timeouts; `end_time` remained null. This incomplete experiment is
not accepted or combined with any later run. Preserve its full log and d33
artifact. The ED-file coverage gate passed at512/578 lines (88.5813%) against80%;
aggregate workspace coverage is not substituted for that file-specific result.

The bounded repair raises only the outer context budget to120minutes, allowing
for its approximately18-minute setup/coverage and72-minute partial mutation run.
The117 catalogue, exact source validator,180-second per-mutant limit,2workers,
coverage floor and all behavioral oracles remain unchanged. Final full mutation
and required aggregate success remain prerequisites; R14 is still unchecked.
Coordinator01a19b9f-69e7-7353-9b75-445d5653feb4 retains the existing integration
worktree and shared Conductor ownership; frozen staging/consumer lanes remain
untouched.

## R14 final engineering acceptance — 4 November 2026

R14 is accepted at parent258bb6b41f4682e140c06cb66a20155eb83d91a2 after
[CI41718206221](https://github.com/edithatogo/careops-sim/actions/runs/41718206221)
and [release41718206130](https://github.com/edithatogo/careops-sim/actions/runs/41718206130)
completed successfully. GitHub tested mergee2724a13d4fb9d3ec6ccbd218d2b7eaef64ce053;
its complete treedfd71e4b1a571d9af68a3a0facff058ec31cbe49 equals the source tree.
These are distinct commits. The later archive/documentation commit does not
become that tested commit; unchanged implementation evidence is reused explicitly.

Both native hosts passed787/787 with zero ignored, skipped or failed tests and
expected red-control exit100. Default-feature workspace, doctests, Clippy, policy,
all companion workflows and the required aggregate passed. Context passed815
top-level plus127 ED Python tests. ED-file coverage512/578=88.5813% exceeds the
unchanged80% floor. One complete terminal117-mutation experiment accounts for
111 caught,2 existing source-bound reviewed equivalents,4 original unviable and
zero timeouts; the unchanged exact-source/catalog/phase validator passed. Its
terminal timestamp is2026-11-03T22:37:04.854641599Z. The earlier incomplete115/117
experiment remains separate and unaccepted.

All26 frozen input/canonical-summary pairs match;14 rehashed negatives and actual
Linux disk-full/zero/oversize/slow-stdout/live-blocked-pipe checks pass within the
unchanged10s/256MiB/8MiB budgets. Maximum normal case0.851725315s and148025344B
child RSS. Actual unpacked Linux ELF4dda9945, package repeat/comparison consumers
and exact lockfile passed. Local12-command small/large ensemble,8 conversions,
2 full-table worker-equivalence checks, historical conversion/60 unchanged files,
populated3-domain FFI, graph/policy and cached external build receipts retain their
actual source/toolchain/cache scopes. No cross-workload speedup, universal zero-copy,
Python-reader RSS, cold-build or external concurrent-load claim is added.

The maintained-code audit retains gross legacy removal and a net increase:
+859 production/runtime,+2756 test/support,+3615 aggregate in its six-prefix
scope. The two source258 assertion substitutions do not change those totals.
Obsolete full-table JSON/schema-DDL/manual-CSV/dispatcher/descriptor production
paths are removed; retained closed-profile readers, control/provenance documents,
scientific numerical logic, SHA/FIPS and boundary oracles have explicit purposes.
No overall net-negative migration is claimed.

Independent reviewer `/root/ffi_packet_prepare` verified immutable source/merge
bindings and the actual terminal hosted/local receipts, accepting bounded R14
engineering with no blockers. Runtime074/data473/Polars964 and all existing
ownership remain. Rollback keeps parent85419da/dataf347 and the preserved
0c2e8487 release; use the archived shared-import adoption contract. The original
failed attempts and immutable JSON receipts remain intact. This closes engineering
R10–R14 and archives the track; no main merge, stable release, renewed A7,
clinical/site or unavailable-hardware acceptance is implied. No new track is dispatched.

Terminal owner: coordinator01a19b9f-69e7-7353-9b75-445d5653feb4, `careops-pr194-checks/careops-sim`. Source258 accepted; this archive commit is documentation reconciliation. Earlier assignments remain history.

## Archive fixture verification — 4 November 2026

The first focused38-task-test run exposed a fixture that copied only active
tracks, omitting the archived Polars prerequisite before its intended override
rejections. The isolated fixture now copies both areas and actual completion
evidence; all four malformed-override assertions remain. The corrected38 task,
16 context,9 dependency-handoff and30 immutable-source tests pass. Actual
context/DAG/MVP checks pass at25 tracks/110 milestones/405 tasks and82/258 MVP
parents/leaves. All20 Polars IDs, three downstream edges, unrelated tasks/handoff
rows and10 immutable receipt/packet hashes remain unchanged apart from the four
accepted R14 flags and intentional archive paths.

This archive-only fixture change adds12/removes6 test lines: final maintained
cost is+859 runtime/production,+2762 tests/support,+3621 aggregate. The qualified
source258/source40 audit remains+3615; its receipt is preserved. No runtime,
manifest, lockfile, native test, benchmark oracle or CI configuration changed in
the archive commit. Existing source258 hosted evidence is reused with this
explicit distinction; no fresh archive-head native pass is inferred.

## Post-archive PR gate — fixed native thread profile

Archive-head900c9cf CI41734017689 completed with Linux/context/default/doc/Clippy/
policy and package success, but macOS ran787:786 passed/1 C63 failure. Its actual
comparison was222.199667ms against the unchanged100ms gate; required aggregate
correctly failed. Preserve that failure. Slot reservation is supported by the
pinned runner and existing real-process control; completion ordering does not
prove overlap. No additional reservation/priority/retry/filter change is justified.

One bounded local fresh-process configuration comparison of the existing
source-identical C63 debug binary passed at29.476833ms with host-default Polars
and33.4855ms with one thread. This does not reproduce the hosted failure or prove
its cause. The native runner previously inherited a host-dependent Polars pool,
unlike the existing benchmark and independent-reader recipes that select1.
The runner now selects `POLARS_MAX_THREADS=1` for the complete native suite and
records the child binding. A real-child regression proves inherited64 becomes1
without mutating the parent environment. This is an explicit resource profile,
not a claimed production optimization or established root-cause fix.

All787 tests, replication worker controls, populated FFI selection, pinned tools,
red control, full-slot reservation,100ms oracle and zero retries remain. Fresh
complete hosted acceptance is required before PR199 can proceed to merge.
R14's source258 engineering receipt and archived history stay source-bound;
archive-head900 is not qualified by that earlier pass. No further source/pin
promotion or new SHA-2 implementation is implied.

Reviewer `/root/ffi_packet_prepare` approved this five-path recipe repair after
18 focused tests and scope review. Its +24 support lines (+9 runner/+15 test)
bring the same-scope cumulative maintenance delta to+3645; source258+3615 and
archive900+3621 remain dated, unchanged audits. Fresh hosted acceptance is pending.
