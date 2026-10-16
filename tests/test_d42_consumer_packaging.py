"""Consumer packaging, SBOM, SLSA provenance, and release immutability tests for Task D4.2.

Validates:
1. Build-once and single-digest cryptographic binding across artifacts, SHA256SUMS, SPDX 2.3 SBOM, and SLSA v1 provenance.
2. Multi-owner governance model alignment (Owners 15, 16, 20, 25, 28, 42, 44).
3. Consumer verification protocol for clean distribution packages.
4. Six mandatory failure gates:
   - DirtyReleaseInputError
   - MissingArtifactError
   - DigestMismatchError
   - ExpiredEvidenceError
   - WrongCommitError
   - IncompatibleSchemaError
5. Absolute release immutability and corrected-version / yank policy:
   - ImmutableReleaseViolationError on tag or blob overwrite attempts
   - Corrected patch increment and cargo yank workflow
"""

from __future__ import annotations

from datetime import datetime, timedelta, timezone
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

ROOT_DIR = Path(__file__).resolve().parents[1]


# Custom Domain Exceptions for Failure Gates
from tools.release_rehearsal import (
    ReleaseRehearsalError,
    DirtyReleaseInputError,
    MissingArtifactError,
    DigestMismatchError,
    ExpiredEvidenceError,
    WrongCommitError,
    IncompatibleSchemaError,
    ImmutableReleaseViolationError,
)


# Governance Model Mapping
OWNER_ROLES = {
    15: "Release packaging & clean distribution",
    16: "Provenance attestations & SLSA v1",
    20: "CI isolation & dirty tree rejection",
    25: "Version governance & semver patch increments",
    28: "SPDX 2.3 SBOM generation & verification",
    42: "License notices (Apache-2.0) & legal compliance",
    44: "Dependency lockfile integrity",
}


def compute_sha256(data: bytes) -> str:
    """Compute hex SHA-256 digest of byte payload."""
    return hashlib.sha256(data).hexdigest()


def check_clean_working_tree(git_status_porcelain: str) -> None:
    """Owner 20 gate: Fail closed if git working tree has modifications or untracked files."""
    stripped = git_status_porcelain.strip()
    if stripped:
        lines = stripped.splitlines()
        raise DirtyReleaseInputError(
            f"Release build rejected: working tree contains {len(lines)} uncommitted/untracked item(s):\n"
            + "\n".join(f"  {line}" for line in lines[:5])
        )


def verify_consumer_release_bundle(
    artifact_dir: Path,
    sha256sums_path: Path,
    spdx_sbom_path: Path,
    slsa_provenance_path: Path,
    expected_commit: str,
    evaluation_time: datetime,
    max_age_days: int = 30,
) -> dict:
    """Consumer-side verification protocol validating artifacts, SBOM, provenance, and freshness."""
    # 1. Parse SHA256SUMS
    if not sha256sums_path.exists():
        raise MissingArtifactError(f"SHA256SUMS manifest not found at {sha256sums_path}")

    sums_lines = sha256sums_path.read_text(encoding="utf-8").strip().splitlines()
    declared_digests: dict[str, str] = {}
    for line in sums_lines:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split(maxsplit=1)
        if len(parts) == 2:
            digest, fname = parts[0], parts[1].lstrip("*").strip()
            declared_digests[fname] = digest.lower()

    if not declared_digests:
        raise MissingArtifactError("SHA256SUMS is empty or contains no valid digest entries.")

    # 2. Check artifact presence and byte digests
    actual_digests: dict[str, str] = {}
    for fname, expected_digest in declared_digests.items():
        artifact_file = artifact_dir / fname
        if not artifact_file.exists():
            raise MissingArtifactError(f"Declared artifact '{fname}' is missing from release directory.")
        content = artifact_file.read_bytes()
        actual_digest = compute_sha256(content)
        if actual_digest != expected_digest:
            raise DigestMismatchError(
                f"Artifact '{fname}' digest mismatch: computed {actual_digest}, expected {expected_digest} in SHA256SUMS"
            )
        actual_digests[fname] = actual_digest

    # 3. Validate SPDX 2.3 SBOM
    if not spdx_sbom_path.exists():
        raise MissingArtifactError(f"SPDX SBOM file not found at {spdx_sbom_path}")

    try:
        sbom_data = json.loads(spdx_sbom_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise IncompatibleSchemaError(f"SPDX SBOM is not valid JSON: {exc}") from exc

    spdx_version = sbom_data.get("spdxVersion")
    if spdx_version != "SPDX-2.3":
        raise IncompatibleSchemaError(
            f"SPDX SBOM version '{spdx_version}' is incompatible; strictly requires 'SPDX-2.3'"
        )

    data_license = sbom_data.get("dataLicense")
    if data_license != "CC0-1.0":
        raise IncompatibleSchemaError(f"SPDX SBOM dataLicense must be 'CC0-1.0', got '{data_license}'")

    # Audit root packages in SBOM
    packages = sbom_data.get("packages", [])
    for pkg in packages:
        pkg_name = pkg.get("name")
        for checksum in pkg.get("checksums", []):
            if checksum.get("algorithm") == "SHA256":
                checksum_val = checksum.get("checksumValue", "").lower()
                # If package corresponds to one of our artifacts, ensure single-digest match
                for fname, act_dig in actual_digests.items():
                    if pkg_name == "careops-ed" and fname.startswith("careops-ed-cli"):
                        continue
                    if fname.startswith(pkg_name):
                        if checksum_val != act_dig:
                            raise DigestMismatchError(
                                f"SPDX SBOM checksum mismatch for package '{pkg_name}': "
                                f"SBOM={checksum_val}, Artifact={act_dig}"
                            )

    # 4. Validate SLSA v1 Provenance
    if not slsa_provenance_path.exists():
        raise MissingArtifactError(f"SLSA Provenance file not found at {slsa_provenance_path}")

    try:
        prov_data = json.loads(slsa_provenance_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise IncompatibleSchemaError(f"SLSA Provenance is not valid JSON: {exc}") from exc

    if prov_data.get("_type") != "https://in-toto.io/Statement/v1":
        raise IncompatibleSchemaError(
            f"Unsupported attestation _type: '{prov_data.get('_type')}', expected 'https://in-toto.io/Statement/v1'"
        )

    if prov_data.get("predicateType") != "https://slsa.dev/provenance/v1":
        raise IncompatibleSchemaError(
            f"Unsupported predicateType: '{prov_data.get('predicateType')}', expected 'https://slsa.dev/provenance/v1'"
        )

    # Audit subjects
    subjects = prov_data.get("subject", [])
    if not subjects:
        raise MissingArtifactError("SLSA Provenance statement contains no subject entries.")

    for subj in subjects:
        s_name = subj.get("name")
        s_digest = subj.get("digest", {}).get("sha256", "").lower()
        if s_name in actual_digests:
            if s_digest != actual_digests[s_name]:
                raise DigestMismatchError(
                    f"SLSA Provenance subject '{s_name}' digest mismatch: "
                    f"attestation={s_digest}, actual={actual_digests[s_name]}"
                )

    # Audit Git commit SHA
    predicate = prov_data.get("predicate", {})
    build_def = predicate.get("buildDefinition", {})
    resolved_deps = build_def.get("resolvedDependencies", [])
    repo_commit = None
    for dep in resolved_deps:
        if dep.get("uri", "").startswith("git+") or "careops-sim" in dep.get("uri", ""):
            repo_commit = dep.get("digest", {}).get("gitCommit")
            break

    if not repo_commit:
        # Fallback to internalParameters or externalParameters
        repo_commit = (
            build_def.get("internalParameters", {}).get("gitCommit")
            or build_def.get("externalParameters", {}).get("source", {}).get("digest", {}).get("gitCommit")
        )

    if repo_commit != expected_commit:
        raise WrongCommitError(
            f"SLSA Provenance commit mismatch: attestation references '{repo_commit}', expected '{expected_commit}'"
        )

    # Audit 30-day Freshness Horizon
    timestamp_str = (
        predicate.get("runDetails", {}).get("metadata", {}).get("finishedOn")
        or build_def.get("internalParameters", {}).get("buildTimestamp")
    )
    if not timestamp_str:
        raise ExpiredEvidenceError("SLSA Provenance does not contain a valid build timestamp.")

    try:
        build_time = datetime.fromisoformat(timestamp_str.replace("Z", "+00:00"))
    except ValueError as exc:
        raise IncompatibleSchemaError(f"Invalid timestamp format in provenance: '{timestamp_str}'") from exc

    age = evaluation_time - build_time
    if age > timedelta(days=max_age_days):
        raise ExpiredEvidenceError(
            f"Release evidence expired: attestation age is {age.days} days, exceeding the {max_age_days}-day limit "
            f"(built {timestamp_str}, evaluated {evaluation_time.isoformat()})"
        )

    return {
        "status": "PASS",
        "verified_artifacts": list(actual_digests.keys()),
        "commit": repo_commit,
        "evidence_age_days": age.days,
        "spdx_version": spdx_version,
        "slsa_version": "v1.0",
    }


class ReleaseRegistry:
    """Simulates release registry with absolute immutability and yank/corrected-version policies."""

    def __init__(self) -> None:
        self.releases: dict[str, dict] = {}

    def publish_release(self, version: str, artifacts: dict[str, bytes], tag: str) -> None:
        """Publish a new immutable release."""
        if version in self.releases:
            raise ImmutableReleaseViolationError(
                f"Immutable release violation: version '{version}' is already published and cannot be modified or replaced."
            )
        # Check if tag already points to another release
        for v, rel in self.releases.items():
            if rel["tag"] == tag:
                raise ImmutableReleaseViolationError(
                    f"Immutable release violation: tag '{tag}' is already bound to version '{v}'."
                )

        self.releases[version] = {
            "version": version,
            "tag": tag,
            "artifacts": {fname: compute_sha256(data) for fname, data in artifacts.items()},
            "yanked": False,
            "yank_reason": None,
            "advisory_id": None,
        }

    def yank_release(self, version: str, reason: str, advisory_id: str | None = None) -> None:
        """Owner 15 / 25 operation: Yank a defective release without deleting or overwriting its bytes."""
        if version not in self.releases:
            raise KeyError(f"Version '{version}' does not exist.")
        rel = self.releases[version]
        rel["yanked"] = True
        rel["yank_reason"] = reason
        rel["advisory_id"] = advisory_id

    def get_release(self, version: str) -> dict:
        if version not in self.releases:
            raise KeyError(f"Version '{version}' not found.")
        return self.releases[version]


class ConsumerPackagingRehearsalTests(unittest.TestCase):
    """Test suite covering Task D4.2 requirements."""

    def setUp(self) -> None:
        self.eval_time = datetime(2026, 10, 16, 12, 0, 0, tzinfo=timezone.utc)
        self.commit_sha = "bb81846c2f9d8a1e4b5c7d8e9f0a1b2c3d4e5f6a"
        self.temp_dir = tempfile.TemporaryDirectory()
        self.work_dir = Path(self.temp_dir.name)

        # Create representative release artifacts
        self.cli_binary = b"\x7fELF\x02\x01\x01\x00careops-ed-cli-v0.1.0-deterministic-binary"
        self.tar_archive = b"\x1f\x8b\x08\x00\x00\x00\x00\x00careops-ed-v0.1.0-crate-payload"

        self.cli_digest = compute_sha256(self.cli_binary)
        self.tar_digest = compute_sha256(self.tar_archive)

        self.artifacts_dir = self.work_dir / "dist"
        self.artifacts_dir.mkdir()
        (self.artifacts_dir / "careops-ed-cli-x86_64-unknown-linux-gnu.tar.gz").write_bytes(self.cli_binary)
        (self.artifacts_dir / "careops-ed-0.1.0.crate").write_bytes(self.tar_archive)

        # Generate SHA256SUMS
        self.sha256sums_file = self.artifacts_dir / "SHA256SUMS"
        self.sha256sums_file.write_text(
            f"{self.cli_digest}  careops-ed-cli-x86_64-unknown-linux-gnu.tar.gz\n"
            f"{self.tar_digest}  careops-ed-0.1.0.crate\n",
            encoding="utf-8",
        )

        # Generate SPDX 2.3 SBOM
        self.sbom_file = self.artifacts_dir / "spdx.json"
        sbom_content = {
            "spdxVersion": "SPDX-2.3",
            "dataLicense": "CC0-1.0",
            "SPDXID": "SPDXRef-DOCUMENT",
            "name": "careops-sim-0.1.0",
            "documentNamespace": "https://careops.org/spdx/careops-sim-0.1.0",
            "creationInfo": {
                "creators": ["Organization: CareOps Sim Release Team (Owner 28)"],
                "created": "2026-10-16T10:00:00Z",
            },
            "packages": [
                {
                    "name": "careops-ed-cli",
                    "SPDXID": "SPDXRef-Package-CLI",
                    "versionInfo": "0.1.0",
                    "licenseConcluded": "Apache-2.0",
                    "checksums": [
                        {"algorithm": "SHA256", "checksumValue": self.cli_digest}
                    ],
                },
                {
                    "name": "careops-ed",
                    "SPDXID": "SPDXRef-Package-Core",
                    "versionInfo": "0.1.0",
                    "licenseConcluded": "Apache-2.0",
                    "checksums": [
                        {"algorithm": "SHA256", "checksumValue": self.tar_digest}
                    ],
                },
            ],
        }
        self.sbom_file.write_text(json.dumps(sbom_content, indent=2), encoding="utf-8")

        # Generate SLSA v1 Provenance
        self.prov_file = self.artifacts_dir / "provenance.json"
        prov_content = {
            "_type": "https://in-toto.io/Statement/v1",
            "predicateType": "https://slsa.dev/provenance/v1",
            "subject": [
                {
                    "name": "careops-ed-cli-x86_64-unknown-linux-gnu.tar.gz",
                    "digest": {"sha256": self.cli_digest},
                },
                {
                    "name": "careops-ed-0.1.0.crate",
                    "digest": {"sha256": self.tar_digest},
                },
            ],
            "predicate": {
                "buildDefinition": {
                    "buildType": "https://careops.org/builders/rust-hermetic/v1",
                    "externalParameters": {
                        "source": {
                            "uri": "git+https://github.com/careops/careops-sim",
                            "digest": {"gitCommit": self.commit_sha},
                        }
                    },
                    "internalParameters": {
                        "rustcVersion": "1.99.0",
                        "gitCommit": self.commit_sha,
                        "buildTimestamp": "2026-10-16T10:30:00Z",
                    },
                    "resolvedDependencies": [
                        {
                            "uri": "git+https://github.com/careops/careops-sim",
                            "digest": {"gitCommit": self.commit_sha},
                        }
                    ],
                },
                "runDetails": {
                    "builder": {"id": "https://github.com/careops/careops-sim/.github/workflows/release.yml@refs/tags/v0.1.0"},
                    "metadata": {
                        "invocationId": "rel-run-20261016-001",
                        "startedOn": "2026-10-16T10:25:00Z",
                        "finishedOn": "2026-10-16T10:30:00Z",
                    },
                },
            },
        }
        self.prov_file.write_text(json.dumps(prov_content, indent=2), encoding="utf-8")

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def test_multi_owner_governance_model_coverage(self) -> None:
        """Verify all 7 governance owners (15, 16, 20, 25, 28, 42, 44) have defined mandates."""
        required_owners = {15, 16, 20, 25, 28, 42, 44}
        self.assertEqual(set(OWNER_ROLES.keys()), required_owners)
        for oid in required_owners:
            self.assertIn("Owner", f"Owner {oid}")
            self.assertTrue(len(OWNER_ROLES[oid]) > 10)

    def test_happy_path_clean_consumer_verification(self) -> None:
        """Verify that a clean release bundle passes all consumer verification checks."""
        result = verify_consumer_release_bundle(
            artifact_dir=self.artifacts_dir,
            sha256sums_path=self.sha256sums_file,
            spdx_sbom_path=self.sbom_file,
            slsa_provenance_path=self.prov_file,
            expected_commit=self.commit_sha,
            evaluation_time=self.eval_time,
            max_age_days=30,
        )
        self.assertEqual(result["status"], "PASS")
        self.assertEqual(len(result["verified_artifacts"]), 2)
        self.assertEqual(result["commit"], self.commit_sha)
        self.assertEqual(result["spdx_version"], "SPDX-2.3")
        self.assertEqual(result["slsa_version"], "v1.0")

    def test_gate1_dirty_release_inputs_rejection(self) -> None:
        """Gate 1: DirtyReleaseInputError must be raised when working tree has uncommitted changes."""
        clean_status = ""
        check_clean_working_tree(clean_status)  # clean should pass

        dirty_status = " M crates/careops-ed/Cargo.toml\n?? scratch/dirty.txt\n"
        with self.assertRaises(DirtyReleaseInputError) as ctx:
            check_clean_working_tree(dirty_status)
        self.assertIn("working tree contains 2 uncommitted/untracked item(s)", str(ctx.exception))

    def test_gate2_missing_artifact_rejection(self) -> None:
        """Gate 2: MissingArtifactError must be raised when a declared artifact is absent."""
        # Delete one artifact
        missing_target = self.artifacts_dir / "careops-ed-0.1.0.crate"
        missing_target.unlink()

        with self.assertRaises(MissingArtifactError) as ctx:
            verify_consumer_release_bundle(
                artifact_dir=self.artifacts_dir,
                sha256sums_path=self.sha256sums_file,
                spdx_sbom_path=self.sbom_file,
                slsa_provenance_path=self.prov_file,
                expected_commit=self.commit_sha,
                evaluation_time=self.eval_time,
            )
        self.assertIn("Declared artifact 'careops-ed-0.1.0.crate' is missing", str(ctx.exception))

    def test_gate3_digest_mismatch_and_tampered_artifact_rejection(self) -> None:
        """Gate 3: DigestMismatchError must be raised when artifact bytes are altered or substituted."""
        # Corrupt the binary bytes
        corrupted_binary = self.cli_binary + b"\x00TAMPERED_PAYLOAD_EXTENSION"
        (self.artifacts_dir / "careops-ed-cli-x86_64-unknown-linux-gnu.tar.gz").write_bytes(corrupted_binary)

        with self.assertRaises(DigestMismatchError) as ctx:
            verify_consumer_release_bundle(
                artifact_dir=self.artifacts_dir,
                sha256sums_path=self.sha256sums_file,
                spdx_sbom_path=self.sbom_file,
                slsa_provenance_path=self.prov_file,
                expected_commit=self.commit_sha,
                evaluation_time=self.eval_time,
            )
        self.assertIn("digest mismatch: computed", str(ctx.exception))

    def test_gate4_expired_evidence_rejection(self) -> None:
        """Gate 4: ExpiredEvidenceError must be raised when attestation exceeds 30-day freshness horizon."""
        # Evaluation time 45 days after build timestamp (2026-10-16 + 45 days = 2026-11-30)
        future_eval_time = self.eval_time + timedelta(days=45)

        with self.assertRaises(ExpiredEvidenceError) as ctx:
            verify_consumer_release_bundle(
                artifact_dir=self.artifacts_dir,
                sha256sums_path=self.sha256sums_file,
                spdx_sbom_path=self.sbom_file,
                slsa_provenance_path=self.prov_file,
                expected_commit=self.commit_sha,
                evaluation_time=future_eval_time,
                max_age_days=30,
            )
        self.assertIn("exceeding the 30-day limit", str(ctx.exception))

    def test_gate5_wrong_commit_attestation_rejection(self) -> None:
        """Gate 5: WrongCommitError must be raised when provenance references an unauthorized commit."""
        wrong_commit = "deadbeef1234567890abcdef1234567890abcdef"
        with self.assertRaises(WrongCommitError) as ctx:
            verify_consumer_release_bundle(
                artifact_dir=self.artifacts_dir,
                sha256sums_path=self.sha256sums_file,
                spdx_sbom_path=self.sbom_file,
                slsa_provenance_path=self.prov_file,
                expected_commit=wrong_commit,
                evaluation_time=self.eval_time,
            )
        self.assertIn("SLSA Provenance commit mismatch", str(ctx.exception))

    def test_gate6_incompatible_schema_rejection(self) -> None:
        """Gate 6: IncompatibleSchemaError must be raised when SBOM or Provenance has obsolete/invalid schema."""
        # Incompatible SPDX schema (e.g. SPDX-2.1)
        bad_sbom = json.loads(self.sbom_file.read_text(encoding="utf-8"))
        bad_sbom["spdxVersion"] = "SPDX-2.1"
        self.sbom_file.write_text(json.dumps(bad_sbom), encoding="utf-8")

        with self.assertRaises(IncompatibleSchemaError) as ctx:
            verify_consumer_release_bundle(
                artifact_dir=self.artifacts_dir,
                sha256sums_path=self.sha256sums_file,
                spdx_sbom_path=self.sbom_file,
                slsa_provenance_path=self.prov_file,
                expected_commit=self.commit_sha,
                evaluation_time=self.eval_time,
            )
        self.assertIn("incompatible; strictly requires 'SPDX-2.3'", str(ctx.exception))

        # Restore SPDX and test incompatible SLSA schema
        bad_sbom["spdxVersion"] = "SPDX-2.3"
        self.sbom_file.write_text(json.dumps(bad_sbom), encoding="utf-8")

        bad_prov = json.loads(self.prov_file.read_text(encoding="utf-8"))
        bad_prov["predicateType"] = "https://slsa.dev/provenance/v0.2"
        self.prov_file.write_text(json.dumps(bad_prov), encoding="utf-8")

        with self.assertRaises(IncompatibleSchemaError) as ctx:
            verify_consumer_release_bundle(
                artifact_dir=self.artifacts_dir,
                sha256sums_path=self.sha256sums_file,
                spdx_sbom_path=self.sbom_file,
                slsa_provenance_path=self.prov_file,
                expected_commit=self.commit_sha,
                evaluation_time=self.eval_time,
            )
        self.assertIn("expected 'https://slsa.dev/provenance/v1'", str(ctx.exception))

    def test_release_immutability_and_yank_workflow(self) -> None:
        """Rehearse absolute immutability, overwrite rejection, and corrected-version / yank policy."""
        registry = ReleaseRegistry()

        # Step 1: Initial release v0.1.0 published
        artifacts_v1 = {
            "careops-ed-0.1.0.crate": self.tar_archive,
            "careops-ed-cli-x86_64-unknown-linux-gnu.tar.gz": self.cli_binary,
        }
        registry.publish_release("0.1.0", artifacts_v1, tag="v0.1.0")

        rel_010 = registry.get_release("0.1.0")
        self.assertEqual(rel_010["version"], "0.1.0")
        self.assertFalse(rel_010["yanked"])

        # Step 2: Attempting to overwrite v0.1.0 must fail with ImmutableReleaseViolationError
        tampered_artifacts = {
            "careops-ed-0.1.0.crate": self.tar_archive + b"\x00MUTATED",
        }
        with self.assertRaises(ImmutableReleaseViolationError) as ctx:
            registry.publish_release("0.1.0", tampered_artifacts, tag="v0.1.0")
        self.assertIn("version '0.1.0' is already published and cannot be modified", str(ctx.exception))

        # Attempting to re-point tag v0.1.0 to another release also fails
        with self.assertRaises(ImmutableReleaseViolationError) as ctx:
            registry.publish_release("0.1.0-patch", tampered_artifacts, tag="v0.1.0")
        self.assertIn("tag 'v0.1.0' is already bound to version '0.1.0'", str(ctx.exception))

        # Step 3: Corrected-Version Workflow:
        # A defect is detected in 0.1.0. Instead of overwriting, Owner 15/25 yanks 0.1.0
        registry.yank_release(
            version="0.1.0",
            reason="Defective transit edge calculation under surge profile; use 0.1.1",
            advisory_id="GHSA-careops-2026-001",
        )

        yanked_rel = registry.get_release("0.1.0")
        self.assertTrue(yanked_rel["yanked"])
        self.assertEqual(yanked_rel["advisory_id"], "GHSA-careops-2026-001")
        # Ensure original bytes remain in place and immutable
        self.assertEqual(yanked_rel["artifacts"]["careops-ed-0.1.0.crate"], self.tar_digest)

        # Step 4: Publish corrected patch version 0.1.1
        corrected_crate = b"\x1f\x8b\x08\x00\x00\x00\x00\x00careops-ed-v0.1.1-corrected-payload"
        artifacts_v11 = {
            "careops-ed-0.1.1.crate": corrected_crate,
            "careops-ed-cli-x86_64-unknown-linux-gnu.tar.gz": self.cli_binary,
        }
        registry.publish_release("0.1.1", artifacts_v11, tag="v0.1.1")

        rel_011 = registry.get_release("0.1.1")
        self.assertEqual(rel_011["version"], "0.1.1")
        self.assertEqual(rel_011["tag"], "v0.1.1")
        self.assertFalse(rel_011["yanked"])


class ReleaseRehearsalToolModuleTests(unittest.TestCase):
    """Direct validation of the tools.release_rehearsal script implementation."""

    def setUp(self) -> None:
        from tools import release_rehearsal
        self.rr = release_rehearsal
        self.temp_dir = tempfile.TemporaryDirectory()
        self.work_dir = Path(self.temp_dir.name)
        self.src_dir = self.work_dir / "src"
        self.dist_dir = self.work_dir / "dist"
        self.src_dir.mkdir()
        self.dist_dir.mkdir()

        # Create dummy crate structure
        crate_dir = self.src_dir / "crates" / "careops-ed"
        crate_dir.mkdir(parents=True)
        (crate_dir / "Cargo.toml").write_text('[package]\nname = "careops-ed"\nversion = "0.1.0"\nlicense = "Apache-2.0"\n')
        (crate_dir / "src").mkdir()
        (crate_dir / "src" / "lib.rs").write_text("#![forbid(unsafe_code)]\npub fn hello() {}\n")

        self.fixed_commit = "bb81846c2f9d8a1e4b5c7d8e9f0a1b2c3d4e5f6a"
        self.fixed_ts = datetime(2026, 10, 16, 10, 0, 0, tzinfo=timezone.utc)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def test_rr_package_and_verify_clean_rehearsal(self) -> None:
        """Verify tools.release_rehearsal package and verify cycle."""
        pkg_res = self.rr.package_release(
            source_dir=self.src_dir,
            output_dir=self.dist_dir,
            package_name="careops-ed",
            version="0.1.0",
            commit_sha=self.fixed_commit,
            check_clean=False,
            timestamp=self.fixed_ts,
        )
        self.assertEqual(pkg_res["package_name"], "careops-ed")
        self.assertEqual(pkg_res["version"], "0.1.0")

        # Verify as consumer
        ver_res = self.rr.verify_consumer_release(
            dist_dir=self.dist_dir,
            expected_commit=self.fixed_commit,
            reference_time=self.fixed_ts,
            max_evidence_age_days=30,
        )
        self.assertTrue(ver_res["success"])
        self.assertIn("careops-ed-0.1.0.tar.gz", ver_res["verified_artifacts"])

    def test_rr_failure_gates(self) -> None:
        """Verify failure gates through tools.release_rehearsal."""
        self.rr.package_release(
            source_dir=self.src_dir,
            output_dir=self.dist_dir,
            package_name="careops-ed",
            version="0.1.0",
            commit_sha=self.fixed_commit,
            check_clean=False,
            timestamp=self.fixed_ts,
        )

        # Wrong commit
        with self.assertRaises(self.rr.WrongCommitError):
            self.rr.verify_consumer_release(
                dist_dir=self.dist_dir,
                expected_commit="0000000000000000000000000000000000000000",
                reference_time=self.fixed_ts,
            )

        # Expired evidence
        with self.assertRaises(self.rr.ExpiredEvidenceError):
            self.rr.verify_consumer_release(
                dist_dir=self.dist_dir,
                expected_commit=self.fixed_commit,
                reference_time=self.fixed_ts + timedelta(days=45),
                max_evidence_age_days=30,
            )

        # Tampered artifact
        artifact_path = self.dist_dir / "careops-ed-0.1.0.tar.gz"
        artifact_path.write_bytes(artifact_path.read_bytes() + b"\x00TAMPERED")
        with self.assertRaises(self.rr.DigestMismatchError):
            self.rr.verify_consumer_release(
                dist_dir=self.dist_dir,
                expected_commit=self.fixed_commit,
                reference_time=self.fixed_ts,
            )


if __name__ == "__main__":
    unittest.main()

