import importlib.util
import json
import unittest
from pathlib import Path
from unittest.mock import patch


MODULE_PATH = Path(__file__).resolve().parents[1] / "tools" / "bootstrap.py"
SPEC = importlib.util.spec_from_file_location("bootstrap", MODULE_PATH)
bootstrap = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(bootstrap)


def tool_output(tool, version="1.99.0", host="aarch64-apple-darwin"):
    return f"{tool} {version} (fixture)\nrelease: {version}\nhost: {host}\n"


class BootstrapTests(unittest.TestCase):
    def invoke(self, host="aarch64-apple-darwin", rustc=None, cargo=None,
               rustup="/fixture/rustup"):
        outputs = {"rustc": rustc or tool_output("rustc"),
                   "cargo": cargo or tool_output("cargo")}

        def run(command, **kwargs):
            self.assertEqual(command[:3], [rustup, "run", "1.99.0"])
            self.assertEqual(command[4:], ["--version", "--verbose"])
            tool = command[3]
            return type("Result", (), {"returncode": 0, "stdout": outputs[tool], "stderr": ""})()

        with patch.object(bootstrap.subprocess, "run", side_effect=run) as runner:
            bootstrap.check(host=host, rustup_path=rustup)
            return runner

    def test_success_uses_read_only_rustup_run_for_both_tools(self):
        runner = self.invoke()
        self.assertEqual(runner.call_count, 2)

    def test_missing_rustup_has_actionable_remediation_without_invocation(self):
        with patch.object(bootstrap.shutil, "which", return_value=None), \
             patch.object(bootstrap.subprocess, "run") as runner:
            with self.assertRaisesRegex(bootstrap.BootstrapError, "rustup.*not found.*rustup toolchain install"):
                bootstrap.check(host="aarch64-apple-darwin")
            runner.assert_not_called()

    def test_wrong_rustc_version_fails(self):
        with patch.object(bootstrap.subprocess, "run", return_value=type(
                "Result", (), {"returncode": 0, "stdout": tool_output("rustc", "1.97.0"), "stderr": ""})()):
            with self.assertRaisesRegex(bootstrap.BootstrapError, "Pinned rustc 1.99.0"):
                bootstrap.check(host="aarch64-apple-darwin", rustup_path="rustup")

    def test_wrong_cargo_version_fails(self):
        calls = 0

        def run(*args, **kwargs):
            nonlocal calls
            calls += 1
            tool = args[0][3]
            version = "1.97.0" if tool == "cargo" else "1.99.0"
            return type("Result", (), {"returncode": 0, "stdout": tool_output(tool, version), "stderr": ""})()

        with patch.object(bootstrap.subprocess, "run", side_effect=run):
            with self.assertRaisesRegex(bootstrap.BootstrapError, "Pinned cargo 1.99.0"):
                bootstrap.check(host="aarch64-apple-darwin", rustup_path="rustup")

    def test_unsupported_host_fails_before_running_tools(self):
        with patch.object(bootstrap.subprocess, "run") as runner:
            with self.assertRaisesRegex(bootstrap.BootstrapError, "Unsupported host"):
                bootstrap.check(host="x86_64-apple-darwin", rustup_path="rustup")
            runner.assert_not_called()

    def test_toolchain_host_mismatch_fails(self):
        def run(command, **kwargs):
            tool = command[3]
            result = type("Result", (), {})()
            result.returncode = 0
            result.stdout = tool_output(tool, host="x86_64-apple-darwin")
            result.stderr = ""
            return result

        with patch.object(bootstrap.subprocess, "run", side_effect=run):
            with self.assertRaisesRegex(bootstrap.BootstrapError, "Host mismatch"):
                bootstrap.check(host="aarch64-apple-darwin", rustup_path="rustup")

    def test_host_mapping_is_limited_to_supported_targets(self):
        self.assertEqual(bootstrap.detected_host("Darwin", "arm64"), "aarch64-apple-darwin")
        self.assertEqual(bootstrap.detected_host("Linux", "x86_64", "glibc"), "x86_64-unknown-linux-gnu")
        self.assertIsNone(bootstrap.detected_host("Linux", "x86_64", "musl"))
        self.assertIsNone(bootstrap.detected_host("Linux", "x86_64", ""))
        self.assertIsNone(bootstrap.detected_host("Linux", "aarch64"))

    def test_core_suite_checks_toolchain_then_runs_five_locked_packages(self):
        with patch.object(bootstrap, "check") as check, \
             patch.object(bootstrap, "KAIROS_PATH") as kairos, \
             patch.object(bootstrap.subprocess, "run") as run:
            kairos.__truediv__.return_value.is_file.return_value = True
            run.return_value.returncode = 0
            bootstrap.test_core()
        check.assert_called_once()
        command = run.call_args.args[0]
        self.assertEqual(command[:5], ["rustup", "run", "1.99.0", "cargo", "test"])
        self.assertIn("--locked", command)
        self.assertEqual(
            [command[index + 1] for index, value in enumerate(command[:-1]) if value == "-p"],
            list(bootstrap.CORE_PACKAGES),
        )
        self.assertEqual(run.call_args.kwargs["cwd"], kairos)

    def test_core_suite_reports_submodule_setup_instructions(self):
        with patch.object(bootstrap, "check"), \
             patch.object(bootstrap, "KAIROS_PATH") as kairos, \
             patch.object(bootstrap.subprocess, "run") as run:
            kairos.__truediv__.return_value.is_file.return_value = False
            with self.assertRaisesRegex(bootstrap.BootstrapError, "git submodule update --init --recursive"):
                bootstrap.test_core()
            run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
