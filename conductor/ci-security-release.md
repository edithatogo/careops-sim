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

Use the pinned Rust 1.99.1 candidate lane for active builds, tests, benchmarks and policy
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

## Cargo lock regeneration and SHA-2 qualification — 19 October 2026

Renovate PR158 changed the SHA-2 declaration without regenerating Cargo.lock;
its locked native checks correctly refused that stale source. The official
registry identifies 0.11.1 as the latest release including development versions.
The candidate pins that exact version, uses Rust 1.99.0 and updates only workspace
resolution and newly required transitive packages; the Kairos pin is unchanged.

The Cargo lock workflow automatically regenerates required resolution on source
PRs, main pushes, weekly schedules and manual dispatch. It runs native tests and
the existing dependency policy, retains the exact candidate and honest stage
receipt even on failure, and refuses source checks whose committed lock differs.
It never commits a repair or promotes a dependency. Three actual-workflow
regressions exercise failure receipts, lock-byte binding and stale-lock refusal.
The first hosted full harness found the rollback validator still expecting
SHA-2 0.10.9. Advance only its exact SHA-2 pin to 0.11.1 with the reviewed
manifest/lock candidate; existing tamper/refusal and restoration tests remain.
All applicable current-head native and source gates remain required before merge.

## Scheduled schema development qualification — 19 October 2026

The schema development candidate tests run weekly or manually, independently of the
canonical Rust 1.99.0 lane. It tests exact Python 3.14.8 and 3.15.0, plus a published
next-runtime preview selected from an immutable discovery manifest. An absent
preview is recorded as unavailable; it is not a successful preview execution.

Candidate declarations are isolated from source pins and resolved with prereleases
allowed, universal integrity hashes and wheel-only installation. Nine affected
schema regression groups execute on each actual candidate interpreter. Security
auditing uses that interpreter's installed distribution inventory, so environment
markers cannot silently exclude a target dependency when the audit tools run on
Python 3.14.8. Execution receipts bind runtime, package inventory and lock hashes;
setup failures, missing evidence and failed tests cannot qualify a candidate.
Four regression tests exercise the actual preparation, discovery and receipt code.
The first main dispatch exposed a missing required discovery CLI argument before
any candidate execution. The corrected call passes the explicit GitHub output
file. Relevant PRs now exercise discovery against the pinned shared tool; only
schedule/manual events execute development dependency candidates. An early
discovery failure is setup failure, not successful preview qualification.

Candidate artifacts never promote versions or change native objects. Promotion
requires a separately reviewed exact-pin update and all applicable hosted checks.
This schema-only lane does not qualify a Rust engine or native release.

## Suite source CI maintenance — 19 October 2026

The CareOps suite CI goal owns this isolated source-workflow update. Retain Rust
1.99.0 bindings, Kairos/Conductor pins, domain tests and the fail-closed Required
checks aggregate. Adopt immutable current checkout/artifact actions and reviewed
checksum-bound Zizmor1.31.1. Add the qualified immutable shared source-security
workflow alongside existing Rust dependency/licence and scoped policy checks.

Install schema-validation tools from a universal hash lock. PR/push qualification
regenerates the exact declarations, installs only verified wheels in an isolated
Python3.14.8 environment, runs all nine affected schema regression groups, audits dependencies using the immutable suite
audit-tool hash lock, and refuses uncommitted lock
repair. Scheduled/manual runs retain candidate lock and actual stage outcomes,
without source promotion. The existing full harness/native gates remain intact;
weekly lock qualification avoids duplicating unrelated native builds. New check protection follows successful hosted
qualification; no existing required/native gate is waived. Development runtime
canaries, actual release SBOM/signatures and optional-interface qualification
remain open suite work; this update does not reopen or recertify accepted MVP
tasks or Kairos native releases.

Local preparation evidence: five hash-locked packages install in a fresh exact
Python3.14.8 environment, lock regeneration is byte-identical, and an altered
candidate is refused by the actual guard. Dependency audit reports no known
vulnerabilities; actionlint and pedantic Zizmor pass. A full670-case harness run
found one stale artifact-pin assertion, now repaired to read the reviewed tool
manifest while retaining failure-report preservation assertions. Targeted repair
and actual workflow receipt/guard regressions pass. Fresh hosted full-suite and
native gates, policy binary execution and protection readback remain required;
no full current-head hosted qualification is inferred from these local results.

First hosted PR159 result: Rust/Linux/macOS tests, fmt/clippy/default/doctest,
policy and shared security passed. Context failed before tests because the
runner-selected Python3.12 required a conditional dependency outside the3.14
lock; schema qualification passed its81cases but audit-tool setup failed because
pinned PyYAML6.0.3 has no usable Python3.15 wheel under wheel-only installation.
Explicitly select the already-qualified3.14.8 schema runtime in context and the
isolated audit environment. Preserve hashes, wheel-only installation and exact
tool versions; do not treat these setup failures as advisory/test failures or
claim current-head hosted qualification before rerun.

## Next exact schema backend candidate

Official PyPI metadata identifies RPDS2026.9.1 as newer than the existing
2026.5.1 schema backend; the other four exact schema packages have no newer
published version or later development candidate in this observation. Prepare
only that RPDS pin and regenerate the universal integrity lock in an isolated
worktree. Qualify all nine schema validator groups and dependency audit with
verified wheels on exactPython3.14.8; retain the other four package versions.
This candidate follows PR159 and is not adopted by that source-CI repair. Full
current-head hosted/domain checks and normal protected merge remain necessary
before promotion; no Rust engine/submodule or native acceptance change implied.

Candidate local evidence: a fresh verified-wheel Python3.14.8 install selects
RPDS2026.9.1 while preserving the other four package versions. All81cases across
the nine affected schema groups pass, and dependency audit reports no known
vulnerabilities. Hosted current-head/full-domain qualification remains pending.

## Isolated Rust development qualification — 19 October 2026

The suite CI goal adds a separate weekly/manual and source-PR canary for newer
stable patches, beta and nightly compilers. Official manifests are frozen by date,
SHA256 and full compiler commit. Qualification re-fetches the dated manifest and
requires the installed release, compiler commit and native host to match before
locked workspace tests, doctests and Clippy execute. Setup, compiler mismatch,
test failures, source drift and invalid terminal evidence remain unqualified.
Receipts retain exact commands, exit codes, log hashes and source identity.

This bounded candidate lane preserves the exact canonical toolchain declared in
`rust-toolchain.toml`,
Cargo lock, engine/submodule pins and ordinary native gates. It does not enable
Miri/libFuzzer, change accepted native evidence, grant a licence or promote any
candidate. A later exact-pin adoption must synchronize canonical policy, manifests
and CI and pass all affected current-head checks. The existing single-file
Rust1.99.1 proposal fails the maintenance scheduler's canonical-pin check; that
failure is not evidence of a compiler or simulation regression.

Local preparation passes9identity/receipt/setup/test/drift regressions, workflow
syntax and pedantic security lint. Real official discovery freezes Rust1.99.1,
beta1.100.0-beta.7 and nightly1.101.0 with full compiler commits and dated manifest
SHA256 verification. This is discovery and infrastructure evidence; actual
candidate native execution and hosted current-head gates remain pending.


### Schema discovery terminal evidence — 19 October 2026

The source candidate adopts qualified immutable CareOps f0e8db2 across source
security, schema lock and canary helper checkouts. An always-run discovery step
records setup/discovery failure, cancellation or missing execution even when no
receipt was produced. Partial evidence is preserved; apparent successful discovery
without a valid receipt fails. Discovery alone never claims runtime tests.
Focused tests execute the actual workflow terminal script; hosted current-head
qualification remains required. Rust pins, schema locks and domain tests remain
unchanged. Revert the source refs and terminal-step change together for recovery.


### Coordinated compiler source proposal — 19 October 2026

Rust 1.99.1 is synchronized across active CI, bootstrap and quality tools.
Official manifest and supported-host checksum bindings are in the bootstrap
profile. Historical runs above remain tied to their actual Rust 1.99.0 sources.
The candidate requires its own current-head Linux/macOS and context checks;
prior canary success does not establish integrated-source acceptance. The Cargo
lock, upstream pins, immutable action SHAs and declared minimum remain unchanged.

