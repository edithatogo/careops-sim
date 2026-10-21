# R1 native baseline — 21 October 2026

Source: careops-sim `1fef8d4778cb0e136c1096bdaf04eb2b1a635e8d`, Kairos `44261eb5cf8275f132f63a3396903d116d701dbe`. No runtime source, dependency lock or submodule pin changed. This is local ARM macOS development evidence, not hosted Linux, hardware, public API, clinical or release acceptance.

## Executed results

| Lane | Result | Scope |
| --- | --- | --- |
| Parent workspace, all features, Rust 1.99.1 | 296 passed, 0 failed | ED and consumed native dependencies, including synthetic holdout/replay comparisons |
| DES/ABM/calibration with Flow/test-support, Rust 1.99.1 | 969 passed, 6 ignored, 0 failed | Existing queue/calibration/checkpoint contracts; ignored cases remain unverified |
| Official Arrow IO / lifecycle / C43, Rust 1.99.1 | 31 / 31 / 22 passed | Physical Rust IPC/Parquet outputs |
| Independent PyArrow reader | Passed | CPython 3.14.8, PyArrow 25.0.1, NumPy 2.5.3 from existing hash lock; fixtures, C43 mutation controls, C44 diagnostics, interoperability and C11 physical readback |
| C2 separate process demo, Rust 1.99.1 | Passed on canonical paths | Save, two byte-identical restores, preexisting sentinel rejected for expected reason and bytes preserved |
| Original C5 process qualifier, Rust 1.99.0 | Passed on canonical paths | Workers 1/2/4 identical; cooperative and hard-kill recovery; original provenance assertion unchanged |
| ED CLI, seed 42 nominal fixture | Two successful runs | Actual JSON artifacts retained; byte-identical repeats |

The first C2/C5 attempt used `/tmp`, a macOS symlink. C5 explicitly rejected that component; C2 returned its invalid/incompatible error. The same binaries and original oracles passed using `/private/tmp`. Original failed receipts remain in `native/`; successful process receipts are in `canonical-process/`. Equal error logs in the first attempt do not establish replay success. Acceptance requires successful save/restores, all four canonical C2 controls and C5 result status pass. The runner records each subprocess result; its own exit zero is not an aggregate pass.

## Reproduction and durable outputs

`runners/` retains executed command orchestration; receipt JSON records argv, cwd, compiler, target directory, exit, elapsed time and log hash. Source/input SHA-256 maps are in `native/source-manifest.json` and `ed-input-hashes.json`. `artifact-manifest.json` hashes every member of `baseline-outputs.tar.gz`, which preserves physical files, checkpoint/restores, process recovery artifacts and generated test evidence. Compiled target directories are excluded; executable hashes are retained separately. Paths in historical logs identify original local execution; the archive is the portable retained copy.

The reader environment was created separately under `/tmp/careops-r1-arrow-reader-20261021`; no global environment or dependency version was changed. Local Git excludes follow existing C2 CI patterns plus generated `/.artifacts/c13-transport/`; they conceal no source edits. Child Git status was clean before the C5 process qualification.

## Capability and rollback gates

R0 found divergent planning-main missing consumed capabilities. Keep the working pin until R2/R3 prove a capability-preserving candidate. Replay/checkpoint formats and fixture seeds remain those of the immutable source and retained outputs; this run does not change their versions. Synthetic holdout comparisons do not establish empirical clinical validity. Constructed E6/E8 tests do not qualify Metal/CUDA or multiple hosts.

Before any candidate comparison, use a separate clean worktree, restore this exact parent source and Kairos pin, verify source/input/archive hashes, then repeat the recorded commands. Compare semantic outputs under their versioned contracts; reject unexplained drift. If candidate qualification fails, leave the parent gitlink at 44261eb. No pin was advanced here, so no source rollback is necessary. R1 phase acceptance requires independent review; mainline publication is separate.

### Frozen identities and protected recovery procedure

The C2 process artifact uses `kairos-c2-replay-journal-v1`, recipe `c2.synthetic.annotated-micro.route5.pause1.resume3.v1`, seed-map version 1, root seed 19, study `c2-replay-test`, case `case-a`, replication 0, schedule `paired`; it is a sealed replay journal, not a claim that every portable checkpoint backend was exercised. C5 emits `kairoecs.calibration.summary.v1`, `kairoecs.c5-provenance.v1` and `kairoecs.c5-calibration-run.v1`; study SHA-256 is `759121ff4bc4605bd567095895ba3820bbe0105a909701c989fab8a59668dc2d`. The ED artifact has manifest/summary schema 1, scenario `nominal-ed-mvp`, seed 42, horizon 20 ticks and input SHA-256 `71215b7ff4a2b47b38ace339cadac15a68c1c49c9738856ff742e9400c2ff9ae`.

For later recovery, first coordinate with active owners, inspect dirty state and preserve all uncommitted files and candidate artifacts. Create a new detached worktree at parent `1fef8d4778cb0e136c1096bdaf04eb2b1a635e8d`, initialize its recorded submodules, and verify `libs/kairos` resolves to `44261eb5cf8275f132f63a3396903d116d701dbe`. Verify retained source/input hashes and use the successful canonical-path receipts. Never reset an occupied or dirty checkout. If a future parent gitlink has been committed, revert that specific pin commit on an isolated review branch with the parent owner, then rerun qualification before integration. This is a documented recovery procedure; actual candidate rollback rehearsal remains R3.

### Durable baseline source

The annotated repository tag `careops-r1-baseline-20261021` retains exact parent source `1fef8d4778cb0e136c1096bdaf04eb2b1a635e8d` independently of PR branch cleanup or squash history. Fetch it explicitly with `git fetch origin tag careops-r1-baseline-20261021`, verify `git rev-parse careops-r1-baseline-20261021^{commit}` equals that SHA, then use the isolated recovery procedure above. Ordered PR integration uses merge commits to preserve the original evidence ancestry as well. This tag is a source retention anchor, not a package release or capability acceptance.
