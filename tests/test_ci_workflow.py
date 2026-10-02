import os
from pathlib import Path
import subprocess
import tempfile
import unittest


WORKFLOW = Path(__file__).parents[1] / ".github" / "workflows" / "ci.yml"
LANES = ("FMT_RESULT", "CLIPPY_RESULT", "TEST_RESULT", "DOCTEST_RESULT", "CONTEXT_RESULT")


def aggregate_script():
    lines = WORKFLOW.read_text().splitlines()
    marker = "        run: |"
    start = len(lines) - 1 - lines[::-1].index(marker) + 1
    block = []
    for line in lines[start:]:
        if line and not line.startswith("          "):
            break
        block.append(line[10:] if line.startswith("          ") else "")
    return "\n".join(block) + "\n"


def run_aggregate(changed, results):
    env = {
        "SCOPE_RESULT": "success",
        "SCOPE_CHANGED": changed,
        **dict(zip(LANES, results)),
    }
    return subprocess.run(
        ["bash", "-c", aggregate_script()],
        env={**os.environ, **env},
        capture_output=True,
        text=True,
        check=False,
    )


def scope_script():
    lines = WORKFLOW.read_text().splitlines()
    step = next(i for i, line in enumerate(lines) if line.strip() == "- id: detect")
    marker = next(i for i in range(step, len(lines)) if lines[i].strip() == "run: |")
    block = []
    for line in lines[marker + 1 :]:
        if line and not line.startswith("          "):
            break
        block.append(line[10:] if line.startswith("          ") else "")
    return "\n".join(block) + "\n"


def git(repo, *args):
    result = subprocess.run(
        ["git", *args], cwd=repo, capture_output=True, text=True, check=False
    )
    if result.returncode:
        raise AssertionError(result.stderr)
    return result.stdout.strip()


class CiWorkflowTests(unittest.TestCase):
    def test_scope_includes_rust_inputs_fixtures_workflows_and_submodule_pin(self):
        workflow = WORKFLOW.read_text()
        scope_step = workflow.split("name: Detect Rust, tool, or Conductor changes", 1)[1].split("  fmt:", 1)[0]
        for path in ("tests", "model-inputs", ".gitmodules", ".github/workflows", "libs/kairos"):
            with self.subTest(path=path):
                self.assertIn(path, scope_step)

    def test_scope_detector_observes_tracked_inputs_and_gitlink_changes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            dependency = root / "dependency"
            dependency.mkdir()
            git(dependency, "init", "-q")
            git(dependency, "config", "user.email", "ci@example.invalid")
            git(dependency, "config", "user.name", "CI fixture")
            (dependency / "revision").write_text("one\n")
            git(dependency, "add", "revision")
            git(dependency, "commit", "-qm", "first")
            first_pin = git(dependency, "rev-parse", "HEAD")
            (dependency / "revision").write_text("two\n")
            git(dependency, "commit", "-qam", "second")
            second_pin = git(dependency, "rev-parse", "HEAD")

            repo = root / "repo"
            repo.mkdir()
            git(repo, "init", "-q")
            git(repo, "config", "user.email", "ci@example.invalid")
            git(repo, "config", "user.name", "CI fixture")
            paths = (
                "Cargo.toml",
                "Cargo.lock",
                "crates/example/src/lib.rs",
                "tests/test_example.py",
                "model-inputs/example.json",
                "tools/check.py",
                "conductor/plan.md",
                ".gitmodules",
                ".github/workflows/other.yml",
            )
            for path in paths:
                target = repo / path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("before\n")
            git(repo, "add", ".")
            git(repo, "update-index", "--add", "--cacheinfo", f"160000,{first_pin},libs/kairos")
            git(repo, "commit", "-qm", "baseline")
            base = git(repo, "rev-parse", "HEAD")
            for path in paths:
                (repo / path).write_text("after\n")
            git(repo, "add", ".")
            git(repo, "update-index", "--add", "--cacheinfo", f"160000,{second_pin},libs/kairos")
            git(repo, "commit", "-qm", "scoped changes")

            output = root / "github-output"
            result = subprocess.run(
                ["bash", "-c", scope_script()],
                cwd=repo,
                env={
                    **os.environ,
                    "EVENT_NAME": "push",
                    "PUSH_BASE_SHA": base,
                    "PR_BASE_SHA": "",
                    "GITHUB_OUTPUT": str(output),
                },
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(output.read_text().strip(), "changed=true")

            output.write_text("")
            unchanged = subprocess.run(
                ["bash", "-c", scope_script()],
                cwd=repo,
                env={
                    **os.environ,
                    "EVENT_NAME": "push",
                    "PUSH_BASE_SHA": git(repo, "rev-parse", "HEAD"),
                    "PR_BASE_SHA": "",
                    "GITHUB_OUTPUT": str(output),
                },
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(unchanged.returncode, 0, unchanged.stdout + unchanged.stderr)
            self.assertEqual(output.read_text().strip(), "changed=false")

    def test_cargo_lanes_initialize_path_dependency_and_pin_evidenced_toolchain(self):
        workflow = WORKFLOW.read_text()
        cargo_jobs = workflow.split("  required:", 1)[0]
        self.assertEqual(cargo_jobs.count("submodules: true"), 4)
        self.assertEqual(cargo_jobs.count("rustup toolchain install 1.98.1"), 4)
        for command in ("cargo +1.98.1 fmt", "cargo +1.98.1 clippy", "cargo +1.98.1 test --workspace", "cargo +1.98.1 test --doc"):
            with self.subTest(command=command):
                self.assertIn(command, cargo_jobs)

    def test_required_lane_failure_fails_aggregate(self):
        result = run_aggregate("true", ["success", "failure", "success", "success", "success"])
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("required lane failed or did not run", result.stdout)

    def test_unchanged_scope_is_an_explicit_successful_skip(self):
        result = run_aggregate("false", ["skipped"] * len(LANES))
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("scope unchanged: all required lanes skipped successfully", result.stdout)

    def test_changed_scope_cannot_hide_a_skipped_lane(self):
        result = run_aggregate("true", ["success", "skipped", "success", "success", "success"])
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
