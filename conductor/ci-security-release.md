# Phased CI, automation, quality and security

## D0/D1: fast local feedback

Pin the supported development profile; run fmt, targeted clippy/tests, doctests,
context checks and the baseline fixture. Add secret scanning and dependency source/
licence/advisory checks from the first dependency/code changes. Keep private data
out of fixtures. Deeper security work is phased later; these cheap controls start
now because retroactive cleanup is costly.

## D2: create GitHub when it provides immediate value

**Creation gate:** E0 has a buildable minimal Rust workspace and a tiny deterministic
fixture; D1 has a tested tool lock/bootstrap; source/licence/data boundaries are
clear; local required checks pass; a reviewed minimal CI workflow is ready to run.
At that point hosted CI, backups and review history justify the remote, before
sustained Q1/C1/E1 implementation. The parent has no remote today; do not create a
placeholder now or defer it until the finished library.

Resolve owner/name/visibility at the gate from current user context; ask only for
missing choices. Check for an existing matching repository first. Prefer private
initial development until publication/readiness is decided, without assuming
that preference is approval of a visibility choice. Then create/link once, push
reviewed source, run CI, read back repository settings and verify required checks.
The user's request authorizes setup at the useful time; it does not authorize
publishing private data or guessing visibility. Budget/plan limitations of hosted
features must be recorded, with local equivalents if necessary.

Protect main with required checks and branch/PR workflow suited to one maintainer.
Do not require an impossible independent human approval on a one-person repo.
Use evidence-backed review and explicit owner merge; keep bypasses documented.
Require the checks on the actual commit/merge candidate. Add `merge_group` if a
merge queue is enabled. Publication and cloud credentials are unnecessary here.

Minimal CI: path-aware fast checks with an always-running aggregate required gate
that explicitly handles unchanged scopes; native Linux and macOS ARM coverage for
supported packages; exact tool pins and lockfiles; bounded timeout/concurrency;
short failed-test artifacts; scoped caches keyed by toolchain/target/lockfile.
Cancellation is appropriate for superseded PR checks, not release attestations.
Use nextest and add doctests separately; reuse build archives only for matching
platform/toolchain/provenance ([nextest guidance](https://nexte.st/docs/ci-features/archiving/)).

## D3: harden as behavior lands

Use the pinned Rust 1.99.0 lane for active builds, tests, benchmarks and policy
checks. Miri and libFuzzer AddressSanitizer runtime jobs are unavailable under the
current owner policy because they require nightly; prior dated-nightly results
remain historical and are not current qualification. This unavailability does
not waive later release obligations for unsafe/FFI review, malformed-input and
resource-exhaustion assurance, sanitizer/Miri or fuzz evidence, or an equivalent
owner-approved method. PR gates include relevant golden/differential/restore
tests; longer stable runs can be scheduled without hiding release blockers.
Flaky tests retain issue, owner, expiry and failure signal; unlimited retries
never produce an honest pass.

Run cargo-deny advisories, sources, licences and bans, plus cargo-audit with
recorded database freshness. Add actionlint and zizmor, full-SHA action pins,
least-privilege permissions, and no secrets for untrusted PR execution. Separate
trusted publishing workflows from untrusted build inputs; never run PR code under
privileged pull_request_target. See [GitHub secure-use guidance](https://docs.github.com/en/actions/reference/security/secure-use).

Current Kairos CodeQL covers JavaScript only. Evaluate/add Rust scanning using
[GitHub's Rust query support](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-queries/rust-built-in-queries),
checking availability/licensing and real extraction coverage. Static analysis
supplements tests, not proof of safety. Use focused mutation tests for queue,
checkpoint and metric invariants; measure branch coverage on changed critical
paths, rather than chasing an arbitrary repository score.

## D4: before release, networking or private data

Threat-model file ingestion/decompression/resource exhaustion, unsafe FFI/Wasm,
dependency/build scripts, agent tool authority and telemetry disclosure. Add input
size/event/probe/runtime limits, malformed-data/fuzz tests and dependency/license
review. Network profiles add auth/TLS, quotas, replay/dedup, failure recovery and
operator boundaries before real multi-node use. Cairns data requires a separate
access/minimization/retention/export review; public CI must use synthetic data.

Qualify a scoped ED native release with install/use in a clean consumer project,
semver/schema checks, package dry-run, docs/examples, reproducible artifacts,
checksums, SBOM and provenance verification. Use trusted publishing/OIDC where the
registry supports it, scoped to the intended repository/ref/environment; otherwise
record the least-privilege alternative. Verify attestations as a consumer.
Preserve upstream 15/16/20/25/28/42/44 holds; an ED profile is not a waiver of global
Kairos release governance. Test rollback/restore and keep last-known-good pins.

## D5: bounded automation after trustworthy gates

Enable grouped dependency PRs, stale-context/link checks and scheduled extended
quality runs with clear budgets and actionable reports. Record source-version
freshness, test duration, flaky-rate, performance drift, unresolved risks and
maintainer intervention rate. Fix proposals stay scoped; secrets/permissions/
publishing changes do not auto-merge. Pause/retry/lease controls prevent repeated
agent jobs from fighting over files. No jobs are enabled in this planning audit.
