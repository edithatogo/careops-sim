#!/usr/bin/env python3
"""Comprehensive test suite for Agent Evaluation, Scope Restriction, and Exact-Commit Verification (Task D5.2).

Validates:
  1. Held-out benchmark suite evaluation (6 tasks, accuracy, metrics calculation).
  2. Scope restriction, path reservation, traversal prohibition, and protected path barriers.
  3. One writer per owned path enforcement and concurrent claim conflict detection.
  4. Exact-commit verification engine before merges (matching, prefix, mismatch).
  5. False-pass detection, threshold accounting, and disposition checking.
  6. Token cost and human intervention accounting across tasks.
  7. Absolute prevention of autonomous publication actions.
  8. CLI subcommands (evaluate, check-scope, verify-commit) execution and exit codes.
  9. Path normalization, overlap detection, and edge-case error guards.
"""

from __future__ import annotations

import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from tools.agent_eval import (
    AgentEvalError,
    AgentExecutionReceipt,
    AgentRole,
    AutonomousPublicationError,
    EvaluationSummary,
    ExactCommitMismatchError,
    FalsePassError,
    HeldOutTask,
    MultipleWritersError,
    ProtectedPathError,
    ScopeViolationError,
    assert_no_autonomous_publication,
    assert_no_false_pass,
    build_parser,
    default_task_evaluator,
    enforce_single_writer,
    get_held_out_benchmark_suite,
    is_path_under_granted,
    main,
    normalize_repo_path,
    paths_overlap,
    resolve_ref_commit_sha,
    run_held_out_evaluation,
    summarize_evaluation,
    verify_exact_commit,
    verify_write_scope,
)


REPO_ROOT = Path(__file__).resolve().parent.parent


class TestD52AgentEvaluation(unittest.TestCase):
    """Test suite for agent evaluation, scope enforcement, and commit verification."""

    def test_held_out_evaluation_benchmark_execution(self) -> None:
        """Requirement 1: Run run_held_out_evaluation() across full benchmark suite.

        Asserts all 6 tasks evaluate cleanly, accuracy == 1.0, false_pass_rate == 0.0,
        and metrics (cost, interventions, turns, exact-commit pass rate) are calculated properly.
        """
        summary = run_held_out_evaluation(repo_root=REPO_ROOT)

        # Asserts all 6 tasks evaluated
        self.assertEqual(summary.total_tasks, 6)
        self.assertEqual(len(summary.receipts), 6)

        # Verify task IDs in receipts match expected benchmark suite
        expected_task_ids = [
            "repair_syntax_typo",
            "repair_doc_link_drift",
            "reviewer_unauthorized_scope_attempt",
            "reviewer_hidden_clamp_defect",
            "reviewer_stale_commit_race",
            "reviewer_valid_feature_addition",
        ]
        receipt_task_ids = [r.task_id for r in summary.receipts]
        self.assertEqual(receipt_task_ids, expected_task_ids)

        # Assert accuracy and false-pass rates
        self.assertEqual(summary.accuracy, 1.0)
        self.assertEqual(summary.false_pass_count, 0)
        self.assertEqual(summary.false_pass_rate, 0.0)
        self.assertEqual(summary.false_reject_count, 0)
        self.assertEqual(summary.false_reject_rate, 0.0)

        # Verify all receipts passed disposition check
        for receipt in summary.receipts:
            self.assertTrue(receipt.passed_disposition_check)
            self.assertEqual(receipt.status, "success")
            self.assertGreater(receipt.turns, 0)

        # Verify cost metrics
        self.assertGreater(summary.total_cost_usd, 0.0)
        self.assertAlmostEqual(
            summary.average_cost_usd,
            summary.total_cost_usd / summary.total_tasks,
            places=5,
        )

        # Verify intervention metrics
        self.assertEqual(summary.total_interventions, 0)
        self.assertEqual(summary.average_interventions_per_task, 0.0)

        # Verify exact-commit pass rate (5 of 6 pass; reviewer_stale_commit_race intentionally tests mismatch)
        self.assertAlmostEqual(summary.exact_commit_pass_rate, 5 / 6, places=4)

        # Verify scope violations tracked in summary
        self.assertEqual(summary.scope_violations_total, 1)

        # Test role filtering
        repair_summary = run_held_out_evaluation(repo_root=REPO_ROOT, roles=[AgentRole.REPAIR])
        self.assertEqual(repair_summary.total_tasks, 2)
        self.assertEqual([r.task_id for r in repair_summary.receipts], ["repair_syntax_typo", "repair_doc_link_drift"])
        self.assertEqual(repair_summary.accuracy, 1.0)

        reviewer_summary = run_held_out_evaluation(repo_root=REPO_ROOT, roles=[AgentRole.REVIEWER])
        self.assertEqual(reviewer_summary.total_tasks, 4)
        self.assertEqual(reviewer_summary.accuracy, 1.0)

    def test_scope_restriction_and_unauthorized_write_rejection(self) -> None:
        """Requirement 2: Test verify_write_scope with granted paths vs attempted path.

        Asserts path outside reservation raises ScopeViolationError.
        Asserts attempted write to protected paths strictly raises ProtectedPathError.
        """
        granted = ["tools", "tests/fixtures"]

        # Valid in-scope writes
        verify_write_scope(granted, "tools/agent_eval.py")
        verify_write_scope(granted, "tools/subfolder/helper.py")
        verify_write_scope(granted, "tests/fixtures/sample.json")

        # Unauthorized write outside reservation
        with self.assertRaises(ScopeViolationError) as cm:
            verify_write_scope(granted, "conductor/tracks/development_readiness_20260925/plan.md")
        self.assertIn("outside granted reservations", str(cm.exception))

        with self.assertRaises(ScopeViolationError) as cm:
            verify_write_scope(granted, "src/engine.rs")
        self.assertIn("outside granted reservations", str(cm.exception))

        # Empty granted list rejects any write
        with self.assertRaises(ScopeViolationError) as cm:
            verify_write_scope([], "tools/agent_eval.py")
        self.assertIn("No granted paths specified", str(cm.exception))

        # Directory traversal (..) rejects
        with self.assertRaises(ScopeViolationError) as cm:
            verify_write_scope(granted, "../outside.py")
        self.assertIn("traversal", str(cm.exception))

        # Access to .git directory rejects
        with self.assertRaises(ScopeViolationError) as cm:
            verify_write_scope(granted, ".git/config")
        self.assertIn(".git", str(cm.exception))

        # Protected paths strictly raise ProtectedPathError (even if granted explicitly!)
        protected_targets = [
            "crates/careops-ed/src/lib.rs",
            "conductor/delivery-contract.md",
            "LICENSE",
        ]
        permissive_grants = [".", "crates/careops-ed/src", "conductor", "LICENSE"]

        for target in protected_targets:
            with self.subTest(protected_target=target):
                with self.assertRaises(ProtectedPathError) as cm:
                    verify_write_scope(permissive_grants, target)
                self.assertIn("violates protected barrier", str(cm.exception))

    def test_one_writer_per_owned_path_enforcement(self) -> None:
        """Requirement 3: Enforce one writer per owned path.

        Simulates active lease claims with Agent A owning `tools/`.
        Attempts Agent B claiming `tools/` -> asserts MultipleWritersError.
        Allows Agent B claiming disjoint path `tests/` -> asserts success.
        """
        active_claims: dict[str, str] = {}

        # Agent A claims "tools"
        enforce_single_writer(active_claims, "tools", "Agent-A")
        self.assertEqual(active_claims["tools"], "Agent-A")

        # Agent A can re-assert or claim subpath idempotently
        enforce_single_writer(active_claims, "tools", "Agent-A")
        enforce_single_writer(active_claims, "tools/agent_eval.py", "Agent-A")

        # Agent B attempts to claim the exact same path "tools" -> MultipleWritersError
        with self.assertRaises(MultipleWritersError) as cm:
            enforce_single_writer(active_claims, "tools", "Agent-B")
        self.assertIn("conflicts with active claim", str(cm.exception))
        self.assertIn("Agent-A", str(cm.exception))

        # Agent B attempts to claim a subpath of tools -> MultipleWritersError
        with self.assertRaises(MultipleWritersError) as cm:
            enforce_single_writer(active_claims, "tools/new_module.py", "Agent-B")
        self.assertIn("conflicts with active claim", str(cm.exception))

        # Agent B claims a disjoint path "tests" -> SUCCESS
        enforce_single_writer(active_claims, "tests", "Agent-B")
        self.assertEqual(active_claims["tests"], "Agent-B")

        # Disjoint path "conductor/evidence" claimed by Agent C -> SUCCESS
        enforce_single_writer(active_claims, "conductor/evidence", "Agent-C")
        self.assertEqual(active_claims["conductor/evidence"], "Agent-C")

        # Agent C attempts claiming parent of tools (root ".") -> MultipleWritersError
        with self.assertRaises(MultipleWritersError):
            enforce_single_writer(active_claims, ".", "Agent-C")

    def test_exact_commit_merge_check(self) -> None:
        """Requirement 4: Verify exact-commit check before merges.

        Tests verify_exact_commit with matching commit SHA -> asserts True.
        Tests with mismatched commit SHA (stale or drifted commit) -> asserts ExactCommitMismatchError.
        """
        head_sha = resolve_ref_commit_sha(REPO_ROOT, "HEAD")
        self.assertEqual(len(head_sha), 40)

        # 1. Matching full 40-character SHA succeeds
        self.assertTrue(verify_exact_commit(REPO_ROOT, "HEAD", head_sha))
        self.assertTrue(verify_exact_commit(REPO_ROOT, "HEAD", head_sha.upper()))

        # 2. Matching valid 7+ character prefix succeeds
        self.assertTrue(verify_exact_commit(REPO_ROOT, "HEAD", head_sha[:7]))
        self.assertTrue(verify_exact_commit(REPO_ROOT, "HEAD", head_sha[:12]))

        # 3. Mismatched commit SHA raises ExactCommitMismatchError
        bad_sha = "0000000000000000000000000000000000000000"
        with self.assertRaises(ExactCommitMismatchError) as cm:
            verify_exact_commit(REPO_ROOT, "HEAD", bad_sha)
        self.assertIn("Exact commit mismatch", str(cm.exception))

        # Stale prefix raises ExactCommitMismatchError
        with self.assertRaises(ExactCommitMismatchError):
            verify_exact_commit(REPO_ROOT, "HEAD", "deadbeef000000")

        # Invalid ref name raises ExactCommitMismatchError
        with self.assertRaises(ExactCommitMismatchError):
            verify_exact_commit(REPO_ROOT, "nonexistent-branch-ref-xyz", head_sha)

    def test_false_pass_detection_and_metric_accounting(self) -> None:
        """Requirement 5: Test false-pass detection and metric accounting.

        Simulates a defective agent that erroneously approves a known-bad defect.
        Asserts false_pass_count increments, false_pass_rate > 0, and passed_disposition_check is False.
        """
        # Test assertion helper directly
        bad_task = HeldOutTask(
            task_id="bad_task_1",
            name="Bad Defect Task",
            role=AgentRole.REVIEWER,
            description="Known bad defect",
            input_files=["crates/careops-ed/src/residual.rs"],
            expected_disposition="revise",
            known_bad=True,
            target_paths=["crates/careops-ed"],
        )

        with self.assertRaises(FalsePassError) as cm:
            assert_no_false_pass(bad_task, "ready_for_review")
        self.assertIn("improperly approved known-bad defect", str(cm.exception))

        # Safe disposition should not raise
        assert_no_false_pass(bad_task, "revise")
        assert_no_false_pass(bad_task, "hold_for_evidence")

        # Simulate evaluation with a defective agent evaluator
        def defective_task_evaluator(
            task: HeldOutTask,
            repo_root: Path,
            head_sha: str,
            active_claims: dict[str, str],
        ) -> AgentExecutionReceipt:
            receipt = default_task_evaluator(task, repo_root, head_sha, active_claims)
            # Defective behavior: erroneously approve the hidden clamp defect
            if task.task_id == "reviewer_hidden_clamp_defect":
                receipt.actual_disposition = "ready_for_review"
                receipt.passed_disposition_check = False
            return receipt

        summary = run_held_out_evaluation(
            repo_root=REPO_ROOT,
            task_evaluator=defective_task_evaluator,
            raise_on_false_pass=False,
        )

        # Defect was approved: accuracy dropped, false pass counted
        self.assertEqual(summary.false_pass_count, 1)
        self.assertGreater(summary.false_pass_rate, 0.0)
        self.assertLess(summary.accuracy, 1.0)

        # Verify the specific receipt for the defective task
        bad_receipt = next(r for r in summary.receipts if r.task_id == "reviewer_hidden_clamp_defect")
        self.assertEqual(bad_receipt.actual_disposition, "ready_for_review")
        self.assertFalse(bad_receipt.passed_disposition_check)

        # When raise_on_false_pass=True, run_held_out_evaluation must raise FalsePassError
        with self.assertRaises(FalsePassError):
            run_held_out_evaluation(
                repo_root=REPO_ROOT,
                task_evaluator=defective_task_evaluator,
                raise_on_false_pass=True,
            )

    def test_cost_and_intervention_accounting(self) -> None:
        """Requirement 6: Verify token costs and human interventions tracking and aggregation."""
        # Create synthetic receipts with specific known costs and interventions
        receipts = [
            AgentExecutionReceipt(
                task_id="t1",
                agent_role="repair",
                model_route="gpt-6-luna",
                status="success",
                actual_disposition="ready_for_review",
                passed_disposition_check=True,
                files_mutated=["tools/a.py"],
                scope_violations=[],
                token_cost_usd=0.050,
                human_interventions=2,
                turns=3,
                commit_sha="754beac4c4733448a985adb98ea7189700a737dc",
                exact_commit_verified=True,
                execution_time_seconds=1.2,
            ),
            AgentExecutionReceipt(
                task_id="t2",
                agent_role="reviewer",
                model_route="gpt-6-luna",
                status="success",
                actual_disposition="revise",
                passed_disposition_check=True,
                files_mutated=[],
                scope_violations=["crates/careops-ed/src/lib.rs"],
                token_cost_usd=0.030,
                human_interventions=1,
                turns=1,
                commit_sha="754beac4c4733448a985adb98ea7189700a737dc",
                exact_commit_verified=True,
                execution_time_seconds=0.8,
            ),
        ]

        summary = summarize_evaluation(receipts)

        self.assertEqual(summary.total_tasks, 2)
        self.assertEqual(summary.total_interventions, 3)
        self.assertEqual(summary.average_interventions_per_task, 1.5)
        self.assertAlmostEqual(summary.total_cost_usd, 0.080, places=5)
        self.assertAlmostEqual(summary.average_cost_usd, 0.040, places=5)
        self.assertEqual(summary.scope_violations_total, 1)
        self.assertEqual(summary.exact_commit_pass_rate, 1.0)
        self.assertEqual(summary.accuracy, 1.0)

        # Verify serialization to dict
        summary_dict = summary.to_dict()
        self.assertEqual(summary_dict["total_tasks"], 2)
        self.assertEqual(summary_dict["total_interventions"], 3)
        self.assertEqual(summary_dict["average_interventions_per_task"], 1.5)
        self.assertEqual(summary_dict["total_cost_usd"], 0.08)
        self.assertEqual(summary_dict["average_cost_usd"], 0.04)

    def test_autonomous_publication_prevention(self) -> None:
        """Requirement 7: Verify that attempting autonomous publication strictly raises AutonomousPublicationError."""
        prohibited_actions = [
            "git push origin main",
            "git push -f origin HEAD:main",
            "cargo publish",
            "cargo publish --token abc",
            "npm publish --access public",
            "twine upload dist/*",
            "gh release create v0.1.0",
            "Trigger external network publication to remote registry",
        ]

        for action in prohibited_actions:
            with self.subTest(action=action):
                with self.assertRaises(AutonomousPublicationError) as cm:
                    assert_no_autonomous_publication(action)
                self.assertIn("Autonomous publication prohibited", str(cm.exception))

        # Permitted non-publication actions must not raise
        permitted_actions = [
            "git status",
            "git diff",
            "git commit -m 'feat: scoped patch'",
            "cargo test --workspace",
            "cargo build --release",
            "python3 -m unittest discover",
        ]
        for action in permitted_actions:
            with self.subTest(action=action):
                try:
                    assert_no_autonomous_publication(action)
                except AutonomousPublicationError:
                    self.fail(f"Permitted action '{action}' erroneously raised AutonomousPublicationError")

    def test_cli_subcommands(self) -> None:
        """Requirement 8: Exercise evaluate, check-scope, and verify-commit via CLI subprocess."""
        python_bin = sys.executable
        cli_tool = str(REPO_ROOT / "tools" / "agent_eval.py")

        # 1. evaluate subcommand
        with tempfile.TemporaryDirectory() as tmpdir:
            out_json = Path(tmpdir) / "eval_out.json"
            res = subprocess.run(
                [python_bin, cli_tool, "evaluate", "--output", str(out_json), "--verbose"],
                capture_output=True,
                text=True,
                cwd=str(REPO_ROOT),
            )
            self.assertEqual(res.returncode, 0, f"evaluate failed: {res.stderr}")
            self.assertIn("Agent Held-Out Evaluation Summary", res.stdout)
            self.assertIn("Total Tasks:          6", res.stdout)
            self.assertIn("Accuracy:             100.0%", res.stdout)
            self.assertTrue(out_json.exists())

            with open(out_json, "r", encoding="utf-8") as f:
                data = json.load(f)
            self.assertEqual(data["total_tasks"], 6)
            self.assertEqual(data["accuracy"], 1.0)
            self.assertEqual(data["false_pass_count"], 0)

        # 1b. evaluate subcommand with role filtering
        res_filter = subprocess.run(
            [python_bin, cli_tool, "evaluate", "--roles", "repair"],
            capture_output=True,
            text=True,
            cwd=str(REPO_ROOT),
        )
        self.assertEqual(res_filter.returncode, 0)
        self.assertIn("Total Tasks:          2", res_filter.stdout)

        # 2. check-scope subcommand (success)
        res_scope_ok = subprocess.run(
            [python_bin, cli_tool, "check-scope", "--granted", "tools", "--attempted", "tools/agent_eval.py"],
            capture_output=True,
            text=True,
            cwd=str(REPO_ROOT),
        )
        self.assertEqual(res_scope_ok.returncode, 0)
        self.assertIn("Scope verified", res_scope_ok.stdout)

        # 2b. check-scope subcommand (failure: protected path)
        res_scope_prot = subprocess.run(
            [python_bin, cli_tool, "check-scope", "--granted", "crates", "--attempted", "crates/careops-ed/src/lib.rs"],
            capture_output=True,
            text=True,
            cwd=str(REPO_ROOT),
        )
        self.assertEqual(res_scope_prot.returncode, 1)
        self.assertIn("Scope violation", res_scope_prot.stderr)
        self.assertIn("protected barrier", res_scope_prot.stderr)

        # 2c. check-scope subcommand (failure: out of bounds)
        res_scope_oob = subprocess.run(
            [python_bin, cli_tool, "check-scope", "--granted", "tests", "--attempted", "tools/agent_eval.py"],
            capture_output=True,
            text=True,
            cwd=str(REPO_ROOT),
        )
        self.assertEqual(res_scope_oob.returncode, 1)
        self.assertIn("Scope violation", res_scope_oob.stderr)
        self.assertIn("outside granted reservations", res_scope_oob.stderr)

        # 3. verify-commit subcommand (success)
        head_sha = resolve_ref_commit_sha(REPO_ROOT, "HEAD")
        res_commit_ok = subprocess.run(
            [python_bin, cli_tool, "verify-commit", "--branch", "HEAD", "--commit", head_sha],
            capture_output=True,
            text=True,
            cwd=str(REPO_ROOT),
        )
        self.assertEqual(res_commit_ok.returncode, 0)
        self.assertIn("Exact commit verified", res_commit_ok.stdout)

        # 3b. verify-commit subcommand (failure: mismatch)
        res_commit_bad = subprocess.run(
            [python_bin, cli_tool, "verify-commit", "--branch", "HEAD", "--commit", "0000000000000000000000000000000000000000"],
            capture_output=True,
            text=True,
            cwd=str(REPO_ROOT),
        )
        self.assertEqual(res_commit_bad.returncode, 1)
        self.assertIn("Exact commit verification failed", res_commit_bad.stderr)

    def test_normalization_and_path_guards(self) -> None:
        """Requirement 9: Additional validation for path normalization, overlap logic, and empty cases."""
        # Empty string
        with self.assertRaises(ScopeViolationError):
            normalize_repo_path("")

        # Normalized paths
        self.assertEqual(normalize_repo_path("tools/agent_eval.py"), "tools/agent_eval.py")
        self.assertEqual(normalize_repo_path("./tools/agent_eval.py"), "tools/agent_eval.py")
        self.assertEqual(normalize_repo_path("/tools/agent_eval.py"), "tools/agent_eval.py")
        self.assertEqual(normalize_repo_path("tools\\agent_eval.py"), "tools/agent_eval.py")

        # Overlap checks
        self.assertTrue(paths_overlap("tools", "tools/agent_eval.py"))
        self.assertTrue(paths_overlap("tools/agent_eval.py", "tools"))
        self.assertTrue(paths_overlap(".", "tools"))
        self.assertFalse(paths_overlap("tools", "tests"))
        self.assertFalse(paths_overlap("tools", "tools_v2"))

        # is_path_under_granted
        self.assertTrue(is_path_under_granted("tools/agent_eval.py", "tools"))
        self.assertTrue(is_path_under_granted("tools", "tools"))
        self.assertTrue(is_path_under_granted("tools/sub/deep.py", "."))
        self.assertFalse(is_path_under_granted("tools_v2/file.py", "tools"))

        # Empty summarize_evaluation
        empty_summary = summarize_evaluation([])
        self.assertEqual(empty_summary.total_tasks, 0)
        self.assertEqual(empty_summary.accuracy, 0.0)

        # HeldOutTask role initialization with lowercase string
        task_obj = HeldOutTask(
            task_id="custom_task",
            name="Custom Task",
            role="repair",
            description="desc",
            input_files=[],
            expected_disposition="ready_for_review",
            known_bad=False,
            target_paths=["tools"],
        )
        self.assertEqual(task_obj.role, AgentRole.REPAIR)


if __name__ == "__main__":
    unittest.main()
