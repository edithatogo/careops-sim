# Generic ED — current executable capability qualification

3 November 2026. Parent base `544a1e2`, runtime pin
`07410266bf54b6f73c59db2f8cbfbc5697c53734`. E9 accepted for the bounded CPU/CLI capability repair at source `18b6e8f`;
independent committed review is ready and applicable hosted source gates pass. The clean
owned checkout moved to `codex/generic-ed-capability-qualification`; the core
branch/PR197 and every other worktree retain their revisions. This is a stacked
source branch on PR197, not a change to main or another lane's environment.

## Repair and evidence boundary

`careops-ed capabilities` exposes schema-version 1 JSON describing actual ED
executable modes. `capabilities --require MODE` succeeds only for `cpu_serial`
and `cpu_replications`. `run --backend cpu_serial` and
`batch --backend cpu_replications` preserve the existing dispatch paths. Known
unavailable Metal/PDES/gRPC/MPI/distributed selections exit 8 before input or
output IO; unknown, duplicate or malformed selections exit 2. Existing defaults,
simulation/RNG/event ordering, scenarios, output/checkpoint schemas, dependencies
and runtime pin are unchanged. This report performs no hardware probe.

Historical E0–E8 flags/receipts remain preserved as dated evidence. The old
R0.3/R2.3 conclusion that all device/socket code is absent is no longer current:
the pin contains real wgpu ABM submission and TCP process transports. Neither
is consumed by ED, and [exact upstream blockers](upstream-blockers.md) prevent
using their runtime labels or smoke tests as current ED G3 acceptance. The
track remains in progress; this phase closes current parent capability ambiguity
without certifying optional hardware/protocol/distributed execution.

## Executed local checks

- New contract controls: first meaningful red run 4/4 failed, then 5/5 pass
  after adding a valid-input/no-output-side-effect and real CPU batch control.
- `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p careops-ed --test execution_capabilities --test e31_public_api_tests --test e32_batch_and_checkpoint_tests --test e33_arrow_parquet_spatial_tests`: 42 passed (5 + 19 + 9 + 9), no skips/retries.
- `.venv/bin/python -m unittest discover -s tests -v`: 766 passed in47.739seconds, using actual repo-local Python3.15.1.
- `CARGO_PROFILE_DEV_DEBUG=0 cargo build --locked -p careops-ed-cli`: passed; actual native executable then exercised.
- [CPU walkthrough](cpu-walkthrough.json): 12 command receipts plus three preserved 4-replication batches at 1/2/4 workers; seed 2026, identical replication results, default versus explicit CPU run identical, supported validation succeeds and unsupported selections create no artifacts. Raw logs/output receipts remain private. Two inspector mistakes (field/enum spellings) were corrected against actual saved artifacts; no outputs, simulation oracle or batch runs were replaced.

Tests and local artifacts establish the stated CPU/CLI boundary. Full hosted
security, lock, coverage, mutation, native and aggregate checks remain mandatory
before integrating this new source. Prior exact544a1e2 gates are unchanged-source
context only, not a hosted pass for this patch. Clinical/Cairns/site/operational
and broader release acceptance remain separate.

## Accepted source gates and review fixes

[CI41499579545](https://github.com/edithatogo/careops-sim/actions/runs/41499579545)
qualified source 18b6e8f:15 applicable successes plus the expected main-only
attestation skip. Hosted full Python 766 passed; critical ED 500/567 lines,
116 canonical mutants: 111 caught, 2 exact reviewed equivalents, 3 unviable, 0 timeouts,
0 viable survivors. Cargo/schema lock qualification is unchanged from the qualified
PR197 base: this patch modifies no dependency, manifest, lock, schema or pin.

Independent committed review found one diagnostic-priority mismatch at 8a283c1;
bfce6be scans complete backend syntax before checking availability. GitHub review
found wrong-command CPU mode requests classified as runtime errors;18b6e8f now
returns usage 2. Tests first failed 8 versus 2 before both corrections; the5 focused
controls pass, and final actual CLI diagnostic readback passes. Both corresponding
GitHub review threads were resolved after independent exact-source confirmation.
See the [review report](review-report.md) and [phase closeout](phase-closeout.json).

E9.1–E9.4 are accepted; Generic ED stays in progress for the remaining optional
backend gates. [PR198](https://github.com/edithatogo/careops-sim/pull/198) is a
draft stacked on unmerged PR197. Final documentation-head hosted gates remain an
integration requirement. The existing ED JSON/schema export paths are not native
IPC/Parquet: actual ED columnar-output acceptance remains with Polars/PR191.
Birthing B7's separate real-IO qualification does not qualify those ED exports.
