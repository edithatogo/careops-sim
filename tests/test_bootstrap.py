import importlib.util
import json
import unittest
from pathlib import Path
from unittest.mock import patch


MODULE_PATH = Path(__file__).resolve().parents[1] / "tools" / "bootstrap.py"
SPEC = importlib.util.spec_from_file_location("bootstrap", MODULE_PATH)
bootstrap = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(bootstrap)


def tool_output(tool, version="1.99.1", host="aarch64-apple-darwin"):
    return f"{tool} {version} (fixture)\nrelease: {version}\nhost: {host}\n"


class BootstrapTests(unittest.TestCase):
    def invoke(self, host="aarch64-apple-darwin", rustc=None, cargo=None,
               rustup="/fixture/rustup"):
        outputs = {"rustc": rustc or tool_output("rustc"),
                   "cargo": cargo or tool_output("cargo")}

        def run(command, **kwargs):
            self.assertEqual(command[:3], [rustup, "run", "1.99.1"])
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
            with self.assertRaisesRegex(bootstrap.BootstrapError, "Pinned rustc 1.99.1"):
                bootstrap.check(host="aarch64-apple-darwin", rustup_path="rustup")

    def test_wrong_cargo_version_fails(self):
        calls = 0

        def run(*args, **kwargs):
            nonlocal calls
            calls += 1
            tool = args[0][3]
            version = "1.97.0" if tool == "cargo" else "1.99.1"
            return type("Result", (), {"returncode": 0, "stdout": tool_output(tool, version), "stderr": ""})()

        with patch.object(bootstrap.subprocess, "run", side_effect=run):
            with self.assertRaisesRegex(bootstrap.BootstrapError, "Pinned cargo 1.99.1"):
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
        with patch.object(bootstrap, "compiler_environment", return_value=("/pinned/bin/cargo", {"RUSTC": "/pinned/bin/rustc"})), \
             patch.object(bootstrap, "check") as check, \
             patch.object(bootstrap, "KAIROS_PATH") as kairos, \
             patch.object(bootstrap.subprocess, "run") as run:
            kairos.__truediv__.return_value.is_file.return_value = True
            run.return_value.returncode = 0
            bootstrap.test_core()
        check.assert_called_once()
        command = run.call_args.args[0]
        self.assertEqual(command[:3], ["/pinned/bin/cargo", "test", "--locked"])
        self.assertEqual(run.call_args.kwargs["env"]["RUSTC"], "/pinned/bin/rustc")
        self.assertIn("--locked", command)
        self.assertEqual(
            [command[index + 1] for index, value in enumerate(command[:-1]) if value == "-p"],
            list(bootstrap.CORE_PACKAGES),
        )
        self.assertEqual(run.call_args.kwargs["cwd"], kairos)

    def test_compiler_binding_overrides_unrelated_path_and_compiler(self):
        def resolve(command, **kwargs):
            return type("Result", (), {"returncode": 0, "stdout": "/pinned/bin/" + command[-1] + "\n"})()
        with patch.object(bootstrap.shutil, "which", return_value="/safe/rustup"), \
             patch.object(bootstrap.subprocess, "run", side_effect=resolve), \
             patch.dict(bootstrap.os.environ, {"PATH": "/other/bin", "RUSTC": "/other/rustc"}):
            cargo, environment = bootstrap.compiler_environment("1.99.1")
        self.assertEqual(cargo, "/pinned/bin/cargo")
        self.assertEqual(environment["RUSTC"], "/pinned/bin/rustc")
        self.assertEqual(environment["RUSTDOC"], "/pinned/bin/rustdoc")
        self.assertEqual(environment["RUSTUP_TOOLCHAIN"], "1.99.1")
        self.assertEqual(environment["PATH"].split(bootstrap.os.pathsep)[0], "/pinned/bin")

    def test_invalid_tool_resolution_is_refused(self):
        for code, output in ((1, "/pinned/bin/cargo"), (0, "cargo"),
                             (0, "/pinned/bin/not-cargo"), (0, "")):
            with self.subTest(code=code, output=output), \
                 patch.object(bootstrap.shutil, "which", return_value="/safe/rustup"), \
                 patch.object(bootstrap.subprocess, "run", return_value=type(
                     "Result", (), {"returncode": code, "stdout": output})()):
                with self.assertRaisesRegex(bootstrap.BootstrapError, "resolve pinned cargo"):
                    bootstrap.compiler_environment("1.99.1")

    def test_mixed_compiler_directories_are_refused(self):
        def resolve(command, **kwargs):
            directory = "/other/bin/" if command[-1] == "rustc" else "/pinned/bin/"
            return type("Result", (), {"returncode": 0, "stdout": directory + command[-1]})()
        with patch.object(bootstrap.shutil, "which", return_value="/safe/rustup"), \
             patch.object(bootstrap.subprocess, "run", side_effect=resolve):
            with self.assertRaisesRegex(bootstrap.BootstrapError, "share one toolchain"):
                bootstrap.compiler_environment("1.99.1")

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
