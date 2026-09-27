# Plan: development readiness

Execution: use the shared [serial/parallel protocol](../../execution-model.md) and
[small-packet decomposition guide](../../execution/decomposition.md). The task
catalog preserves every prerequisite and phase closeout. Prepare and validate
bounded worker packets before dispatch; gpt-6-luna is a candidate worker, not an
assumed authority for unresolved contracts or acceptance decisions.

Status: bootstrap slice implemented; remaining work planned. All phase completion
requires [workflow](../../workflow.md), evidence and relevant upstream owner review.


## Research already completed and remaining acceptance

Completed: requested research responses received, duplicates reconciled, embedded
material indexed and relevant findings incorporated into specifications/plans.
See [research handoff](../../research/research-handoff.md) and its evidence links.
This completed intake is distinct from the numbered implementation/qualification
tasks below. No broad repeat search or separate-file recovery is a prerequisite.

Remaining: reuse supplied tables, payloads and narrative; verify relevant primary
sources/current source code, document citations and transformations, reconcile
conflicts, and fill only demonstrated gaps. Mark locally authored structured
records as transcribed or derived; never claim recovery of an unseen original.
A missing source stays unverified or an explicit synthetic assumption with a
validity limit. Acceptance requires this track's actual outputs and tests.

## D0 — Audit and context foundation

- [x] D0.1 Inspect module/CI/toolchain boundaries; record readiness and live registry
  candidates; add AGENTS map, context resume/check and negative regression tests.
  Evidence: [audit receipt](../../evidence/development-audit.md).
- [x] D0.2 Turn the module table into capability/owner/gate records, refresh the
  upstream DAG, reconcile prerequisite contracts and classify required vs deferred
  APIs. Test that missing prerequisite evidence blocks a readiness claim.
  Reconcile Kairos commit/date and Cargo/README versus LICENSE metadata
  using local objects; report 10 repository identity claims are not authoritative.
  Consume reports 10/11/26/28 as completed research intake; verify each adopted capability/CI/harness claim against the current pinned source and document verified, contradicted or unresolved status.
Evidence: [D0.2 execution receipt](../../evidence/d0.2-execution-receipt-20260927.md),
source-backed [capability inventory](../../evidence/d0.2-kairos-capability-inventory-20260927.json),
[prerequisite/identity audit](../../evidence/d0.2-prerequisite-identity-audit-20260927.json),
and [reports 10/11](../../evidence/d0.2-research-audit-reports-10-11-20260927.md)/
[26/28](../../evidence/d0.2-research-audit-reports-26-28-20260927.md) reviews.
- [x] D0.3 Define the ED-native support/release profile with 25/30 and independent
  review checklist; preserve upstream global publication holds. Record ADRs.
  Classify every capability as MVP, native v1 or post-v1 per the delivery contract. Do not require CAD/UI/Wasm/device/network backends or optional repair automation for native acceptance.

Evidence: [ADR-0001](../../decisions/ADR-0001-ed-native-support-profile.md),
[capability profile](../../evidence/d0.3-ed-native-profile-20260927.json), and
[execution/review receipt](../../evidence/d0.3-execution-receipt-20260928.md).
- [ ] D0.4 Conductor — review and verify phase (workflow.md).

Exit: reviewed dependency closure, reproducible context recovery, no false G0 claim.
Manual verification: resume in a fresh session and reach the exact next task.

## D1 — Reproducible current toolchain and evaluated agent skills

- [ ] D1.1 Add bootstrap tests for missing/wrong tools and clean macOS ARM/Linux
  environments. Resolve Rust 1.98.1/current registry candidates; pin versions and
  record platform checksums. Test minimum/current consumer dependency resolution.
- [ ] D1.2 Resolve Arrow/TOML/Rayon versus MSRV with owners 25/30. Update affected
  manifests, policy checks and docs together; keep existing core promises unless
  an approved transition applies. Separate stable baseline and dated canary lane.
  Review report 26/28 version claims against live registries and supported targets; no reported latest version is approved by intake. Test advertised MSRV explicitly and record any reviewed transition; keep MLX C/C++ and Rust-wrapper toolchains distinct.
- [ ] D1.3 Inventory available Conductor/security/browser skills; source or author
  context-resume, deterministic-review, calibration-review and dependency-upgrade
  skills only for missing behaviors. Record source commit/licence/hash and tools.
- [ ] D1.4 Write held-out harness/skill evals first, including fake pass, pin drift,
  hidden clamp error and untrusted instructions; test the candidate skills and
  approve only those improving correctness/intervention cost over the baseline.
  Compare no-skill/pinned-skill and serial/parallel arms on undisclosed
  variants; split correct unit conversions from injected wrong-output cases. Report
  known-bad acceptance and bad-among-accepted separately; account for repeated fixtures.
- [ ] D1.5 Add structured command receipts, input/output hashes and checkpoint
  validation without private payloads. Exercise kill/restart, stale context and
  concurrent-writer rejection; log tool/model version where available.
  Recheck dependency/input/skill hashes at dispatch and acceptance; preserve
  immutable attempt lineage, actual changed paths and exact command evidence.
  Keep current packet/DAG authority; prove hidden-oracle isolation before claims.
- [ ] D1.6 Conductor — review and verify phase (workflow.md).

Exit: one-command minimal native bootstrap and measured skill evaluations; no
runtime Python dependency. Manual verification: reproduce from a clean checkout.

## D2 — GitHub creation and minimal dependable CI

Entry: D1 and E0's buildable skeleton/fixture. Follow
[the creation gate](../../ci-security-release.md); no placeholder remote earlier.

- [ ] D2.1 Write local CI smoke/negative fixtures: failing test must fail required
  gate, unchanged path is explicit, untrusted PR lacks secrets, missing artifact
  cannot pass. Prepare fmt/clippy/unit/doctest/context lanes before creation.
- [ ] D2.2 Resolve owner/name/visibility; check for existing repo; create or link
  once at the value gate. Push reviewed source, configure one-maintainer branch
  rules and verify actual required-check behavior on a small PR.
- [ ] D2.3 Pin action commit SHAs and tool versions; run actionlint/zizmor, secret
  scan and full dependency policy. Set narrow permissions, budgets, concurrency,
  cache keys and native Linux/macOS ARM lanes appropriate to the support promise.
  Audit full cargo-deny advisories/bans/licenses/sources, immutable action SHAs including rejection of mutable version tags, recursive submodule identity and cache trust separation. Report 28 upstream workflow findings require source readback; upstream R0–R4 labels do not replace D phases.
- [ ] D2.4 Add exact-parent-pin Kairos integration plus upstream owner CI; verify
  the parent fails against an incompatible pin even if upstream tests passed.
- [ ] D2.5 Conductor — review and verify phase (workflow.md).

Exit: actual hosted CI/settings evidence and a deliberately failing-check proof,
not workflow presence alone. Manual verification: clone remotely and run the fixture.

## D3 — Risk-focused quality and security depth

- [ ] D3.1 Add failing malformed-input, state-machine, resume and seed-change
  cases as implementations land; promote minimized property/fuzz failures to fixtures.
- [ ] D3.2 Add a tested dated nightly fuzz/sanitizer lane, selective Miri/unsafe
  checks and Rust CodeQL extraction where available. Keep unsupported analysis
  explicit; preserve advisories/licence/bans/source gates.
- [ ] D3.3 Add nextest/doctest, feature/MSRV/semver/schema and mutation/coverage
  checks for critical paths. Audit flaky tests with owner/expiry, not hidden retries.
  Use a bounded supported feature matrix, independent bidirectional Arrow interoperability and separate doctests. Mutation thresholds and semver comparisons require legitimate baselines; nightly/Miri/security-advisory claims need verified versions before adoption.
- [ ] D3.4 Establish representative ED/load/memory/cancellation/soak benchmarks;
  set recorded budgets before release and maintain existing upstream thresholds.
  Test slow consumers, oversized input, zero-time loops and exhausted storage.
- [ ] D3.5 Conductor — review and verify phase (workflow.md).

Exit: Q/C/E behavior has targeted failure detection and measured costs. Manual
verification: inject a known queue/RNG/metric defect and verify the proper gate fails.

## D4 — Release, threat and provenance qualification

Entry: E3 integrated runner, Q5 and C6; native library packaging is finalized in E4.

- [ ] D4.1 Review input/FFI/agent/build/output trust boundaries and misuse cases;
  resolve release-blocking findings and test resource limits. Record local-data
  and networking entry gates without presuming real EHR access.
  Treat Arrow C Data pointers as trusted in-process interfaces only; use validated bounded IPC across process boundaries. Apply sanitizers/Miri to actual unsafe surfaces and concurrency checks when real parallel code exists.
- [ ] D4.2 Rehearse clean-consumer package/install, licence notices, SBOM, checksum
  and provenance generation/verification with owners 15/16/20/25/28/42/44.
  Test expired/wrong-commit evidence and an incompatible schema as failures.
  Build once from clean locked source; bind checksums, SBOM and attestations to the same actual artifact bytes and verify as consumer. Reject missing/substituted artifacts and dirty release inputs; rehearse corrected-version/yank policy without rewriting immutable releases.
- [ ] D4.3 Rehearse rollback to last-known-good pins and checkpoint migration;
  verify reproducibility across claimed platforms and create release notes and
  explicit unsupported-feature list. No autonomous public publication.
- [ ] D4.4 Conductor — review and verify phase (workflow.md).

Exit: verifiable release candidate evidence, no unresolved critical release risks.
Manual verification: independent install and attestation/checksum verification.

## D5 — Bounded maintenance automation

Entry: E4 hardened native v1 and D4; this optional extension cannot block v1.

- [ ] D5.1 Add schedules only after CI is reliable: version freshness, documentation
  drift, dependency groups and extended tests. Validate no-change silence,
  actionable reports, idempotency, timeout, retry ceiling and pause controls.
- [ ] D5.2 Evaluate optional repair/reviewer agents on held-out tasks; restrict
  write scope and permissions, record cost/false-pass/intervention metrics and
  require exact-commit checks before merges. Keep one writer per owned path.
- [ ] D5.3 Track quality debt, flaky tests and performance drift; review whether
  automation saves maintainer effort. Roll back workflows that weaken evidence.
- [ ] D5.4 Conductor — review and verify phase (workflow.md).

Exit: demonstrated maintenance loop with bounded cost and correct failures.
Manual verification: pause and resume a job without duplicate edits or publication.

## MVP gpt-6-luna workpack

Every task in this track that is an ancestor of E2.4 is decomposed in the
[MVP leaf recipes](../../execution/mvp/README.md) and
[readable work breakdown](../../execution/mvp/work-breakdown.md). These recipes
are mandatory preparation inputs: freeze/bind interfaces, source slices, paths,
commands and reviewer acceptance before dispatch. Parent tasks close only after
all leaf instances and the original phase acceptance pass. Post-MVP tasks are
outside this workpack. No Luna execution or qualification is implied by coverage.
