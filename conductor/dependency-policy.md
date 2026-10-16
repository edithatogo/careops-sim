# Current dependency and toolchain policy

## Canonical stable toolchain — 2026-10-04

The Kairos owner directs the canonical development/integration toolchain to be the current stable Rust release: **1.99.0**, released 2026-10-01. Exact 1.99.0 pins in the toolchain, bootstrap profile and CI retain reproducibility; refresh these together promptly when a new stable release is adopted. This supersedes the historical 1.98.1 decision below. Prior receipts remain evidence for their actual compiler and source, and are not rewritten as 1.99.0 results.

Rust 1.99.0 is the sole declared and executed Rust toolchain. The default-feature workspace lane, stable quality gates and optional feature checks use the same exact 1.99.0 toolchain; no older compatibility floor, beta, or nightly lane is maintained. The official manifest and both host checksums are recorded in [the adoption receipt](evidence/rust-1.99.0-canonical-20261004.json). Historical 1.76/1.88 receipts describe only their original commits and are not current compatibility qualifications. Miri and libFuzzer AddressSanitizer runtime checks require nightly and are unavailable under this owner policy; their historical evidence remains historical, and later release security obligations remain open. Other refreshed package candidates remain unapproved.

Live registry observations are recorded in
[dependency-candidates.json](evidence/dependency-candidates.json), including source
URLs, fetch time, release date and declared MSRV. Refresh with
`python3 tools/refresh_versions.py`; it never installs or updates dependencies.
The snapshot is a candidate inventory, not a tested lockfile or promise that all
packages should be adopted. Unknown MSRV means unknown, not compatible.

## Active dependency freshness audit — 2026-10-16

The 33-entry registry snapshot was refreshed at 2026-10-16T07:01 UTC via `python3 tools/refresh_versions.py` and compared with active parent pins, Kairos direct manifests, and root/nested lockfiles. All entries remain `candidate_unvalidated`. Key observations:
- Stable Rust toolchain: 1.99.1 observed (released 2026-10-13). Pinned canonical development toolchain remains exact 1.99.0; adoption of 1.99.1 requires formal owner authorization and manifest/CI synchronization.
- Kairos Arrow array, schema, IPC, and Parquet match the 60.0.0 pin.
- Serde 1.0.229 matches root `Cargo.lock`; Kairos lock resolves 1.0.228. Serde JSON 1.0.154 observed in registry (root lock is 1.0.151, Kairos is 1.0.149).
- CLI sha2 remains pinned at 0.10.9; 0.11.0 is a breaking 0.x change requiring golden manifest verification prior to evaluation.
- Dev tools: nextest 0.9.148 observed (pinned 0.9.146); cargo-deny 0.20.2 and cargo-audit 0.22.2 match pins; zizmor 1.31.1 observed.
- GitHub Actions: actions/checkout v7.0.1, upload-artifact v7.0.2, download-artifact v8.0.2, and codeql-action v4.39.0 observed; workflows preserve exact immutable commit SHAs.
- No production dependency, lockfile, or toolchain was altered by this read-only audit.

## Historical dependency freshness audit — 2026-10-04

The 33-entry registry snapshot was refreshed at 09:09 UTC and compared with
active parent pins, Kairos direct manifests, root/nested locks and workflows.
Full read-only reviews are retained in the Rust adoption receipt. This is a
freshness assessment, not proof that every latest version is installed.

- Parent Serde/JSON and nextest/actionlint/gitleaks/cargo-deny/zizmor match the
  snapshot; Kairos Arrow/Parquet are already pinned to 60.0.0.
- CI checkout v6.0.2 and upload-artifact v4.6.2 trail v7.0.1. Evaluate verified
  immutable release SHAs, runner support, permissions and artifact behavior in
  a bounded workflow update with policy tests and hosted checks.
- Kairos Serde 1.0.228/JSON 1.0.149 trail 1.0.229/1.0.151; thiserror's exact
  2.0.20 pin trails 2.0.21. Review patch resolution, advisories/licences and
  Rust 1.99.0 default and optional feature checks in separately owned changes.
- Parent CLI sha2 0.10.9 trails 0.11.0; this is a breaking 0.x line change.
  Preserve Rust 1.76 support and exact input-byte/SHA-256 manifest outputs with
  golden checks before adoption. Parent lockfile Proptest 1.8.0 is not in the
  active parent graph; stale lock cleanup is separate from a runtime upgrade.
- Wasm bindings trail their latest release and require matching generated-tool
  and target tests. Fuzz libfuzzer-sys already resolves to 0.4.13.
- Preserve constrained Clap/Criterion and upstream-selected rand versions until
  API/MSRV/determinism migrations have explicit contracts. Do not force a
  transitive RNG major upgrade.
- New semver-checks 0.51.0 and Renovate 44.132.5 are candidate observations;
  neither establishes an active binary pin. Preserve the reviewed advisory
  mitigation pin instead of equating newer with security clearance.

The open Kairos website npm advisory remains a separate release/security hold.
No manifest or lockfile upgrade was made by this Q4 freshness assessment.

## Historical registry observations — 2026-10-03

The complete 33-entry snapshot was refreshed successfully from the public registry
endpoints recorded in the JSON. Four candidates changed since the prior snapshot:
Rust 1.99.0, insta 1.49.0, Renovate 44.132.2, and Codex CLI rust-v0.160.0.
All remain `candidate_unvalidated`; no toolchain, manifest, lockfile, workflow pin,
installed tool, compatibility decision, or desktop app version was changed.
The reviewed baseline below remains historical decision context. Rust 1.98.1
was the approved integration pin at that observation; the owner-authorized
1.99.0 adoption above supersedes this historical decision.

## Reviewed candidate baseline, recorded 2026-09-28

| Area | Registry candidate | Adoption decision |
| --- | --- | --- |
| Rust | 1.98.1 | Owner-approved exact development/integration pin; tested with the full default-feature workspace |
| Arrow array/schema/IPC, Parquet | 60.0.0; MSRV 1.88 | Owner-approved future package boundary; defer dependency adoption until C1 schema/interoperability review |
| serde / serde_json | 1.0.229 / 1.0.151 | Evaluate compatible lockfile update in owning change |
| toml | 1.1.6+spec-1.1.0; MSRV 1.85 | Owner-approved only as a future isolated config/tool package if needed; keep out of core |
| rayon | 1.12.0; MSRV 1.80 | No mandatory within-run scheduler dependency; retain deterministic event order |
| crossbeam-epoch | Patched `0.9.20` lock pin (MSRV 1.61); registry latest observed `0.9.21` | Lock-only repair for RUSTSEC-2026-0204 in the benchmark-only Criterion/Rayon graph; cargo-deny advisory/source gate passes. |
| clap / clap_builder | 4.6.7; MSRV 1.85 | The original Kairos benchmark lock resolved Clap 4.6.1, which Cargo 1.76 cannot parse. The Kairos owner approved a six-package lock-only resolution, committed on `codex/d12-bench-lock-msrv` and tested across the default-feature workspace at Rust 1.76.0 and 1.98.1; the parent pin/evidence records the exact commit. |
| rand | 0.10.3; MSRV 1.85 | Optional sampling candidate; do not replace deterministic RNG algorithm silently |
| wgpu | 30.0.1; MSRV 1.87 | E6 real Metal candidate, gate shader/API/device compatibility |
| Burn | 0.21.0; MSRV 1.92 | Deferred existing ML track; no dependency added now |
| nextest / deny / audit | 0.9.146 / 0.20.2 / 0.22.2 | D1 pin dev tools; installed audit observed 0.22.1, upgrade remains pending |
| semver-checks / llvm-cov / mutants | 0.50.0 / 0.9.1 / 27.1.0 | D3 targeted quality tools with overhead measured |
| proptest / criterion / insta | 1.11.0 / 0.8.2 / 1.48.0 | Targeted invariant/benchmark/snapshot tooling; no unnecessary runtime dependencies |
| cargo-fuzz / zizmor | 0.13.2 / 1.30.1 | D3 fuzz/nightly; D2 workflow analysis |
| checkout / upload-artifact / download-artifact | v7.0.1 / v7.0.1 / v8.0.1 | Resolve releases to full verified commit SHAs and check runner compatibility in D2 |
| CodeQL action | v4.38.2 | Use action release, not the separate codeql-bundle tag |
| Renovate / actionlint / gitleaks | 44.115.12 / v1.7.12 / v8.30.1 | Pin tested automation/tool versions in D1/D2 |
| Codex CLI | rust-v0.157.1 | Observed upstream candidate; not a claim about this desktop app's version |

Sources are the linked crates.io/GitHub/Rust distribution endpoints in the JSON.
No production dependency, host toolchain or application model was upgraded by
this audit. The upstream lock contains the owner-approved benchmark-only Clap-family
compatibility resolution plus a separate advisory fix for crossbeam-epoch; it does not change Kairos manifests, runtime dependencies or MSRV.
Conductor v0.4.1 is still its latest observed release; default-branch HEAD is now
`6e8f9a860bcdd6a2c423473c12e745200688c633`. Evaluate that commit in an isolated
canary before deciding whether an unreleased change improves this workflow.

## Bleeding-edge with reproducibility

Maintain two lanes: an exact tested integration baseline and an advisory current/
preview canary. Adopt the newest useful compatible stable release promptly;
prerelease/nightly/HEAD experiments use exact commits/dates, bounded benchmarks,
rollback evidence and explicit capability labels. Do not leave required builds
on floating `latest`, `stable`, action tags or unbounded `cargo install` commands.

Record exact dependency resolutions in Cargo.lock and tool/action pins in the
owning repository. CI uses `--locked`; publishable Rust libraries still need
appropriate dependency ranges and minimum/current resolution tests. Pinning only
a library's lockfile does not constrain downstream consumer resolution.

D1's prior compatibility direction is historical and superseded by the explicit
Rust 1.99.0 sole-toolchain owner policy above. The former Rust 1.76 default floor,
1.88 Arrow candidate floor, and beta advisory canary are not maintained as
current execution or compatibility promises. Package/API boundary reviews,
interoperability checks and dependency approval remain independent obligations;
historical receipts and decisions retain their original scope and are not rewritten.

Updates: weekly grouped low-risk patch/minor review, immediate advisory triage,
separate major/API/RNG/schema changes. New versions are reverified immediately
before adoption. Run conformance, resume, feature/MSRV and affected performance
checks. Automatic dependency merges only after required checks and stable policy;
never auto-merge RNG/time/schema, security-permission or public-release changes.
Automation is planned here; no recurring task has been enabled by this audit.

## D4 Trust Boundaries, FFI Safety, and Release Misuse Policy

Ahead of packaging, distribution, and release qualification (D4.1–D4.4), explicit threat modeling establishes system boundaries, resource constraints, data governance gates, and unsafe code boundaries to protect against misuse, corruption, and unauthorized data exposure.

### Explicit Trust Boundaries and Misuse Modeling

1. **Input Boundary (Untrusted to Engine Core)**:
   - *Threat Surface*: Scenario JSON documents, CLI arguments, and tabular data files (Arrow IPC, Parquet, CSV) ingested from user or external sources.
   - *Mitigations & Misuse Constraints*: All deserialization enforces strict schema conformance via `#[serde(deny_unknown_fields)]`. Bounded allocation caps apply to arrival counts, horizon ticks, entity allocations, and resource configurations. Inputs causing non-terminating event loops, zero-time cycles, unbounded recursion, or disk/memory exhaustion are rejected fail-closed with structured error messages before simulation admission.

2. **FFI and C-Data Boundary (Native Rust to External Runtimes)**:
   - *Threat Surface*: Foreign Function Interface (FFI) bindings for C ABI, WebAssembly, and language bridges (UniFFI, Diplomat).
   - *Mitigations & Misuse Constraints*: Raw memory pointers and foreign function pointers are quarantined within isolated interface crates (`kairo-ecs-ffi`, `kairo-ecs-fmi`, `kairo-ecs-wasm`). All incoming pointers undergo exhaustive validation (null checks, alignment, buffer length bounds, and lifetime invariants) prior to dereferencing or conversion into Rust domain references.

3. **Agent and Tool Boundary (Autonomous Automation to Environment)**:
   - *Threat Surface*: Autonomous coding agents, subagents, and automated maintenance/repair runners operating on repository and system assets.
   - *Mitigations & Misuse Constraints*: Agents operate under bounded workpackets, immutable command receipts, and explicit advisory file claims ([AGENTS.md](../AGENTS.md) and [single-maintainer.md](harness/single-maintainer.md)). One writer per owned path is strictly enforced. Automated tools and agents are forbidden from executing unbounded shell scripts, modifying CI security credentials/permissions, publishing artifacts autonomously, or mutating files outside pre-authorized write scopes.

4. **Build and Dependency Boundary (Upstream Supply Chain to Binaries)**:
   - *Threat Surface*: Compromised transitive dependencies, malicious build scripts (`build.rs`), unpinned compiler toolchains, or registry poisoning.
   - *Mitigations & Misuse Constraints*: Hermetic builds enforce the exact canonical Rust 1.99.0 toolchain and strictly locked lockfiles (`Cargo.lock` enforced via `--locked` in CI). Dependencies undergo continuous scanning via `cargo-deny` (advisories, bans, licenses, sources) and `cargo-audit`. Build scripts and procedural macros are forbidden from initiating network connections or executing untrusted ambient binaries.

5. **Output and Privacy Boundary (Engine Telemetry to External Storage)**:
   - *Threat Surface*: Execution summaries, trace logs, statistical diagnostics, and performance benchmarks leaking sensitive operational or individual-level data.
   - *Mitigations & Misuse Constraints*: Outputs are deterministic, reproducible artifacts bound to cryptographic hashes (SHA-256) of input configurations and simulation seeds. Run manifests, diagnostic tables, and Arrow/Parquet outputs emit strictly aggregate statistical distributions and anonymized entity identifiers; no Protected Health Information (PHI) or unmasked patient attributes are generated or stored.

### Arrow C Data Interface vs. Inter-Process Communication (IPC) Safety

- **In-Process C Data Interface**: Raw pointers exposed via the Arrow C Data interface (`ArrowArray`, `ArrowSchema`) are treated strictly as **trusted in-process interfaces only**. These raw pointer representations rely on shared virtual memory address space assumptions and lack transport serialization or cross-process memory protection. Exposing or passing Arrow C Data raw pointers across process, host, or security container boundaries is strictly prohibited.
- **Inter-Process and Network IPC**: All communication crossing process boundaries, container barriers, or network interfaces MUST use validated, bounded Arrow IPC streams (`arrow::ipc::reader::StreamReader`) or self-contained Parquet files. IPC streams must validate encapsulated metadata, schema definitions, dictionary encodings, alignment, message length limits, and record batch counts to prevent malformed-buffer memory corruption and allocation bomb attacks.

### Local-Data Entry Gate: Clinical and Patient Privacy

- **Synthetic Default**: All public continuous integration, development workspaces, automated test fixtures, calibration baselines, and performance benchmarks use strictly synthetic, parametric, or generic public hospital data. No actual patient data is required or presumed for engine validation.
- **Real EHR Entry Gate**: Entry of real electronic health records (EHR), Cairns Hospital operational extracts, or clinical datasets into the CareOps Sim environment is barred until all of the following entry conditions are independently audited and fulfilled:
  1. **Ethics & Governance Approval**: Valid Institutional Review Board (IRB) / Human Research Ethics Committee (HREC) approval and authorized data custodian transfer agreement.
  2. **De-Identification Verification**: Formal statistical and algorithmic certification of de-identification (complying with HIPAA Privacy Rule Safe Harbor / Expert Determination and Australian Privacy Principles), ensuring removal of all direct and indirect identifiers and verification that re-identification risk on sparse event logs is within approved thresholds.
  3. **Airgapped Encrypted Storage**: Storage on dedicated on-premise, encrypted volumes (AES-256 at rest) with hardware-enforced access controls and strict role-based authorization (RBAC). Real patient records or derivative identifiable embeddings must never be committed to Git repositories, uploaded to hosted cloud CI runners, or stored in public test artifact caches.

### Networking Entry Gate: Airgap and Distributed Communication

- **Zero-Network Airgap for Core Engine**: The native DES/ABM simulation engine (`careops-ed`, `careops-ed-cli`, `careops-sim`, and Kairos core) operates under a strict **zero-network airgap policy**. Core execution, calibration fitting, and CLI report generation must function completely offline without opening network sockets, making remote RPC calls, or resolving external DNS requests.
- **Networking Entry Gate for Distributed Profiles**: Any future distributed runner, remote visualization service, or multi-node parallel DES profile (planned in post-v1 extensions E5/E7/E8) must satisfy the networking entry gate before activation:
  - **Transport Security**: Mandatory TLS 1.3 encryption with strict certificate validation and modern cryptographic cipher suites; unencrypted plain-text communication is forbidden.
  - **Mutual Authentication**: Mutual TLS (mTLS) or cryptographically signed ephemeral identity tokens for all distributed nodes and operator interfaces.
  - **Rate Limiting & DoS Protection**: Bounded connection concurrency, request size ceilings, per-peer rate limiting, and aggressive socket timeout policies to prevent exhaustion of file descriptors or memory.
  - **Replay Mitigation**: Cryptographic nonce validation, monotonically increasing sequence numbers, and idempotent operation processing to neutralize message replay or spoofing attacks.

### Unsafe Code and Concurrency Discipline

- **Forbid Unsafe Code in Domain Modules**: All simulation domain crates (`careops-ed`, `careops-ed-cli`, `kairo-ecs-core`, `kairo-ecs-des`, `kairo-ecs-abm`, `kairo-ecs-calibration`, `kairo-ecs-types`, `kairo-ecs-rng`) strictly enforce `#![forbid(unsafe_code)]`. Any required `unsafe` blocks are quarantined exclusively in dedicated FFI adapter boundary crates (`kairo-ecs-ffi`, `kairo-ecs-fmi`) under exhaustive invariant checks and documentation.
- **Message-Passing Concurrency**: Shared-memory multithreading with mutable shared state is architecturally prohibited. Concurrent simulation workers communicate exclusively via deterministic message-passing channels and immutable, chunked data partitions. Race conditions, data races, and mutex deadlocks are eliminated by construction.
- **Miri and Sanitizer Tooling Status & Release Risk Assessment**:
  - *Toolchain Availability*: Under the canonical Rust 1.99.0 stable toolchain directed by project policy, runtime Miri undefined-behavior interpretation and libFuzzer/AddressSanitizer (ASan) runtime instrumentation require nightly toolchain features and are not executable in stable CI lanes.
  - *Risk Assessment*: Because domain crates enforce `#![forbid(unsafe_code)]` and concurrency is restricted to message-passing without shared mutable state, memory safety risks are confined entirely to the isolated FFI boundary layer.
  - *Mitigation & Release Obligation*: Prior to tagging and distributing native library release candidates (D4.2–D4.4), a dedicated dated-nightly validation pass using Miri and AddressSanitizer must be executed against all FFI entry points (`kairo-ecs-ffi`, `kairo-ecs-diplomat`, `kairo-ecs-uniffi`). Stable CI unavailability during daily development does not waive this release gate.

## D4.2 Consumer Packaging, SBOM, Provenance, and Release Immutability Policy

Prior to packaging, publishing, and distributing CareOps Sim release candidates, strict supply-chain controls bind release artifacts, cryptographic checksums, Software Bill of Materials (SBOM), and provenance attestations into a verifiable, immutable release bundle.

### Build-Once and Single-Digest Cryptographic Binding

1. **Build-Once Invariant**:
   - Every distribution artifact (crate tarball `.crate`, standalone CLI binary tarball `.tar.gz`, wheel `.whl`) is built exactly once within an isolated, hermetic continuous integration environment from clean, locked source (`cargo build --locked --release`).
   - Downstream consumers, testing harnesses, and verification workflows must evaluate the identical compiled binary artifact bytes. Rebuilding binaries from source in subsequent packaging or attestation steps is strictly prohibited.
2. **Single-Digest Cryptographic Binding**:
   - The cryptographic SHA-256 digest computed across the canonical output bytes of each release artifact is the sole authoritative subject identifier across all supply-chain documents:
     - `SHA256SUMS`: Plaintext record mapping hexadecimal SHA-256 digests to distribution artifact filenames.
     - SPDX 2.3 SBOM (`spdx.json`): Package checksum record binding each delivery archive (`Checksum: SHA256 <hex>`).
     - SLSA Provenance v1 (`provenance.json`): Attestation `subject` array recording artifact name and hex digest (`subject[].digest.sha256`).
   - If any artifact byte differs between delivery bundles, checksums, SBOM, or provenance, the release is invalid and fails verification.

### Multi-Owner Governance Model

Packaging, attestation, and distribution responsibilities are partitioned across seven distinct Conductor owners, each exercising sole authority over their operational domain:

| Owner | Domain & Architectural Responsibility | Governance Mandate |
|---|---|---|
| **Owner 15** | Release packaging & clean distribution | Owns distribution packaging scripts, archive layout (`tar.gz`, `.crate`), reproducible permissions (`tar --mode=0755`), and consumer download staging. |
| **Owner 16** | Provenance attestations & SLSA v1 | Owns generation and signing of in-toto SLSA Provenance v1.0 attestations, builder identity verification, materials tracking, and build invocation parameter locking. |
| **Owner 20** | CI isolation & dirty tree rejection | Enforces clean ephemeral build runners (`git status --porcelain` check == 0), isolated network sandboxing post-fetch (`--network=none`), and hermetic compiler environments. |
| **Owner 25** | Version governance & semver patch increments | Governs semantic versioning contracts (`Cargo.toml`), tag taxonomy (`vX.Y.Z`), patch increment authorization, and immutable release lifecycle management. |
| **Owner 28** | SPDX 2.3 SBOM generation & verification | Governs Software Bill of Materials (SPDX 2.3 JSON), component dependency graph capture, package verification codes, and external reference validation. |
| **Owner 42** | License notices (Apache-2.0) & legal compliance | Governs statutory legal compliance, top-level `LICENSE` and `NOTICE` files, crate manifest license expressions (`Apache-2.0`), copyright headers, and third-party license audit clean-status via `cargo-deny`. |
| **Owner 44** | Dependency lockfile integrity | Governs `Cargo.lock` reproducibility, exact dependency commit/version pins, canonical MSRV enforcement (`1.99.0`), and prevention of floating dependency ranges. |

### Consumer-Side Verification Protocol

An end-consumer or downstream automated deployment agent verifies the release bundle strictly from published assets without requiring access to source code or build infrastructure:

1. **Artifact Byte Validation against `SHA256SUMS`**:
   - The consumer downloads `SHA256SUMS` along with distribution archives.
   - Computes independent SHA-256 digests of downloaded archives.
   - Verifies bit-for-bit equality against `SHA256SUMS`.
2. **SPDX 2.3 Schema and Component Digest Audit**:
   - Validates SBOM schema conformity against SPDX 2.3 JSON specification (`spdxVersion == "SPDX-2.3"`, `dataLicense == "CC0-1.0"`).
   - Verifies that root package component checksums match the computed artifact SHA-256 digest.
   - Confirms declared licenses match `Apache-2.0` and third-party dependency licenses comply with repository policy.
3. **SLSA v1 Provenance and Freshness Audit**:
   - Validates provenance against in-toto attestation schema (`_type == "https://in-toto.io/Statement/v1"`, `predicateType == "https://slsa.dev/provenance/v1"`).
   - Validates that `subject` digest matches the artifact SHA-256 digest.
   - Verifies that git commit SHA in `resolvedDependencies` or `invocation.configSource` matches the authorized clean git commit tag.
   - Evaluates evidence freshness: attestation timestamp (`predicate.buildDefinition.internalParameters.buildTimestamp` or metadata) must fall within a **30-day freshness horizon** ($\Delta t \le 30\text{ days}$) relative to the evaluation date. Evidence older than 30 days is rejected.

### Six Mandatory Failure Gates

Release evaluation scripts and consumer verifiers must enforce six mandatory failure gates, failing closed upon any detected anomaly:

1. **Dirty Release Inputs (`DirtyReleaseInputError`)**:
   - *Trigger*: Uncommitted modifications, staged changes, or untracked files present in the build tree (`git status --porcelain` non-empty) during packaging.
   - *Behavior*: Build aborts immediately with `DirtyReleaseInputError`. No release artifacts, checksums, or attestations are generated.
2. **Missing Artifact (`MissingArtifactError`)**:
   - *Trigger*: An artifact declared in `SHA256SUMS`, SBOM packages, or SLSA provenance subjects is absent from the distribution payload.
   - *Behavior*: Verification fails closed with `MissingArtifactError`. Partial release bundles are rejected.
3. **Substituted / Corrupted Artifact Bytes (`DigestMismatchError`)**:
   - *Trigger*: The SHA-256 hash of an artifact byte stream differs from the digest recorded in `SHA256SUMS`, SPDX SBOM, or SLSA Provenance.
   - *Behavior*: Immediate fail-closed abort with `DigestMismatchError`. Artifact is discarded as tampered or corrupted.
4. **Expired Evidence (`ExpiredEvidenceError`)**:
   - *Trigger*: Attestation or verification evidence has a generation timestamp older than the 30-day freshness window.
   - *Behavior*: Verification halts with `ExpiredEvidenceError`. Stale release evidence cannot qualify a release.
5. **Wrong-Commit Attestation (`WrongCommitError`)**:
   - *Trigger*: SLSA Provenance materials, git commit claims, or SBOM metadata reference a commit hash that does not match the signed release tag commit SHA.
   - *Behavior*: Aborts with `WrongCommitError`. Prevents misattribution of build provenance to divergent or unvetted branches.
6. **Incompatible Schema Version (`IncompatibleSchemaError`)**:
   - *Trigger*: SBOM or Provenance documents specify unsupported schema specifications (e.g., deprecated SPDX 2.1 / CycloneDX 1.2, or legacy SLSA v0.2 when SLSA v1 is required).
   - *Behavior*: Aborts with `IncompatibleSchemaError`. Prevents parser confusion or evasion attacks against security tools.

### Immutability and Corrected-Version / Yank Policy

1. **Absolute Immutability of Released Versions and Tags**:
   - Once a version tag (e.g., `v0.1.0`) is published and attached to release artifacts, it is **permanently immutable**.
   - Git tags, release binaries, checksum digests, and attestation documents must never be modified, deleted, overwritten, or re-pointed in place.
   - Any attempt to overwrite an existing release tag or replace released archive bytes triggers a critical security halt with `ImmutableReleaseViolationError`.
2. **Corrected-Version Workflow**:
   - If a published release is found to contain a severe defect, security vulnerability, or corrupted binary:
     1. **Do Not Overwrite**: The defective release remains in place to preserve cryptographic auditability and prevent breakages in existing downstream lockfiles.
     2. **Increment Semver Patch**: Owner 25 authorizes a semantic version patch increment (e.g., from `0.1.0` to `0.1.1`).
     3. **Clean-Source Rebuild**: The patch release is built from clean source with the defect resolved.
     4. **Yank Defective Version**: Owner 15 executes registry withdrawal (`cargo yank --version 0.1.0 careops-ed`) or marks the release deprecated in the distribution index. This prevents new projects from adopting the flawed release while allowing existing lockfiles to continue resolving.
     5. **Publish Advisory**: A formal security/defect advisory (RUSTSEC / GitHub Security Advisory) is published explaining the defect, affected versions, and upgrade path to `0.1.1`.

## D4.3 Rollback, Checkpoint Migration, and Cross-Platform Reproducibility Policy

Prior to publishing v0.1.0 and authorizing general deployment, explicit policies govern rollbacks to last-known-good pins, schema migration and state conservation for simulation checkpoints, cross-platform reproducibility bounds across claimed architectures, and non-negotiable boundaries for unsupported features.

### Submodule Pin Rollback Protocol

1. **Strict Reviewed Pinning**:
   - The upstream Kairos submodule (`libs/kairos`) is pinned strictly to the owner-reviewed, native-accepted commit: `44261eb5cf8275f132f63a3396903d116d701dbe`.
   - Floating branches, detached unverified heads, or unreviewed submodule upgrades are prohibited.
2. **Fail-Closed Verification Gate**:
   - The verification script [`tools/check_kairos_pin.py`](../tools/check_kairos_pin.py) enforces continuous integrity of the submodule pin. It checks:
     - Exact commit identity in [`conductor/evidence/d2.4-kairos-contract.json`](evidence/d2.4-kairos-contract.json).
     - Submodule pin declaration in [`conductor/current-state.json`](current-state.json).
     - Committed and index gitlinks (`160000` gitlink tree entry).
     - Clean checkout status and exact `HEAD` rev-parse in `libs/kairos`.
     - Cryptographic SHA-256 verification of all tracked upstream source files.
   - Any discrepancy, divergence, uncommitted tracked change, or missing acceptance evidence halts CI with exit code 1.
3. **Rollback Procedure**:
   - When an unreviewed, incompatible, or failing upstream commit is detected:
     1. **Revert Gitlink**: `git update-index --cacheinfo 160000,44261eb5cf8275f132f63a3396903d116d701dbe,libs/kairos` or `git checkout HEAD -- libs/kairos`.
     2. **Restore Upstream Tree**: `git -C libs/kairos checkout 44261eb5cf8275f132f63a3396903d116d701dbe && git -C libs/kairos clean -fd`.
     3. **Reconcile Evidence**: Restore reviewed contract in [`conductor/evidence/d2.4-kairos-contract.json`](evidence/d2.4-kairos-contract.json) and state metadata in [`conductor/current-state.json`](current-state.json).
     4. **Execute Verification**: Run `python3 tools/check_kairos_pin.py` to confirm full restoration.

### Dependency Pin Rollback Protocol

1. **Strict Cargo.lock Adherence**:
   - All builds, continuous integration pipelines, and release rehearsals execute with `cargo build --locked` and `cargo test --locked`.
   - Floating version ranges or ambient updating of dependencies without formal review is strictly prohibited.
2. **Unauthorized Tampering Detection & Rollback**:
   - Any unauthorized edit or drift in `Cargo.lock` triggers an immediate build failure under `--locked` (`error: the lock file needs to be updated but --locked was passed`).
   - Rollback procedure:
     1. Discard untrusted modifications: `git checkout HEAD -- Cargo.lock`.
     2. Reverification: Execute `cargo check --locked --quiet` and `cargo audit` to confirm integrity against project security policies.

### Checkpoint Schema Migration and Rollback Protocol

1. **Schema Versioning Support**:
   - Simulation state checkpoints generated during long-running batch experiments support explicit integer schema versioning (`schema_version: 1` $\to$ `schema_version: 2`).
   - Legacy schema version 1 represents minimal batch progress (`BatchCheckpoint` containing completed replication arrays and pending indices).
   - Schema version 2 introduces enhanced provenance tracking, including `checkpoint_version: "2.0.0"`, `simulation_engine: "careops-ed-v1"`, `migration_metadata` (timestamps, operator identity, SHA-256 replication checksum), and cryptographic `provenance_digest`.
2. **State Conservation Law**:
   - Any checkpoint migration from schema version 1 to 2 MUST satisfy the strict State Conservation Law:
     $$\text{len}(\text{completed\_replications}) + \text{len}(\text{pending\_indices}) = \text{total\_replications}$$
   - Migrations MUST preserve 100% of completed replication records bit-for-bit without data loss, reordering, index mutation, or precision truncation.
   - Base seeds, scenario configurations, horizon overrides, and batch identifiers must remain invariant across migrations.
   - Any missing replication, duplicate index, or set divergence raises `StateConservationViolationError` and fails closed.
3. **Reversible Migration (Rollback Protocol)**:
   - Checkpoint migration is strictly bidirectional and reversible.
   - If downstream tools, older worker nodes, or external analytical scripts require schema version 1, schema version 2 checkpoints can be downgraded cleanly via `tools/checkpoint_migration.py rollback`.
   - The rollback procedure validates schema version 2 cryptographic digests and replication checksums before safely stripping version 2 envelope metadata, producing an exact, conforming schema version 1 document.
   - Resuming a batch from a rolled-back v1 checkpoint produces identical mathematical and operational results as resuming directly from v2.

### Cross-Platform Reproducibility Policy

1. **Claimed Supported Platforms**:
   - CareOps Sim v0.1.0 officially supports and qualifies exactly two Tier 1 host platforms:
     - `x86_64-unknown-linux-gnu` (Linux x86_64)
     - `aarch64-apple-darwin` (macOS Apple Silicon ARM64)
   - Other target triples (including `x86_64-pc-windows-msvc`, `i686-unknown-linux-gnu`, `wasm32-unknown-unknown`, and `wasm32-wasi`) are unverified and unsupported in v0.1.0.
2. **Deterministic Reproducibility Invariant**:
   - For any identical scenario specification, input configuration, and integer base seed:
     - **Integer Equivalence**: All discrete state metrics, event counts, patient arrivals, completions, unfinished counts, queue lengths, discrete tick totals, and PRNG draw sequences must match **100% bit-for-bit** across both claimed platforms.
     - **Floating-Point Reduction Tolerance**: Floating-point reductions (such as aggregate mean waiting times, standard errors, and resource utilizations) are bounded by strict IEEE 754 floating-point reduction tolerance:
       $$|\Delta_{\text{metric}}| \le 1.0 \times 10^{-12}$$
   - Any simulation run exceeding this tolerance or exhibiting integer drift triggers `PlatformReproducibilityError` and fails release qualification.

### Explicit Unsupported-Feature Boundary Policy

To prevent misuse, avoid scope creep, and protect patient safety, the following six operational boundaries are explicitly defined as **unsupported and prohibited** in CareOps Sim v0.1.0:

1. **No Real EHR or Private Clinical Patient Data**:
   - The engine is qualified exclusively on synthetic, parametric, or generic open hospital operational data.
   - Ingestion of real Electronic Health Record (EHR) data or identifiable patient event streams is strictly barred until independent Institutional Review Board (IRB) / Human Research Ethics Committee (HREC) approval, formal HIPAA Safe Harbor / APP de-identification verification, and encrypted on-premise local storage are certified.
2. **No Autonomous Public Publication**:
   - Automated scripts, background CI runners, and AI agent harnesses are strictly forbidden from publishing packages to public registries (crates.io, PyPI, npm) or creating public GitHub Releases.
   - All external releases require explicit human maintainer authorization, verifiable PGP commit signatures, and the deliberate assertion of `CAREOPS_HUMAN_MAINTAINER_AUTHORITY=1`.
3. **No Hardware Acceleration (GPU / Metal / CUDA / MLX) in Core**:
   - The simulation engine core is architected strictly as single-core or multi-core CPU discrete-event / agent-based simulation.
   - GPU, CUDA, Apple Metal, or WebGPU compute kernels are not bundled into the core engine. GPU acceleration exploration is deferred to Track 32.
4. **No Distributed Networking (MPI / gRPC / Sockets) in Core**:
   - The native core executes under a strict zero-network airgap. No network sockets, RPC listeners, or distributed consensus protocols are included in core binaries.
   - Multi-node distributed clustering is deferred to Track 35.
5. **No Interactive GUI / CAD / PixiJS Renderer in Core**:
   - CareOps Sim v0.1.0 is a headless computational engine and CLI tool (`careops-ed`, `careops-ed-cli`).
   - Interactive 2D/3D graphical interfaces, CAD spatial visualizers, and web canvases (PixiJS) are not bundled in core and are deferred to visualization tracks (Tracks 05 and 09).
6. **No Cross-Process Arrow C Data Sharing**:
   - Raw C Data pointers (`ArrowArray`, `ArrowSchema`) are restricted exclusively to in-process FFI memory boundaries within the same address space.
   - Passing raw memory pointers across OS processes, containers, or network boundaries is prohibited; all IPC must utilize bounded Arrow IPC streaming buffers (`arrow::ipc`) or self-contained Parquet files.

## D5.1 Bounded Maintenance Automation and Scheduled Quality Policy

Following the successful completion and acceptance of Milestone G1 (Hardened native v1 closeout across Phases E4 and D4), scheduled maintenance automation is unlocked under strict bounding constraints. Automated schedules must never precede reliable, deterministic continuous integration. Under Task D5.1, background maintenance operates as an advisory, bounded quality guardian designed to detect drift early without creating maintainer alert fatigue, mutation loops, or unreviewed repository state.

### Scope & Principles

1. **Prerequisite Reliability Gate**: Scheduled jobs run only against proven, stable CI baselines. If core compilation, unit tests, or platform checks are unstable, automated maintenance schedules remain disabled.
2. **Advisory by Default**: Scheduled routines audit, verify, and diagnose. They do NOT execute autonomous code edits, unreviewed git commits, pull request merges, or releases.
3. **One Writer per Path**: Any remediation suggested by automated maintenance must be explicitly authorized and claimed by the designated Conductor owner in accordance with [AGENTS.md](../AGENTS.md) and [single-maintainer.md](harness/single-maintainer.md).

### The Four Scheduled Maintenance Routines

1. **Version Freshness**:
   - *Scope*: Continuous audit of `Cargo.lock`, compiler toolchains, GitHub Actions workflows, and upstream submodule pins against reviewed baselines.
   - *Cadence & Execution*: Weekly candidate review; triggered via [`tools/refresh_versions.py`](../tools/refresh_versions.py) and [`tools/maintenance_scheduler.py`](../tools/maintenance_scheduler.py).
   - *Invariants*: Canonical Rust toolchain is exact `1.99.0`; `libs/kairos` submodule pin must strictly match [`conductor/evidence/d2.4-kairos-contract.json`](evidence/d2.4-kairos-contract.json); CI workflow actions must resolve to immutable commit SHAs, never floating major tags (`@v4`, `@v7`).
2. **Documentation Drift**:
   - *Scope*: Real-time cross-validation of markdown links, CLI help flags, scenario schema definitions, and evidence registers against actual code and filesystem reality.
   - *Cadence & Execution*: Pre-commit and bi-weekly scheduled check; executed via [`tools/context.py check`](../tools/context.py) and schema validation scripts.
   - *Invariants*: Zero dead markdown links; CLI `--help` parameters exactly match argument parser declarations; scenario schemas in `model-inputs/ed/schema/` (`capacity-location.schema.json`, `example-profile-pack.schema.json`) accurately reflect parser expectations.
3. **Dependency Groups**:
   - *Scope*: Comprehensive audit of crate dependency graphs, open-source license declarations, forbidden dependency trees, and `#![forbid(unsafe_code)]` compliance.
   - *Cadence & Execution*: Weekly automated audit; executed via `cargo-deny` checks, manifest inspection, and AST/regex tree scanning.
   - *Invariants*: All workspace crates declare `Apache-2.0` license expressions; domain crates (`careops-ed`, `careops-sim`) and CLI roots (`careops-ed-cli`) strictly enforce `#![forbid(unsafe_code)]`; prohibited licenses (GPL, AGPL, SSPL) and unvetted network crates are rejected fail-closed.
4. **Extended Tests**:
   - *Scope*: Periodic deep soak tests, property-based invariants (`proptest`), long-horizon memory benchmarks, and deterministic PRNG replay stress checks.
   - *Cadence & Execution*: Nightly or weekly scheduled matrix runs; executed via `cargo test --locked` and dedicated benchmark harnesses.
   - *Invariants*: Memory utilization remains bounded and leak-free ($O(1)$ heap growth after transient warm-up); discrete event counts and PRNG draw sequences remain 100% bit-for-bit reproducible across seeds.

### Six Core Operational Controls

All automated routines orchestrated by [`tools/maintenance_scheduler.py`](../tools/maintenance_scheduler.py) are bounded by six non-negotiable operational controls:

1. **No-Change Silence**:
   - Maintenance runs that detect no drift, anomalies, or policy violations MUST remain completely silent (`silent_on_clean = True`).
   - Clean executions emit zero terminal output to `stdout`/`stderr`, send zero email/Slack/webhook notifications, and create zero redundant pull requests or commits. Maintainers are notified only when actionable defects arise.
2. **Actionable Reports**:
   - When drift or violations occur, the system emits a structured, deterministic machine-readable report (JSON) accompanied by human-readable diagnostic output.
   - Every violation record explicitly specifies:
     - `routine`: Name of the maintenance routine (e.g., `version_freshness`).
     - `error_class`: Strict machine-readable error class (e.g., `SubmodulePinDriftError`, `LicenseDeclarationDriftError`).
     - `file_path`: Exact file path relative to repository root.
     - `line_number`: Specific offending line number.
     - `conductor_owner`: Designated Conductor owner responsible for the domain (e.g., `Owner 13 (Upstream Kairos Core)`, `Owner 42 (Legal & Compliance)`).
     - `remediation_command`: Deterministic shell command to immediately resolve or test the fix.
     - `message`: Diagnostic explanation of the drift.
3. **Idempotency**:
   - Execution of maintenance jobs is strictly idempotent.
   - Sequential invocations against unchanged repository state produce zero compounding side-effects, zero file modifications, zero uncommitted git changes, and identical diagnostic findings.
4. **Timeout Enforcement**:
   - Every scheduled routine is subject to a strict execution timeout (default: 60 seconds per routine, configurable via `--timeout`).
   - If a routine hangs, blocks on disk I/O, or enters an infinite loop, the scheduler cleanly terminates the worker process with `SchedulerTimeoutError` and logs an actionable failure report.
5. **Retry Ceiling**:
   - Transient failures (e.g., temporary disk lock contention, transient process spawn failure) are bounded by a strict retry ceiling (default: 3 retries, configurable via `--retry-ceiling`).
   - If a routine fails consistently across all attempts, execution halts immediately with `RetryCeilingExceededError`. Infinite retry loops and recursive scheduling cascades are strictly forbidden.
6. **Pause and Resume Controls**:
   - Human maintainers can pause the scheduler at any time via the command:
     ```bash
     python3 tools/maintenance_scheduler.py pause --reason "<audit or maintenance justification>"
     ```
   - While paused, the scheduler state file (`conductor/maintenance-scheduler-state.json`) records `paused: true`, timestamp, operator identity, and reason.
   - Any scheduled or background run attempted while paused **fails closed** immediately with `SchedulerPausedError` (exit code 2) without executing any routine or mutating state.
   - When manual intervention or auditing is complete, the human maintainer restores automated scheduling via:
     ```bash
     python3 tools/maintenance_scheduler.py resume
     ```
   - Resumption restores normal scheduled execution safely without state corruption or duplicate queued runs.

### Absolute Prohibition of Autonomous Publication

Under no circumstances may automated maintenance routines, CI bots, background schedules, or AI coding agents autonomously push tags, publish packages to registries (crates.io, PyPI, npm), or generate public releases:
- Any routine attempting `cargo publish`, `git push`, `gh release create`, or credentialed external uploads triggers an immediate security halt with `AutonomousPublicationViolationError`.
- All release publication requires explicit human maintainer verification, cryptographic signature, and the manual assertion of `CAREOPS_HUMAN_MAINTAINER_AUTHORITY=1`.

## D5.2 Agent Governance, Scope Restriction, and Exact-Commit Merge Policy

Task D5.2 establishes rigorous governance, security boundaries, and empirical evaluation standards for optional repair and reviewer agents deployed across CareOps Sim workflows.

### 1. Role Definitions & Advisory Authority Boundaries

1. **Optional Repair Agents**:
   - *Scope & Function*: Assist human maintainers with narrowly bounded bug fixes, syntax repairs, compiler diagnostic remediation, and documentation alignment within explicitly assigned, owned paths.
   - *Authority Boundary*: Repair agents possess **zero merge or publication authority**. Their outputs constitute proposals requiring deterministic validation, test harness passes, and independent review.
2. **Optional Reviewer Agents**:
   - *Scope & Function*: Provide independent, read-only analysis of candidate diffs, property invariants, deterministic PRNG replay sequences, statistical calibration residuals, and schema compatibility.
   - *Authority Boundary*: Reviewers can challenge diffs, demand additional evidence, or emit `hold_for_evidence` / `reject` dispositions. However, consensus between agents does **not** constitute proof of correctness; human maintainer authority remains the sole certification of acceptance.

### 2. Strict Write Scope Restrictions & Protected Paths

1. **Path-Level and Directory-Level Reservations**:
   - Prior to making any workspace modifications, agents must declare and acquire an explicit, exclusive path reservation covering only the files necessary for their active task (orchestrated via [`tools/agent_eval.py`](../tools/agent_eval.py) and [`tools/agent_session.py`](../tools/agent_session.py)).
   - Writes targeting any path outside the active reservation fail closed immediately with `ScopeViolationError`.
2. **Absolute Prohibition on Protected Core Files**:
   - The following repository-critical artifacts are permanently protected from agent modification:
     - `crates/careops-ed/src/lib.rs` (core ED domain contract and module root)
     - `conductor/delivery-contract.md` (delivery and acceptance contract)
     - `LICENSE` (Apache-2.0 legal grant)
   - Any attempt by an agent or automated script to write, truncate, delete, or rename these files triggers an unbypassable, fail-closed `ProtectedPathError`.
3. **Escaped and Traversal Guardrails**:
   - Direct access to `.git`, directory traversal sequences (`..`), and symlinks resolving outside the repository root fail closed with `ValueError` and immediate claim revocation.

### 3. Single-Writer-Per-Owned-Path Invariant

To eliminate write races, clobbering, and merge divergence across concurrent agent sessions:
- **Single-Writer Rule**: At most one active writer process or agent lease may hold a write reservation on any given path or directory at any instant across all worktrees.
- **Hierarchical Overlap Detection**: Reservations are checked hierarchically and normalized against Unicode NFC and casefolding. A claim on `tools` precludes any concurrent claim on `tools/context.py`, and vice-versa.
- **Fail-Closed Conflict**: Any overlapping claim triggers an immediate `MultipleWritersError`, halting the competing agent before any filesystem write occurs.

### 4. Exact-Commit Merge Rule

Automated merges and qualification reviews must guarantee exact commit identity to prevent TOCTOU (time-of-check to time-of-use) vulnerabilities:
- **Exact-Commit Verification**: A candidate branch or patch may only be merged if required CI test runs, local harnesses, and reviewer approvals were executed against the **exact Git commit SHA** that is current HEAD on the target branch.
- **Stale CI Invalidation**: Reviews, status checks, or test receipts performed against an older ancestor commit cannot qualify a subsequent commit. Any commit drift between test execution and merge triggers an unrecoverable `ExactCommitMismatchError`.
- **Pre-Merge Gate**: Pre-merge validation requires:
  1. `tested_commit_sha == target_head_sha`
  2. All required CI test matrix jobs reported passing status.
  3. Reviewer verdict equals `pass` (zero unresolved `hold_for_evidence` or `reject` notices).

### 5. Agent Performance, Safety, and Cost Accounting

All agent invocations and evaluation workflows must capture structured telemetry:
1. **Token Usage and Cost Tracking**:
   - Input tokens, output tokens, context length, and dollar cost must be logged per invocation.
2. **Zero-Tolerance False-Pass Rate on Safety Defects**:
   - **Critical Threshold**: **0.0%** false-pass rate on safety-critical defects (PRNG determinism regressions, trust boundary violations, schema mismatches, protected path tamper attempts).
   - Any agent model or configuration that emits a false pass on a known safety regression is immediately disqualified from automated workflow participation.
3. **False-Reject Rate & Human Interventions**:
   - False-rejects (unnecessary blocking of clean patches) and maintainer intervention events (lease recoveries, conflict resolutions, manual overrides) are tracked to assess net maintainer burden.

### 6. Absolute Prohibition of Autonomous Publication

- Autonomous pushing (`git push`), tag generation (`git tag`), GitHub release publication (`gh release create`), and package publishing (`cargo publish`, `npm publish`, `twine upload`) remain strictly prohibited.
- Release publication demands explicit human maintainer cryptographic signing and the environment assertion `CAREOPS_HUMAN_MAINTAINER_AUTHORITY=1`.

## D5.3 Quality Debt, Flaky Test Quarantine, Performance Drift, and Workflow Rollback Policy

Task D5.3 establishes continuous quality surveillance, maintainer effort protection, performance benchmarking guardrails, and automated workflow rollback mechanisms.

### 1. Scope & Principles: Ongoing Quality Surveillance & Maintainer Effort Protection

1. **Evidence Protection Mandate**: Automation must strengthen, never dilute, repository verification evidence. Automated tooling that generates ambient noise, false alarms, or unverified churn undermines developer confidence and shall be rolled back.
2. **Maintainer Effort Conservation**: Automated maintenance and CI workflows must deliver a net reduction in human maintainer cognitive load. When the maintenance overhead or triage burden of an automated workflow exceeds the time it saves, the workflow is classified as burdensome and must be disabled.
3. **Continuous Auditing**: Quality debt, flaky test inventories, runtime throughput, and heap memory profiles are monitored continuously against canonical baselines.

### 2. Quality Debt Tracking Policy

1. **Continuous Debt Marker Scanning**:
   - Technical debt markers (`TODO`, `FIXME`, `HACK`, `XXX`, `DEPRECATED`) across all tracked source, documentation, and configuration files are systematically inventoried via [`tools/quality_tracker.py`](../tools/quality_tracker.py).
   - Weighted scoring is assigned per marker severity: `FIXME` (5), `HACK` (4), `XXX` (3), `TODO` (2), `DEPRECATED` (1).
2. **Compiler Warning and Hygiene Deficits**:
   - Zero-warning invariant on canonical stable Rust (`rustc 1.99.0` with `#![forbid(unsafe_code)]` and `-D warnings`).
   - Line coverage floor ($\ge 80.0\%$) and zero viable surviving mutant policy established in D3.3 are maintained on critical engine crates.
3. **Hard Debt Threshold Ceilings**:
   - Repository-wide quality debt score must remain below the strict ceiling of **150 points**.
   - Any commit or PR that causes quality debt to breach this ceiling fails CI closed immediately with `QualityDebtExceededError`.
   - New debt markers introduced in core simulation modules require explicit owner approval and tracking issue linkage.

### 3. Flaky Test Quarantine Policy

1. **Zero Ambient Flake Tolerance**:
   - Non-deterministic test execution, hidden test retries (e.g., `nextest --retries > 0`), and unquarantined intermittent failures are strictly forbidden in CI and local verification harnesses.
2. **Strict Quarantine Requirements**:
   - Any test exhibiting non-deterministic behavior must be immediately documented in the quarantine inventory ([`.config/d33-quality-gates.json`](../.config/d33-quality-gates.json)).
   - Each quarantine entry must define all five required fields:
     - `test_id`: Fully qualified test identifier.
     - `owner`: Designated maintainer responsible for resolution (e.g., `Owner 13 (Kairos Core)`).
     - `reason`: Root-cause hypothesis or reproduction notes.
     - `issue_url`: Canonical issue tracker URL.
     - `expires_on`: Hard expiration date (ISO 8601 `YYYY-MM-DD`).
3. **Hard 14-Day Expiration Ceiling**:
   - Quarantine grants are temporary mitigations with a **maximum lifetime of 14 calendar days**.
   - Permanent or indefinite quarantine is strictly prohibited.
4. **Fail-Closed Expiry Enforcement**:
   - [`tools/quality_tracker.py flakes`](../tools/quality_tracker.py) validates active quarantine inventories on every CI run.
   - Any expired entry or entry lacking required attribution fails CI closed immediately with `UnquarantinedFlakeError`.

### 4. Performance Drift & Memory Leak Policy

1. **Regular Benchmark Tracking Against Native Baseline**:
   - Discrete-event simulation throughput and memory footprint are benchmarked against the certified native baseline established in Task E4.2:
     - Baseline Simulation Throughput: $\ge \mathbf{540,000\,\text{steps/sec}}$ ($\ge 270,000$ completed patients/sec).
     - Baseline Memory Footprint: Peak RSS $\le \mathbf{10.0\,\text{MB}}$ across multi-day (2,500-patient) runs.
     - Memory Growth Slope: $0.00\,\text{KB}$ leak over 20-run soak repetition.
2. **Performance Regression Thresholds**:
   - **Throughput Drop Tolerance**: Any sustained throughput drop $> 15.0\%$ relative to canonical baseline triggers `PerformanceRegressionError` (exit code 1).
   - **Memory RSS Growth Tolerance**: Any heap growth $> 15.0\%$ over baseline or a positive memory leak slope across soak iterations triggers `PerformanceRegressionError`.
3. **Automated Enforcement**:
   - Performance regression evaluation is enforced programmatically via `evaluate_performance()` in [`tools/quality_tracker.py`](../tools/quality_tracker.py).

### 5. Maintainer Effort & Automation ROI Policy

1. **Automation Efficiency Evaluation**:
   - All automated workflows, maintenance schedulers, bot commenters, and agent assistance pipelines are audited for operational return-on-investment (ROI).
   - Metric tracking captures: maintainer hours saved ($H_{\text{saved}}$), triage and investigation hours ($H_{\text{triage}}$), false alerts count ($N_{\text{false}}$), and workflow maintenance hours ($H_{\text{maint}}$).
2. **Automation Efficiency Ratio ($AER$)**:
   - Defined as:
     $$AER = \frac{H_{\text{saved}}}{H_{\text{triage}} + H_{\text{maint}} + (N_{\text{false}} \times 0.5\,\text{hrs})}$$
   - **Retention Threshold**: Sustained automation requires $AER \ge 2.0$ and net time savings $\ge 3.0\,\text{hours}$.
   - **Classification Rules**:
     - **Beneficial ($AER \ge 2.0$)**: Retained. Delivers clear maintainer effort savings with low false alarm burden.
     - **Neutral ($1.0 \le AER < 2.0$)**: Monitored. Marginal ROI; requires tuning of alert thresholds to prevent evidence erosion.
     - **Burdensome ($AER < 1.0$ or $N_{\text{false}} \ge 5$ with low savings)**: Target for immediate rollback.

### 6. Workflow Rollback Mandate

1. **Mandatory Rollback Trigger**:
   - Any automated workflow, scheduled task, or agent evaluation pipeline that weakens evidence, generates excessive false alerts, or drains maintainer focus must be rolled back immediately.
2. **Rollback Procedure**:
   - Execution of rollback via [`tools/quality_tracker.py rollback`](../tools/quality_tracker.py):
     ```bash
     python3 tools/quality_tracker.py rollback --workflow-id "<id>" --config "<config_path>" --reason "<justification>" --user "<maintainer>" --receipt-dir ".artifacts/receipts"
     ```
   - Workflow state is set to `enabled: false` in the owning configuration file.
   - An immutable JSON rollback receipt is generated recording `receipt_id`, UTC timestamp, `workflow_id`, `reason`, `user`, and `config_change`.
   - A post-mortem review is filed in `conductor/evidence/` prior to any prospective re-enablement.

### 7. Absolute Prohibition of Autonomous Publication

Under no circumstances may automated quality tracking routines, benchmark scripts, or rollback tools autonomously publish packages or push commits:
- Execution of `git push`, `git push --tags`, `cargo publish`, `gh release create`, or credentialed registry uploads triggers an immediate security halt with `AutonomousPublicationError`.
- All release publication requires explicit human maintainer verification, cryptographic signature, and the manual assertion of `CAREOPS_HUMAN_MAINTAINER_AUTHORITY=1`.



