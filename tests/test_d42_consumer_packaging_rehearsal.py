"""Test suite for Task D4.2: Clean-consumer packaging, license notices, SBOM, single-digest binding, provenance, and immutable release rehearsal.

Validates:
1. test_clean_consumer_package_and_license_notices
2. test_build_once_single_digest_binding
3. test_rejection_of_dirty_release_inputs
4. test_rejection_of_missing_artifacts
5. test_rejection_of_substituted_corrupted_artifacts
6. test_rejection_of_expired_evidence
7. test_rejection_of_wrong_commit_evidence
8. test_rejection_of_incompatible_schema
9. test_immutable_release_and_yank_policy_rehearsal
10. test_cli_runner_package_and_verify
"""

from __future__ import annotations

from datetime import datetime, timedelta, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import unittest
from unittest import mock

from tools.release_rehearsal import (
    APACHE_2_0_NOTICE,
    CAREOPS_NOTICE,
    UPSTREAM_OWNERS,
    DigestMismatchError,
    DirtyReleaseInputError,
    ExpiredEvidenceError,
    ImmutableReleaseViolationError,
    IncompatibleSchemaError,
    MissingArtifactError,
    ReleaseRegistry,
    ReleaseRehearsalError,
    WrongCommitError,
    check_clean_git,
    compute_sha256,
    get_package_license,
    package_release,
    verify_consumer_release,
)

ROOT_DIR = Path(__file__).resolve().parents[1]


class TestD42ConsumerPackagingRehearsal(unittest.TestCase):
    """Test suite for Task D4.2 packaging, verification, and immutable release rehearsal."""

    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.work_path = Path(self.temp_dir.name)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def test_clean_consumer_package_and_license_notices(self) -> None:
        """Validate Cargo.toml license declarations and clean packaging with license notices."""
        # 1. Validate root workspace Cargo.toml
        ws_cargo = ROOT_DIR / "Cargo.toml"
        self.assertTrue(ws_cargo.is_file(), "Workspace Cargo.toml must exist")
        with open(ws_cargo, "rb") as f:
            ws_toml = tomllib.load(f)
        ws_license = ws_toml.get("workspace", {}).get("package", {}).get("license")
        self.assertEqual(
            ws_license,
            "Apache-2.0",
            f"Workspace Cargo.toml package license must be Apache-2.0, found: {ws_license}",
        )

        # 2. Validate crates/careops-ed Cargo.toml license declaration (inherited from workspace)
        ed_crate_dir = ROOT_DIR / "crates/careops-ed"
        self.assertTrue(ed_crate_dir.is_dir(), "crates/careops-ed must exist")
        ed_license = get_package_license(ed_crate_dir, workspace_root=ROOT_DIR)
        self.assertEqual(
            ed_license,
            "Apache-2.0",
            f"crates/careops-ed license declaration must resolve to Apache-2.0, found: {ed_license}",
        )

        # 3. Package clean artifacts into distribution directory
        dist_dir = self.work_path / "dist"
        result = package_release(
            source_dir=ROOT_DIR,
            output_dir=dist_dir,
            package_name="careops-ed",
            version="0.1.0",
            check_clean=False,
        )
        self.assertTrue(result["success"])

        # Check standalone license notice files in dist
        license_path = dist_dir / "LICENSE-APACHE"
        notice_path = dist_dir / "NOTICE"
        self.assertTrue(license_path.is_file(), "LICENSE-APACHE must exist in dist directory")
        self.assertTrue(notice_path.is_file(), "NOTICE must exist in dist directory")

        license_content = license_path.read_text(encoding="utf-8")
        notice_content = notice_path.read_text(encoding="utf-8")
        self.assertIn("Apache License", license_content)
        self.assertIn("CareOps Sim", notice_content)

        # Check license notices bundled inside the package tarball
        tarball_path = dist_dir / "careops-ed-0.1.0.tar.gz"
        self.assertTrue(tarball_path.is_file(), "Package tarball must exist")
        with tarfile.open(tarball_path, "r:gz") as tar:
            members = tar.getnames()
            self.assertIn("careops-ed-0.1.0/LICENSE-APACHE", members)
            self.assertIn("careops-ed-0.1.0/NOTICE", members)
            self.assertIn("careops-ed-0.1.0/Cargo.toml", members)

    def test_build_once_single_digest_binding(self) -> None:
        """Build package once into dist dir, bind exact single digest across checksums, SBOM, and SLSA, and verify."""
        dist_dir = self.work_path / "dist_bind"
        fixed_commit = "abcdef0123456789abcdef0123456789abcdef01"
        now_utc = datetime.now(timezone.utc)

        result = package_release(
            source_dir=ROOT_DIR,
            output_dir=dist_dir,
            package_name="careops-ed",
            version="0.1.0",
            commit_sha=fixed_commit,
            check_clean=False,
            timestamp=now_utc,
        )
        self.assertTrue(result["success"])

        artifact_file = dist_dir / "careops-ed-0.1.0.tar.gz"
        self.assertTrue(artifact_file.is_file())
        disk_digest = compute_sha256(artifact_file)

        # 1. Verify SHA256SUMS file
        sha256sums_file = dist_dir / "SHA256SUMS"
        self.assertTrue(sha256sums_file.is_file())
        sums_text = sha256sums_file.read_text(encoding="utf-8").strip()
        sums_digest, sums_name = sums_text.split()
        self.assertEqual(sums_name, "careops-ed-0.1.0.tar.gz")
        self.assertEqual(sums_digest, disk_digest)

        # 2. Verify SPDX 2.3 SBOM package verification digest
        sbom_file = dist_dir / "sbom.spdx.json"
        self.assertTrue(sbom_file.is_file())
        with open(sbom_file, "r", encoding="utf-8") as f:
            sbom_data = json.load(f)
        self.assertEqual(sbom_data["spdxVersion"], "SPDX-2.3")
        self.assertEqual(len(sbom_data["packages"]), 1)
        spdx_pkg = sbom_data["packages"][0]
        spdx_checksum = spdx_pkg["checksums"][0]["checksumValue"]
        spdx_pvc = spdx_pkg["packageVerificationCode"]["packageVerificationCodeValue"]
        self.assertEqual(spdx_checksum, disk_digest)
        self.assertEqual(spdx_pvc, disk_digest)

        # 3. Verify SLSA Provenance subject digest
        provenance_file = dist_dir / "provenance.slsa.json"
        self.assertTrue(provenance_file.is_file())
        with open(provenance_file, "r", encoding="utf-8") as f:
            slsa_data = json.load(f)
        self.assertEqual(slsa_data["predicateType"], "https://slsa.dev/provenance/v0.2")
        self.assertEqual(len(slsa_data["subject"]), 1)
        slsa_subject = slsa_data["subject"][0]
        self.assertEqual(slsa_subject["name"], "careops-ed-0.1.0.tar.gz")
        slsa_digest = slsa_subject["digest"]["sha256"]
        self.assertEqual(slsa_digest, disk_digest)

        # Asserts EXACT same SHA-256 digest is bound across all four representations
        self.assertEqual(disk_digest, sums_digest)
        self.assertEqual(disk_digest, spdx_checksum)
        self.assertEqual(disk_digest, slsa_digest)

        # 4. Consumer verification succeeds
        verify_result = verify_consumer_release(
            dist_dir=dist_dir,
            expected_commit=fixed_commit,
            reference_time=now_utc,
        )
        self.assertTrue(verify_result["success"])
        self.assertEqual(verify_result["verified_artifacts"], ["careops-ed-0.1.0.tar.gz"])
        self.assertEqual(verify_result["commit_sha"], fixed_commit)
        self.assertEqual(verify_result["upstream_owners"], sorted(UPSTREAM_OWNERS))

    def test_rejection_of_dirty_release_inputs(self) -> None:
        """Assert DirtyReleaseInputError is raised when working tree has uncommitted/dirty changes."""
        dist_dir = self.work_path / "dist_dirty"

        # Mock git status to return dirty status output
        mock_dirty_output = subprocess.CompletedProcess(
            args=["git", "status", "--porcelain"],
            returncode=0,
            stdout=" M Cargo.toml\n?? untracked_script.py\n",
            stderr="",
        )

        with mock.patch("subprocess.run", return_value=mock_dirty_output):
            with self.assertRaises(DirtyReleaseInputError) as ctx:
                package_release(
                    source_dir=ROOT_DIR,
                    output_dir=dist_dir,
                    check_clean=True,
                )
            self.assertIn("Working tree is dirty", str(ctx.exception))

            with self.assertRaises(DirtyReleaseInputError):
                check_clean_git(ROOT_DIR)

    def test_rejection_of_missing_artifacts(self) -> None:
        """Assert verify_consumer_release raises MissingArtifactError if any artifact or metadata is missing."""
        dist_dir = self.work_path / "dist_missing"
        package_release(
            source_dir=ROOT_DIR,
            output_dir=dist_dir,
            package_name="careops-ed",
            version="0.1.0",
            check_clean=False,
        )

        # Case 1: Missing artifact file listed in SHA256SUMS
        artifact_file = dist_dir / "careops-ed-0.1.0.tar.gz"
        self.assertTrue(artifact_file.is_file())
        artifact_file.unlink()

        with self.assertRaises(MissingArtifactError) as ctx:
            verify_consumer_release(dist_dir)
        self.assertIn("missing on disk", str(ctx.exception).lower())

        # Case 2: Missing evidence file (SHA256SUMS)
        sha256sums_file = dist_dir / "SHA256SUMS"
        sha256sums_file.unlink()

        with self.assertRaises(MissingArtifactError) as ctx:
            verify_consumer_release(dist_dir)
        self.assertIn("missing required release evidence file", str(ctx.exception).lower())

    def test_rejection_of_substituted_corrupted_artifacts(self) -> None:
        """Assert verify_consumer_release raises DigestMismatchError when an artifact is modified by 1 byte."""
        dist_dir = self.work_path / "dist_corrupted"
        package_release(
            source_dir=ROOT_DIR,
            output_dir=dist_dir,
            package_name="careops-ed",
            version="0.1.0",
            check_clean=False,
        )

        artifact_file = dist_dir / "careops-ed-0.1.0.tar.gz"
        original_bytes = bytearray(artifact_file.read_bytes())
        # Mutate 1 byte
        original_bytes[0] ^= 0xFF
        artifact_file.write_bytes(original_bytes)

        with self.assertRaises(DigestMismatchError) as ctx:
            verify_consumer_release(dist_dir)
        self.assertIn("Checksum mismatch", str(ctx.exception))

    def test_rejection_of_expired_evidence(self) -> None:
        """Assert verify_consumer_release raises ExpiredEvidenceError when evidence is > 30 days old."""
        dist_dir = self.work_path / "dist_expired"
        build_time = datetime(2026, 1, 1, 12, 0, 0, tzinfo=timezone.utc)

        package_release(
            source_dir=ROOT_DIR,
            output_dir=dist_dir,
            package_name="careops-ed",
            version="0.1.0",
            check_clean=False,
            timestamp=build_time,
        )

        # Verification at 35 days later (> 30 days)
        verify_time = build_time + timedelta(days=35)
        with self.assertRaises(ExpiredEvidenceError) as ctx:
            verify_consumer_release(
                dist_dir=dist_dir,
                reference_time=verify_time,
                max_evidence_age_days=30,
            )
        self.assertIn("Release evidence is expired", str(ctx.exception))

        # Verification within 30 days (e.g. 10 days) succeeds
        valid_time = build_time + timedelta(days=10)
        verify_res = verify_consumer_release(
            dist_dir=dist_dir,
            reference_time=valid_time,
            max_evidence_age_days=30,
        )
        self.assertTrue(verify_res["success"])

    def test_rejection_of_wrong_commit_evidence(self) -> None:
        """Assert verify_consumer_release raises WrongCommitError when commit does not match expected."""
        dist_dir = self.work_path / "dist_commit"
        actual_commit = "1111111111111111111111111111111111111111"

        package_release(
            source_dir=ROOT_DIR,
            output_dir=dist_dir,
            package_name="careops-ed",
            version="0.1.0",
            commit_sha=actual_commit,
            check_clean=False,
        )

        wrong_commit = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        with self.assertRaises(WrongCommitError) as ctx:
            verify_consumer_release(
                dist_dir=dist_dir,
                expected_commit=wrong_commit,
            )
        self.assertIn("Release provenance commit mismatch", str(ctx.exception))
        self.assertIn(wrong_commit, str(ctx.exception))

    def test_rejection_of_incompatible_schema(self) -> None:
        """Assert verify_consumer_release raises IncompatibleSchemaError for invalid schema versions."""
        dist_dir = self.work_path / "dist_schema"
        package_release(
            source_dir=ROOT_DIR,
            output_dir=dist_dir,
            package_name="careops-ed",
            version="0.1.0",
            check_clean=False,
        )

        # 1. Mutate SPDX version to unsupported SPDX-99.0
        sbom_path = dist_dir / "sbom.spdx.json"
        with open(sbom_path, "r", encoding="utf-8") as f:
            sbom_data = json.load(f)
        original_spdx = sbom_data["spdxVersion"]
        sbom_data["spdxVersion"] = "SPDX-99.0"
        with open(sbom_path, "w", encoding="utf-8") as f:
            json.dump(sbom_data, f, indent=2)

        with self.assertRaises(IncompatibleSchemaError) as ctx:
            verify_consumer_release(dist_dir)
        self.assertIn("SPDX-99.0", str(ctx.exception))

        # Restore SPDX version
        sbom_data["spdxVersion"] = original_spdx
        with open(sbom_path, "w", encoding="utf-8") as f:
            json.dump(sbom_data, f, indent=2)

        # 2. Mutate SLSA predicateType to invalid URI
        provenance_path = dist_dir / "provenance.slsa.json"
        with open(provenance_path, "r", encoding="utf-8") as f:
            slsa_data = json.load(f)
        slsa_data["predicateType"] = "https://example.com/unsupported/slsa/v99"
        with open(provenance_path, "w", encoding="utf-8") as f:
            json.dump(slsa_data, f, indent=2)

        with self.assertRaises(IncompatibleSchemaError) as ctx:
            verify_consumer_release(dist_dir)
        self.assertIn("https://example.com/unsupported/slsa/v99", str(ctx.exception))

    def test_immutable_release_and_yank_policy_rehearsal(self) -> None:
        """Assert published releases are immutable, yanking preserves artifacts, and corrected patches succeed."""
        registry_dir = self.work_path / "registry"
        registry = ReleaseRegistry(registry_dir=registry_dir)

        artifact_v1 = self.work_path / "careops-ed-0.1.0.tar.gz"
        artifact_v1.write_bytes(b"v0.1.0-release-bytes")

        # 1. Model publication of 0.1.0
        entry_v1 = registry.publish(
            version="0.1.0",
            artifact_path=artifact_v1,
            metadata={"description": "Initial ED preview"},
        )
        self.assertEqual(entry_v1["version"], "0.1.0")
        self.assertFalse(entry_v1["yanked"])
        self.assertIsNone(entry_v1["yank_reason"])

        # 2. Attempt to overwrite 0.1.0 with different artifacts -> ImmutableReleaseViolationError
        artifact_v1_replacement = self.work_path / "careops-ed-0.1.0-replacement.tar.gz"
        artifact_v1_replacement.write_bytes(b"v0.1.0-overwritten-bytes")

        with self.assertRaises(ImmutableReleaseViolationError) as ctx:
            registry.publish(
                version="0.1.0",
                artifact_path=artifact_v1_replacement,
            )
        self.assertIn("already published and immutable", str(ctx.exception))

        # 3. Rehearse yanking 0.1.0 with advisory reason
        advisory = "Security advisory: non-validated capacity parameter overflow in 0.1.0; please use 0.1.1"
        yanked_entry = registry.yank(version="0.1.0", reason=advisory)
        self.assertTrue(yanked_entry["yanked"])
        self.assertEqual(yanked_entry["yank_reason"], advisory)

        # 4. Release corrected patch 0.1.1
        artifact_v2 = self.work_path / "careops-ed-0.1.1.tar.gz"
        artifact_v2.write_bytes(b"v0.1.1-corrected-bytes")
        entry_v2 = registry.publish(
            version="0.1.1",
            artifact_path=artifact_v2,
            metadata={"description": "Corrected capacity handling"},
        )
        self.assertEqual(entry_v2["version"], "0.1.1")
        self.assertFalse(entry_v2["yanked"])

        # 5. Assert 0.1.0 remains immutably in registry marked yanked: True, and 0.1.1 is active
        stored_v1 = registry.get_version("0.1.0")
        stored_v2 = registry.get_version("0.1.1")
        self.assertTrue(stored_v1["yanked"])
        self.assertEqual(stored_v1["yank_reason"], advisory)
        self.assertFalse(stored_v2["yanked"])

        all_versions = registry.list_versions()
        self.assertEqual(len(all_versions), 2)
        self.assertEqual([v["version"] for v in all_versions], ["0.1.0", "0.1.1"])

        # 6. Verify persistence across registry reload
        reloaded_registry = ReleaseRegistry(registry_dir=registry_dir)
        self.assertTrue(reloaded_registry.get_version("0.1.0")["yanked"])
        self.assertFalse(reloaded_registry.get_version("0.1.1")["yanked"])

    def test_cli_runner_package_and_verify(self) -> None:
        """Test running tools/release_rehearsal.py via CLI subprocess for package and verify subcommands."""
        dist_dir = self.work_path / "dist_cli"

        # 1. Run package via CLI
        pkg_cmd = [
            sys.executable,
            str(ROOT_DIR / "tools/release_rehearsal.py"),
            "package",
            "--source",
            str(ROOT_DIR),
            "--output",
            str(dist_dir),
            "--name",
            "careops-ed",
            "--version",
            "0.1.0",
            "--allow-dirty",
        ]
        pkg_run = subprocess.run(pkg_cmd, capture_output=True, text=True, check=False)
        self.assertEqual(
            pkg_run.returncode,
            0,
            f"CLI package failed (code {pkg_run.returncode}):\nSTDOUT: {pkg_run.stdout}\nSTDERR: {pkg_run.stderr}",
        )
        self.assertTrue((dist_dir / "SHA256SUMS").is_file())
        self.assertTrue((dist_dir / "sbom.spdx.json").is_file())
        self.assertTrue((dist_dir / "provenance.slsa.json").is_file())

        # 2. Run verify via CLI
        verify_cmd = [
            sys.executable,
            str(ROOT_DIR / "tools/release_rehearsal.py"),
            "verify",
            "--dist",
            str(dist_dir),
        ]
        verify_run = subprocess.run(verify_cmd, capture_output=True, text=True, check=False)
        self.assertEqual(
            verify_run.returncode,
            0,
            f"CLI verify failed (code {verify_run.returncode}):\nSTDOUT: {verify_run.stdout}\nSTDERR: {verify_run.stderr}",
        )
        verify_data = json.loads(verify_run.stdout)
        self.assertTrue(verify_data.get("success"))

        # 3. Test CLI failure when artifact is corrupted
        artifact_file = dist_dir / "careops-ed-0.1.0.tar.gz"
        corrupted_bytes = bytearray(artifact_file.read_bytes())
        corrupted_bytes[0] ^= 0xFF
        artifact_file.write_bytes(corrupted_bytes)

        corrupt_verify_run = subprocess.run(verify_cmd, capture_output=True, text=True, check=False)
        self.assertNotEqual(corrupt_verify_run.returncode, 0)
        self.assertIn("Verification failed", corrupt_verify_run.stderr)


if __name__ == "__main__":
    unittest.main()
