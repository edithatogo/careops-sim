import hashlib
import io
import json
import os
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
from unittest import mock


ROOT = Path(__file__).parents[1]
sys.path.insert(0, str(ROOT / "tools"))

import install_nextest
import nextest_ci


def make_archive(entries):
    """Build a small deterministic gzip tar for installer boundary tests."""
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w:gz") as archive:
        for name, kind, payload in entries:
            info = tarfile.TarInfo(name)
            if kind == "file":
                info.size = len(payload)
                archive.addfile(info, io.BytesIO(payload))
            elif kind == "symlink":
                info.type = tarfile.SYMTYPE
                info.linkname = "cargo-nextest"
                archive.addfile(info)
            elif kind == "hardlink":
                info.type = tarfile.LNKTYPE
                info.linkname = "cargo-nextest"
                archive.addfile(info)
            elif kind == "fifo":
                info.type = tarfile.FIFOTYPE
                archive.addfile(info)
            else:
                raise AssertionError(kind)
    return output.getvalue()


def fake_binary(version="0.9.146"):
    return f"#!/bin/sh\nprintf 'cargo-nextest {version} (fixture)\\n'\n".encode()


class NextestInstallerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        (ROOT / ".artifacts" / "ci").mkdir(parents=True, exist_ok=True)

    def test_supported_archives_are_exact_and_unknown_hosts_fail(self):
        self.assertEqual(install_nextest.release_for_host("Darwin", "arm64"), "universal-apple-darwin")
        self.assertEqual(install_nextest.release_for_host("Linux", "x86_64"), "x86_64-unknown-linux-gnu")
        for system, machine in (("Darwin", "x86_64"), ("Linux", "aarch64"), ("Windows", "AMD64")):
            with self.subTest(system=system, machine=machine), self.assertRaises(install_nextest.InstallError):
                install_nextest.release_for_host(system, machine)

    def test_valid_archive_installs_one_verified_version_atomically(self):
        archive = make_archive([("cargo-nextest-0.9.146/bin/cargo-nextest", "file", fake_binary())])
        digest = hashlib.sha256(archive).hexdigest()
        with tempfile.TemporaryDirectory(dir=ROOT / ".artifacts/ci") as temp:
            with mock.patch.dict(install_nextest.ARCHIVES, {
                "universal-apple-darwin": {"url": "fixture://archive", "sha256": digest}
            }):
                result = install_nextest.install_nextest(
                    Path(temp) / "tools", "Darwin", "arm64", downloader=lambda _url: archive
                )
            binary = Path(result["path"])
            self.assertEqual(result["version"], "cargo-nextest 0.9.146 (fixture)")
            self.assertEqual(result["sha256"], hashlib.sha256(fake_binary()).hexdigest())
            self.assertEqual(binary.read_bytes(), fake_binary())
            self.assertTrue(os.access(binary, os.X_OK))
            self.assertEqual(list(binary.parent.glob(".cargo-nextest-*")), [])

    def test_checksum_is_checked_before_archive_selection(self):
        archive = b"not the pinned official release"
        with mock.patch.dict(install_nextest.ARCHIVES, {
            "universal-apple-darwin": {"url": "fixture://archive", "sha256": "0" * 64}
        }), mock.patch.object(install_nextest, "select_binary") as select:
            with self.assertRaisesRegex(install_nextest.InstallError, "SHA-256 mismatch"):
                install_nextest.install_nextest(".artifacts/ci/unused", "Darwin", "arm64",
                                                downloader=lambda _url: archive)
            select.assert_not_called()

    def test_unsupported_host_fails_before_download(self):
        with mock.patch.object(install_nextest, "download_archive") as download:
            with self.assertRaisesRegex(install_nextest.InstallError, "unsupported"):
                install_nextest.install_nextest(".artifacts/ci/unused", "Linux", "aarch64")
            download.assert_not_called()

    def test_malformed_archive_is_rejected(self):
        with self.assertRaises(install_nextest.InstallError):
            install_nextest.select_binary(b"not a tar.gz")

    def test_missing_binary_and_duplicate_binary_are_rejected(self):
        with self.assertRaisesRegex(install_nextest.InstallError, "found 0"):
            install_nextest.select_binary(make_archive([("README", "file", b"release")]))
        duplicate = make_archive([
            ("one/cargo-nextest", "file", fake_binary()),
            ("two/cargo-nextest", "file", fake_binary()),
        ])
        with self.assertRaisesRegex(install_nextest.InstallError, "found 2"):
            install_nextest.select_binary(duplicate)

    def test_unsafe_paths_links_and_special_members_are_rejected(self):
        for name, kind in (("../cargo-nextest", "file"), ("/cargo-nextest", "file"),
                           ("cargo-nextest", "symlink"), ("cargo-nextest", "hardlink"),
                           ("cargo-nextest", "fifo")):
            with self.subTest(name=name, kind=kind):
                archive = make_archive([(name, kind, fake_binary())])
                with self.assertRaises(install_nextest.InstallError):
                    install_nextest.select_binary(archive)

    def test_oversized_member_and_unpacked_stream_are_rejected(self):
        archive = make_archive([("cargo-nextest", "file", b"x" * 32)])
        with mock.patch.object(install_nextest, "MAX_MEMBER_BYTES", 16):
            with self.assertRaisesRegex(install_nextest.InstallError, "size limit"):
                install_nextest.select_binary(archive)
        with mock.patch.object(install_nextest, "MAX_UNPACKED_BYTES", 64):
            with self.assertRaisesRegex(install_nextest.InstallError, "size limit"):
                install_nextest.select_binary(archive)

    def test_compressed_pax_metadata_bomb_is_bounded_after_decompression(self):
        output = io.BytesIO()
        with tarfile.open(fileobj=output, mode="w:gz", format=tarfile.PAX_FORMAT) as archive:
            info = tarfile.TarInfo("x" * 4000 + "/cargo-nextest")
            payload = fake_binary()
            info.size = len(payload)
            archive.addfile(info, io.BytesIO(payload))
        compressed = output.getvalue()
        self.assertLess(len(compressed), 2048)
        with mock.patch.object(install_nextest, "MAX_UNPACKED_BYTES", 512):
            with self.assertRaisesRegex(install_nextest.InstallError, "unpacked archive exceeds size limit"):
                install_nextest.select_binary(compressed)

    def test_existing_bin_symlink_is_rejected_before_writing(self):
        archive = make_archive([("cargo-nextest", "file", fake_binary())])
        digest = hashlib.sha256(archive).hexdigest()
        with tempfile.TemporaryDirectory(dir=ROOT / ".artifacts/ci") as temp:
            base = Path(temp)
            install_dir = base / "tools"
            install_dir.mkdir()
            (install_dir / "bin").symlink_to(base / "outside", target_is_directory=True)
            with mock.patch.dict(install_nextest.ARCHIVES, {
                "universal-apple-darwin": {"url": "fixture://archive", "sha256": digest}
            }), self.assertRaisesRegex(install_nextest.InstallError, "must not be a symlink"):
                install_nextest.install_nextest(install_dir, "Darwin", "arm64",
                                                downloader=lambda _url: archive)
            self.assertFalse((base / "outside" / "cargo-nextest").exists())

    def test_wrong_runtime_version_leaves_no_installed_binary(self):
        archive = make_archive([("cargo-nextest", "file", fake_binary("9.9.9"))])
        digest = hashlib.sha256(archive).hexdigest()
        with tempfile.TemporaryDirectory(dir=ROOT / ".artifacts/ci") as temp:
            install_dir = Path(temp) / "tools"
            with mock.patch.dict(install_nextest.ARCHIVES, {
                "universal-apple-darwin": {"url": "fixture://archive", "sha256": digest}
            }), self.assertRaisesRegex(install_nextest.InstallError, "unexpected.*runtime version"):
                install_nextest.install_nextest(install_dir, "Darwin", "arm64",
                                                downloader=lambda _url: archive)
            bin_dir = install_dir / "bin"
            self.assertFalse((bin_dir / "cargo-nextest").exists())
            self.assertEqual(list(bin_dir.glob(".cargo-nextest-*")), [])


class NextestDiscoveryTests(unittest.TestCase):
    def test_cargo_and_nextest_names_and_ignored_state_compare_as_multisets(self):
        cargo_all = (
            "Running unittests src/lib.rs (target/debug/deps/lib-same-111)\n"
            "running 2 tests\na::same: test\nc::ignored: test (ignored)\n"
            "Running tests/integration.rs (target/debug/deps/integration-222)\n"
            "running 1 test\nb::same: test\n"
        )
        cargo_ignored = (
            "Running unittests src/lib.rs (target/debug/deps/lib-same-111)\n"
            "running 1 test\nc::ignored: test\n"
            "Running tests/integration.rs (target/debug/deps/integration-222)\n"
            "running 0 tests\n"
        )
        nextest = json.dumps({
            "test-count": 3,
            "rust-suites": {
                "suite-a": {"binary-path": "/workspace/target/debug/deps/lib-same-111",
                            "testcases": {"a::same": {"ignored": False}, "c::ignored": {"ignored": True}}},
                "suite-b": {"binary-path": "/workspace/target/debug/deps/integration-222",
                            "testcases": {"b::same": {"ignored": False}}},
            },
        })
        names, ignored = nextest_ci.parse_nextest_listing(nextest)
        cargo_names, _cargo_all_ignored = nextest_ci.parse_cargo_listing(cargo_all)
        _ignored_only, cargo_ignored_names = nextest_ci.parse_cargo_listing(
            cargo_ignored, assume_ignored=True
        )
        counts = nextest_ci.compare_discoveries(
            cargo_names, cargo_ignored_names, names, ignored
        )
        self.assertEqual(counts, {"test_count": 3, "ignored_count": 1})
        with self.assertRaises(nextest_ci.QualificationError):
            nextest_ci.compare_discoveries(
                cargo_names, cargo_ignored_names,
                [("integration-222", name) for _, name in names],
                [("integration-222", name) for _, name in ignored],
            )

    def test_discovery_name_ignored_count_and_metadata_defects_fail(self):
        for cargo_all, cargo_ignored, listing in (
            ([], [], '{"test-count": 0, "rust-suites": {}}'),
            ([ ("a-bin", "a"), ("a-bin", "b") ], [], '{"test-count": 1, "rust-suites": {"s": {"binary-path": "/tmp/a-bin", "testcases": {"a": {"ignored": false}}}}}'),
            ([ ("a-bin", "a") ], [("a-bin", "a")], '{"test-count": 1, "rust-suites": {"s": {"binary-path": "/tmp/a-bin", "testcases": {"a": {"ignored": false}}}}}'),
            ([ ("a-bin", "a") ], [], '{"test-count": 1, "rust-suites": {"s": {"binary-path": "/tmp/a-bin", "testcases": {"a": {}}}}}'),
        ):
            with self.subTest(cargo_all=cargo_all, cargo_ignored=cargo_ignored), self.assertRaises(nextest_ci.QualificationError):
                names, ignored = nextest_ci.parse_nextest_listing(listing)
                nextest_ci.compare_discoveries(cargo_all, cargo_ignored, names, ignored)

    def test_only_exact_named_single_nextest_failure_proves_red_control(self):
        failure = (
            "FAIL [ 0.001s] nextest-red-control::tests::injected_failure_is_detected\n"
            "Summary [0.001s] 1 test run: 0 passed, 1 failed, 0 skipped\n"
        )
        self.assertTrue(nextest_ci.red_control_proved(100, failure))
        self.assertTrue(nextest_ci.red_control_proved(100, failure + failure.splitlines()[0] + "\n"))
        for exit_code, output in (
            (101, failure),
            (4, "no tests run\n"),
            (100, failure.replace("injected_failure_is_detected", "other_test")),
            (100, failure + failure),
            (100, failure + "FAIL nextest-red-control::other\n"),
            (100, failure.replace("1 failed", "2 failed")),
            (100, "error: could not compile\n"),
        ):
            with self.subTest(exit_code=exit_code, output=output):
                self.assertFalse(nextest_ci.red_control_proved(exit_code, output))

    def test_run_summary_records_pass_fail_and_ignored_skip_counts(self):
        summary = nextest_ci.parse_run_summary(
            "Summary [1s] 2 tests run: 2 passed, 1 skipped, 0 failed\n"
        )
        self.assertEqual(summary,
                         {"run_count": 2, "passed": 2, "failed": 0, "skipped": 1})
        nextest_ci.validate_run_summary(summary, {"test_count": 3, "ignored_count": 1})
        for invalid in (
            {"run_count": 3, "passed": 2, "failed": 0, "skipped": 1},
            {"run_count": 2, "passed": 2, "failed": 0, "skipped": 0},
            {"run_count": 2, "passed": 1, "failed": 0, "skipped": 1},
        ):
            with self.subTest(summary=invalid), self.assertRaises(nextest_ci.QualificationError):
                nextest_ci.validate_run_summary(invalid, {"test_count": 3, "ignored_count": 1})

    def test_helper_rejects_unsupported_and_wrong_hosts(self):
        self.assertEqual(nextest_ci.supported_host("Darwin", "arm64"), "aarch64-apple-darwin")
        self.assertEqual(nextest_ci.supported_host("Linux", "x86_64"), "x86_64-unknown-linux-gnu")
        for system, machine in (("Darwin", "x86_64"), ("Linux", "aarch64")):
            with self.subTest(system=system, machine=machine), self.assertRaises(nextest_ci.QualificationError):
                nextest_ci.supported_host(system, machine)


if __name__ == "__main__":
    unittest.main()
