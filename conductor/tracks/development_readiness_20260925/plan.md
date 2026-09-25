# Plan: development readiness

Execution: use the shared [serial/parallel protocol](../../execution-model.md) and
[small-packet decomposition guide](../../execution/decomposition.md). The task
catalog preserves every prerequisite and phase closeout. Prepare and validate
bounded worker packets before dispatch; gpt-6-luna is a candidate worker, not an
assumed authority for unresolved contracts or acceptance decisions.

Status: bootstrap slice implemented; remaining work planned. All phase completion
requires [workflow](../../workflow.md), evidence and relevant upstream owner review.

## D0 — Audit and context foundation

- [x] D0.1 Inspect module/CI/toolchain boundaries; record readiness and live registry
  candidates; add AGENTS map, context resume/check and negative regression tests.
  Evidence: [audit receipt](../../evidence/development-audit.md).
- [ ] D0.2 Turn the module table into capability/owner/gate records, refresh the
  upstream DAG, reconcile prerequisite contracts and classify required vs deferred
  APIs. Test that missing prerequisite evidence blocks a readiness claim.
- [ ] D0.3 Define the ED-native support/release profile with 25/30 and independent
  review checklist; preserve upstream global publication holds. Record ADRs.
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
- [ ] D1.3 Inventory available Conductor/security/browser skills; source or author
  context-resume, deterministic-review, calibration-review and dependency-upgrade
  skills only for missing behaviors. Record source commit/licence/hash and tools.
- [ ] D1.4 Write held-out harness/skill evals first, including fake pass, pin drift,
  hidden clamp error and untrusted instructions; test the candidate skills and
  approve only those improving correctness/intervention cost over the baseline.
- [ ] D1.5 Add structured command receipts, input/output hashes and checkpoint
  validation without private payloads. Exercise kill/restart, stale context and
  concurrent-writer rejection; log tool/model version where available.
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
- [ ] D4.2 Rehearse clean-consumer package/install, licence notices, SBOM, checksum
  and provenance generation/verification with owners 15/16/20/25/28/42/44.
  Test expired/wrong-commit evidence and an incompatible schema as failures.
- [ ] D4.3 Rehearse rollback to last-known-good pins and checkpoint migration;
  verify reproducibility across claimed platforms and create release notes and
  explicit unsupported-feature list. No autonomous public publication.
- [ ] D4.4 Conductor — review and verify phase (workflow.md).

Exit: verifiable release candidate evidence, no unresolved critical release risks.
Manual verification: independent install and attestation/checksum verification.

## D5 — Bounded maintenance automation

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
