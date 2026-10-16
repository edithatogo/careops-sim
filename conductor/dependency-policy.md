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
