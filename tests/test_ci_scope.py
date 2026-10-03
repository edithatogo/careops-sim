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

    def test_invalid_base_or_git_errors_select_every_lane(self):
        self.assert_all_selected(ci_scope.classify("not-a-revision", self.base))
        with mock.patch.object(ci_scope, "_git", side_effect=RuntimeError("fixture git error")):
            self.assert_all_selected(ci_scope.classify(self.base, self.base))

    def test_malformed_or_unknown_status_and_unsafe_path_are_rejected(self):
        for data in (b"M\0", b"Z\0path\0", b"R100\0only-one\0", b"A\0../escape\0", b"A\0\xff\0"):
            with self.subTest(data=data), self.assertRaises(ValueError):
                ci_scope._parse_name_status(data)

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
