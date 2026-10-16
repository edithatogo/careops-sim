#!/usr/bin/env python3
"""Tests for Agent Evaluation, Scope Restriction, and Exact-Commit Verification (Task D5.2).

Validates:
  - Exception hierarchy and inheritance from AgentEvalError
  - Scope restriction and protected path barriers
  - Single-writer-per-path claim enforcement and concurrency protection
  - Exact-commit SHA verification engine
  - Autonomous publication prohibition
  - False pass detection and metric calculations
  - Built-in held-out benchmark suite (6 tasks)
  - CLI commands (evaluate, check-scope, verify-commit)
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
from unittest.mock import patch

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
    enforce_single_writer,
    get_held_out_benchmark_suite,
    is_path_under_granted,
    main,
    normalize_repo_path,
    paths_overlap,
    run_held_out_evaluation,
    summarize_evaluation,
    verify_exact_commit,
    verify_write_scope,
)


class TestExceptionHierarchy(unittest.TestCase):
    """Verify that all domain exceptions derive from AgentEvalError."""

    def test_all_exceptions_inherit_from_agent_eval_error(self):
        exceptions = [
            ScopeViolationError,
            MultipleWritersError,
            ProtectedPathError,
            ExactCommitMismatchError,
            FalsePassError,
            AutonomousPublicationError,
        ]
        for exc_cls in exceptions:
            with self.subTest(exc=exc_cls.__name__):
                self.assertTrue(issubclass(exc_cls, AgentEvalError))
                self.assertTrue(issubclass(exc_cls, Exception))


class TestModelsAndDataclasses(unittest.TestCase):
    """Verify dataclasses, enums, and serialization."""

    def test_agent_role_values(self):
        self.assertEqual(AgentRole.REPAIR, "repair")
        self.assertEqual(AgentRole.REVIEWER, "reviewer")

    def test_held_out_task_normalization(self):
        task = HeldOutTask(
            task_id="t1",
            name="Task 1",
            role="REPAIR",
            description="desc",
            input_files=["tools/foo.py"],
            expected_disposition="ready_for_review",
            known_bad=False,
            target_paths=["tools"],
        )
        self.assertEqual(task.role, AgentRole.REPAIR)

    def test_agent_execution_receipt_to_dict(self):
        receipt = AgentExecutionReceipt(
            task_id="t1",
            agent_role=AgentRole.REPAIR.value,
            model_route="route1",
            status="success",
            actual_disposition="ready_for_review",
            passed_disposition_check=True,
            files_mutated=["tools/foo.py"],
            scope_violations=[],
            token_cost_usd=0.015,
            human_interventions=0,
            turns=1,
            commit_sha="abcd1234efgh5678abcd1234efgh5678abcd1234",
            exact_commit_verified=True,
            execution_time_seconds=0.5,
        )
        d = receipt.to_dict()
        self.assertEqual(d["task_id"], "t1")
        self.assertEqual(d["agent_role"], "repair")
        self.assertTrue(d["passed_disposition_check"])
        self.assertEqual(d["token_cost_usd"], 0.015)

    def test_evaluation_summary_to_dict(self):
        summary = EvaluationSummary(
            total_tasks=2,
            accuracy=1.0,
            false_pass_count=0,
            false_pass_rate=0.0,
            false_reject_count=0,
            false_reject_rate=0.0,
            total_interventions=1,
            average_interventions_per_task=0.5,
            total_cost_usd=0.05,
            average_cost_usd=0.025,
            scope_violations_total=0,
            exact_commit_pass_rate=1.0,
            receipts=[],
        )
        d = summary.to_dict()
        self.assertEqual(d["total_tasks"], 2)
        self.assertEqual(d["accuracy"], 1.0)
        self.assertEqual(d["average_interventions_per_task"], 0.5)


class TestScopeRestriction(unittest.TestCase):
    """Verify write scope boundaries and protected path enforcement."""

    def test_normalize_repo_path(self):
        self.assertEqual(normalize_repo_path("tools/agent_eval.py"), "tools/agent_eval.py")
        self.assertEqual(normalize_repo_path("./tools/agent_eval.py"), "tools/agent_eval.py")
        self.assertEqual(normalize_repo_path("/tools/agent_eval.py"), "tools/agent_eval.py")
        self.assertEqual(normalize_repo_path("tools/sub/../agent_eval.py"), "tools/agent_eval.py")

        with self.assertRaises(ScopeViolationError):
            normalize_repo_path("")

        with self.assertRaises(ScopeViolationError):
            normalize_repo_path("../outside.py")

        with self.assertRaises(ScopeViolationError):
            normalize_repo_path(".git/config")

    def test_is_path_under_granted(self):
        self.assertTrue(is_path_under_granted("tools/agent_eval.py", "tools"))
        self.assertTrue(is_path_under_granted("tools", "tools"))
        self.assertTrue(is_path_under_granted("tools/agent_eval.py", "."))
        self.assertFalse(is_path_under_granted("tooling/agent_eval.py", "tools"))
        self.assertFalse(is_path_under_granted("tests/test_eval.py", "tools"))

    def test_verify_write_scope_valid(self):
        verify_write_scope(["tools", "tests"], "tools/agent_eval.py")
        verify_write_scope(["tools", "tests"], "tests/test_agent_eval.py")
        verify_write_scope(["."], "docs/index.md")

    def test_verify_write_scope_unauthorized(self):
        with self.assertRaises(ScopeViolationError):
            verify_write_scope(["tools"], "tests/test_agent_eval.py")

        with self.assertRaises(ScopeViolationError):
            verify_write_scope([], "tools/agent_eval.py")

    def test_verify_write_scope_protected_paths_blocked(self):
        protected_targets = [
            "crates/careops-ed/src/lib.rs",
            "conductor/delivery-contract.md",
            "LICENSE",
        ]
        for prot in protected_targets:
            with self.subTest(path=prot):
                # Even if granted scope covers the whole tree
                with self.assertRaises(ProtectedPathError):
                    verify_write_scope(["."], prot)

                # Even if granted scope specifically grants parent dir
                parent_dir = str(Path(prot).parent)
                with self.assertRaises(ProtectedPathError):
                    verify_write_scope([parent_dir], prot)


class TestSingleWriterEnforcement(unittest.TestCase):
    """Verify one-writer-per-path reservation protocol."""

    def test_paths_overlap(self):
        self.assertTrue(paths_overlap("tools", "tools/agent_eval.py"))
        self.assertTrue(paths_overlap("tools/agent_eval.py", "tools"))
        self.assertTrue(paths_overlap("tools/agent_eval.py", "tools/agent_eval.py"))
        self.assertTrue(paths_overlap(".", "tools/agent_eval.py"))
        self.assertFalse(paths_overlap("tools/a.py", "tools/b.py"))
        self.assertFalse(paths_overlap("tools", "tests"))

    def test_single_writer_sequential_claims(self):
        claims: dict[str, str] = {}
        enforce_single_writer(claims, "tools/a.py", "agent1")
        enforce_single_writer(claims, "tools/b.py", "agent2")
        # Same agent claiming again is idempotent
        enforce_single_writer(claims, "tools/a.py", "agent1")

        self.assertIn("tools/a.py", claims)
        self.assertIn("tools/b.py", claims)

    def test_single_writer_conflict_raises(self):
        claims = {"tools/a.py": "agent1"}
        with self.assertRaises(MultipleWritersError):
            enforce_single_writer(claims, "tools/a.py", "agent2")

        with self.assertRaises(MultipleWritersError):
            enforce_single_writer(claims, "tools", "agent2")


class TestExactCommitVerification(unittest.TestCase):
    """Verify exact commit SHA verification engine."""

    def test_verify_exact_commit_current_head(self):
        repo_root = Path(__file__).resolve().parent.parent
        # Get HEAD sha via git
        sha = subprocess.check_output(
            ["git", "-C", str(repo_root), "rev-parse", "HEAD"], text=True
        ).strip()

        # Exact match
        self.assertTrue(verify_exact_commit(repo_root, "HEAD", sha))
        # Prefix match (>= 7 chars)
        self.assertTrue(verify_exact_commit(repo_root, "HEAD", sha[:10]))

    def test_verify_exact_commit_mismatch_raises(self):
        repo_root = Path(__file__).resolve().parent.parent
        with self.assertRaises(ExactCommitMismatchError):
            verify_exact_commit(
                repo_root,
                "HEAD",
                "0000000000000000000000000000000000000000",
            )

    def test_verify_exact_commit_invalid_ref_raises(self):
        repo_root = Path(__file__).resolve().parent.parent
        with self.assertRaises(ExactCommitMismatchError):
            verify_exact_commit(repo_root, "non_existent_branch_ref_xyz", "abcd1234")


class TestAutonomousPublicationProhibition(unittest.TestCase):
    """Verify absolute ban on autonomous push or external publication."""

    def test_prohibited_publication_commands(self):
        prohibited = [
            "git push origin main",
            "git push --force",
            "cargo publish",
            "npm publish --access public",
            "twine upload dist/*",
            "gh release create v1.0.0",
            "external network publication",
        ]
        for cmd in prohibited:
            with self.subTest(cmd=cmd):
                with self.assertRaises(AutonomousPublicationError):
                    assert_no_autonomous_publication(cmd)

    def test_allowed_local_commands(self):
        allowed = [
            "git status",
            "git commit -m 'local commit'",
            "cargo build",
            "cargo test",
            "python3 -m unittest",
        ]
        for cmd in allowed:
            with self.subTest(cmd=cmd):
                # Should not raise
                assert_no_autonomous_publication(cmd)


class TestFalsePassDetection(unittest.TestCase):
    """Verify detection of false passes and metric calculations."""

    def test_assert_no_false_pass_raises_on_defect_approved(self):
        task = HeldOutTask(
            task_id="defect_1",
            name="Defect 1",
            role=AgentRole.REVIEWER,
            description="Hidden defect",
            input_files=[],
            expected_disposition="revise",
            known_bad=True,
            target_paths=["tools"],
        )
        with self.assertRaises(FalsePassError):
            assert_no_false_pass(task, "ready_for_review")

        # revise or hold_for_evidence should not raise
        assert_no_false_pass(task, "revise")
        assert_no_false_pass(task, "hold_for_evidence")

    def test_summarize_evaluation_metrics(self):
        tasks = [
            HeldOutTask("t1", "Good 1", AgentRole.REPAIR, "", [], "ready_for_review", False, ["tools"]),
            HeldOutTask("t2", "Good 2", AgentRole.REVIEWER, "", [], "ready_for_review", False, ["tools"]),
            HeldOutTask("t3", "Bad 1", AgentRole.REVIEWER, "", [], "revise", True, ["tools"]),
            HeldOutTask("t4", "Bad 2", AgentRole.REVIEWER, "", [], "revise", True, ["tools"]),
        ]

        # Simulate agent responses:
        # t1: ready_for_review (correct)
        # t2: revise (false reject!)
        # t3: ready_for_review (false pass!)
        # t4: revise (correct)
        receipts = [
            AgentExecutionReceipt("t1", "repair", "route", "success", "ready_for_review", True, [], [], 0.01, 0, 1, "sha", True, 0.1),
            AgentExecutionReceipt("t2", "reviewer", "route", "success", "revise", False, [], [], 0.01, 1, 1, "sha", True, 0.1),
            AgentExecutionReceipt("t3", "reviewer", "route", "success", "ready_for_review", False, [], [], 0.01, 0, 1, "sha", True, 0.1),
            AgentExecutionReceipt("t4", "reviewer", "route", "success", "revise", True, [], [], 0.01, 0, 1, "sha", True, 0.1),
        ]

        summary = summarize_evaluation(receipts, tasks=tasks)
        self.assertEqual(summary.total_tasks, 4)
        self.assertEqual(summary.accuracy, 0.5)  # 2 correct out of 4
        self.assertEqual(summary.false_pass_count, 1)
        self.assertEqual(summary.false_pass_rate, 0.5)  # 1 bad run accepted out of 2 bad runs
        self.assertEqual(summary.false_reject_count, 1)
        self.assertEqual(summary.false_reject_rate, 0.5)  # 1 good run rejected out of 2 good runs
        self.assertEqual(summary.total_interventions, 1)
        self.assertEqual(summary.average_interventions_per_task, 0.25)
        self.assertAlmostEqual(summary.total_cost_usd, 0.04)


class TestHeldOutBenchmarkSuite(unittest.TestCase):
    """Verify the 6 held-out benchmark tasks and evaluation execution."""

    def test_suite_task_ids_and_attributes(self):
        suite = get_held_out_benchmark_suite()
        self.assertEqual(len(suite), 6)

        task_ids = [t.task_id for t in suite]
        expected_ids = [
            "repair_syntax_typo",
            "repair_doc_link_drift",
            "reviewer_unauthorized_scope_attempt",
            "reviewer_hidden_clamp_defect",
            "reviewer_stale_commit_race",
            "reviewer_valid_feature_addition",
        ]
        self.assertEqual(task_ids, expected_ids)

        repair_tasks = [t for t in suite if t.role == AgentRole.REPAIR]
        self.assertEqual(len(repair_tasks), 2)
        reviewer_tasks = [t for t in suite if t.role == AgentRole.REVIEWER]
        self.assertEqual(len(reviewer_tasks), 4)

    def test_run_held_out_evaluation_full_suite(self):
        repo_root = Path(__file__).resolve().parent.parent
        summary = run_held_out_evaluation(repo_root=repo_root)

        self.assertEqual(summary.total_tasks, 6)
        self.assertEqual(summary.accuracy, 1.0)
        self.assertEqual(summary.false_pass_count, 0)
        self.assertEqual(summary.false_pass_rate, 0.0)
        self.assertEqual(summary.false_reject_count, 0)
        self.assertEqual(summary.false_reject_rate, 0.0)
        self.assertEqual(summary.total_interventions, 0)
        self.assertGreater(summary.total_cost_usd, 0.0)
        self.assertEqual(len(summary.receipts), 6)

    def test_run_held_out_evaluation_filter_roles(self):
        repo_root = Path(__file__).resolve().parent.parent
        summary_repair = run_held_out_evaluation(repo_root=repo_root, roles=[AgentRole.REPAIR])
        self.assertEqual(summary_repair.total_tasks, 2)
        self.assertEqual(summary_repair.accuracy, 1.0)

        summary_reviewer = run_held_out_evaluation(repo_root=repo_root, roles=["reviewer"])
        self.assertEqual(summary_reviewer.total_tasks, 4)
        self.assertEqual(summary_reviewer.accuracy, 1.0)

    def test_run_held_out_evaluation_raise_on_false_pass(self):
        repo_root = Path(__file__).resolve().parent.parent

        def defective_agent(task, root, sha, claims):
            # Blindly approves everything
            return AgentExecutionReceipt(
                task_id=task.task_id,
                agent_role=str(task.role),
                model_route="defective",
                status="success",
                actual_disposition="ready_for_review",
                passed_disposition_check=not task.known_bad,
                files_mutated=[],
                scope_violations=[],
                token_cost_usd=0.01,
                human_interventions=0,
                turns=1,
                commit_sha=sha,
                exact_commit_verified=True,
                execution_time_seconds=0.01,
            )

        with self.assertRaises(FalsePassError):
            run_held_out_evaluation(
                repo_root=repo_root,
                task_evaluator=defective_agent,
                raise_on_false_pass=True,
            )


class TestCLI(unittest.TestCase):
    """Verify CLI commands and exit codes."""

    def test_cli_evaluate(self):
        repo_root = str(Path(__file__).resolve().parent.parent)
        with patch("sys.stdout", new=io.StringIO()) as fake_out:
            code = main(["evaluate", "--repo-root", repo_root, "--verbose"])
            self.assertEqual(code, 0)
            output = fake_out.getvalue()
            self.assertIn("Agent Held-Out Evaluation Summary", output)
            self.assertIn("Accuracy:             100.0%", output)

    def test_cli_evaluate_with_output(self):
        repo_root = str(Path(__file__).resolve().parent.parent)
        with tempfile.TemporaryDirectory() as tmpdir:
            out_json = Path(tmpdir) / "summary.json"
            code = main(["evaluate", "--repo-root", repo_root, "--output", str(out_json)])
            self.assertEqual(code, 0)
            self.assertTrue(out_json.is_file())
            data = json.loads(out_json.read_text(encoding="utf-8"))
            self.assertEqual(data["total_tasks"], 6)
            self.assertEqual(data["accuracy"], 1.0)

    def test_cli_check_scope_success(self):
        code = main(["check-scope", "--granted", "tools", "tests", "--attempted", "tools/agent_eval.py"])
        self.assertEqual(code, 0)

    def test_cli_check_scope_violation(self):
        code = main(["check-scope", "--granted", "tools", "--attempted", "crates/careops-ed/src/lib.rs"])
        self.assertEqual(code, 1)

    def test_cli_verify_commit_success(self):
        repo_root = Path(__file__).resolve().parent.parent
        sha = subprocess.check_output(
            ["git", "-C", str(repo_root), "rev-parse", "HEAD"], text=True
        ).strip()
        code = main(["verify-commit", "--branch", "HEAD", "--commit", sha])
        self.assertEqual(code, 0)

    def test_cli_verify_commit_mismatch(self):
        code = main(["verify-commit", "--branch", "HEAD", "--commit", "0000000000000000000000000000000000000000"])
        self.assertEqual(code, 1)


if __name__ == "__main__":
    unittest.main()
