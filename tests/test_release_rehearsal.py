"""Unit tests for tools/release_rehearsal.py (Task D4.2).

Validates:
- Exception hierarchy
- ReleaseArtifact dataclass
- Check clean git status and dirty release input rejection
- Build and package artifacts from locked source
- SHA256SUMS manifest generation
- SPDX 2.3 SBOM generation and validation
- SLSA v1 Provenance generation and validation
- Consumer release verification and 6 failure gates
- Release immutability, duplicate rejection, and yank/patch workflow
- CLI interface (package, verify, rehearse-yank)
"""

from __future__ import annotations

from datetime import datetime, timedelta, timezone
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from tools.release_rehearsal import (
    ReleaseRehearsalError,
    DirtyReleaseInputError,
    MissingArtifactError,
    DigestMismatchError,
    ExpiredEvidenceError,
    WrongCommitError,
    IncompatibleSchemaError,
    ImmutableReleaseViolationError,
    ReleaseArtifact,
    compute_sha256,
    check_clean_working_tree,
    check_clean_git_status,
    build_and_package_artifacts,
    package_release,
    generate_checksums_manifest,
    generate_spdx_sbom,
    generate_slsa_provenance,
    verify_consumer_release,
    verify_consumer_release_bundle,
    rehearse_immutable_release,
    rehearse_yank_and_patch,
    ReleaseRegistry,
    main as cli_main,
)

ROOT_DIR = Path(__file__).resolve().parents[1]


class ReleaseRehearsalUnitTests(unittest.TestCase):
    """Exhaustive tests for release_rehearsal module components."""

    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.work_dir = Path(self.temp_dir.name)
        self.repo_dir = self.work_dir / "mock_repo"
        self.repo_dir.mkdir()
        self.dist_dir = self.work_dir / "dist"
        self.dist_dir.mkdir()

        # Initialize mock repo structure
        (self.repo_dir / "crates" / "careops-ed" / "src").mkdir(parents=True)
        (self.repo_dir / "crates" / "careops-ed" / "Cargo.toml").write_text(
            '[package]\nname = "careops-ed"\nversion = "0.1.0"\nlicense = "Apache-2.0"\n',
            encoding="utf-8",
        )
        (self.repo_dir / "crates" / "careops-ed" / "src" / "lib.rs").write_text(
            "pub fn simulate() {}\n", encoding="utf-8"
        )

        (self.repo_dir / "crates" / "careops-ed-cli" / "src").mkdir(parents=True)
        (self.repo_dir / "crates" / "careops-ed-cli" / "Cargo.toml").write_text(
            '[package]\nname = "careops-ed-cli"\nversion = "0.1.0"\nlicense = "Apache-2.0"\n',
            encoding="utf-8",
        )
        (self.repo_dir / "crates" / "careops-ed-cli" / "src" / "main.rs").write_text(
            "fn main() {}\n", encoding="utf-8"
        )

        (self.repo_dir / "model-inputs" / "ed" / "schema").mkdir(parents=True)
        (self.repo_dir / "model-inputs" / "ed" / "schema" / "example-profile-pack.schema.json").write_text(
            json.dumps({"$schema": "https://json-schema.org/draft/2020-12/schema", "type": "object"}),
            encoding="utf-8",
        )

        (self.repo_dir / "crates" / "careops-ed" / "examples").mkdir(parents=True)
        (self.repo_dir / "crates" / "careops-ed" / "examples" / "nominal_mvp.json").write_text(
            json.dumps({"profile_id": "nominal_mvp", "seed": 42}),
            encoding="utf-8",
        )

        (self.repo_dir / "LICENSE").write_text(
            "Apache License, Version 2.0\nhttp://www.apache.org/licenses/\n",
            encoding="utf-8",
        )

        self.fixed_commit = "bb81846c2f9d8a1e4b5c7d8e9f0a1b2c3d4e5f6a"
        self.fixed_ts = datetime(2026, 10, 16, 12, 0, 0, tzinfo=timezone.utc)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def test_exception_hierarchy(self) -> None:
        """Verify all custom exceptions inherit from ReleaseRehearsalError."""
        exceptions = [
            DirtyReleaseInputError,
            MissingArtifactError,
            DigestMismatchError,
            ExpiredEvidenceError,
            WrongCommitError,
            IncompatibleSchemaError,
            ImmutableReleaseViolationError,
        ]
        for exc in exceptions:
            self.assertTrue(issubclass(exc, ReleaseRehearsalError))
            self.assertTrue(issubclass(exc, Exception))

    def test_release_artifact_dataclass(self) -> None:
        """Verify ReleaseArtifact attributes and types."""
        fpath = self.work_dir / "test.bin"
        fpath.write_bytes(b"hello world")
        digest = compute_sha256(fpath)
        art = ReleaseArtifact(
            name="test.bin",
            path=fpath,
            sha256=digest,
            size_bytes=11,
            content_type="application/octet-stream",
        )
        self.assertEqual(art.name, "test.bin")
        self.assertEqual(art.path, fpath)
        self.assertEqual(art.sha256, digest)
        self.assertEqual(art.size_bytes, 11)
        self.assertEqual(art.content_type, "application/octet-stream")

    def test_check_clean_working_tree_gate(self) -> None:
        """Verify check_clean_working_tree raises on uncommitted changes."""
        check_clean_working_tree("")  # Clean
        check_clean_working_tree("   \n")  # Whitespace only is clean

        dirty = " M Cargo.toml\n?? scratch/dirty.tmp\n"
        with self.assertRaises(DirtyReleaseInputError) as ctx:
            check_clean_working_tree(dirty)
        self.assertIn("working tree contains 2 uncommitted/untracked item(s)", str(ctx.exception))

    def test_check_clean_git_status_with_mock_git(self) -> None:
        """Verify check_clean_git_status in an actual initialized clean and dirty git repo."""
        # Initialize a real git repo in repo_dir
        subprocess.run(["git", "init"], cwd=self.repo_dir, capture_output=True, check=True)
        subprocess.run(["git", "config", "user.name", "Tester"], cwd=self.repo_dir, check=True)
        subprocess.run(["git", "config", "user.email", "test@test.org"], cwd=self.repo_dir, check=True)
        subprocess.run(["git", "add", "."], cwd=self.repo_dir, check=True)
        subprocess.run(["git", "commit", "-m", "initial"], cwd=self.repo_dir, check=True)

        # 1. Clean repo should return HEAD commit
        commit_sha = check_clean_git_status(self.repo_dir)
        self.assertEqual(len(commit_sha), 40)

        # 2. Add an untracked file -> dirty
        (self.repo_dir / "dirty.txt").write_text("dirty", encoding="utf-8")
        with self.assertRaises(DirtyReleaseInputError):
            check_clean_git_status(self.repo_dir)

    def test_build_and_package_artifacts(self) -> None:
        """Verify build_and_package_artifacts bundles the 6 mandatory items."""
        artifacts = build_and_package_artifacts(
            repo_root=self.repo_dir,
            output_dir=self.dist_dir,
            version="0.1.0",
            allow_dirty=True,
        )
        artifact_names = {a.name for a in artifacts}
        expected_names = {
            "careops-ed-cli",
            "careops-ed-0.1.0.crate",
            "careops-ed-cli-0.1.0.crate",
            "schema_v1.json",
            "example_scenario.json",
            "LICENSE",
        }
        self.assertTrue(expected_names.issubset(artifact_names))

        for a in artifacts:
            self.assertTrue(a.path.exists())
            self.assertEqual(a.sha256, compute_sha256(a.path))
            self.assertEqual(a.size_bytes, a.path.stat().st_size)

    def test_checksums_manifest_generation(self) -> None:
        """Verify SHA256SUMS formatting `{sha256}  {filename}`."""
        artifacts = build_and_package_artifacts(
            repo_root=self.repo_dir,
            output_dir=self.dist_dir,
            version="0.1.0",
            allow_dirty=True,
        )
        sums_path = generate_checksums_manifest(artifacts, self.dist_dir)
        self.assertTrue(sums_path.exists())

        lines = sums_path.read_text(encoding="utf-8").strip().splitlines()
        self.assertEqual(len(lines), len(artifacts))
        for line in lines:
            parts = line.split("  ")
            self.assertEqual(len(parts), 2)
            sha, fname = parts[0], parts[1]
            self.assertEqual(len(sha), 64)
            self.assertTrue((self.dist_dir / fname).exists())

    def test_spdx_sbom_generation(self) -> None:
        """Verify SPDX 2.3 SBOM generation with packages, files, checksums, and Apache-2.0 license."""
        artifacts = build_and_package_artifacts(
            repo_root=self.repo_dir,
            output_dir=self.dist_dir,
            version="0.1.0",
            allow_dirty=True,
        )
        sbom_path = self.dist_dir / "sbom.spdx.json"
        sbom = generate_spdx_sbom(artifacts, self.repo_dir, "0.1.0", sbom_path, timestamp=self.fixed_ts)

        self.assertEqual(sbom["spdxVersion"], "SPDX-2.3")
        self.assertEqual(sbom["dataLicense"], "CC0-1.0")
        self.assertEqual(sbom["SPDXID"], "SPDXRef-DOCUMENT")

        pkg_names = {p["name"] for p in sbom["packages"]}
        expected_pkgs = {"careops-sim", "careops-ed", "careops-ed-cli", "serde", "serde_json", "sha2", "kairo-ecs-core"}
        self.assertTrue(expected_pkgs.issubset(pkg_names))

        file_names = {f["fileName"] for f in sbom["files"]}
        for a in artifacts:
            self.assertIn(f"./{a.name}", file_names)

    def test_slsa_provenance_generation(self) -> None:
        """Verify SLSA v1 in-toto provenance attestation generation."""
        artifacts = build_and_package_artifacts(
            repo_root=self.repo_dir,
            output_dir=self.dist_dir,
            version="0.1.0",
            allow_dirty=True,
        )
        prov_path = self.dist_dir / "provenance.slsa.json"
        prov = generate_slsa_provenance(
            artifacts=artifacts,
            repo_root=self.repo_dir,
            commit_sha=self.fixed_commit,
            version="0.1.0",
            output_path=prov_path,
            validity_days=30,
            timestamp=self.fixed_ts,
        )

        self.assertEqual(prov["_type"], "https://in-toto.io/Statement/v1")
        self.assertEqual(prov["predicateType"], "https://slsa.dev/provenance/v1")

        subj_names = {s["name"] for s in prov["subject"]}
        for a in artifacts:
            self.assertIn(a.name, subj_names)

        predicate = prov["predicate"]
        self.assertIn("builder", predicate)
        self.assertEqual(predicate["buildDefinition"]["source"]["commit_sha"], self.fixed_commit)
        self.assertEqual(predicate["runDetails"]["metadata"]["validityHorizonDays"], 30)

    def test_full_packaging_and_consumer_verification_e2e(self) -> None:
        """E2E test: package release, generate manifests, verify as clean consumer."""
        artifacts = build_and_package_artifacts(
            repo_root=self.repo_dir,
            output_dir=self.dist_dir,
            version="0.1.0",
            allow_dirty=True,
        )
        generate_checksums_manifest(artifacts, self.dist_dir)
        generate_spdx_sbom(artifacts, self.repo_dir, "0.1.0", self.dist_dir / "sbom.spdx.json", timestamp=self.fixed_ts)
        generate_slsa_provenance(
            artifacts,
            self.repo_dir,
            self.fixed_commit,
            "0.1.0",
            self.dist_dir / "provenance.slsa.json",
            validity_days=30,
            timestamp=self.fixed_ts,
        )

        # Consumer verification
        res = verify_consumer_release(
            dist_dir=self.dist_dir,
            expected_commit=self.fixed_commit,
            reference_time_utc=self.fixed_ts,
            max_age_days=30,
        )
        self.assertEqual(res["status"], "PASS")
        self.assertTrue(res["success"])
        self.assertEqual(res["commit"], self.fixed_commit)
        self.assertEqual(len(res["verified_artifacts"]), len(artifacts))

    def test_consumer_verification_failure_gates(self) -> None:
        """Verify all failure gates in verify_consumer_release."""
        artifacts = build_and_package_artifacts(
            repo_root=self.repo_dir,
            output_dir=self.dist_dir,
            version="0.1.0",
            allow_dirty=True,
        )
        generate_checksums_manifest(artifacts, self.dist_dir)
        generate_spdx_sbom(artifacts, self.repo_dir, "0.1.0", self.dist_dir / "sbom.spdx.json", timestamp=self.fixed_ts)
        generate_slsa_provenance(
            artifacts,
            self.repo_dir,
            self.fixed_commit,
            "0.1.0",
            self.dist_dir / "provenance.slsa.json",
            validity_days=30,
            timestamp=self.fixed_ts,
        )

        # 1. Missing artifact
        (self.dist_dir / "LICENSE").unlink()
        with self.assertRaises(MissingArtifactError):
            verify_consumer_release(self.dist_dir, expected_commit=self.fixed_commit, reference_time_utc=self.fixed_ts)
        (self.dist_dir / "LICENSE").write_bytes((self.repo_dir / "LICENSE").read_bytes())

        # 2. Digest mismatch (tampered file)
        target = self.dist_dir / "schema_v1.json"
        orig_bytes = target.read_bytes()
        target.write_bytes(orig_bytes + b"\n/* tampering */")
        with self.assertRaises(DigestMismatchError):
            verify_consumer_release(self.dist_dir, expected_commit=self.fixed_commit, reference_time_utc=self.fixed_ts)
        target.write_bytes(orig_bytes)

        # 3. Wrong commit
        with self.assertRaises(WrongCommitError):
            verify_consumer_release(self.dist_dir, expected_commit="0" * 40, reference_time_utc=self.fixed_ts)

        # 4. Expired evidence
        with self.assertRaises(ExpiredEvidenceError):
            verify_consumer_release(
                self.dist_dir,
                expected_commit=self.fixed_commit,
                reference_time_utc=self.fixed_ts + timedelta(days=31),
                max_age_days=30,
            )

        # 5. Incompatible SPDX schema
        bad_sbom = self.dist_dir / "sbom.spdx.json"
        sbom_obj = json.loads(bad_sbom.read_text(encoding="utf-8"))
        sbom_obj["spdxVersion"] = "SPDX-1.2"
        bad_sbom.write_text(json.dumps(sbom_obj), encoding="utf-8")
        with self.assertRaises(IncompatibleSchemaError):
            verify_consumer_release(self.dist_dir, expected_commit=self.fixed_commit, reference_time_utc=self.fixed_ts)
        sbom_obj["spdxVersion"] = "SPDX-2.3"
        bad_sbom.write_text(json.dumps(sbom_obj), encoding="utf-8")

        # 6. Incompatible SLSA schema
        bad_prov = self.dist_dir / "provenance.slsa.json"
        prov_obj = json.loads(bad_prov.read_text(encoding="utf-8"))
        prov_obj["predicateType"] = "https://slsa.dev/provenance/v0.1"
        bad_prov.write_text(json.dumps(prov_obj), encoding="utf-8")
        with self.assertRaises(IncompatibleSchemaError):
            verify_consumer_release(self.dist_dir, expected_commit=self.fixed_commit, reference_time_utc=self.fixed_ts)

    def test_rehearse_immutable_release_and_yank_and_patch(self) -> None:
        """Verify immutable release records, overwrite prevention, and yank/patch workflow."""
        state: dict = {"releases": {}}
        artifacts_v1 = [
            ReleaseArtifact(
                name="careops-ed-0.1.0.crate",
                path=Path("mock.crate"),
                sha256="a" * 64,
                size_bytes=100,
                content_type="application/gzip",
            )
        ]

        # 1. Publish initial version
        rehearse_immutable_release(state, "0.1.0", artifacts_v1)
        self.assertIn("0.1.0", state["releases"])
        self.assertFalse(state["releases"]["0.1.0"]["yanked"])

        # 2. Overwrite attempt must raise ImmutableReleaseViolationError
        with self.assertRaises(ImmutableReleaseViolationError):
            rehearse_immutable_release(state, "0.1.0", artifacts_v1)

        # 3. Yank and patch
        artifacts_v2 = [
            ReleaseArtifact(
                name="careops-ed-0.1.1.crate",
                path=Path("mock_v2.crate"),
                sha256="b" * 64,
                size_bytes=105,
                content_type="application/gzip",
            )
        ]
        rehearse_yank_and_patch(
            state,
            defective_version="0.1.0",
            patch_version="0.1.1",
            patch_artifacts=artifacts_v2,
            reason="Security defect in 0.1.0",
        )

        # 0.1.0 is yanked but immutable artifact data preserved
        self.assertTrue(state["releases"]["0.1.0"]["yanked"])
        self.assertEqual(state["releases"]["0.1.0"]["yank_reason"], "Security defect in 0.1.0")
        self.assertEqual(state["releases"]["0.1.0"]["artifacts"]["careops-ed-0.1.0.crate"], "a" * 64)

        # 0.1.1 is active
        self.assertIn("0.1.1", state["releases"])
        self.assertFalse(state["releases"]["0.1.1"]["yanked"])

    def test_cli_subcommands(self) -> None:
        """Verify CLI subcommands package, verify, and rehearse-yank."""
        # 1. CLI package
        rc = cli_main([
            "package",
            "--repo-root", str(self.repo_dir),
            "--output-dir", str(self.dist_dir),
            "--version", "0.1.0",
            "--commit", "mock000000000000000000000000000000000000",
            "--allow-dirty",
        ])
        self.assertEqual(rc, 0)
        self.assertTrue((self.dist_dir / "SHA256SUMS").exists())
        self.assertTrue((self.dist_dir / "sbom.spdx.json").exists())
        self.assertTrue((self.dist_dir / "provenance.slsa.json").exists())

        # 2. CLI verify
        rc_ver = cli_main([
            "verify",
            "--dist-dir", str(self.dist_dir),
            "--expected-commit", "mock000000000000000000000000000000000000",
            "--max-age-days", "30",
        ])
        self.assertEqual(rc_ver, 0)

        # 3. CLI rehearse-yank
        rc_yank = cli_main([
            "rehearse-yank",
            "--dist-dir", str(self.dist_dir),
            "--defective-version", "0.1.0",
            "--patch-version", "0.1.1",
        ])
        self.assertEqual(rc_yank, 0)


if __name__ == "__main__":
    unittest.main()
