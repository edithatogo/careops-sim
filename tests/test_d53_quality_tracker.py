#!/usr/bin/env python3
"""Comprehensive test suite for Quality Debt, Flaky Test Quarantine, and Performance Drift Tracker (Task D5.3).

Validates:
  1. Exception hierarchy and inheritance from QualityTrackingError.
  2. Dataclasses & Models (DebtMetrics, FlakeRecord, PerformanceSnapshot, AutomationROI, WorkflowRollbackReceipt).
  3. Quality debt scanning (debt markers, compiler warnings, coverage deficits, composite scoring).
  4. Flaky test quarantine audit (expiry checking, quarantine enforcement, UnquarantinedFlakeError).
  5. Performance drift audit (throughput slowdown, memory growth, PerformanceRegressionError).
  6. Automation ROI evaluation and verdict classification (beneficial, neutral, burdensome).
  7. Evidence-weakening workflow rollback and receipt generation with state hashes.
  8. Autonomous publication prohibition (git push, cargo publish, npm publish).
  9. CLI subcommands (debt, flakes, performance, roi, rollback) and exit codes.
"""

from __future__ import annotations

from datetime import date, datetime, timezone
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

from tools.quality_tracker import (
    AutonomousPublicationError,
    AutomationROI,
    DebtMetrics,
    FlakeRecord,
    PerformanceRegressionError,
    PerformanceSnapshot,
    QualityDebtExceededError,
    QualityTrackingError,
    UnquarantinedFlakeError,
    WorkflowEvidenceWeakenedError,
    WorkflowRollbackReceipt,
    assert_no_autonomous_publication,
    assert_workflow_evidence_integrity,
    audit_flaky_tests,
    audit_performance_drift,
    audit_quality_debt,
    build_parser,
    evaluate_automation_roi,
    main,
    rollback_weakened_workflow,
)


class TestD53ExceptionHierarchy(unittest.TestCase):
    """Test 1: Verify all custom exceptions inherit from QualityTrackingError."""

    def test_exception_inheritance(self) -> None:
        self.assertTrue(issubclass(QualityDebtExceededError, QualityTrackingError))
        self.assertTrue(issubclass(UnquarantinedFlakeError, QualityTrackingError))
        self.assertTrue(issubclass(PerformanceRegressionError, QualityTrackingError))
        self.assertTrue(issubclass(WorkflowEvidenceWeakenedError, QualityTrackingError))
        self.assertTrue(issubclass(AutonomousPublicationError, QualityTrackingError))
        self.assertTrue(issubclass(QualityTrackingError, Exception))


class TestD53DataModels(unittest.TestCase):
    """Test 2: Verify dataclasses, fields, serialization and deserialization."""

    def test_debt_metrics_model(self) -> None:
        metrics = DebtMetrics(
            todo_markers_count=5,
            warning_count=2,
            uncovered_lines=10,
            debt_score=35.0,
            status="acceptable",
            details=["Score within threshold"],
        )
        data = metrics.to_dict()
        self.assertEqual(data["todo_markers_count"], 5)
        self.assertEqual(data["warning_count"], 2)
        self.assertEqual(data["uncovered_lines"], 10)
        self.assertEqual(data["debt_score"], 35.0)
        self.assertEqual(data["status"], "acceptable")
        self.assertEqual(data["details"], ["Score within threshold"])

    def test_flake_record_model(self) -> None:
        rec = FlakeRecord(
            test_id="crate::test_sim_race",
            owner="Track25 / ED Team",
            reason="Non-deterministic scheduling race",
            issue_url="https://github.com/careops/sim/issues/101",
            expires_on="2026-11-01",
            is_expired=False,
            is_quarantined=True,
        )
        data = rec.to_dict()
        self.assertEqual(data["test_id"], "crate::test_sim_race")
        self.assertFalse(data["is_expired"])
        self.assertTrue(data["is_quarantined"])

    def test_performance_snapshot_roundtrip(self) -> None:
        snap = PerformanceSnapshot(
            benchmark_id="ed_benchmarks::step_10k",
            throughput_steps_per_sec=12500.5,
            memory_rss_bytes=104857600,
            elapsed_seconds=0.8,
            timestamp_utc="2026-10-17T00:00:00Z",
            drift_percent=-2.5,
        )
        data = snap.to_dict()
        restored = PerformanceSnapshot.from_dict(data)
        self.assertEqual(restored.benchmark_id, snap.benchmark_id)
        self.assertEqual(restored.throughput_steps_per_sec, snap.throughput_steps_per_sec)
        self.assertEqual(restored.memory_rss_bytes, snap.memory_rss_bytes)
        self.assertEqual(restored.elapsed_seconds, snap.elapsed_seconds)
        self.assertEqual(restored.timestamp_utc, snap.timestamp_utc)
        self.assertEqual(restored.drift_percent, snap.drift_percent)

    def test_automation_roi_model(self) -> None:
        roi = AutomationROI(
            workflow_name="cargo_deps_freshness",
            hours_saved=8.0,
            triage_hours_spent=1.0,
            intervention_count=1,
            false_alerts_count=0,
            efficiency_ratio=5.3333,
            verdict="beneficial",
        )
        data = roi.to_dict()
        self.assertEqual(data["workflow_name"], "cargo_deps_freshness")
        self.assertEqual(data["verdict"], "beneficial")

    def test_workflow_rollback_receipt_model(self) -> None:
        receipt = WorkflowRollbackReceipt(
            workflow_name="auto_flaky_retry",
            rolled_back_at_utc="2026-10-17T01:00:00Z",
            rolled_back_by="lead_maintainer",
            reason="Retries weaken test evidence without quarantine",
            evidence_weakening_details="Automated retries mask non-deterministic races",
            state_hash="a" * 64,
        )
        data = receipt.to_dict()
        self.assertEqual(data["workflow_name"], "auto_flaky_retry")
        self.assertEqual(len(data["state_hash"]), 64)


class TestD53QualityDebtAudit(unittest.TestCase):
    """Test 3: Verify audit_quality_debt scanning, formulas, and threshold checking."""

    def test_audit_debt_clean_repo(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            # Create a clean file
            clean_file = tmppath / "clean.rs"
            clean_file.write_text("pub fn compute() -> i32 { 42 }\n", encoding="utf-8")

            metrics = audit_quality_debt(tmppath, max_score=50.0, warning_count=0, uncovered_lines=0)
            self.assertEqual(metrics.todo_markers_count, 0)
            self.assertEqual(metrics.warning_count, 0)
            self.assertEqual(metrics.uncovered_lines, 0)
            self.assertEqual(metrics.debt_score, 0.0)
            self.assertEqual(metrics.status, "acceptable")

    def test_audit_debt_formula_calculation(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            # Write file with markers: TODO, FIXME, HACK, DEPRECATED
            debt_file = tmppath / "debt.py"
            debt_file.write_text(
                "# " + "TODO: implement step 1\n"
                "# " + "FIXME: handle edge case\n"
                "# " + "HACK: temporary workaround\n"
                "# " + "DEPRECATED: legacy api\n",
                encoding="utf-8",
            )
            # todo_markers_count = 4
            # warning_count = 2 -> 2 * 5.0 = 10.0
            # uncovered_lines = 5 -> 5 * 2.0 = 10.0
            # debt_score = 4 + 10 + 10 = 24.0
            metrics = audit_quality_debt(
                tmppath,
                max_score=50.0,
                warning_count=2,
                uncovered_lines=5,
            )
            self.assertEqual(metrics.todo_markers_count, 4)
            self.assertEqual(metrics.warning_count, 2)
            self.assertEqual(metrics.uncovered_lines, 5)
            self.assertEqual(metrics.debt_score, 24.0)
            self.assertEqual(metrics.status, "acceptable")

    def test_audit_debt_exceeded_error_raised(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            debt_file = tmppath / "heavy.rs"
            debt_file.write_text(
                "// " + "TODO: task a\n// " + "TODO: task b\n// " + "TODO: task c\n",
                encoding="utf-8",
            )
            # debt_score = 3.0 * 1.0 = 3.0. Max threshold = 2.0 -> Exceeded!
            with self.assertRaises(QualityDebtExceededError):
                audit_quality_debt(tmppath, max_score=2.0, warning_count=0, uncovered_lines=0, raise_on_error=True)

    def test_audit_debt_reads_warnings_and_coverage_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            warnings_path = tmppath / "warnings.json"
            warnings_path.write_text(json.dumps([{"code": "W01"}, {"code": "W02"}, {"code": "W03"}]), encoding="utf-8")

            coverage_path = tmppath / "coverage.json"
            coverage_path.write_text(json.dumps({"uncovered_lines": 8}), encoding="utf-8")

            metrics = audit_quality_debt(
                tmppath,
                max_score=100.0,
                warnings_file=warnings_path,
                coverage_file=coverage_path,
            )
            self.assertEqual(metrics.warning_count, 3)
            self.assertEqual(metrics.uncovered_lines, 8)
            # warnings: 3*5.0=15, uncovered: 8*2.0=16 -> debt_score = 31.0
            self.assertEqual(metrics.debt_score, 31.0)


class TestD53FlakyTestAudit(unittest.TestCase):
    """Test 4: Verify audit_flaky_tests quarantine validation and expiry checking."""

    def test_valid_quarantined_flake(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            flake_cfg = {
                "schema_version": 1,
                "flakes": [
                    {
                        "test_id": "test_ed_capacity_stress",
                        "owner": "Owner 12 (ED Engine)",
                        "reason": "Queue contention under CPU starvation",
                        "issue_url": "https://github.com/careops/sim/issues/501",
                        "expires_on": "2026-12-31",
                    }
                ],
            }
            records = audit_flaky_tests(
                tmppath,
                flake_config=flake_cfg,
                today="2026-10-17",
                raise_on_error=True,
            )
            self.assertEqual(len(records), 1)
            self.assertEqual(records[0].test_id, "test_ed_capacity_stress")
            self.assertFalse(records[0].is_expired)
            self.assertTrue(records[0].is_quarantined)

    def test_expired_flake_raises_unquarantined_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            flake_cfg = {
                "schema_version": 1,
                "flakes": [
                    {
                        "test_id": "test_des_expired",
                        "owner": "Owner 25",
                        "reason": "Old flake exception",
                        "issue_url": "https://github.com/careops/sim/issues/99",
                        "expires_on": "2026-09-01",  # Expired relative to 2026-10-17
                    }
                ],
            }
            with self.assertRaises(UnquarantinedFlakeError):
                audit_flaky_tests(
                    tmppath,
                    flake_config=flake_cfg,
                    today="2026-10-17",
                    raise_on_error=True,
                )

    def test_unquarantined_flake_flagged(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            flake_cfg = {
                "schema_version": 1,
                "flakes": [
                    {
                        "test_id": "test_unquarantined_active",
                        "owner": "Owner 05",
                        "reason": "Flaking in main suite",
                        "issue_url": "https://github.com/careops/sim/issues/22",
                        "expires_on": "2026-12-01",
                        "is_quarantined": False,
                    }
                ],
            }
            with self.assertRaises(UnquarantinedFlakeError):
                audit_flaky_tests(
                    tmppath,
                    flake_config=flake_cfg,
                    today="2026-10-17",
                    raise_on_error=True,
                )


class TestD53PerformanceDriftAudit(unittest.TestCase):
    """Test 5: Verify audit_performance_drift throughput slowdown and memory regression checks."""

    def setUp(self) -> None:
        self.baseline = PerformanceSnapshot(
            benchmark_id="ed_steps",
            throughput_steps_per_sec=10000.0,
            memory_rss_bytes=100000000,
            elapsed_seconds=1.0,
            timestamp_utc="2026-10-01T00:00:00Z",
        )

    def test_performance_within_tolerance(self) -> None:
        current = PerformanceSnapshot(
            benchmark_id="ed_steps",
            throughput_steps_per_sec=9500.0,  # 5% slowdown <= 15% tolerance
            memory_rss_bytes=105000000,       # 5% memory growth <= 50% tolerance
            elapsed_seconds=1.05,
            timestamp_utc="2026-10-17T00:00:00Z",
        )
        res = audit_performance_drift(self.baseline, current, max_slowdown_percent=15.0, raise_on_error=True)
        self.assertFalse(res["is_regression"])
        self.assertEqual(res["status"], "passed")
        self.assertEqual(res["slowdown_percent"], 5.0)

    def test_throughput_slowdown_regression(self) -> None:
        current = PerformanceSnapshot(
            benchmark_id="ed_steps",
            throughput_steps_per_sec=8000.0,  # 20% slowdown > 15% tolerance
            memory_rss_bytes=100000000,
            elapsed_seconds=1.25,
            timestamp_utc="2026-10-17T00:00:00Z",
        )
        with self.assertRaises(PerformanceRegressionError):
            audit_performance_drift(self.baseline, current, max_slowdown_percent=15.0, raise_on_error=True)

    def test_memory_leak_regression(self) -> None:
        current = PerformanceSnapshot(
            benchmark_id="ed_steps",
            throughput_steps_per_sec=9800.0,  # throughput ok
            memory_rss_bytes=220000000,       # 120% memory growth > 50% tolerance
            elapsed_seconds=1.02,
            timestamp_utc="2026-10-17T00:00:00Z",
        )
        with self.assertRaises(PerformanceRegressionError):
            audit_performance_drift(
                self.baseline,
                current,
                max_slowdown_percent=15.0,
                max_memory_growth_percent=50.0,
                raise_on_error=True,
            )


class TestD53AutomationROI(unittest.TestCase):
    """Test 6: Verify evaluate_automation_roi calculation and verdict classification."""

    def test_beneficial_workflow(self) -> None:
        # saved=10, triage=1, interv=1 -> denominator = 1 + 0.5 = 1.5 -> ratio = 6.67 >= 2.0, alerts=1 <= 2 -> beneficial
        roi = evaluate_automation_roi("cargo_update_bot", hours_saved=10.0, triage_hours=1.0, interventions=1, false_alerts=1)
        self.assertEqual(roi.verdict, "beneficial")
        self.assertGreaterEqual(roi.efficiency_ratio, 2.0)

    def test_neutral_workflow(self) -> None:
        # saved=3, triage=1.5, interv=1 -> denominator = 1.5 + 0.5 = 2.0 -> ratio = 1.5 -> neutral
        roi = evaluate_automation_roi("doc_link_checker", hours_saved=3.0, triage_hours=1.5, interventions=1, false_alerts=1)
        self.assertEqual(roi.verdict, "neutral")

    def test_burdensome_workflow_low_ratio(self) -> None:
        # saved=1, triage=2, interv=2 -> denominator = 2 + 1.0 = 3.0 -> ratio = 0.33 < 1.0 -> burdensome
        roi = evaluate_automation_roi("noisy_linter", hours_saved=1.0, triage_hours=2.0, interventions=2, false_alerts=1)
        self.assertEqual(roi.verdict, "burdensome")

    def test_burdensome_workflow_high_false_alerts(self) -> None:
        # ratio is high (10/1 = 10.0) but false_alerts = 6 > 5 -> burdensome
        roi = evaluate_automation_roi("flaky_test_alerter", hours_saved=10.0, triage_hours=0.5, interventions=1, false_alerts=6)
        self.assertEqual(roi.verdict, "burdensome")


class TestD53WorkflowRollbackAndGuard(unittest.TestCase):
    """Test 7: Verify evidence-weakening workflow rollback, receipt generation, and publication guard."""

    def test_prohibited_publication_guard(self) -> None:
        with self.assertRaises(AutonomousPublicationError):
            assert_no_autonomous_publication("git push origin main")

        with self.assertRaises(AutonomousPublicationError):
            assert_no_autonomous_publication("cargo publish --token xxx")

        with self.assertRaises(AutonomousPublicationError):
            assert_no_autonomous_publication("npm publish")

        with self.assertRaises(AutonomousPublicationError):
            assert_no_autonomous_publication("twine upload dist/*")

    def test_rollback_weakened_workflow(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            evidence_dir = tmppath / "evidence"
            config_file = tmppath / "workflows.json"
            config_file.write_text(
                json.dumps({
                    "workflows": {
                        "auto_retry_suite": {"enabled": True, "status": "active"}
                    }
                }),
                encoding="utf-8",
            )

            receipt = rollback_weakened_workflow(
                workflow_name="auto_retry_suite",
                reason="Workflows weakening evidence by retrying failures without quarantine",
                user="coordinator",
                evidence_dir=evidence_dir,
                workflows_config_path=config_file,
            )

            self.assertEqual(receipt.workflow_name, "auto_retry_suite")
            self.assertEqual(receipt.rolled_back_by, "coordinator")
            self.assertTrue(len(receipt.state_hash) == 64)

            # Check workflow was disabled in config file
            updated_cfg = json.loads(config_file.read_text(encoding="utf-8"))
            self.assertFalse(updated_cfg["workflows"]["auto_retry_suite"]["enabled"])
            self.assertEqual(updated_cfg["workflows"]["auto_retry_suite"]["status"], "disabled")

            # Check immutable receipt was written
            receipt_file = evidence_dir / "rollback_auto_retry_suite.json"
            self.assertTrue(receipt_file.exists())
            loaded_receipt = json.loads(receipt_file.read_text(encoding="utf-8"))
            self.assertEqual(loaded_receipt["state_hash"], receipt.state_hash)

    def test_rollback_blocks_publication_attempt(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            with self.assertRaises(AutonomousPublicationError):
                rollback_weakened_workflow(
                    workflow_name="git push workflow",
                    reason="Attempts publication",
                    evidence_dir=tmppath,
                )

    def test_assert_workflow_evidence_integrity(self) -> None:
        # Legitimate actions pass
        assert_workflow_evidence_integrity("ci_check", ["cargo check", "cargo test --all-targets"])

        # Evidence weakening actions fail
        with self.assertRaises(WorkflowEvidenceWeakenedError):
            assert_workflow_evidence_integrity("flaky_masker", ["pytest --continue-on-error: true"])

        with self.assertRaises(WorkflowEvidenceWeakenedError):
            assert_workflow_evidence_integrity("coverage_reducer", ["lower coverage floor to 50%"])


class TestD53CommandLineInterface(unittest.TestCase):
    """Test 8: Verify CLI subcommands."""

    def test_cli_debt_subcommand(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            (tmppath / "code.rs").write_text("fn test() {}\n", encoding="utf-8")
            output_json = tmppath / "out.json"

            code = main(["debt", "--repo-root", str(tmppath), "--max-score", "50.0", "--output", str(output_json)])
            self.assertEqual(code, 0)
            self.assertTrue(output_json.exists())

    def test_cli_performance_subcommand(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            b_file = tmppath / "base.json"
            c_file = tmppath / "curr.json"

            base_snap = PerformanceSnapshot("ed_bench", 1000.0, 50000000, 1.0, "2026-10-01T00:00:00Z")
            curr_snap = PerformanceSnapshot("ed_bench", 950.0, 52000000, 1.05, "2026-10-17T00:00:00Z")

            b_file.write_text(json.dumps(base_snap.to_dict()), encoding="utf-8")
            c_file.write_text(json.dumps(curr_snap.to_dict()), encoding="utf-8")

            code = main(["performance", "--baseline", str(b_file), "--current", str(c_file), "--max-slowdown", "15.0"])
            self.assertEqual(code, 0)

    def test_cli_roi_subcommand(self) -> None:
        code = main([
            "roi",
            "--workflow", "extended_tests",
            "--saved", "12.0",
            "--triage", "1.0",
            "--interventions", "1",
            "--false-alerts", "0",
        ])
        self.assertEqual(code, 0)

    def test_cli_rollback_subcommand(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            evidence_dir = tmppath / "evidence"

            code = main([
                "rollback",
                "--workflow", "broken_automation",
                "--reason", "Disables unit test assertions",
                "--user", "qa_maintainer",
                "--evidence-dir", str(evidence_dir),
            ])
            self.assertEqual(code, 0)
            self.assertTrue((evidence_dir / "rollback_broken_automation.json").exists())


if __name__ == "__main__":
    unittest.main()
