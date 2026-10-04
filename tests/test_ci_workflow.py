import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest


WORKFLOW = Path(__file__).parents[1] / ".github" / "workflows" / "ci.yml"
LANES = ("FMT_RESULT", "CLIPPY_RESULT", "TEST_RESULT", "MSRV_RESULT", "DOCTEST_RESULT", "CONTEXT_RESULT", "POLICY_RESULT", "FUZZ_RESULT", "MIRI_RESULT")


def workflow_job(workflow, name):
    match = re.search(rf"(?ms)^  {re.escape(name)}:\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)", workflow)
    if not match:
        raise AssertionError(f"missing workflow job {name}")
    return match.group(0)


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


def run_aggregate(changed, results, selectors=None):
    selectors = selectors or {
        "native": changed,
        "context": changed,
        "policy": changed,
    }
    env = {
        "SCOPE_RESULT": "success",
        "SCOPE_CHANGED": changed,
        "SCOPE_NATIVE": selectors["native"],
        "SCOPE_CONTEXT": selectors["context"],
        "SCOPE_POLICY": selectors["policy"],
        **dict(zip(LANES, results)),
    }
    return subprocess.run(
        ["bash", "-c", aggregate_script()],
        env={**os.environ, **env},
        capture_output=True,
        text=True,
        check=False,
    )


def git(repo, *args):
    result = subprocess.run(
        ["git", *args], cwd=repo, capture_output=True, text=True, check=False
    )
    if result.returncode:
        raise AssertionError(result.stderr)
    return result.stdout.strip()


def untrusted_contract_errors(workflow):
    errors = []
    lines = workflow.splitlines()
    for line in lines:
        if re.match(r"^\s*-\s*!", line):
            errors.append("workflow step tags are not supported by the trust checker")
        if re.match(r"^\s*(?:-\s*)?(?:\"(?:\\.|[^\"])*\"|'(?:''|[^'])*')\s*:", line):
            errors.append("workflow mapping keys must use the supported plain YAML form")
        if re.match(r"^\s*(?:-\s*)?uses:", line):
            match = re.fullmatch(
                r"\s*(?:-\s*)?uses:\s*(?:'([^']+)'|\"([^\"]+)\"|([^#\s]+))(?:\s+#.*)?",
                line,
            )
            action = next((part for part in match.groups() if part is not None), None) if match else None
            if not action or not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+@[0-9a-f]{40}", action):
                errors.append("actions must use simple full-SHA references")
    declared_action_references = len(
        re.findall(r"\b[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+@[^\s#]+", workflow)
    )
    parsed_uses_keys = sum(1 for line in lines if re.match(r"^\s*(?:-\s*)?uses:", line))
    if declared_action_references != parsed_uses_keys:
        errors.append("action references must use supported plain uses entries")
    permission_headers = [i for i, line in enumerate(lines) if line.strip() == "permissions:"]
    if len(permission_headers) != 1:
        errors.append("workflow must have one global permissions block")
    elif permission_headers:
        index = permission_headers[0]
        indent = len(lines[index]) - len(lines[index].lstrip())
        entries = []
        for line in lines[index + 1 :]:
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            line_indent = len(line) - len(line.lstrip())
            if line_indent <= indent:
                break
            entries.append(line.strip())
        if entries != ["contents: read"]:
            errors.append("workflow permissions must be exactly contents: read")
    if re.search(r"\$\{\{[^}]*\bsecrets\b", workflow, flags=re.DOTALL):
        errors.append("workflow must not reference secrets")
    lines = workflow.splitlines()
    checkouts = []
    for index, line in enumerate(lines):
        if re.match(r"^\s*-\s+uses:\s*['\"]?actions/checkout@", line):
            indent = len(line) - len(line.lstrip())
            step = []
            for following in lines[index + 1 :]:
                if following.strip() and len(following) - len(following.lstrip()) <= indent:
                    break
                step.append(following)
            checkouts.append("\n".join(step))
    if not checkouts:
        errors.append("workflow must contain checkout steps")
    # The lightweight fixture checker only understands the canonical plain
    # `uses:` form. Fail closed if a quoted key or another YAML spelling adds
    # an action reference that the checkout-step scanner did not recognize.
    checkout_references = len(re.findall(r"actions/checkout@", workflow))
    if checkout_references != len(checkouts):
        errors.append("checkout action references must use the supported plain uses key")
    for step in checkouts:
        step_lines = step.splitlines()
        with_headers = [i for i, line in enumerate(step_lines) if line.strip() == "with:"]
        has_disabled_credentials = False
        if len(with_headers) == 1:
            index = with_headers[0]
            indent = len(step_lines[index]) - len(step_lines[index].lstrip())
            for line in step_lines[index + 1 :]:
                if not line.strip() or line.lstrip().startswith("#"):
                    continue
                line_indent = len(line) - len(line.lstrip())
                if line_indent <= indent:
                    break
                if line_indent == indent + 2 and line.strip() == "persist-credentials: false":
                    has_disabled_credentials = True
                    break
        if not has_disabled_credentials:
            errors.append("every checkout must disable persisted credentials")
    return errors


class CiWorkflowTests(unittest.TestCase):
    def test_scope_requires_explicit_boolean_outputs_and_consistency(self):
        for field, value in (("changed", ""), ("native", "typo"), ("context", "null")):
            selectors = {"native": "false", "context": "false", "policy": "false"}
            changed = "false"
            if field == "changed":
                changed = value
            else:
                selectors[field] = value
            with self.subTest(field=field, value=value):
                result = run_aggregate(changed, ["skipped"] * len(LANES), selectors)
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        inconsistent = run_aggregate(
            "false",
            ["skipped"] * len(LANES),
            {"native": "true", "context": "false", "policy": "false"},
        )
        self.assertNotEqual(inconsistent.returncode, 0)

    def test_native_failure_cannot_be_masked_by_log_capture(self):
        native = WORKFLOW.read_text().split("  test:\n", 1)[1].split("  doctest:\n", 1)[0]
        self.assertIn("set -o pipefail", native)
        result = subprocess.run(["bash", "-c", "set -o pipefail; false | tee /dev/null"], capture_output=True)
        self.assertNotEqual(result.returncode, 0)

    def test_platform_and_policy_lanes_are_required(self):
        workflow = WORKFLOW.read_text()
        self.assertIn("os: [ubuntu-24.04, macos-15]", workflow)
        self.assertIn('test "$(uname -m)" = arm64', workflow)
        self.assertIn("fail-fast: false", workflow)
        self.assertIn("POLICY_RESULT", aggregate_script())
        self.assertEqual(run_aggregate("true", ["success"] * len(LANES)).returncode, 0)
        results = ["success"] * len(LANES)
        results[LANES.index("POLICY_RESULT")] = "failure"
        self.assertEqual(run_aggregate("true", results).returncode, 1)

    def test_ci_budgets_and_cache_trust_boundary(self):
        workflow = WORKFLOW.read_text()
        jobs = workflow.split("jobs:\n", 1)[1]
        blocks = re.split(r"^  [a-z_]+:\n", jobs, flags=re.MULTILINE)[1:]
        self.assertTrue(blocks)
        for block in blocks:
            timeout = re.search(r"^    timeout-minutes: ([0-9]+)$", block, re.MULTILINE)
            self.assertIsNotNone(timeout)
            self.assertLessEqual(int(timeout.group(1)), 20)
            self.assertGreater(int(timeout.group(1)), 0)
        self.assertNotIn("actions/cache@", workflow)
        self.assertNotIn("pull_request_target", workflow)
        self.assertIn("cancel-in-progress: ${{ github.event_name == 'pull_request' }}", workflow)

    def test_unit_lane_does_not_repeat_documentation_tests(self):
        workflow = WORKFLOW.read_text()
        native = workflow.split("  test:\n", 1)[1].split("  doctest:\n", 1)[0]
        self.assertIn("python3 tools/nextest_ci.py --toolchain 1.99.0", native)
        self.assertIn("test --doc --workspace --all-features --locked", workflow)

    def test_context_check_fetches_pinned_submodule_documents(self):
        context = WORKFLOW.read_text().split("  context:\n", 1)[1].split("  required:\n", 1)[0]
        self.assertIn("          submodules: recursive\n", context)

    def test_untrusted_pull_request_has_no_privileged_trigger_or_credentials(self):
        workflow = WORKFLOW.read_text()
        trigger_block = workflow.split("\non:", 1)[1].split("\npermissions:", 1)[0]
        triggers = {
            line.strip().split(":", 1)[0]
            for line in trigger_block.splitlines()
            if line.startswith("  ") and not line.startswith("    ")
        }
        self.assertEqual({"pull_request", "push"}, triggers)

        self.assertEqual([], untrusted_contract_errors(workflow))

    def test_untrusted_contract_fixture_rejects_extra_permissions_and_indexed_secrets(self):
        workflow = WORKFLOW.read_text()
        writable = workflow.replace(
            "permissions:\n  contents: read",
            "permissions:\n  contents: read\n  id-token: write",
            1,
        )
        self.assertTrue(untrusted_contract_errors(writable))

        blank_separated_permission = workflow.replace(
            "permissions:\n  contents: read",
            "permissions:\n  contents: read\n\n  id-token: write",
            1,
        )
        self.assertTrue(untrusted_contract_errors(blank_separated_permission))

        indexed_secret = workflow + '\n# ${{ secrets["TOKEN"] }}\n'
        self.assertIn("workflow must not reference secrets", untrusted_contract_errors(indexed_secret))

        whole_secret_context = workflow + "\n# ${{ toJSON(secrets) }}\n"
        self.assertIn("workflow must not reference secrets", untrusted_contract_errors(whole_secret_context))

        quoted_checkout = workflow.replace(
            "- uses: actions/checkout@",
            "- uses: 'actions/checkout@",
            1,
        ).replace(" # v6.0.2", "' # v6.0.2", 1)
        quoted_checkout = quoted_checkout.replace("persist-credentials: false", "persist-credentials: true", 1)
        self.assertIn(
            "every checkout must disable persisted credentials",
            untrusted_contract_errors(quoted_checkout),
        )

        quoted_key_checkout = workflow.replace(
            "- uses: actions/checkout@",
            '- "uses": actions/checkout@',
            1,
        ).replace("persist-credentials: false", "persist-credentials: true", 1)
        self.assertIn(
            "checkout action references must use the supported plain uses key",
            untrusted_contract_errors(quoted_key_checkout),
        )

        for quoted_key in ('"uses"', '"us\\u0065s"'):
            with self.subTest(quoted_key=quoted_key):
                escaped_key_action = workflow.replace(
                    "- name: Install Rust 1.99.0",
                    f"- {quoted_key}: actions/setup-go@v6\n      - name: Install Rust 1.99.0",
                    1,
                )
                self.assertIn(
                    "workflow mapping keys must use the supported plain YAML form",
                    untrusted_contract_errors(escaped_key_action),
                )

        nested_quoted_key_action = workflow.replace(
            "- name: Install Rust 1.99.0",
            '- name: Install Rust 1.99.0\n        "uses": actions/setup-go@v6',
            1,
        )
        self.assertIn(
            "workflow mapping keys must use the supported plain YAML form",
            untrusted_contract_errors(nested_quoted_key_action),
        )

        tagged_key_action = workflow.replace(
            "- name: Install Rust 1.99.0",
            "- !!str uses: actions/setup-go@v6\n      - name: Install Rust 1.99.0",
            1,
        )
        self.assertIn(
            "workflow step tags are not supported by the trust checker",
            untrusted_contract_errors(tagged_key_action),
        )

        env_only_checkout = workflow.replace(
            "with:\n          fetch-depth: 0\n          persist-credentials: false",
            "with:\n          fetch-depth: 0\n        env:\n          persist-credentials: false",
            1,
        )
        self.assertIn(
            "every checkout must disable persisted credentials",
            untrusted_contract_errors(env_only_checkout),
        )

        folded_checkout = re.sub(
            r"uses: actions/checkout@([0-9a-f]{40}) # v6.0.2",
            r"uses: >-\n          actions/checkout@\1",
            workflow,
            count=1,
        ).replace("persist-credentials: false", "persist-credentials: true", 1)
        self.assertTrue(untrusted_contract_errors(folded_checkout))

    def test_scope_outputs_and_lane_selectors_are_explicit(self):
        workflow = WORKFLOW.read_text()
        scope_job = workflow.split("  scope:\n", 1)[1].split("\n  fmt:", 1)[0]
        for selector in ("changed", "native", "context", "policy"):
            with self.subTest(selector=selector):
                self.assertIn(f"{selector}: ${{{{ steps.detect.outputs.{selector} }}}}", scope_job)
        self.assertIn("python3 tools/ci_scope.py --base", scope_job)
        for job in ("fmt", "clippy", "test", "msrv", "fuzz", "miri", "doctest"):
            with self.subTest(job=job):
                section = workflow_job(workflow, job)
                self.assertIn("if: needs.scope.outputs.native == 'true'", section)
        for job in ("context", "policy"):
            section = workflow_job(workflow, job)
            self.assertIn(f"if: needs.scope.outputs.{job} == 'true'", section)

    def test_planning_only_scope_runs_context_and_policy_without_native_lanes(self):
        selectors = {"native": "false", "context": "true", "policy": "true"}
        results = ["skipped"] * len(LANES)
        results[LANES.index("CONTEXT_RESULT")] = "success"
        results[LANES.index("POLICY_RESULT")] = "success"
        self.assertEqual(run_aggregate("true", results, selectors).returncode, 0)
        results[LANES.index("POLICY_RESULT")] = "skipped"
        self.assertNotEqual(run_aggregate("true", results, selectors).returncode, 0)

    def test_fuzz_changes_are_in_scope_and_fuzz_is_required_fail_closed(self):
        workflow = WORKFLOW.read_text()
        self.assertIn("if: needs.scope.outputs.native == 'true'", workflow.split("  fuzz:\n", 1)[1].split("  miri:\n", 1)[0])
        fuzz_job = workflow.split("  fuzz:\n", 1)[1].split("  doctest:\n", 1)[0]
        self.assertIn("nightly-2026-10-02", fuzz_job)
        self.assertIn("cargo-fuzz", fuzz_job)
        self.assertIn("cargo install cargo-fuzz --version 0.13.2 --locked", fuzz_job)
        self.assertNotIn("cargo +nightly-2026-10-02", fuzz_job)
        self.assertIn("tee .artifacts/ci/fuzz/install.log", fuzz_job)
        self.assertIn("tee -a .artifacts/ci/fuzz/install.log", fuzz_job)
        self.assertIn("grep -Fx 'host: x86_64-unknown-linux-gnu'", fuzz_job)
        self.assertIn('test "$(uname -m)" = x86_64', fuzz_job)
        self.assertIn("--locked", fuzz_job)
        self.assertIn("git diff --quiet -- fuzz/Cargo.lock", fuzz_job)
        self.assertIn("RUSTUP_TOOLCHAIN: nightly-2026-10-02", fuzz_job)
        self.assertIn('export RUSTC="$nightly_rustc"', fuzz_job)
        self.assertIn('export RUSTDOC="$nightly_bin/rustdoc"', fuzz_job)
        self.assertIn('export PATH="$nightly_bin:$PATH"', fuzz_job)
        self.assertIn('export PATH="$nightly_bin:$GITHUB_WORKSPACE/.artifacts/ci/fuzz-tools/bin:$PATH"', fuzz_job)
        self.assertIn("cargo fuzz run --sanitizer address parse_scenario_json", fuzz_job)
        self.assertNotIn("cargo +nightly-2026-10-02 fuzz", fuzz_job)
        self.assertIn("test -s .artifacts/ci/fuzz/fuzz.log", fuzz_job)
        self.assertIn("fuzz/artifacts/parse_scenario_json/", fuzz_job)
        self.assertIn("if-no-files-found: ignore", fuzz_job)
        for flag in (
            "-seed=20261003",
            "-runs=100000",
            "-max_total_time=120",
            "-max_len=32768",
            "-timeout=5",
            "-rss_limit_mb=1024",
        ):
            with self.subTest(flag=flag):
                self.assertIn(flag, fuzz_job)
        self.assertNotIn("continue-on-error", fuzz_job)
        self.assertIn("cargo deny --manifest-path fuzz/Cargo.toml", workflow)
        self.assertIn("FUZZ_RESULT", aggregate_script())
        self.assertIn("fuzz", workflow.split("  required:\n", 1)[1].split("    runs-on:", 1)[0])
        self.assertEqual(run_aggregate("true", ["success"] * len(LANES)).returncode, 0)
        for index in range(len(LANES)):
            results = ["success"] * len(LANES)
            results[index] = "failure"
            with self.subTest(failed_lane=LANES[index]):
                self.assertNotEqual(run_aggregate("true", results).returncode, 0)

    def test_fuzz_records_actual_address_sanitizer_binary_and_retains_proof(self):
        workflow = WORKFLOW.read_text()
        fuzz_job = workflow.split("  fuzz:\n", 1)[1].split("  miri:\n", 1)[0]
        for evidence in (
            "cargo fuzz build --sanitizer address parse_scenario_json",
            "cargo fuzz run --sanitizer address parse_scenario_json",
            'test -x "$fuzz_binary"',
            'nm --defined-only "$fuzz_binary"',
            "__asan_init$",
            'sha256sum "$fuzz_binary"',
            "sha256sum --check .artifacts/ci/fuzz/binary-sha256.txt",
            "name: parser-fuzz-instrumentation",
            "if: always()",
            "sanitizer-symbols.log",
            "binary-sha256.txt",
            "build.log",
        ):
            with self.subTest(evidence=evidence):
                self.assertIn(evidence, fuzz_job)
        proof = fuzz_job.split("name: parser-fuzz-instrumentation", 1)[1]
        self.assertIn("if-no-files-found: error", proof)
        self.assertNotIn("private-claim", proof)
        self.assertLess(fuzz_job.index("nm --defined-only"), fuzz_job.index("cargo fuzz run"))
        self.assertLess(fuzz_job.index("cargo fuzz run"), fuzz_job.index("sha256sum --check"))

    def test_fuzz_manifest_isolated_and_license_exception_is_version_scoped(self):
        fuzz_manifest = (WORKFLOW.parents[2] / "fuzz" / "Cargo.toml").read_text()
        fuzz_policy = (WORKFLOW.parents[2] / "fuzz" / "deny.toml").read_text()
        self.assertIn("publish = false", fuzz_manifest)
        self.assertIn('license = "Apache-2.0"', fuzz_manifest)
        self.assertIn("[workspace]", fuzz_manifest)
        self.assertIn('libfuzzer-sys = "=0.4.13"', fuzz_manifest)
        self.assertIn('careops-ed = { path = "../crates/careops-ed", version = "=0.1.0" }', fuzz_manifest)
        self.assertIn('serde_json = "=1.0.151"', fuzz_manifest)
        fuzz_lock = (WORKFLOW.parents[2] / "fuzz" / "Cargo.lock").read_text()
        self.assertIn('name = "libfuzzer-sys"\nversion = "0.4.13"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = ', fuzz_lock)
        workflow = WORKFLOW.read_text()
        self.assertIn("cargo deny --manifest-path fuzz/Cargo.toml --config fuzz/deny.toml --locked check", workflow)
        self.assertIn('version = "=0.4.13"', fuzz_policy)
        self.assertIn('name = "libfuzzer-sys"', fuzz_policy)
        self.assertIn('allow = ["NCSA"]', fuzz_policy)
        for root_policy in (
            'yanked = "deny"',
            'multiple-versions = "deny"',
            'allow = ["MIT", "Apache-2.0", "BSD-3-Clause", "Unicode-3.0"]',
            'unknown-registry = "deny"',
            'unknown-git = "deny"',
        ):
            with self.subTest(root_policy=root_policy):
                self.assertIn(root_policy, fuzz_policy)

    def test_miri_is_a_dated_linux_ffi_target_with_explicit_toolchain_and_logs(self):
        workflow = WORKFLOW.read_text()
        miri_job = workflow.split("  miri:\n", 1)[1].split("  doctest:\n", 1)[0]
        self.assertIn("timeout-minutes: 20", miri_job)
        self.assertIn("needs: scope", miri_job)
        self.assertIn("if: needs.scope.outputs.native == 'true'", miri_job)
        self.assertIn("runs-on: ubuntu-24.04", miri_job)
        self.assertIn("nightly-2026-10-02", miri_job)
        self.assertIn("--component miri --component rust-src", miri_job)
        self.assertIn('test "$(uname -s)" = Linux', miri_job)
        self.assertIn('test "$(uname -m)" = x86_64', miri_job)
        self.assertIn("host: x86_64-unknown-linux-gnu", miri_job)
        self.assertIn('export PATH="$nightly_bin:$PATH"', miri_job)
        self.assertIn('export RUSTUP_TOOLCHAIN=nightly-2026-10-02', miri_job)
        self.assertIn('export MIRI="$nightly_bin/miri"', miri_job)
        self.assertIn('test "$(command -v rustc)" = "$nightly_rustc"', miri_job)
        self.assertIn('test "$(command -v cargo)" = "$nightly_cargo"', miri_job)
        self.assertIn('cargo miri setup', miri_job)
        self.assertIn("cargo miri test --manifest-path libs/kairos/Cargo.toml -p kairo-ecs-ffi --test ffi_integration --locked", miri_job)
        self.assertIn(".artifacts/ci/miri/setup.log", miri_job)
        self.assertIn(".artifacts/ci/miri/test.log", miri_job)
        self.assertIn("retention-days: 7", miri_job)
        self.assertIn("MIRI_RESULT", workflow.split("  required:\n", 1)[1])

    def test_miri_result_is_fail_closed_in_changed_scope_aggregate(self):
        self.assertIn("MIRI_RESULT", aggregate_script())
        self.assertEqual(run_aggregate("true", ["success"] * len(LANES)).returncode, 0)
        for result in ("failure", "skipped", "cancelled"):
            results = ["success"] * len(LANES)
            results[LANES.index("MIRI_RESULT")] = result
            with self.subTest(miri_result=result):
                failed = run_aggregate("true", results)
                self.assertNotEqual(failed.returncode, 0, failed.stdout + failed.stderr)

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
            git(repo, "update-index", "--add", "--cacheinfo", f"160000,{first_pin},extensions/conductor")
            git(repo, "commit", "-qm", "baseline")
            base = git(repo, "rev-parse", "HEAD")
            for path in paths:
                (repo / path).write_text("after\n")
            git(repo, "add", ".")
            git(repo, "update-index", "--add", "--cacheinfo", f"160000,{second_pin},libs/kairos")
            git(repo, "update-index", "--add", "--cacheinfo", f"160000,{second_pin},extensions/conductor")
            git(repo, "commit", "-qm", "scoped changes")
            head = git(repo, "rev-parse", "HEAD")

            output = root / "github-output"
            command = [
                "python3", str(WORKFLOW.parents[2] / "tools" / "ci_scope.py"),
                "--base", base, "--head", head, "--output", str(output),
            ]
            result = subprocess.run(command, cwd=repo, capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(output.read_text(), "changed=true\nnative=true\ncontext=true\npolicy=true\n")

            output.write_text("")
            unchanged = subprocess.run(
                ["python3", command[1], "--base", head, "--head", head, "--output", str(output)],
                cwd=repo,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(unchanged.returncode, 0, unchanged.stdout + unchanged.stderr)
            self.assertEqual(output.read_text(), "changed=false\nnative=false\ncontext=false\npolicy=false\n")

    def test_cargo_lanes_initialize_path_dependency_and_pin_evidenced_toolchain(self):
        workflow = WORKFLOW.read_text()
        cargo_jobs = workflow.split("  required:", 1)[0]
        self.assertEqual(cargo_jobs.count("submodules: recursive"), 9)
        self.assertEqual(cargo_jobs.count("rustup toolchain install 1.99.0"), 5)
        for command in ("cargo +1.99.0 fmt", "cargo +1.99.0 clippy --locked", "python3 tools/nextest_ci.py --toolchain 1.99.0", "cargo +1.99.0 test --doc"):
            with self.subTest(command=command):
                self.assertIn(command, cargo_jobs)
        msrv = workflow_job(workflow, "msrv")
        self.assertIn("rustup toolchain install 1.76.0", msrv)
        self.assertIn("cargo test --workspace --lib --bins --tests --locked", msrv)
        self.assertNotIn("--all-features", msrv)

    def test_msrv_result_fails_when_selected_and_must_skip_when_unselected(self):
        self.assertIn("MSRV_RESULT", aggregate_script())
        results = ["success"] * len(LANES)
        results[LANES.index("MSRV_RESULT")] = "failure"
        failed = run_aggregate("true", results)
        self.assertNotEqual(failed.returncode, 0, failed.stdout + failed.stderr)
        results[LANES.index("MSRV_RESULT")] = "skipped"
        self.assertNotEqual(run_aggregate("true", results).returncode, 0)
        self.assertEqual(
            run_aggregate("false", ["skipped"] * len(LANES)).returncode,
            0,
        )

    def test_required_lane_failure_fails_aggregate(self):
        result = run_aggregate("true", ["success", "failure"] + ["success"] * (len(LANES) - 2))
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("selected required lane", result.stdout)

    def test_unchanged_scope_is_an_explicit_successful_skip(self):
        result = run_aggregate("false", ["skipped"] * len(LANES))
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("all selected required lanes passed; changed=false", result.stdout)

    def test_changed_scope_cannot_hide_a_skipped_lane(self):
        result = run_aggregate("true", ["success", "skipped"] + ["success"] * (len(LANES) - 2))
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)

    def test_changed_scope_fails_when_required_lane_result_is_absent(self):
        missing = "MIRI_RESULT"
        env = {
            key: value
            for key, value in os.environ.items()
            if key not in LANES and key not in {"SCOPE_RESULT", "SCOPE_CHANGED", "SCOPE_NATIVE", "SCOPE_CONTEXT", "SCOPE_POLICY"}
        }
        env.update(
            {
                "SCOPE_RESULT": "success",
                "SCOPE_CHANGED": "true",
                "SCOPE_NATIVE": "true",
                "SCOPE_CONTEXT": "true",
                "SCOPE_POLICY": "true",
                **{lane: "success" for lane in LANES if lane != missing},
            }
        )
        self.assertNotIn(missing, env)

        result = subprocess.run(
            ["bash", "-c", aggregate_script()],
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
