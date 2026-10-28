# Copyright (c) 2026 CareOps contributors.
# SPDX-License-Identifier: Apache-2.0
# ruff: noqa: PT009, INP001 -- standalone standard-library unittest, no pytest dependency.
"""Exercise privacy gates using fictional content and temporary Git repositories."""

import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

GIT = shutil.which("git")
SH = shutil.which("sh")

SPEC = importlib.util.spec_from_file_location("privacy", Path(__file__).with_name("check.py"))
privacy = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(privacy)


class PrivacyTests(unittest.TestCase):
    """Verify privacy behavior."""

    def test_text_rules(self) -> None:
        """Text rules."""
        cases = {
            "organisation-email": b"fictional@" + b"health.example.gov.au",
            "private-machine-path": b"/" + b"Users/" + b"fictional/private/",
            "patient-identifier": b"MRN=" + b"12345678",
            "private-key": b"-----BEGIN " + b"PRIVATE KEY-----",
        }
        for rule, blob in cases.items():
            with self.subTest(rule=rule):
                self.assertIn(rule, privacy.findings("sample.txt", blob, {}))

    def test_environment(self) -> None:
        """Environment."""
        self.assertIn("environment-file", privacy.findings(".env", b"A=B", {}))
        self.assertEqual([], privacy.findings(".env.example", b"A=placeholder", {}))

    def test_private_directories_cannot_be_inventory_approved(self) -> None:
        """Refuse private exports even if someone lists their paths in an inventory."""
        for path in [
            ".local/snapshot.json",
            "private/export.csv",
            "receipts/run.json",
            ".venv/cache.txt",
        ]:
            with self.subTest(path=path):
                self.assertIn(
                    "private-or-generated-directory", privacy.findings(path, b"fictional", {})
                )

    def test_document_review_is_exact(self) -> None:
        """Document review is exact."""
        blob = b"fictional document"
        approval = {
            "fixture.pdf": {
                "sha256": hashlib.sha256(blob).hexdigest(),
                "reason": "Fictional test artifact inspected",
            }
        }
        self.assertEqual([], privacy.findings("fixture.pdf", blob, approval))
        self.assertIn(
            "document-or-binary-review-required",
            privacy.findings("fixture.pdf", blob + b"changed", approval),
        )
        self.assertIn(
            "document-or-binary-review-required", privacy.findings("image.bin", b"\0", {})
        )

    def test_size(self) -> None:
        """Size."""
        self.assertIn(
            "oversize-file-review-required",
            privacy.findings("large.txt", b"a" * (privacy.MAX_BYTES + 1), {}),
        )

    def test_real_index_inventory_and_partial_staging(self) -> None:
        """Real index inventory and partial staging."""
        with tempfile.TemporaryDirectory() as d:
            old = Path.cwd()
            try:
                os.chdir(d)
                subprocess.run(  # noqa: S603 -- controlled local process arguments, no shell
                    [GIT, "init", "-q"], check=True
                )
                Path(".careops-security").mkdir()
                Path(".careops-security/policy.json").write_text('{"reviewedArtifacts":{}}')
                files = [
                    ".careops-security/policy.json",
                    ".careops-security/inventory.json",
                    "sample.txt",
                ]
                Path(".careops-security/inventory.json").write_text(json.dumps({"files": files}))
                Path("sample.txt").write_text("clean\n")
                subprocess.run(  # noqa: S603 -- controlled local process arguments, no shell
                    [GIT, "add", "."], check=True
                )
                Path("sample.txt").write_bytes(b"MRN=" + b"12345678")
                self.assertEqual([], privacy.check())
                subprocess.run(  # noqa: S603 -- controlled local process arguments, no shell
                    [GIT, "add", "sample.txt"], check=True
                )
                Path("sample.txt").write_text("clean working copy")
                self.assertIn(("patient-identifier", "sample.txt"), privacy.check())
                Path("unexpected.txt").write_text("clean")
                subprocess.run(  # noqa: S603 -- controlled local process arguments, no shell
                    [GIT, "add", "unexpected.txt"], check=True
                )
                self.assertIn(("inventory-mismatch", "unexpected.txt"), privacy.check())
            finally:
                os.chdir(old)


class CommitPrivacyTests(unittest.TestCase):
    """Verify commitprivacy behavior."""

    def setUp(self) -> None:
        """Prepare an isolated fictional repository."""
        self.old = Path.cwd()
        self.temp = tempfile.TemporaryDirectory()
        os.chdir(self.temp.name)
        self.addCleanup(self.temp.cleanup)
        self.addCleanup(os.chdir, self.old)
        self.git("init", "-q")
        Path(".careops-security").mkdir()
        Path(".careops-security/policy.json").write_text('{"reviewedArtifacts":{}}')
        Path(".careops-security/inventory.json").write_text(
            json.dumps(
                {
                    "files": [
                        ".careops-security/policy.json",
                        ".careops-security/inventory.json",
                        "sample.txt",
                    ]
                }
            )
        )
        Path("sample.txt").write_text("clean")
        self.commit()
        self.base = self.git("rev-parse", "HEAD").strip().decode()

    def git(self, *args: str) -> bytes:
        """Git."""
        return subprocess.check_output(  # noqa: S603 -- fixed Git command, no shell
            [
                GIT,
                "-c",
                "user.name=Fictional",
                "-c",
                "user.email=fictional@example.invalid",
                *args,
            ],
            stderr=subprocess.DEVNULL,
        )

    def commit(self) -> None:
        """Commit."""
        self.git("add", ".")
        self.git("commit", "-qm", "fictional test")

    def test_intermediate_exposure(self) -> None:
        """Intermediate exposure."""
        Path("sample.txt").write_bytes(b"MRN=" + b"12345678")
        self.commit()
        Path("sample.txt").write_text("clean")
        self.commit()
        tip = self.git("rev-parse", "HEAD").strip().decode()
        self.assertIn(("patient-identifier", "sample.txt"), privacy.check(self.base, revision=tip))

    def test_outgoing_revision_independent_of_index(self) -> None:
        """Outgoing revision independent of index."""
        Path("sample.txt").write_bytes(b"MRN=" + b"12345678")
        self.commit()
        target = self.git("rev-parse", "HEAD").strip().decode()
        Path("sample.txt").write_text("clean")
        self.git("add", "sample.txt")
        self.assertEqual([], privacy.check())
        self.assertIn(
            ("patient-identifier", "sample.txt"), privacy.check(self.base, revision=target)
        )

    def test_unsupported_mode(self) -> None:
        """Unsupported mode."""
        Path("sample.txt").unlink()
        Path("sample.txt").symlink_to("elsewhere")
        self.git("add", "sample.txt")
        self.assertIn(("unsupported-file-mode", "sample.txt"), privacy.check())

    def test_revoke_unchanged_document_approval(self) -> None:
        """Revoke unchanged document approval."""
        blob = b"fictional"
        Path("sample.txt").write_bytes(b"\0" + blob)
        digest = hashlib.sha256(b"\0" + blob).hexdigest()
        Path(".careops-security/policy.json").write_text(
            json.dumps(
                {
                    "reviewedArtifacts": {
                        "sample.txt": {"sha256": digest, "reason": "fictional fixture"}
                    }
                }
            )
        )
        self.commit()
        Path(".careops-security/policy.json").write_text('{"reviewedArtifacts":{}}')
        self.git("add", ".careops-security/policy.json")
        self.assertIn(("document-or-binary-review-required", "sample.txt"), privacy.check())


class PushSelectionTests(unittest.TestCase):
    """Verify pushselection behavior."""

    def test_explicit_outgoing_objects_are_scanned(self) -> None:
        """Explicit outgoing objects are scanned."""
        hook = Path(__file__).with_name("hooks") / "pre-push"
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            subprocess.run(  # noqa: S603 -- controlled local process arguments, no shell
                [GIT, "init", "-q", d], check=True
            )
            tree = (
                subprocess.check_output(  # noqa: S603 -- fixed Git command, no shell
                    [GIT, "-C", d, "mktree"], input=b""
                )
                .strip()
                .decode()
            )
            command = [
                GIT,
                "-C",
                d,
                "-c",
                "user.name=Fictional",
                "-c",
                "user.email=fictional@example.invalid",
                "commit-tree",
                tree,
            ]
            oid = (
                subprocess.check_output(  # noqa: S603 -- fixed Git command, no shell
                    command, input=b"fictional unreferenced commit\n"
                )
                .strip()
                .decode()
            )
            binpath = root / "bin"
            binpath.mkdir()
            log = root / "calls"
            for name in ["gitleaks", "repo-python"]:
                f = binpath / name
                f.write_text('#!/bin/sh\nprintf "%s\\n" "$@" >> "$CALL_LOG"\n')
                f.chmod(0o755)
            env = dict(
                os.environ,
                PATH=str(binpath) + os.pathsep + os.environ["PATH"],
                REPO_PYTHON=str(binpath / "repo-python"),
                CALL_LOG=str(log),
            )
            for base in ["0" * 40, "1" * 40]:
                log.write_text("")
                r = subprocess.run(  # noqa: S603 -- controlled local process arguments, no shell
                    [SH, str(hook.resolve())],
                    cwd=d,
                    check=False,
                    env=env,
                    input=f"HEAD {oid} refs/heads/test {base}\n",
                    text=True,
                    capture_output=True,
                )
                self.assertEqual(0, r.returncode, r.stderr)
                calls = log.read_text().splitlines()
                self.assertIn(
                    "--log-opts=" + (oid if base == "0" * 40 else base + ".." + oid), calls
                )
                self.assertIn(oid, calls)


if __name__ == "__main__":
    unittest.main()
