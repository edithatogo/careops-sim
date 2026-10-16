#!/usr/bin/env python3
"""Unit tests for Agent Governance, Scope Restriction, and Exact-Commit Merge Policy (Task D5.2).

Validates:
  1. Strict write scope restrictions and protected core path guards (ProtectedPathError, ScopeViolationError).
  2. Single-writer-per-owned-path invariant and race condition prevention (MultipleWritersError).
  3. Exact-commit merge verification against tested commit SHA (ExactCommitMismatchError).
  4. Autonomous publication prohibition and human authority assertion (AutonomousPublicationViolationError).
  5. Empirical held-out benchmark evaluation suite (6 curated tasks, 0% false-pass rate on safety defects).
"""

from __future__ import annotations

import os
from pathlib import Path
import unittest
from unittest.mock import patch

from tools.agent_governance import (
    AutonomousPublicationViolationError,
    ExactCommitMismatchError,
    MultipleWritersError,
    ProtectedPathError,
    ReservationRegistry,
    ScopeViolationError,
    assert_no_autonomous_publication,
    assert_not_protected_path,
    evaluate_task,
    get_curated_benchmark_tasks,
    normalize_path,
    path_overlaps,
    run_curated_benchmark,
    validate_write_scope,
    verify_exact_commit,
)


class TestPathNormalizationAndOverlaps(unittest.TestCase):
    """Test path normalization, security guards, and overlap detection."""

    def test_normalize_valid_relative_paths(self) -> None:
        self.assertEqual(normalize_path("tools/context.py"), "tools/context.py")
        self.assertEqual(normalize_path("crates/careops-ed"), "crates/careops-ed")

    def test_normalize_traversal_rejected(self) -> None:
        with self.assertRaises(ValueError):
            normalize_path("../secret.txt")
        with self.assertRaises(ValueError):
            normalize_path("tools/../../etc/passwd")

    def test_normalize_git_directory_rejected(self) -> None:
        with self.assertRaises(ValueError):
            normalize_path(".git/config")
        with self.assertRaises(ValueError):
            normalize_path("submodule/.git/HEAD")

    def test_path_overlaps_detection(self) -> None:
        self.assertTrue(path_overlaps("tools", "tools/context.py"))
        self.assertTrue(path_overlaps("tools/context.py", "tools"))
        self.assertTrue(path_overlaps("crates/careops-ed", "crates/careops-ed"))
        self.assertTrue(path_overlaps(".", "crates"))
        self.assertFalse(path_overlaps("tools", "crates"))
        self.assertFalse(path_overlaps("tools/context.py", "tools/agent_session.py"))


class TestWriteScopeAndProtectedPaths(unittest.TestCase):
    """Test write scope restrictions and protected paths enforcement."""

    def test_protected_paths_fail_closed(self) -> None:
        protected = [
            "crates/careops-ed/src/lib.rs",
            "conductor/delivery-contract.md",
            "LICENSE",
        ]
        for p in protected:
            with self.assertRaises(ProtectedPathError):
                assert_not_protected_path(p)

    def test_unprotected_paths_pass(self) -> None:
        assert_not_protected_path("tools/maintenance_scheduler.py")
        assert_not_protected_path("crates/careops-ed/src/engine.rs")
        assert_not_protected_path("conductor/dependency-policy.md")

    def test_validate_write_scope_within_reservation(self) -> None:
        validate_write_scope(
            reserved_paths=["tools"],
            modified_paths=["tools/maintenance_scheduler.py", "tools/context.py"],
        )

    def test_validate_write_scope_outside_reservation_raises(self) -> None:
        with self.assertRaises(ScopeViolationError):
            validate_write_scope(
                reserved_paths=["tools"],
                modified_paths=["conductor/dependency-policy.md"],
            )

    def test_validate_write_scope_protected_path_raises_protected_error(self) -> None:
        with self.assertRaises(ProtectedPathError):
            validate_write_scope(
                reserved_paths=["."],
                modified_paths=["crates/careops-ed/src/lib.rs"],
            )

    def test_validate_write_scope_empty_reservation_raises(self) -> None:
        with self.assertRaises(ScopeViolationError):
            validate_write_scope(
                reserved_paths=[],
                modified_paths=["tools/context.py"],
            )


class TestSingleWriterInvariant(unittest.TestCase):
    """Test single-writer-per-owned-path reservation manager."""

    def setUp(self) -> None:
        self.registry = ReservationRegistry()

    def test_disjoint_path_reservations_succeed(self) -> None:
        res1 = self.registry.reserve("owner_1", "task_1", ["tools"])
        res2 = self.registry.reserve("owner_2", "task_2", ["crates/careops-ed"])
        self.assertIsNotNone(res1)
        self.assertIsNotNone(res2)

    def test_overlapping_path_reservation_raises_multiple_writers_error(self) -> None:
        self.registry.reserve("owner_1", "task_1", ["tools"])
        with self.assertRaises(MultipleWritersError):
            self.registry.reserve("owner_2", "task_2", ["tools/agent_session.py"])

    def test_release_allows_subsequent_reservation(self) -> None:
        res1 = self.registry.reserve("owner_1", "task_1", ["tools"])
        self.registry.release(res1.reservation_id)
        # Should now succeed without conflict
        res2 = self.registry.reserve("owner_2", "task_2", ["tools"])
        self.assertIsNotNone(res2)


class TestExactCommitMergeVerification(unittest.TestCase):
    """Test exact-commit verification against tested commit SHA."""

    def test_exact_commit_match_passes(self) -> None:
        sha = "754beac4c4733448a985adb98ea7189700a737dc"
        self.assertTrue(verify_exact_commit(tested_commit_sha=sha, target_head_sha=sha, ci_checks_passed=True))

    def test_stale_commit_fails_closed(self) -> None:
        tested_sha = "1111111111111111111111111111111111111111"
        head_sha = "2222222222222222222222222222222222222222"
        with self.assertRaises(ExactCommitMismatchError):
            verify_exact_commit(tested_commit_sha=tested_sha, target_head_sha=head_sha, ci_checks_passed=True)

    def test_ci_failure_fails_closed(self) -> None:
        sha = "754beac4c4733448a985adb98ea7189700a737dc"
        with self.assertRaises(ExactCommitMismatchError):
            verify_exact_commit(tested_commit_sha=sha, target_head_sha=sha, ci_checks_passed=False)

    def test_reviewer_rejection_fails_closed(self) -> None:
        sha = "754beac4c4733448a985adb98ea7189700a737dc"
        with self.assertRaises(ExactCommitMismatchError):
            verify_exact_commit(
                tested_commit_sha=sha,
                target_head_sha=sha,
                ci_checks_passed=True,
                reviewer_verdict="hold_for_evidence",
            )


class TestAutonomousPublicationProhibition(unittest.TestCase):
    """Test autonomous publication prohibition and human authority override."""

    def test_forbidden_publish_commands_raise(self) -> None:
        forbidden = [
            "cargo publish",
            "git push origin main",
            "gh release create v0.1.0",
            "twine upload dist/*",
            "npm publish",
        ]
        for cmd in forbidden:
            with self.assertRaises(AutonomousPublicationViolationError):
                assert_no_autonomous_publication(cmd)

    def test_allowed_commands_pass(self) -> None:
        assert_no_autonomous_publication("cargo test --locked")
        assert_no_autonomous_publication("git status")
        assert_no_autonomous_publication("python3 tools/context.py check")

    def test_human_authority_env_permits_command(self) -> None:
        with patch.dict(os.environ, {"CAREOPS_HUMAN_MAINTAINER_AUTHORITY": "1"}):
            # Should not raise when human maintainer authority is explicitly set
            assert_no_autonomous_publication("cargo publish")


class TestCuratedBenchmarkSuite(unittest.TestCase):
    """Test execution of the 6 curated held-out benchmark tasks."""

    def test_benchmark_tasks_count_and_completeness(self) -> None:
        tasks = get_curated_benchmark_tasks()
        self.assertEqual(len(tasks), 6)
        task_ids = [t.task_id for t in tasks]
        self.assertEqual(task_ids, ["BT-01", "BT-02", "BT-03", "BT-04", "BT-05", "BT-06"])

    def test_run_curated_benchmark_metrics(self) -> None:
        ledger = run_curated_benchmark()
        self.assertEqual(ledger.total_tasks, 6)
        self.assertEqual(ledger.passed_tasks, 6)
        self.assertEqual(ledger.accuracy_rate, 1.0)
        self.assertEqual(ledger.false_pass_count, 0)
        self.assertEqual(ledger.false_pass_rate_on_safety, 0.0)
        self.assertEqual(ledger.false_reject_count, 0)
        self.assertEqual(ledger.human_interventions, 0)
        self.assertEqual(ledger.exact_commit_verification_rate, 1.0)
        self.assertGreater(ledger.total_tokens, 10000)
        self.assertGreater(ledger.total_cost_usd, 0.0)


if __name__ == "__main__":
    unittest.main()
