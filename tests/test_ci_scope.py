import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

from tools import ci_scope


class GitRepo:
    def __init__(self, root):
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        self.git("init", "-q")
        self.git("config", "user.email", "ci@example.invalid")
        self.git("config", "user.name", "CI fixture")

    def git(self, *args):
        result = subprocess.run(["git", *args], cwd=self.root, text=True, capture_output=True)
        if result.returncode:
            raise AssertionError(result.stderr)
        return result.stdout.strip()

    def commit(self, message):
        self.git("add", "-A")
        self.git("commit", "-qm", message)
        return self.git("rev-parse", "HEAD")

    def write(self, name, content="sample\n"):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
        return path


class CiScopeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repo = GitRepo(Path(self.temporary.name) / "repo")
        self.repo.write("conductor/tracks/example.md", "baseline\n")
        self.base = self.repo.commit("baseline")
        previous = Path.cwd()
        os.chdir(self.repo.root)
        self.addCleanup(os.chdir, previous)

    def commit_and_classify(self, name, content="changed\n"):
        self.repo.write(name, content)
        head = self.repo.commit("change")
        return ci_scope.classify(self.base, head)

    def assert_all_selected(self, result):
        self.assertEqual(result, {"changed": True, "native": True, "context": True, "policy": True})

    def test_empty_diff_skips_every_lane(self):
        self.assertEqual(ci_scope.classify(self.base, self.base), {"changed": False, "native": False, "context": False, "policy": False})

    def test_regular_planning_add_modify_delete_selects_context_and_policy_only(self):
        result = self.commit_and_classify("conductor/design/new.json", "{}\n")
        self.assertEqual(result, {"changed": True, "native": False, "context": True, "policy": True})

        prior = self.repo.git("rev-parse", "HEAD")
        self.repo.write("conductor/design/new.json", "{\"revision\": 2}\n")
        modified = self.repo.commit("modify planning")
        self.assertEqual(ci_scope.classify(prior, modified), {"changed": True, "native": False, "context": True, "policy": True})

        prior = self.repo.git("rev-parse", "HEAD")
        (self.repo.root / "conductor/design/new.json").unlink()
        deleted = self.repo.commit("delete planning")
        self.assertEqual(ci_scope.classify(prior, deleted), {"changed": True, "native": False, "context": True, "policy": True})

    def test_runtime_and_unknown_paths_select_every_lane(self):
        self.assert_all_selected(self.commit_and_classify("crates/engine/src/lib.rs"))
        self.assert_all_selected(self.commit_and_classify("conductor/other/notes.md"))
        self.assert_all_selected(self.commit_and_classify("conductor/tracks/notes.yaml"))

    def test_executable_and_symlink_planning_paths_select_every_lane(self):
        executable = self.repo.write("conductor/tracks/run.md", "#!/bin/sh\n")
        executable.chmod(0o755)
        first = self.repo.commit("add executable planning path")
        self.assert_all_selected(ci_scope.classify(self.base, first))

        previous = self.repo.git("rev-parse", "HEAD")
        target = self.repo.root / "conductor/design/link.json"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.symlink_to("../tracks/example.md")
        second = self.repo.commit("add symlink planning path")
        self.assert_all_selected(ci_scope.classify(previous, second))

    def test_gitlink_planning_path_selects_every_lane(self):
        dependency = Path(self.temporary.name) / "dependency"
        dep = GitRepo(dependency)
        dep.write("file", "one\n")
        pin = dep.commit("dependency")
        self.repo.git("update-index", "--add", "--cacheinfo", f"160000,{pin},conductor/tracks/gitlink.md")
        self.repo.git("commit", "-qm", "add gitlink")
        head = self.repo.git("rev-parse", "HEAD")
        self.assert_all_selected(ci_scope.classify(self.base, head))

    def test_rename_and_copy_records_keep_both_paths(self):
        changes = ci_scope._parse_name_status(
            b"R100\0conductor/tracks/old.md\0conductor/design/new.md\0"
            b"C087\0conductor/evidence/source.json\0conductor/execution/copy.json\0"
        )
        self.assertEqual(changes[0], ci_scope.Change("R100", "conductor/tracks/old.md", "conductor/design/new.md"))
        self.assertEqual(changes[1], ci_scope.Change("C087", "conductor/evidence/source.json", "conductor/execution/copy.json"))

    def test_rename_from_planning_to_runtime_selects_every_lane(self):
        source = self.repo.root / "conductor/tracks/example.md"
        target = self.repo.root / "crates/runtime/notes.md"
        target.parent.mkdir(parents=True)
        source.rename(target)
        head = self.repo.commit("move planning into runtime")
        self.assert_all_selected(ci_scope.classify(self.base, head))

    def test_real_git_runtime_to_planning_rename_selects_every_lane(self):
        source = self.repo.root / "crates/runtime/policy.md"
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text("runtime policy\n")
        baseline = self.repo.commit("add runtime policy")
        target = self.repo.root / "conductor/tracks/policy.md"
        target.parent.mkdir(parents=True, exist_ok=True)
        source.rename(target)
        head = self.repo.commit("rename runtime policy into planning")
        diff = ci_scope._git(["diff", "--name-status", "-z", "--find-renames", baseline, head, "--"])
        changes = ci_scope._parse_name_status(diff)
        self.assertEqual(changes, [ci_scope.Change("R100", "crates/runtime/policy.md", "conductor/tracks/policy.md")])
        self.assert_all_selected(ci_scope.classify(baseline, head))

    def test_real_git_mixed_runtime_and_planning_changes_select_every_lane(self):
        self.repo.write("crates/engine/src/lib.rs", "baseline\n")
        baseline = self.repo.commit("add runtime source")
        self.repo.write("crates/engine/src/lib.rs", "runtime changed\n")
        self.repo.write("conductor/evidence/assessment.json", "{}\n")
        head = self.repo.commit("mix runtime and planning")
        self.assert_all_selected(ci_scope.classify(baseline, head))

    def test_real_git_runtime_input_matrix_and_both_gitlinks_select_every_lane(self):
        dependency = GitRepo(Path(self.temporary.name) / "matrix-dependency")
        dependency.write("revision", "first\n")
        first_pin = dependency.commit("first dependency revision")
        dependency.write("revision", "second\n")
        second_pin = dependency.commit("second dependency revision")
        paths = (
            "Cargo.toml",
            "Cargo.lock",
            ".github/workflows/extra.yml",
            "tools/check.py",
            "tests/test_runtime_matrix.py",
            "model-inputs/default.json",
            "conductor/plan.md",
            "conductor/tracks/plan.json",
            ".gitmodules",
        )
        for path in paths:
            self.repo.write(path, "baseline\n")
        self.repo.git("add", "-A")
        self.repo.git("update-index", "--add", "--cacheinfo", f"160000,{first_pin},libs/kairos")
        self.repo.git("update-index", "--add", "--cacheinfo", f"160000,{first_pin},extensions/conductor")
        self.repo.git("commit", "-qm", "runtime input matrix baseline")
        baseline = self.repo.git("rev-parse", "HEAD")

        for path in paths:
            self.repo.write(path, "changed\n")
        self.repo.git("add", "-A")
        self.repo.git("update-index", "--add", "--cacheinfo", f"160000,{second_pin},libs/kairos")
        self.repo.git("update-index", "--add", "--cacheinfo", f"160000,{second_pin},extensions/conductor")
        self.repo.git("commit", "-qm", "runtime input matrix changed")
        head = self.repo.git("rev-parse", "HEAD")
        diff = ci_scope._git(["diff", "--name-status", "-z", baseline, head, "--"])
        changes = ci_scope._parse_name_status(diff)
        changed_paths = {path for change in changes for path in (change.old_path, change.new_path) if path is not None}
        self.assertTrue(set(paths).issubset(changed_paths))
        self.assertIn("libs/kairos", changed_paths)
        self.assertIn("extensions/conductor", changed_paths)
        self.assert_all_selected(ci_scope.classify(baseline, head))

    def test_absent_zero_nonexistent_base_and_invalid_head_select_every_lane(self):
        missing_object = "f" * 40
        self.assert_all_selected(ci_scope.classify("", self.base))
        self.assert_all_selected(ci_scope.classify("0" * 40, self.base))
        self.assert_all_selected(ci_scope.classify(missing_object, self.base))
        self.assert_all_selected(ci_scope.classify(self.base, "not-a-head"))

    def test_real_git_typechange_selects_every_lane(self):
        target = self.repo.write("conductor/tracks/typechange.md", "regular file\n")
        baseline = self.repo.commit("add regular planning file")
        target.unlink()
        target.symlink_to("example.md")
        head = self.repo.commit("replace planning file with symlink")
        diff = ci_scope._git(["diff", "--name-status", "-z", baseline, head, "--"])
        changes = ci_scope._parse_name_status(diff)
        self.assertIn("T", [change.status for change in changes])
        self.assert_all_selected(ci_scope.classify(baseline, head))

    def test_invalid_base_or_git_errors_select_every_lane(self):
        self.assert_all_selected(ci_scope.classify("not-a-revision", self.base))
        with mock.patch.object(ci_scope, "_git", side_effect=RuntimeError("fixture git error")):
            self.assert_all_selected(ci_scope.classify(self.base, self.base))

    def test_nonempty_name_status_requires_terminal_nul(self):
        with self.assertRaises(ValueError):
            ci_scope._parse_name_status(b"M\0conductor/tracks/example.md")

    def test_malformed_or_unknown_status_and_unsafe_path_are_rejected(self):
        for data in (b"M\0", b"Z\0path\0", b"R100\0only-one\0", b"A\0../escape\0", b"A\0\xff\0"):
            with self.subTest(data=data), self.assertRaises(ValueError):
                ci_scope._parse_name_status(data)

    def test_injected_missed_native_selection_is_caught_by_real_git_oracle(self):
        runtime = self.repo.write("crates/engine/src/lib.rs", "native change\n")
        head = self.repo.commit("runtime change for selector oracle")
        expected = {"changed": True, "native": True, "context": True, "policy": True}
        self.assertEqual(ci_scope.classify(self.base, head), expected)
        with mock.patch.object(ci_scope, "_planning_path", return_value=True):
            injected_miss = ci_scope.classify(self.base, head)
        self.assertEqual(injected_miss, {"changed": True, "native": False, "context": True, "policy": True})
        self.assertNotEqual(injected_miss, expected)

    def test_mode_must_be_known_for_every_changed_side(self):
        with mock.patch.object(ci_scope, "_tree_modes", side_effect=[{}, {}]):
            self.assert_all_selected(self.commit_and_classify("conductor/tracks/new.md"))

    def test_output_is_literal_and_contains_all_selectors(self):
        output = Path(self.temporary.name) / "output"
        result = subprocess.run(
            [os.sys.executable, str(Path(ci_scope.__file__)), "--base", self.base, "--head", self.base, "--output", str(output)],
            cwd=self.repo.root,
            text=True,
            capture_output=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(output.read_text(), "changed=false\nnative=false\ncontext=false\npolicy=false\n")


if __name__ == "__main__":
    unittest.main()
