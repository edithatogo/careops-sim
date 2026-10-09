import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import learning


class LearningTests(unittest.TestCase):
    def setUp(self):
        self.original_store = learning.STORE
        self.original_root = learning.ROOT
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.addCleanup(self._restore_globals)
        self.root = Path(self.temp.name)
        self.store = self.root / "lessons.jsonl"
        learning.STORE = self.store
        learning.ROOT = self.root
        self.evidence = self.root / "source.md"
        self.evidence.write_text("source one", encoding="utf-8")

    def _restore_globals(self):
        learning.STORE = self.original_store
        learning.ROOT = self.original_root

    def lesson(self, lesson_id="lesson-one"):
        return {"id": lesson_id, "title": "Exact worktree binding", "topics": ["git", "worktree"],
                "procedure": "Check exact HEAD before writing.",
                "evidence": [{"path": "source.md", "sha256": learning.digest(self.evidence)}]}

    def test_promotion_requires_review_and_fresh_evidence(self):
        with self.assertRaises(learning.LearningError):
            learning.validate("lesson-one")
        learning.propose(self.lesson())
        with self.assertRaises(learning.LearningError):
            learning.validate("lesson-one")
        learning.review("lesson-one", "reviewer", "Evidence supports this bounded procedure")
        self.evidence.write_text("source changed", encoding="utf-8")
        with self.assertRaisesRegex(learning.LearningError, "stale evidence"):
            learning.validate("lesson-one")

    def test_tampered_history_is_rejected(self):
        learning.propose(self.lesson())
        raw = self.store.read_text(encoding="utf-8").replace("Exact worktree binding", "Changed worktree binding")
        self.store.write_text(raw, encoding="utf-8")
        with self.assertRaisesRegex(learning.LearningError, "event chain invalid"):
            learning.search("worktree")

    def test_seed_is_resumable_without_duplicate_history(self):
        self.evidence.write_text("", encoding="utf-8")
        # Seed source paths are repository-specific; verify idempotence through
        # the public command with a patched seed set.
        original_seed = learning._seed
        learning._seed = lambda: [self.lesson("seed-one")]
        try:
            self.assertEqual(learning.main(["seed"]), 0)
            first = self.store.read_bytes()
            self.assertEqual(learning.main(["seed"]), 0)
            self.assertEqual(self.store.read_bytes(), first)
        finally:
            learning._seed = original_seed

    def test_append_only_history_and_outcome_feedback(self):
        learning.propose(self.lesson())
        first = self.store.read_bytes()
        learning.review("lesson-one", "reviewer", "Reviewed")
        self.assertTrue(self.store.read_bytes().startswith(first))
        learning.validate("lesson-one")
        learning.record_use("lesson-one", "Applied to the current task")
        learning.outcome("lesson-one", "helped", "Reduced repeated setup")
        learning.outcome("lesson-one", "did_not_help", "Did not fit this task")
        lesson = learning.search("git worktree")[0]
        self.assertEqual(lesson["status"], "validated")
        self.assertEqual(len(lesson["uses"]), 1)
        self.assertEqual(lesson["use_count"], 1)
        self.assertEqual(lesson["outcomes"][0]["result"], "helped")
        self.assertEqual(lesson["outcome_counts"]["helped"], 1)
        self.assertEqual(lesson["outcome_counts"]["did_not_help"], 1)
        self.assertEqual(len(lesson["outcomes"]), 2)

    def test_proposal_requires_evidence_and_duplicate_ids_rejected(self):
        invalid = self.lesson()
        invalid["evidence"] = []
        with self.assertRaises(learning.LearningError):
            learning.propose(invalid)
        learning.propose(self.lesson())
        with self.assertRaisesRegex(learning.LearningError, "cannot be reused"):
            learning.propose(self.lesson())

    def test_proposal_rejects_malformed_types_and_unknown_supersedes_target(self):
        for update in ({"title": []}, {"topics": "git"}, {"evidence": ["source.md"]}):
            invalid = {**self.lesson(), **update}
            with self.assertRaises(learning.LearningError):
                learning.propose(invalid)
        invalid = {**self.lesson("successor"), "supersedes": "missing"}
        with self.assertRaisesRegex(learning.LearningError, "existing earlier lesson"):
            learning.propose(invalid)

    def test_supersedes_cycle_is_rejected(self):
        cyclic_state = {
            "first": {"id": "first", "supersedes": "second"},
            "second": {"id": "second", "supersedes": "first"},
        }
        with self.assertRaisesRegex(learning.LearningError, "cycle"):
            learning._check_supersedes({**self.lesson("third"), "supersedes": "first"}, cyclic_state)

    def test_oversized_evidence_is_rejected_from_stat_before_hashing(self):
        oversized = self.root / "oversized.md"
        with oversized.open("wb") as stream:
            stream.truncate(learning.MAX_EVIDENCE + 1)
        with self.assertRaisesRegex(learning.LearningError, "exceeds .* byte limit"):
            learning.digest(oversized)

    def test_validated_successor_supersedes_predecessor_without_rewriting_it(self):
        learning.propose(self.lesson("first"))
        learning.review("first", "coordinator", "Reviewed procedure")
        learning.validate("first")
        before = self.store.read_bytes()
        successor = {**self.lesson("second"), "supersedes": "first"}
        learning.propose(successor)
        self.assertEqual(learning.show("first")["status"], "validated")
        self.assertEqual([item["id"] for item in learning.search("exact worktree")], ["first", "second"])
        learning.review("second", "reviewer", "Reviewed revision")
        learning.validate("second")
        self.assertEqual(learning.show("first")["status"], "superseded")
        self.assertEqual(learning.show("first")["superseded_by"], "second")
        self.assertEqual([item["id"] for item in learning.search("exact worktree")], ["second"])
        self.assertEqual([item["id"] for item in learning.search("exact worktree", include_superseded=True)],
                         ["first", "second"])
        self.assertTrue(self.store.read_bytes().startswith(before))

    def test_search_checks_evidence_freshness_and_store_bound(self):
        learning.propose(self.lesson())
        self.assertEqual(len(learning.search("exact worktree")), 1)
        with self.assertRaisesRegex(learning.LearningError, "only validated"):
            learning.record_use("lesson-one", "candidate cannot be treated as instruction")
        self.evidence.write_text("new", encoding="utf-8")
        with self.assertRaisesRegex(learning.LearningError, "stale evidence"):
            learning.search("worktree")
        self.store.write_bytes(b"x" * (learning.MAX_STORE + 1))
        with self.assertRaisesRegex(learning.LearningError, "exceeds 1 MB"):
            learning.search("worktree")

    def test_unrelated_stale_lesson_does_not_block_search(self):
        other = self.root / "other.md"
        other.write_text("other evidence", encoding="utf-8")
        first = self.lesson("target-one")
        second = {**self.lesson("unrelated"), "title": "Unrelated topic",
                  "topics": ["elsewhere"],
                  "evidence": [{"path": "other.md", "sha256": learning.digest(other)}]}
        learning.propose(first)
        learning.propose(second)
        other.write_text("changed", encoding="utf-8")
        self.assertEqual([item["id"] for item in learning.search("exact worktree")], ["target-one"])
        with self.assertRaisesRegex(learning.LearningError, "stale evidence"):
            learning.search("unrelated")

    def test_search_is_bounded_and_show_returns_detail(self):
        for i in range(7):
            learning.propose(self.lesson(f"lesson-{i}"))
        self.assertEqual(len(learning.search("worktree")), 5)
        self.assertEqual(len(learning.search("worktree", limit=2)), 2)
        self.assertEqual(learning.show("lesson-6")["id"], "lesson-6")
        with self.assertRaisesRegex(learning.LearningError, "between 1 and 20"):
            learning.search("worktree", limit=21)

    def test_malformed_store_fails_cleanly_and_symlink_store_is_rejected(self):
        self.store.write_text("[]\n", encoding="utf-8")
        with self.assertRaisesRegex(learning.LearningError, "expected an object"):
            learning.search("worktree")
        self.store.unlink()
        outside = self.root.parent / (self.root.name + "-outside.jsonl")
        outside.write_text("", encoding="utf-8")
        self.addCleanup(lambda: outside.unlink(missing_ok=True))
        self.store.symlink_to(outside)
        with self.assertRaisesRegex(learning.LearningError, "must not be a symlink"):
            learning.search("worktree")

    def test_concurrent_subprocess_proposals_keep_one_valid_chain(self):
        code = (
            "from pathlib import Path; import sys; sys.path.insert(0, sys.argv[1]); "
            "import learning; learning.ROOT=Path(sys.argv[2]); learning.STORE=Path(sys.argv[3]); "
            "e=Path(sys.argv[2])/'source.md'; "
            "lesson={'id':sys.argv[4], 'title':'Concurrent lesson', 'procedure':'Check chain', "
            "'topics':['concurrency'], 'evidence':[{'path':'source.md','sha256':learning.digest(e)}]}; "
            "learning.propose(lesson)"
        )
        env = {**os.environ, "PYTHONPATH": str(Path(learning.__file__).parent)}
        processes = [subprocess.Popen([sys.executable, "-c", code,
                       str(Path(learning.__file__).parent), str(self.root), str(self.store), f"p-{i}"],
                       env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                     for i in range(8)]
        results = [process.communicate(timeout=20) + (process.returncode,) for process in processes]
        self.assertEqual([result[2] for result in results], [0] * len(processes), results)
        with learning._locked_store() as fd:
            state = learning._state(fd)
        self.assertEqual(len(state), 8)
        self.assertEqual(len(learning.search("concurrency", limit=20)), 8)
        raw = self.store.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(raw), 8)
        self.assertEqual(len({json.loads(line)["event_hash"] for line in raw}), 8)


if __name__ == "__main__":
    unittest.main()
