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
