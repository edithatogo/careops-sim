# CareOps Sim Support Matrix & Migration Policy

Document Version: 1.0.0  
Lifecycle Stage: Hardened Native v1 (Conductor Milestone G1 Closeout / Phase E4)  
Effective Date: 2026-10-16  

---

## 1. Supported Platform Matrix

CareOps Sim (`careops-ed` and `careops-ed-cli`) provides Tier 1 native support for local CPU-based discrete-event and agent-based operational simulation across primary enterprise and research architectures.

### 1.1 Tier 1 Primary Platforms

| Target Triple | Operating System | Architecture | Toolchain & MSRV | Execution Mode | Verification Status |
|---|---|---|---|---|---|
| `x86_64-unknown-linux-gnu` | Linux (glibc ≥ 2.31, Ubuntu 20.04+, Debian 11+, RHEL 8+) | x86-64 (AMD64) | Rust 1.99.0 (`stable`) | Local CPU (1–N cores) | **TIER 1 SUPPORTED** (CI & Container Verified) |
| `aarch64-apple-darwin` | macOS 13+ (Ventura, Sonoma, Sequoia) | Apple Silicon (M1/M2/M3/M4) | Rust 1.99.0 (`stable`) | Local CPU (1–N cores) | **TIER 1 SUPPORTED** (Workstation Verified) |

### 1.2 Toolchain & Compiler Dependencies

- **Rust Version**: Pin is Rust 1.99.0 (`rust-toolchain.toml` at repository root).
- **Minimum Supported Rust Version (MSRV)**: Rust 1.99.0 (`rust-version = "1.99"` in `Cargo.toml`).
- **Cargo Resolver**: Version 2 (`resolver = "2"`).
- **Submodule Dependencies**: Pinned Kairos engine (`libs/kairos`) at audited commit hash.
- **Python Integration Test Suite**: Python 3.11 or 3.12 (standard library only; no external package dependencies required for basic harness).

---

## 2. Explicit Unsupported Boundaries (Out of Scope for Native v1)

In accordance with [`conductor/delivery-contract.md`](../conductor/delivery-contract.md), the hardened native v1 release enforces strict, non-negotiable architectural and clinical boundaries:

| Out-of-Scope Capability | Boundary Specification | Rationale & Policy |
|---|---|---|
| **Live EHR / Clinical Feeds** | **STRICTLY EXCLUDED** | No live HL7, FHIR, DICOM, or EHR ingestion is implemented. Inputs are strictly bounded synthetic or public-aggregate datasets. |
| **Site-Specific Calibration** | **STRICTLY EXCLUDED** | Software verification does not imply domain validation. No operational endorsement of Cairns Hospital or any clinical site is claimed. |
| **Autonomous Publication** | **STRICTLY EXCLUDED** | Release rehearsal forbids autonomous publishing or artifact distribution without explicit human coordinator authorization. |
| **Hardware GPU / Metal** | **POST-V1 EXTENSION** | Native v1 executes exclusively on CPU. Metal and GPU acceleration are reserved for Phase E6 / Track 32. |
| **Distributed / MPI Clusters** | **POST-V1 EXTENSION** | Parallel execution is bounded to multi-core shared-memory batch execution via worker threads. Distributed IPC is excluded. |
| **Graphical User Interface (GUI)** | **POST-V1 EXTENSION** | Native v1 is strictly headless CLI/API. Spatial replay, interactive PixiJS, and dashboard components belong to Phase E5. |

---

## 3. Schema Versioning & Forward Migration Policy

CareOps Sim schemas follow semantic versioning principles to guarantee backward compatibility, deterministic error detection, and safe migrations.

### 3.1 Schema Version Registry

| Component | Current Schema Version | Specification File | Compatible Range |
|---|---|---|---|
| **Scenario Configuration** (`ScenarioConfig`) | `1` | [`crates/careops-ed/examples/nominal_mvp.json`](../crates/careops-ed/examples/nominal_mvp.json) | Version 1 only (Strict equality) |
| **Capacity & Location Profile** | `3` | [`crates/careops-ed/src/lib.rs`](../crates/careops-ed/src/lib.rs) | Version 3 only |
| **Run Document & Manifest** | `1` | [`crates/careops-ed/src/api_runner.rs`](../crates/careops-ed/src/api_runner.rs) | Version 1 only |
| **Tabular Arrow Schema** | `1.0` | [`crates/careops-ed/src/arrow_export.rs`](../crates/careops-ed/src/arrow_export.rs) | Version 1.0 |

### 3.2 Migration Guarantees & Enforcement

1. **Strict Reject on Mismatch**:
   - Any scenario configuration declaring an unsupported schema version (e.g. `schema_version: 2`) is rejected during `validate` and `run` with exit code `4` (`MALFORMED_CONFIG`).
   - Mismatched schema versions between base and candidate runs during `compare` are rejected with exit code `5` (`INCOMPATIBLE_SCENARIO`).
2. **Deny Unknown Fields**:
   - All scenario structs utilize Serde's `#[serde(deny_unknown_fields)]`. Undocumented or speculative attributes trigger immediate parse errors rather than silent omission.
3. **Migration Path for Future Releases**:
   - Future schema updates (e.g., Schema v2 for Phase E5 spatial attributes) will provide offline schema migration scripts in `tools/` with migration tests in `tests/test_d43_checkpoint_migration.py`.
   - Legacy schema files will be translated deterministically without altering simulation seeds or patient itineraries.

---

## 4. Deterministic Exit Codes & Error Taxonomy

The CLI and API guarantee deterministic exit codes across all platforms:

| Exit Code | Def Name | Meaning | Troubleshooting Guidance |
|---|---|---|---|
| **0** | `Success` | Execution completed without error. | Inspect stdout or output JSON/CSV/Arrow artifact. |
| **2** | `UsageError` | Missing command or invalid argument format. | Run `careops-ed --help` to inspect command syntax. |
| **3** | `FileNotFound` | Input scenario or run document path does not exist. | Verify path spelling and file permissions. |
| **4** | `MalformedConfig` | Schema version mismatch, invalid JSON, or out-of-range field. | Run `careops-ed validate <path>` to diagnose schema errors. |
| **5** | `IncompatibleScenario` | Mismatched time units or schema versions in `compare`. | Ensure both runs use identical time units and schema versions. |
| **6** | `FileAlreadyExists` | Target output file exists without `--overwrite` flag. | Add `--overwrite` or supply a new target path. |
| **7** | `OutputWriteFailed` | I/O failure, permission denied, or disk space exhausted. | Check disk quota, write permissions, and parent directory existence. |
| **8** | `SimulationFailed` | Invariant violation or capacity state error during execution. | Check that staffed bay counts are non-negative and capacity is available. |
| **9** | `Interrupted` | User cancelled execution (SIGINT/SIGTERM) or timeout tripped. | Rerun simulation or resume batch from last saved checkpoint. |
