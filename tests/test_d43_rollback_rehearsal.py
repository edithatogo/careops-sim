#!/usr/bin/env python3
"""Tests for Rollback Rehearsal, Pin Verification, and Publication Guard (Task D4.3)."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from tools import check_kairos_pin
from tools.rollback_rehearsal import (
    AutonomousPublicationError,
    AutonomousPublicationGuard,
    CargoLockfileRehearsal,
    IncompatiblePinError,
    RollbackRehearsalError,
    RollbackVerificationError,
    SubmodulePinRehearsal,
    TamperedLockfileError,
    UnsupportedFeatureGuard,
    UnsupportedFeatureViolationError,
    generate_release_notes,
    main as cli_main,
)


def git(root: Path, *args: str) -> str:
    """Helper to run git commands in a specific directory."""
    res = subprocess.run(
        ["git", "-C", str(root), *args],
        capture_output=True,
        text=True,
        check=True,
    )
    return res.stdout.strip()


class RollbackRehearsalTests(unittest.TestCase):
    """Test suite covering Task D4.3 rollback rehearsal drills and publication guards."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

        # Set up isolated repository mimicking careops-sim parent and libs/kairos child
        self.child = self.root / "libs/kairos"
        self.child.mkdir(parents=True)

        for repo in (self.root, self.child):
            git(repo, "init")
            git(repo, "config", "user.email", "rehearsal@careops.invalid")
            git(repo, "config", "user.name", "Rehearsal Engineer")

        # Initial reviewed files in child repo
        (self.child / "source").write_text("reviewed engine source\n", encoding="utf-8")
        (self.child / "auxiliary").write_text("reviewed auxiliary source\n", encoding="utf-8")
        git(self.child, "add", "source", "auxiliary")
        git(self.child, "commit", "-m", "initial reviewed child commit")
        self.lkg_sha = git(self.child, "rev-parse", "HEAD")

        # Contract matching check_kairos_pin.py expectations
        (self.root / "conductor/evidence").mkdir(parents=True)
        self.contract = {
            "kairos_commit": self.lkg_sha,
            "owner_ci": {
                "head_sha": self.lkg_sha,
                "conclusion": "success",
                "passed_hosts": [
                    "x86_64-unknown-linux-gnu",
                    "aarch64-apple-darwin",
                ],
            },
            "source_sha256": {
                "source": hashlib.sha256((self.child / "source").read_bytes()).hexdigest(),
                "auxiliary": hashlib.sha256((self.child / "auxiliary").read_bytes()).hexdigest(),
            },
        }
        self.save_contract()

        # State matching check_kairos_pin.py expectations
        (self.root / "conductor/current-state.json").write_text(
            json.dumps({"submodule_pins": {"libs/kairos": self.lkg_sha}}, indent=2) + "\n",
            encoding="utf-8",
        )

        git(self.root, "add", "conductor")
        git(self.root, "update-index", "--add", "--cacheinfo", f"160000,{self.lkg_sha},libs/kairos")
        git(self.root, "commit", "-m", "reviewed initial parent commit")

    def save_contract(self) -> None:
        (self.root / "conductor/evidence/d2.4-kairos-contract.json").write_text(
            json.dumps(self.contract, indent=2) + "\n",
            encoding="utf-8",
        )

    def test_submodule_pin_failure_and_rollback_drill(self) -> None:
        """Complete rehearsal of submodule drift injection and rollback to clean LKG baseline."""
        # 0. Baseline verification passes cleanly
        initial_errors = check_kairos_pin.check(self.root)
        self.assertEqual(initial_errors, [])
        SubmodulePinRehearsal.verify_submodule_pin(self.root)

        # 1. Injects incompatible submodule commit -> asserts pin check fails
        mock_drift_commit = "1" * 40
        drift_receipt = SubmodulePinRehearsal.rehearse_submodule_drift(self.root, mock_drift_commit)
        self.assertTrue(drift_receipt["drift_detected"])
        self.assertIn("committed gitlink differs from reviewed compatibility commit", drift_receipt["detected_errors"])

        with self.assertRaises(IncompatiblePinError):
            SubmodulePinRehearsal.verify_submodule_pin(self.root)

        # 2. Injects uncommitted tracked changes -> asserts pin check fails
        # First reset the gitlink back to lkg_sha
        git(self.root, "update-index", "--cacheinfo", f"160000,{self.lkg_sha},libs/kairos")
        git(self.root, "commit", "-m", "reset index to lkg", "--allow-empty")
        self.assertEqual(check_kairos_pin.check(self.root), [])

        # Invalidate with uncommitted dirty change in tracked source
        (self.child / "source").write_text("uncommitted dirty edit\n", encoding="utf-8")
        dirty_errors = check_kairos_pin.check(self.root)
        self.assertIn("Kairos has uncommitted tracked changes", dirty_errors)

        # 3. Injects unreviewed source file changes -> asserts pin check fails
        # Commit the dirty change in child so working tree is clean, but digest diverges
        git(self.child, "commit", "-am", "unreviewed source file commit")
        new_child_sha = git(self.child, "rev-parse", "HEAD")

        # Update contract commit and gitlink to new sha, but retain old expected source_sha256 digest
        self.contract["kairos_commit"] = new_child_sha
        self.contract["owner_ci"]["head_sha"] = new_child_sha
        self.save_contract()
        (self.root / "conductor/current-state.json").write_text(
            json.dumps({"submodule_pins": {"libs/kairos": new_child_sha}}, indent=2) + "\n",
            encoding="utf-8",
        )
        git(self.root, "add", "conductor")
        git(self.root, "update-index", "--cacheinfo", f"160000,{new_child_sha},libs/kairos")
        git(self.root, "commit", "-m", "point parent to unreviewed child", "--allow-empty")

        divergent_errors = check_kairos_pin.check(self.root)
        self.assertTrue(any("reviewed source drift: source" in err for err in divergent_errors))

        # 4. Rehearses rollback to last-known-good pin -> asserts pin check passes with 0 errors!
        rollback_receipt = SubmodulePinRehearsal.rehearse_submodule_rollback(
            self.root, target_commit=self.lkg_sha
        )
        self.assertTrue(rollback_receipt["verified"])
        self.assertEqual(rollback_receipt["errors"], [])

        # Double check pin tool reports 0 errors directly
        final_errors = check_kairos_pin.check(self.root)
        self.assertEqual(final_errors, [])
        SubmodulePinRehearsal.verify_submodule_pin(self.root)

    def test_cargo_lockfile_tamper_and_rollback_drill(self) -> None:
        """Simulates tampering critical Cargo.lock pins and verifies clean rollback."""
        repo_cargo_lock = Path(__file__).resolve().parents[1] / "Cargo.lock"
        self.assertTrue(repo_cargo_lock.exists(), "Root Cargo.lock must exist")

        # 1. Verifies actual Cargo.lock against pinned specifications
        clean_errors = CargoLockfileRehearsal.inspect_cargo_lock_pins(repo_cargo_lock)
        self.assertEqual(clean_errors, [], f"Repository Cargo.lock must be clean: {clean_errors}")
        CargoLockfileRehearsal.validate_cargo_lock_pins(repo_cargo_lock)

        # Create isolated temporary Cargo.lock copy for drill
        temp_lock = self.root / "drill_Cargo.lock"
        original_content = repo_cargo_lock.read_text(encoding="utf-8")
        temp_lock.write_text(original_content, encoding="utf-8")

        # 2. Simulates tampering a critical dependency in Cargo.lock -> asserts TamperedLockfileError
        tamper_receipt = CargoLockfileRehearsal.rehearse_lockfile_tamper(
            temp_lock, package_name="serde", bad_version="99.99.99"
        )
        self.assertTrue(tamper_receipt["tamper_detected"])
        self.assertEqual(tamper_receipt["tampered_package"], "serde")

        with self.assertRaises(TamperedLockfileError) as ctx:
            CargoLockfileRehearsal.validate_cargo_lock_pins(temp_lock)
        self.assertIn("serde", str(ctx.exception))

        # Also test with raise_on_tamper=True
        with self.assertRaises(TamperedLockfileError):
            CargoLockfileRehearsal.rehearse_lockfile_tamper(
                temp_lock, package_name="sha2", bad_version="0.0.0", raise_on_tamper=True
            )

        # 3. Rehearses rollback to clean baseline -> asserts clean pass
        rollback_receipt = CargoLockfileRehearsal.rehearse_lockfile_rollback(
            temp_lock, backup_content=original_content
        )
        self.assertTrue(rollback_receipt["verified"])
        self.assertEqual(rollback_receipt["errors"], [])

        # Verification of restored baseline has zero errors
        restored_errors = CargoLockfileRehearsal.inspect_cargo_lock_pins(temp_lock)
        self.assertEqual(restored_errors, [])
        CargoLockfileRehearsal.validate_cargo_lock_pins(temp_lock)

    def test_autonomous_publication_prevention(self) -> None:
        """Guards against autonomous publication commands unless explicit human override is present."""
        env_key = AutonomousPublicationGuard.AUTH_ENV_VAR

        # Ensure environment authority is absent
        original_val = os.environ.pop(env_key, None)
        try:
            # 1. Tests that attempting publication commands without authority strictly raises AutonomousPublicationError
            unauthorized_cases = [
                ("cargo publish", []),
                ("git", ["push", "--tags"]),
                ("git push --tags", []),
                ("gh release create", ["v0.1.0", "--notes", "autonomous attempt"]),
            ]
            for action, flags in unauthorized_cases:
                with self.subTest(action=action, flags=flags):
                    with self.assertRaises(AutonomousPublicationError):
                        AutonomousPublicationGuard.prevent_autonomous_publication(action, flags)

            # Non-publishing commands are permitted even without authority
            non_pub_receipt = AutonomousPublicationGuard.prevent_autonomous_publication(
                "cargo test", ["--workspace"]
            )
            self.assertTrue(non_pub_receipt["authorized"])
            self.assertFalse(non_pub_receipt["is_publication"])

            # 2. Tests that when explicit human maintainer authority is present, dry-run check succeeds
            os.environ[env_key] = "1"
            authorized_receipt = AutonomousPublicationGuard.prevent_autonomous_publication(
                "cargo publish", ["--dry-run"]
            )
            self.assertTrue(authorized_receipt["authorized"])
            self.assertTrue(authorized_receipt["is_publication"])
            self.assertEqual(authorized_receipt["authority"], f"{env_key}=1")

        finally:
            if original_val is not None:
                os.environ[env_key] = original_val
            else:
                os.environ.pop(env_key, None)

    def test_unsupported_features_and_release_notes(self) -> None:
        """Verifies explicit unsupported feature list rejection and release notes generation."""
        unsupported = UnsupportedFeatureGuard.list_unsupported_features()
        self.assertIn("autonomous_publication", unsupported)
        self.assertIn("metal_gpu_queue_execution", unsupported)
        self.assertIn("unpinned_cargo_dependencies", unsupported)

        # Validates that invoking an unsupported feature raises UnsupportedFeatureViolationError
        for feat in ("autonomous_publication", "metal-gpu-queue-execution", "unreviewed-live-telemetry"):
            with self.subTest(feature=feat):
                with self.assertRaises(UnsupportedFeatureViolationError):
                    UnsupportedFeatureGuard.check_feature(feat)

        # Release notes generation includes all required components
        notes = generate_release_notes(version="0.1.0")
        self.assertIn("CareOps Sim Release Notes v0.1.0", notes)
        self.assertIn("x86_64-unknown-linux-gnu", notes)
        self.assertIn("aarch64-apple-darwin", notes)
        self.assertIn("autonomous_publication", notes)

    def test_cli_runner_submodule_and_lockfile(self) -> None:
        """Tests CLI invocation of tools/rollback_rehearsal.py via subprocess."""
        tool_path = Path(__file__).resolve().parents[1] / "tools/rollback_rehearsal.py"
        python_bin = sys.executable

        # 1. Rehearse lockfile CLI drill on a temporary lockfile
        temp_lock = self.root / "cli_drill_Cargo.lock"
        repo_cargo_lock = Path(__file__).resolve().parents[1] / "Cargo.lock"
        shutil.copyfile(repo_cargo_lock, temp_lock)

        res_lock = subprocess.run(
            [
                python_bin,
                str(tool_path),
                "rehearse-lockfile",
                "--lockfile",
                str(temp_lock),
                "--package",
                "serde",
                "--bad-version",
                "99.0.0",
            ],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res_lock.returncode, 0, f"CLI rehearse-lockfile failed: {res_lock.stderr}")
        data_lock = json.loads(res_lock.stdout)
        self.assertTrue(data_lock["tamper"]["tamper_detected"])
        self.assertTrue(data_lock["rollback"]["verified"])

        # 2. Guard publication CLI test without authority -> exit code 1
        env_no_auth = os.environ.copy()
        env_no_auth.pop(AutonomousPublicationGuard.AUTH_ENV_VAR, None)
        res_guard_fail = subprocess.run(
            [
                python_bin,
                str(tool_path),
                "guard-publication",
                "--action",
                "cargo publish",
            ],
            capture_output=True,
            text=True,
            env=env_no_auth,
        )
        self.assertNotEqual(res_guard_fail.returncode, 0)
        self.assertIn("AutonomousPublicationError", res_guard_fail.stderr)

        # 3. Guard publication CLI test with authority -> exit code 0
        env_with_auth = os.environ.copy()
        env_with_auth[AutonomousPublicationGuard.AUTH_ENV_VAR] = "1"
        res_guard_ok = subprocess.run(
            [
                python_bin,
                str(tool_path),
                "guard-publication",
                "--action",
                "cargo publish",
            ],
            capture_output=True,
            text=True,
            env=env_with_auth,
        )
        self.assertEqual(res_guard_ok.returncode, 0, f"Authorized publication check failed: {res_guard_ok.stderr}")
        data_guard = json.loads(res_guard_ok.stdout)
        self.assertTrue(data_guard["authorized"])

        # 4. Rehearse submodule CLI drill on self.root repo
        res_sub = subprocess.run(
            [
                python_bin,
                str(tool_path),
                "rehearse-submodule",
                "--repo-root",
                str(self.root),
                "--target-commit",
                self.lkg_sha,
                "--mock-commit",
                "2" * 40,
            ],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res_sub.returncode, 0, f"CLI rehearse-submodule failed: {res_sub.stderr}")
        data_sub = json.loads(res_sub.stdout)
        self.assertEqual(len(data_sub["rehearsals"]), 2)
        self.assertTrue(data_sub["rehearsals"][0]["drift_detected"])
        self.assertTrue(data_sub["rehearsals"][1]["verified"])


if __name__ == "__main__":
    unittest.main()
