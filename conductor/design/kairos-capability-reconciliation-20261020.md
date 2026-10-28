# Kairos capability reconciliation and abstraction plan — 20 October 2026

**Repository scope:** This document concerns **kairos** and **careops-sim** (`${PRIVATE_HOME}/Documents/careops-sim`). The separate **careops** repository is outside scope. Earlier uses of “CareOps” in this audit meant the careops-sim simulation repository.

## Current decision

This maintainer-requested addendum updates the enhancement/readiness plan without editing the active roadmap owner's registry, task DAG, new domain tracks or draft upgrade packet. It is not a pin change or a claim that their work has been integrated. Baseline: careops-sim `8ca0e5ecec2a41a86ebce32361adbde08dd191d3`; Kairos pin `44261eb5cf8275f132f63a3396903d116d701dbe`; compared with Kairos main `f0d30e28c3299dcdb6ca138d95319ecaaf29a54a`. The paired Kairos change adds `conductor/capability-reconciliation-20261020.md` with detailed source evidence and an owner plan; that file may remain on its review branch until integration.

**Do not advance the pin solely because main is newer or its registry says Done.** The pin is not an ancestor of that main revision; their merge base is `384e8546d69f9cbf2746fcb2ab646263256e6dec`. The pin contains real `kairo-ecs-arrow-io` using Arrow/Parquet 60.0.0, calibration and extensive Flow/ABM transit/checkpoint contracts absent from the audited main tree. A direct replacement can lose qualified capabilities. Historical Q/C/D acceptance remains bounded by its recorded revision and receipts.

## Integration leaves for the current roadmap owner

These leaves supplement the proposed repository-reconciliation/Kairos-upgrade and careops-sim-core-domain-abstractions tracks. The owner should map them into its existing task IDs/DAG; this addendum does not create duplicate tracks or dispatch runtime work.

- [ ] **C-R0 — inventory before upgrade:** compare crate/module symbols, features, schemas, fixtures, Cargo dependencies, replay/checkpoint contracts and retained Q/C/ED receipts between both revisions. Record missing equivalents explicitly; distinguish alternative implementations from history deletion.
- [ ] **C-R1 — preserving revision:** coordinate Kairos owners 03/04/21/22/25 to integrate missing qualified capability or prove a replacement; retain current pin until a candidate passes physical Arrow/Parquet reader interoperability, Q/C behavioral suites, ED consumer API parity, checkpoint/recovery and rollback. No date/hash-only upgrade acceptance.
- [ ] **C-R2 — correct capability assumptions:** the audited main revision's Metal backend executes CPU fallback, Python optional ctypes loads a version function but not a native scheduler, and AllOf/AnyOf declarations do not implement atomic waits. Draft plans must consume evidence-qualified capability gates, not those names or historical Done labels.
- [ ] **C-R3 — audit E6/E8 receipts:** `e62_wgpu_wgsl_device_path.rs` contains literal WGSL and constructed device metadata; `e83_multinode_cluster_execution.rs` uses in-process threads/mpsc and synthetic efficiency. Keep these as contract tests; require actual device submission and physically distinct hosts with measured times before E6/E8 acceptance.
- [ ] **C-R4 — generic experiment services first:** adapt `batch_runner` and `api_runner` orchestration to existing Kairos experiment interfaces: seed partitioning, worker invariance, progress/cancel, manifests, resume and deterministic recovery. Maintain careops-sim facade and compare before/after traces across 1/2/4 workers and interrupted runs.
- [ ] **C-R5 — allocation/work mechanisms next:** review operations' shifts, held work, handover, preemption, eligibility and dispatch. Extract generic leases/state transitions/policy hooks only after acquire-all, cancellation, rollback and stale-completion semantics pass. `claim_atomic_multi_resource` is unsupported; do not build a plan assuming atomic allocation is already available.
- [ ] **C-R6 — two-consumer boundary proof:** exercise each proposed generic mechanism with ED plus a distinct consumer (initially nonclinical logistics fixture; qualify a separate domain adapter before broad adoption). Retain healthcare policies/types in careops-sim; preserve old adapters until parity and recovery pass. Do not move merely similar code before shared semantics are demonstrated.
- [ ] **C-R7 — qualification receipts:** bind commands, cwd, source/submodule hashes, toolchain, deterministic input/seed, host/device identity, exit codes and raw artifact checksums. Repository/contract tests, native execution, operational acceptance and clinical validation remain separate.

## Ownership boundary

| Belongs in Kairos if qualified | Remains in careops-sim core/domain/site layers |
| --- | --- |
| Deterministic execution, stream/seed partitioning, reusable experiment lifecycle | Patient and clinical pathway semantics; clinical validation policy |
| Atomic allocations, generic work continuation, interruption and dispatch hooks | Clinical triage vocabulary, priority mappings, staffing/clinical eligibility rules |
| Calibration mechanics, metrics, trace/checkpoint/provenance contracts | Empirical sources, clinical interpretation, site parameter mappings |
| Generic entity relations and policy-neutral resource identities | Maternal-neonatal linkage rules, healthcare capabilities, specialty adapters |

Already-upstream Flow/calibration on the current pin should be reconciled rather than extracted twice. Do not promote the count-conditioned synthetic arrival generator to a Poisson model without a separate statistical implementation and oracle. Avoid a new common framework where an existing primitive or adapter suffices.

## Free execution plan: three distinct hosts

1. **Mac (verified M1 Max, 32 GiB):** run native contracts and actual Metal via a reviewed wgpu/Metal implementation. Gate on real kernel dispatch/readback, device identity, CPU parity and measured timings; CPU fallback tests cannot qualify E6.
2. **Pleiades NAS (verified x86_64 Linux i3-N300, approximately 32 GiB, Docker, one NUMA node):** Linux/x86 consumer tests, real affinity, container OpenModelica/FMU and single-node Slurm integration. Use as a CPU transport peer after runtime compatibility checks. It does not prove multi-NUMA locality, NVIDIA execution or a parallel filesystem.
3. **Separate Pleiades CUDA PC (user-reported):** first verify OS/architecture, GPU, driver/toolkit, memory and remote execution access; this audit's SSH probe timed out. Then actual CUDA buffers/kernel/CPU parity, cleanup and timing. The existing SSH alias `pleiades` routes to the NAS; qualify separate host identities before any dispatch.
4. **Two-host E8:** real transport plus failure injection/restart, deterministic trace parity and measured speedup/efficiency. A process launcher, a single-node Slurm run or thread pool alone is insufficient. Mixed OS/architecture launch needs explicit wire/runtime qualification.

Use free [wgpu](https://wgpu.rs/), [Colima](https://github.com/abiosoft/colima), documented [OpenModelica macOS container/source routes](https://openmodelica.org/download/download-mac/), [Open MPI](https://docs.open-mpi.org/en/main/launching-apps/ssh.html), and [Slurm](https://slurm.schedmd.com/quickstart_admin.html) as applicable. Do not assume an OpenModelica macOS cask or Linux container GPU passthrough exists. No software was installed remotely and no host settings were changed for this plan.

## Review and coordination

The main checkout's uncommitted roadmap/domain work was left intact. Existing advisory claims, including expired Kairos solver/ensemble reservations, were not recovered or taken over. This change owns only this addendum and links in `kairos-enhancements.md` / `module-readiness.md`. The roadmap owner retains its task registry and draft upgrade/domain packets; incorporation into those owned files remains an explicit integration handoff. No messages were sent to other chats during the 20 October audit.

Documentation-only review: no pin/API/runtime/clinical-policy changes. Acceptance of this addendum does not close the leaves above or establish GPU/cluster readiness. Run `python3 tools/tasks.py check` and `python3 tools/context.py check` on the isolated checkout with the exact pinned submodules populated; report hosted validation separately.

## Planning validation receipt

Date: 2026-10-20. Working directory: `${PRIVATE_VOLUME}/codex-worktrees/careops-capability-plans-20261020`. Base commit: `8ca0e5ecec2a41a86ebce32361adbde08dd191d3`. Tools: Python 3.15.0. Deterministic seed: not applicable (documentation validators). Table hashes cover concatenated stdout and stderr of the actual commands; this table is the retained summary artifact, not a substitute for hosted logs. Source inputs are the planning diff against this base; integration must bind the final commit from the PR.

| Command | Exit | Output SHA-256 |
| --- | --- | --- |
| `git diff --check` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `python3 tools/tasks.py check` | 0 | `7f502d933e08ab53d2495da2c611315cca2f96b9564f51ec1e280004a26209ad` |
| `python3 tools/context.py check` | 0 | `faa4774b819524edd59271a87ae20bf55b4a174deed581102e95783acb417fc0` |

Full runtime CI was not rerun for these documentation-only changes. Hosted checks and owner lifecycle reconciliation remain separate.

## Review update — 21 October 2026

The source and hardware findings above describe the explicit 20 October revisions/readbacks, not a fresh audit of remote main. GitHub CLI HTTPS requests, Git HTTPS/SSH fetches and the GitHub connector failed during this review; current remote heads, review threads, merge conflicts and hosted check results could not be verified. Do not mark either PR merge-ready from the local checks alone. No runtime capability or track closure is advanced by these review fixes.

The maintainer explicitly authorized coordination, and the active roadmap owner received C-R0–C-R7 for incorporation into its existing plans/task DAG. The previously inspected owner branch already qualifies candidate selection and states that AllOf is not atomic allocation; the addendum does not imply those corrections are still absent everywhere. The specific generic-runner, hardware and receipt leaves require an explicit task mapping and integration commit.

Paired source: [Kairos PR #266 audit](https://github.com/edithatogo/kairos/pull/266); [this careops-sim PR #173](https://github.com/edithatogo/careops-sim/pull/173). The upstream path named above is a cross-repository reference, not an assumed local file on the current submodule pin.

Local review verification on 21 October at parent `356f54b965daec47bd18269c635468443f523df1`: `git diff --check` passed; `tools/tasks.py check` passed for 145 records and `tools/context.py check` passed for 5 tracks, 34 milestones and 1,368 links. Working directory and validator toolchain are the same as the retained planning receipt above. No new tests were added for this prose/catalog correction. Source inspection reconfirmed the audited capability boundaries; full runtime CI was not rerun.

## Further refinements — 21 October 2026

### Reconcile executable plans before expanding scope

Read-only inspection found divergent planning copies: careops-sim main checkout at `fc4fdd6` with uncommitted plans still prescribes an automatic `f0d30e2` upgrade (R2.1) and AllOf/AnyOf atomic team allocation (A3.2). The contracts worktree at `576f50b` instead qualifies candidate selection and explicitly rejects AllOf as an allocator. The uncommitted `inter-agent-coordination-20261020.md` claims all active agents are aligned; that is not established by these conflicting copies. Preserve both versions and let the existing roadmap owner integrate the reviewed corrections and regenerate/check its task catalog. A clean lease store alone is not permission to overwrite uncommitted proposals.

The local child branch `codex/careops-domain-capabilities-20261020` at `0157d0c` does register proposed Tracks 67–69. This is a branch-specific fact, not a change to the audit's 67-track mainline baseline or proof of remote acceptance. Tracks 68 and 69 both claim the whole DES crate; Track 67 overlaps existing core/state/ABM owners. Before dispatch, map exact files and interfaces to one writer, record owner handoffs, and qualify prerequisites from disputed Tracks 64/66. Do not infer that every new track needs all of Track 66: identify the particular RNG/experiment contract it consumes. The specs' non-breaking assertions are acceptance targets until public API/checkpoint/schema compatibility is demonstrated.

### Small acceptance refinements, using existing tasks

| Existing owner / plan | Refinement | Minimum behavioral oracle |
| --- | --- | --- |
| Kairos 29 + careops-sim R0/R2 | Keep lifecycle status separate from exercised capability; bind each consumed capability to immutable source, feature set, host/backend, command, input and retained output | Missing, mismatched or stale evidence leaves upgrade blocked; a zero-exit structural validator cannot certify physical IO, GPU, MPI or clinical fitness |
| Kairos 03/22 + careops-sim C-R4/A0 | First extraction is a small experiment lifecycle interface, preserving careops-sim facade; benchmark whether further movement is worthwhile | Same results for 1/2/4 workers and restart; cancellation never publishes partial output as complete; manifest points only to committed output; crash between output write and completion record is recoverable |
| Kairos 03/64 + careops-sim A3 | Freeze allocation commit and cancellation rules before a shared resource framework | Contending requests, timeout at grant time, partial failure and preemption conserve claims after every event; explicitly distinguish atomic bundle grant from staged acquisition |
| Proposed 67 + careops-sim A5 | Separate typed relationships from ownership/destruction policy; support one-to-many and independent lifetimes | Removing a parent relationship cannot implicitly destroy a surviving dependent; stale handles, cycles, duplicate spawn, deterministic IDs/streams and checkpoint restore are tested with neutral and clinical adapters |
| Proposed 68 + careops-sim A4 | Keep transfer semantics independent of transport; start with one scheduler and typed commands | Duplicate/reordered acknowledgements, cancellation after reservation, expiry at arrival and restart preserve exactly one domain transition per operation ID; avoid claims of network exactly-once delivery |
| Proposed 69 + careops-sim A1/B1 | Define elapsed-age/residual-time semantics, covariate changes and RNG draw ownership before a generic hazard engine | Analytic exponential competing-risk reference, zero-hazard case, tie ordering, unchanged-covariate invariance, time-varying reference and censoring tests; failed calibration is not fixed by resampling unrelated clocks |
| Kairos 52/55 + careops-sim E6/E8 | Require explicit backend provenance and measured benefit before promotion | CPU fallback cannot satisfy device acceptance; initialization/transfer/compute timings separated; throughput and latency compared at equal workload; include memory limits and cancellation/recovery, not only ideal output parity |

CC-01–CC-08 in careops-sim already cover identity, transfer, allocation, progression, capacity, policy/clocks, reproducibility and observation. Reuse those contracts; the rows above refine implementation oracles rather than create another competing framework. Keep scientific/clinical parameter validation in its existing domain process. Retain the headless CPU baseline as usable delivery while optional acceleration remains unqualified.

### Execution order and coordination boundary

1. Reconcile planning copies and status claims; establish exact-file ownership and capability dependencies.
2. Preserve the current qualified pin and freeze consumer/IO/recovery baselines.
3. Implement and qualify one neutral primitive with two concrete consumers, then migrate the adapter incrementally.
4. Qualify native device and physically distributed execution separately; promote only measured, supported capabilities.

These refinements were sent to the existing careops-sim roadmap chat under the maintainer's coordination authorization. Message delivery is not confirmed plan incorporation; retain task mappings and the integration commit before reporting completion. No new runtime tracks, source edits, pin changes or takeovers were made by this refinement. GitHub readback still timed out, so remote branch/PR status is not inferred from local refs.

Refinement validation: documentation diff against `545696596e721aee19bd217f5c7d949a7eb8f888`, in the same worktree/toolchain as the planning receipt. On 21 October, `git diff --check` and task catalog and context validators (145 tasks; 1,368 links) exited 0. These checks validate document structure, not the proposed runtime oracles.

## Confirmed careops-sim plan incorporation

Read back local integration commit `07a67fd` and naming clarification `7b671c3` on `codex/careops-roadmap-contracts-20261020`. C-R0–C-R7 are mapped into existing R/A task IDs, acceptance matrices and the regenerated catalog, with evidence in `conductor/evidence/pr173-roadmap-integration-20261021.md` on that branch. The owner retained 145 accepted / 80 pending tasks and reported passing context/catalog/MVP checks and 32 task-harness tests. No implementation checkbox was promoted by this planning integration. The original main checkout's divergent uncommitted proposals remain preserved; branch incorporation does not mean merge into main.

This incorporation concerns **careops-sim**, not **careops**. Local validation of this documentation slice: diff whitespace checks and careops-sim catalog/context checks passed (145 records on this PR branch; the separate owner integration branch contains 225 records). Hosted state remains unverified.
