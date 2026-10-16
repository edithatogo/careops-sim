#!/usr/bin/env python3
"""Tests for Bounded Maintenance Automation and Scheduled Quality Policy (Task D5.1).

Validates:
  - Four Scheduled Maintenance Routines:
      1. Version Freshness
      2. Documentation Drift
      3. Dependency Groups
      4. Extended Tests
  - Six Core Operational Controls:
      1. No-Change Silence
      2. Actionable Reports
      3. Idempotency
      4. Timeout Enforcement
      5. Retry Ceiling
      6. Pause and Resume Controls
  - Absolute Prohibition of Autonomous Publication
  - CLI Subcommands (status, pause, resume, run)
"""

from __future__ import annotations

import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

from tools.maintenance_scheduler import (
    AutonomousPublicationError,
    AutonomousPublicationViolationError,
    DriftViolation,
    IdempotencyViolationError,
    JobKind,
    JobResult,
    JobStatus,
    JobTimeoutError,
    MaintenanceError,
    MaintenancePausedError,
    MaintenanceReport,
    MaintenanceScheduler,
    MaintenanceSchedulerError,
    RetryCeilingExceededError,
    SchedulerPausedError,
    SchedulerTimeoutError,
    assert_no_autonomous_publication,
    audit_dependency_groups,
    audit_documentation_drift,
    audit_extended_tests,
    audit_version_freshness,
    check_dependency_groups,
    check_documentation_drift,
    check_version_freshness,
    get_scheduler_status,
    load_scheduler_state,
    pause,
    pause_scheduler,
    resume,
    resume_scheduler,
    run_extended_tests,
    save_scheduler_state,
    status,
)


class MaintenanceSchedulerTests(unittest.TestCase):
    """Test suite covering Task D5.1 scheduled maintenance routines and operational controls."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

        # Create mock repository structure
        (self.root / "conductor/evidence").mkdir(parents=True)
        (self.root / "model-inputs/ed/schema").mkdir(parents=True)
        (self.root / "crates/careops-ed/src").mkdir(parents=True)
        (self.root / "crates/careops-sim/src").mkdir(parents=True)
        (self.root / "crates/careops-ed-cli/src").mkdir(parents=True)
        (self.root / "tools").mkdir(parents=True)
        (self.root / "tests").mkdir(parents=True)
        (self.root / ".github/workflows").mkdir(parents=True)

        # Baseline clean files
        (self.root / "conductor/current-state.json").write_text(
            json.dumps({"submodule_pins": {"libs/kairos": "commit-abc-123"}}), encoding="utf-8"
        )
        (self.root / "conductor/evidence/d2.4-kairos-contract.json").write_text(
            json.dumps({"kairos_commit": "commit-abc-123"}), encoding="utf-8"
        )
        (self.root / "rust-toolchain.toml").write_text(
            '[toolchain]\nchannel = "1.99.0"\n', encoding="utf-8"
        )
        (self.root / "Cargo.toml").write_text(
            '[workspace.package]\nlicense = "Apache-2.0"\n', encoding="utf-8"
        )
        (self.root / "crates/careops-ed/src/lib.rs").write_text(
            "#![forbid(unsafe_code)]\npub fn run() {}\n", encoding="utf-8"
        )
        (self.root / "crates/careops-sim/src/lib.rs").write_text(
            "#![forbid(unsafe_code)]\npub fn run() {}\n", encoding="utf-8"
        )
        (self.root / "crates/careops-ed-cli/src/main.rs").write_text(
            "#![forbid(unsafe_code)]\nfn main() {}\n", encoding="utf-8"
        )
        (self.root / "model-inputs/ed/schema/capacity-location.schema.json").write_text(
            '{"type": "object"}', encoding="utf-8"
        )
        (self.root / "model-inputs/ed/schema/example-profile-pack.schema.json").write_text(
            '{"type": "object"}', encoding="utf-8"
        )
        (self.root / ".github/workflows/ci.yml").write_text(
            "name: CI\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683\n",
            encoding="utf-8",
        )

        self.state_file = self.root / "conductor/maintenance-scheduler-state.json"
        self.scheduler = MaintenanceScheduler(root=self.root, state_file=self.state_file)

    # -----------------------------------------------------------------------
    # Requirement 1: test_no_change_silence_on_clean_repository
    # -----------------------------------------------------------------------

    def test_no_change_silence_on_clean_repository(self) -> None:
        """Runs all 4 maintenance jobs on clean repository; asserts zero drift, silence, minimal noise."""
        repo_root = Path(__file__).resolve().parents[1]
        state_file = self.root / "clean-repo-state.json"
        scheduler = MaintenanceScheduler(root=repo_root, state_file=state_file)

        f_stdout = io.StringIO()
        f_stderr = io.StringIO()
        with patch("sys.stdout", f_stdout), patch("sys.stderr", f_stderr):
            report = scheduler.run(silent_on_clean=True, verbose=False)

        # Asserts all 4 jobs pass
        self.assertEqual(len(report.results), 4)
        for job_res in report.results:
            self.assertIn(job_res.status, (JobStatus.SUCCESS, "success"))
            self.assertEqual(job_res.exit_code, 0)

        # Asserts report.drift_detected is False and report.is_silent is True
        self.assertFalse(report.drift_detected)
        self.assertTrue(report.is_silent)
        self.assertTrue(report.is_clean)
        self.assertEqual(len(report.violations), 0)

        # Asserts zero noise / minimal output emitted
        self.assertEqual(f_stdout.getvalue(), "")
        self.assertEqual(f_stderr.getvalue(), "")

    # -----------------------------------------------------------------------
    # Requirement 2: test_actionable_report_generation_on_detected_drift
    # -----------------------------------------------------------------------

    def test_actionable_report_generation_on_detected_drift(self) -> None:
        """Injects synthetic drift, asserts drift_detected is True, is_silent is False, and report is actionable."""
        # Inject synthetic drift into Cargo.toml and current-state.json
        (self.root / "Cargo.toml").write_text('[package]\nlicense = "GPL-3.0"\n', encoding="utf-8")
        (self.root / "conductor/current-state.json").write_text(
            json.dumps({"submodule_pins": {"libs/kairos": "drifted-sha-000"}}), encoding="utf-8"
        )

        f_stderr = io.StringIO()
        with patch("sys.stderr", f_stderr):
            report = self.scheduler.run(silent_on_clean=True, verbose=False)

        # Asserts report.drift_detected is True and report.is_silent is False
        self.assertTrue(report.drift_detected)
        self.assertFalse(report.is_silent)
        self.assertFalse(report.is_clean)
        self.assertGreater(len(report.violations), 0)

        # Asserts report contains actionable remediation instructions, offending files, and designated owner
        remediation_texts = [v.remediation_command for v in report.violations]
        offending_files = [v.file_path for v in report.violations]
        owners = [v.conductor_owner for v in report.violations]

        self.assertTrue(any("Cargo.toml" in f for f in offending_files))
        self.assertTrue(any("Apache-2.0" in r for r in remediation_texts))
        self.assertTrue(any("Owner 42" in o for o in owners))

        # Check property helpers
        for v in report.violations:
            self.assertTrue(len(v.actionable_instructions) > 0)
            self.assertTrue(len(v.offending_files) > 0)
            self.assertTrue(len(v.designated_owner) > 0)

        # Diagnostic JSON was emitted to stderr
        err_output = f_stderr.getvalue()
        self.assertIn("LicenseDeclarationDriftError", err_output)
        self.assertIn("Cargo.toml", err_output)
        self.assertIn("Owner 42", err_output)
        self.assertIn("Apache-2.0", err_output)

    # -----------------------------------------------------------------------
    # Requirement 3: test_idempotency_guarantee
    # -----------------------------------------------------------------------

    def test_idempotency_guarantee(self) -> None:
        """Runs maintenance scheduler twice consecutively; asserts identical status and zero corruption."""
        report1 = self.scheduler.run()
        state1 = status(state_file=self.state_file)

        report2 = self.scheduler.run()
        state2 = status(state_file=self.state_file)

        # Both runs yield identical clean status
        self.assertEqual(report1.drift_detected, report2.drift_detected)
        self.assertEqual(report1.is_silent, report2.is_silent)
        self.assertEqual(len(report1.results), len(report2.results))
        self.assertEqual(len(report1.violations), len(report2.violations))

        # Results statuses match
        for r1, r2 in zip(report1.results, report2.results):
            self.assertEqual(r1.kind, r2.kind)
            self.assertEqual(r1.status, r2.status)
            self.assertEqual(r1.exit_code, r2.exit_code)

        # State preserves history monotonically without record corruption
        self.assertEqual(state2["total_runs"], state1["total_runs"] + 1)
        self.assertFalse(state1["paused"])
        self.assertFalse(state2["paused"])

    # -----------------------------------------------------------------------
    # Requirement 4: test_timeout_enforcement
    # -----------------------------------------------------------------------

    def test_timeout_enforcement(self) -> None:
        """Configures a job that sleeps longer than timeout; asserts JobTimeoutError and timeout status."""
        def slow_job(root: Path) -> list[DriftViolation]:
            time.sleep(0.08)
            return []

        with patch.dict("tools.maintenance_scheduler.ROUTINE_DISPATCH", {"extended_tests": slow_job}):
            # Test 1: When raise_on_error=True (default), handles JobTimeoutError
            with self.assertRaises((JobTimeoutError, SchedulerTimeoutError)) as ctx:
                self.scheduler.run(jobs=["extended_tests"], timeout=0.02, raise_on_error=True)
            self.assertIn("exceeded timeout", str(ctx.exception))

            # Test 2: When raise_on_error=False, records status == "timeout"
            with patch("sys.stderr", io.StringIO()):
                report = self.scheduler.run(jobs=["extended_tests"], timeout=0.02, raise_on_error=False)
            self.assertTrue(report.drift_detected)
            self.assertEqual(len(report.results), 1)
            self.assertEqual(report.results[0].status, "timeout")
            self.assertEqual(report.results[0].exit_code, 124)

    # -----------------------------------------------------------------------
    # Requirement 5: test_retry_ceiling_and_bounded_failure
    # -----------------------------------------------------------------------

    def test_retry_ceiling_and_bounded_failure(self) -> None:
        """Configures a failing job with max_retries = 3; asserts exactly 3 retries and RetryCeilingExceededError."""
        attempts = [0]

        def failing_job(root: Path) -> list[DriftViolation]:
            attempts[0] += 1
            raise ConnectionError(f"Transient I/O failure attempt {attempts[0]}")

        with patch.dict("tools.maintenance_scheduler.ROUTINE_DISPATCH", {"version_freshness": failing_job}):
            # Test 1: When raise_on_error=True, raises RetryCeilingExceededError
            with self.assertRaises(RetryCeilingExceededError) as ctx:
                self.scheduler.run(jobs=["version_freshness"], max_retries=3, raise_on_error=True)
            self.assertEqual(attempts[0], 3)
            self.assertIn("retry ceiling (3)", str(ctx.exception))

            # Test 2: When raise_on_error=False, records status == "failed"
            attempts[0] = 0
            with patch("sys.stderr", io.StringIO()):
                report = self.scheduler.run(jobs=["version_freshness"], max_retries=3, raise_on_error=False)
            self.assertEqual(attempts[0], 3)
            self.assertTrue(report.drift_detected)
            self.assertEqual(len(report.results), 1)
            self.assertEqual(report.results[0].status, "failed")
            self.assertEqual(report.results[0].retries_attempted, 3)

    # -----------------------------------------------------------------------
    # Requirement 6: test_pause_and_resume_controls
    # -----------------------------------------------------------------------

    def test_pause_and_resume_controls(self) -> None:
        """Tests pause, fail-closed while paused, and resume controls."""
        # 1. Invokes pause(reason="Emergency freeze", user="maintainer")
        paused_state = pause(reason="Emergency freeze", user="maintainer", state_file=self.state_file)
        self.assertTrue(paused_state["paused"])

        # 2. Asserts status() shows paused == True
        curr_status = status(state_file=self.state_file)
        self.assertTrue(curr_status["paused"])
        self.assertEqual(curr_status["reason"], "Emergency freeze")
        self.assertEqual(curr_status["paused_by"], "maintainer")

        # 3. Attempts to run jobs while paused -> asserts MaintenancePausedError is raised
        with self.assertRaises((MaintenancePausedError, SchedulerPausedError)) as ctx:
            self.scheduler.run()
        self.assertIn("Emergency freeze", str(ctx.exception))

        # 4. Invokes resume(user="maintainer")
        resumed_state = resume(user="maintainer", state_file=self.state_file)
        self.assertFalse(resumed_state["paused"])

        # 5. Asserts status() shows paused == False
        curr_status2 = status(state_file=self.state_file)
        self.assertFalse(curr_status2["paused"])

        # 6. Runs jobs successfully after resumption without duplicate edits or state loss
        report = self.scheduler.run()
        self.assertFalse(report.drift_detected)
        self.assertTrue(report.is_clean)
        self.assertGreaterEqual(curr_status2["total_runs"], 0)

    # -----------------------------------------------------------------------
    # Requirement 7: test_autonomous_publication_prevention
    # -----------------------------------------------------------------------

    def test_autonomous_publication_prevention(self) -> None:
        """Tests that any attempt to execute publish or push actions raises AutonomousPublicationError."""
        forbidden_commands = [
            "cargo publish",
            "cargo publish --token SECRET",
            "git push origin main",
            "git push --tags",
            "gh release create v0.1.0",
            "twine upload dist/*",
            "npm publish",
        ]

        for cmd in forbidden_commands:
            with self.subTest(command=cmd):
                with self.assertRaises((AutonomousPublicationError, AutonomousPublicationViolationError)) as ctx:
                    assert_no_autonomous_publication(cmd)
                self.assertIn("Autonomous publication command prohibited", str(ctx.exception))

        # Permitted non-publishing commands do not raise
        permitted_commands = [
            "cargo test --locked",
            "git status --porcelain",
            "git diff --check",
            "python3 tools/context.py check",
            "cargo check --workspace",
        ]
        for cmd in permitted_commands:
            with self.subTest(command=cmd):
                assert_no_autonomous_publication(cmd)

    # -----------------------------------------------------------------------
    # Requirement 8: test_cli_interface_subcommands
    # -----------------------------------------------------------------------

    def test_cli_interface_subcommands(self) -> None:
        """Tests CLI subcommands via subprocess: status, pause, resume, run."""
        cli_script = Path(__file__).resolve().parents[1] / "tools/maintenance_scheduler.py"
        test_state = self.root / "cli-test-state.json"

        # 1. Test 'status' subcommand
        res_status = subprocess.run(
            [sys.executable, str(cli_script), "status", "--state-file", str(test_state)],
            capture_output=True,
            text=True,
            check=True,
        )
        data = json.loads(res_status.stdout)
        self.assertFalse(data["paused"])

        # 2. Test 'pause' subcommand
        res_pause = subprocess.run(
            [
                sys.executable,
                str(cli_script),
                "pause",
                "--reason",
                "CLI emergency freeze",
                "--user",
                "operator",
                "--state-file",
                str(test_state),
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        self.assertIn("Scheduler paused: CLI emergency freeze", res_pause.stdout)

        # Check status reflects pause
        res_status_paused = subprocess.run(
            [sys.executable, str(cli_script), "status", "--state-file", str(test_state)],
            capture_output=True,
            text=True,
            check=True,
        )
        data_paused = json.loads(res_status_paused.stdout)
        self.assertTrue(data_paused["paused"])
        self.assertEqual(data_paused["reason"], "CLI emergency freeze")

        # 3. Test 'run' while paused -> exits with code 2
        res_run_paused = subprocess.run(
            [sys.executable, str(cli_script), "run", "--state-file", str(test_state)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(res_run_paused.returncode, 2)
        self.assertIn("FAILED (PAUSED)", res_run_paused.stderr)

        # 4. Test 'resume' subcommand
        res_resume = subprocess.run(
            [
                sys.executable,
                str(cli_script),
                "resume",
                "--user",
                "operator",
                "--state-file",
                str(test_state),
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        self.assertIn("Scheduler resumed.", res_resume.stdout)

        # 5. Test 'run' subcommand after resume -> exits with code 0 and silent output
        res_run = subprocess.run(
            [
                sys.executable,
                str(cli_script),
                "run",
                "--jobs",
                "version_freshness",
                "dependency_groups",
                "--state-file",
                str(test_state),
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        self.assertEqual(res_run.returncode, 0)
        self.assertEqual(res_run.stdout, "")
        self.assertEqual(res_run.stderr, "")

    # -----------------------------------------------------------------------
    # Routine Granular Unit Tests
    # -----------------------------------------------------------------------

    def test_version_freshness_clean_passes(self) -> None:
        violations = audit_version_freshness(self.root)
        self.assertEqual(len(violations), 0)
        job_res = check_version_freshness(self.root)
        self.assertEqual(job_res.status, JobStatus.SUCCESS)

    def test_version_freshness_detects_submodule_drift(self) -> None:
        (self.root / "conductor/current-state.json").write_text(
            json.dumps({"submodule_pins": {"libs/kairos": "drifted-sha-999"}}), encoding="utf-8"
        )
        violations = audit_version_freshness(self.root)
        self.assertEqual(len(violations), 1)
        v = violations[0]
        self.assertEqual(v.error_class, "SubmodulePinDriftError")
        self.assertIn("Owner 13", v.conductor_owner)
        self.assertIn("check_kairos_pin.py", v.remediation_command)

        job_res = check_version_freshness(self.root)
        self.assertEqual(job_res.status, JobStatus.FAILED)
        self.assertIn("SubmodulePinDriftError", job_res.actionable_remediation)

    def test_version_freshness_detects_toolchain_drift(self) -> None:
        (self.root / "rust-toolchain.toml").write_text(
            '[toolchain]\nchannel = "nightly"\n', encoding="utf-8"
        )
        violations = audit_version_freshness(self.root)
        self.assertEqual(len(violations), 1)
        v = violations[0]
        self.assertEqual(v.error_class, "ToolchainDriftError")
        self.assertIn("Owner 44", v.conductor_owner)

    def test_version_freshness_detects_floating_ci_action_tag(self) -> None:
        (self.root / ".github/workflows/ci.yml").write_text(
            "steps:\n  - uses: actions/checkout@v7.0.1\n", encoding="utf-8"
        )
        violations = audit_version_freshness(self.root)
        self.assertEqual(len(violations), 1)
        v = violations[0]
        self.assertEqual(v.error_class, "FloatingActionTagError")
        self.assertIn("Owner 20", v.conductor_owner)

    def test_documentation_drift_clean_passes(self) -> None:
        violations = audit_documentation_drift(self.root)
        self.assertEqual(len(violations), 0)
        job_res = check_documentation_drift(self.root)
        self.assertEqual(job_res.status, JobStatus.SUCCESS)

    def test_documentation_drift_detects_missing_schema(self) -> None:
        (self.root / "model-inputs/ed/schema/capacity-location.schema.json").unlink()
        violations = audit_documentation_drift(self.root)
        self.assertEqual(len(violations), 1)
        v = violations[0]
        self.assertEqual(v.error_class, "MissingScenarioSchemaError")
        self.assertIn("Owner 28", v.conductor_owner)

        job_res = check_documentation_drift(self.root)
        self.assertEqual(job_res.status, JobStatus.FAILED)

    def test_dependency_groups_clean_passes(self) -> None:
        violations = audit_dependency_groups(self.root)
        self.assertEqual(len(violations), 0)
        job_res = check_dependency_groups(self.root)
        self.assertEqual(job_res.status, JobStatus.SUCCESS)

    def test_dependency_groups_detects_license_drift(self) -> None:
        (self.root / "Cargo.toml").write_text('[package]\nlicense = "GPL-3.0"\n', encoding="utf-8")
        violations = audit_dependency_groups(self.root)
        self.assertEqual(len(violations), 1)
        v = violations[0]
        self.assertEqual(v.error_class, "LicenseDeclarationDriftError")
        self.assertIn("Owner 42", v.conductor_owner)

    def test_dependency_groups_detects_missing_forbid_unsafe(self) -> None:
        (self.root / "crates/careops-ed-cli/src/main.rs").write_text("fn main() {}\n", encoding="utf-8")
        violations = audit_dependency_groups(self.root)
        self.assertEqual(len(violations), 1)
        v = violations[0]
        self.assertEqual(v.error_class, "UnsafeCodePolicyViolationError")
        self.assertIn("Owner 42", v.conductor_owner)

    def test_extended_tests_clean_passes(self) -> None:
        violations = audit_extended_tests(self.root)
        self.assertEqual(len(violations), 0)
        job_res = run_extended_tests(self.root)
        self.assertEqual(job_res.status, JobStatus.SUCCESS)

    def test_extended_tests_detects_missing_harness(self) -> None:
        with tempfile.TemporaryDirectory() as empty_dir:
            violations = audit_extended_tests(Path(empty_dir))
            self.assertEqual(len(violations), 1)
            v = violations[0]
            self.assertEqual(v.error_class, "MissingTestHarnessError")
            self.assertIn("Owner 27", v.conductor_owner)


if __name__ == "__main__":
    unittest.main()
