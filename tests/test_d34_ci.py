import json
from pathlib import Path
import re
import unittest


WORKFLOW = Path(__file__).parents[1] / ".github" / "workflows" / "ci.yml"


def workflow_job(workflow, name):
    match = re.search(
        rf"(?ms)^  {re.escape(name)}:\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)",
        workflow,
    )
    if not match:
        raise AssertionError(f"missing workflow job {name}")
    return match.group(0)


class D34CiTests(unittest.TestCase):
    def test_context_benchmark_runs_after_quality_inventory_without_native_gate(self):
        context = workflow_job(WORKFLOW.read_text(), "context")
        quality = context.index("- name: Check reviewed support and flaky-test inventory")
        benchmark = context.index("- name: Run representative ED benchmark")
        retention = context.index("- name: Retain D3.4 benchmark report")
        coverage = context.index("- name: Measure bounded synthetic ED library coverage")
        self.assertLess(quality, benchmark)
        self.assertLess(benchmark, retention)
        self.assertLess(retention, coverage)

        build = context.index("- name: Build native CLI for Python consumers")
        harness = context.index("- name: Run harness and incompatible-pin regressions")
        self.assertLess(build, harness)
        self.assertLess(harness, quality)
        self.assertIn("CAREOPS_ED_PROFILE: 'release'", context)
        step = context[build:harness]
        self.assertNotIn("if:", step)
        self.assertNotIn("if:", context[benchmark:retention])
        self.assertNotIn("continue-on-error", context[benchmark:retention])
        self.assertIn("rustup toolchain install 1.99.1 --profile minimal", step)
        self.assertIn(
            "cargo build --locked --release --package careops-ed-cli",
            step,
        )
        for binding in (
            'rustc_path="$(rustup which rustc --toolchain 1.99.1)"',
            'rustdoc_path="$(rustup which rustdoc --toolchain 1.99.1)"',
            'cargo_path="$(rustup which cargo --toolchain 1.99.1)"',
            'toolchain_bin="$(dirname "$rustc_path")"',
            'export PATH="$toolchain_bin:$PATH" RUSTUP_TOOLCHAIN=1.99.1 RUSTC="$rustc_path" RUSTDOC="$rustdoc_path"',
            'test "$(command -v rustc)" = "$rustc_path"',
            'test "$(command -v rustdoc)" = "$rustdoc_path"',
            'test "$(command -v cargo)" = "$cargo_path"',
        ):
            with self.subTest(binding=binding):
                self.assertIn(binding, step)
        self.assertIn(
            '"$CAREOPS_PYTHON" tools/ed_native_benchmarks.py --binary target/release/careops-ed '
            "--output .artifacts/ci/d34/report.json",
            context[benchmark:retention],
        )
        self.assertNotIn("continue-on-error", step)
        self.assertEqual("90", re.search(r"^    timeout-minutes: (\d+)$", context, re.M).group(1))
        self.assertIn("CARGO_PROFILE_DEV_DEBUG: '0'", context)
        self.assertIn("CARGO_PROFILE_TEST_DEBUG: '0'", context)
        self.assertIn("--timeout 180", context)

    def test_benchmark_report_is_retained_even_after_failure(self):
        context = workflow_job(WORKFLOW.read_text(), "context")
        retention = context.split("- name: Retain D3.4 benchmark report", 1)[1]
        step = retention.split("      - name:", 1)[0]
        self.assertIn("if: always()", step)
        pins = json.loads((WORKFLOW.parents[2] / "tools/ci-tools.json").read_text())
        self.assertIn("uses: actions/upload-artifact@" + pins["actions"]["upload-artifact"], step)
        self.assertIn("name: d34-benchmarks-${{ github.run_id }}", step)
        self.assertIn("path: .artifacts/ci/d34/report.json", step)
        self.assertIn("if-no-files-found: error", step)
        self.assertIn("retention-days: 7", step)


if __name__ == "__main__":
    unittest.main()
