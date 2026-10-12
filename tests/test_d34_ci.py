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
        benchmark = context.index("- name: Build and run representative ED benchmark")
        retention = context.index("- name: Retain D3.4 benchmark report")
        coverage = context.index("- name: Measure bounded synthetic ED library coverage")
        self.assertLess(quality, benchmark)
        self.assertLess(benchmark, retention)
        self.assertLess(retention, coverage)

        step = context[benchmark:retention]
        self.assertNotIn("if:", step)
        self.assertIn("rustup toolchain install 1.99.0 --profile minimal", step)
        self.assertIn(
            "cargo build --locked --release --package careops-ed-cli",
            step,
        )
        for binding in (
            'rustc_path="$(rustup which rustc --toolchain 1.99.0)"',
            'rustdoc_path="$(rustup which rustdoc --toolchain 1.99.0)"',
            'cargo_path="$(rustup which cargo --toolchain 1.99.0)"',
            'toolchain_bin="$(dirname "$rustc_path")"',
            'export PATH="$toolchain_bin:$PATH" RUSTUP_TOOLCHAIN=1.99.0 RUSTC="$rustc_path" RUSTDOC="$rustdoc_path"',
            'test "$(command -v rustc)" = "$rustc_path"',
            'test "$(command -v rustdoc)" = "$rustdoc_path"',
            'test "$(command -v cargo)" = "$cargo_path"',
        ):
            with self.subTest(binding=binding):
                self.assertIn(binding, step)
        self.assertIn(
            "python3 tools/ed_benchmarks.py --binary target/release/careops-ed "
            "--output .artifacts/ci/d34/report.json",
            step,
        )
        self.assertNotIn("continue-on-error", step)
        self.assertEqual("20", re.search(r"^    timeout-minutes: (\d+)$", context, re.M).group(1))

    def test_benchmark_report_is_retained_even_after_failure(self):
        context = workflow_job(WORKFLOW.read_text(), "context")
        retention = context.split("- name: Retain D3.4 benchmark report", 1)[1]
        step = retention.split("      - name:", 1)[0]
        self.assertIn("if: always()", step)
        self.assertIn(
            "uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02",
            step,
        )
        self.assertIn("name: d34-benchmarks-${{ github.run_id }}", step)
        self.assertIn("path: .artifacts/ci/d34/report.json", step)
        self.assertIn("if-no-files-found: error", step)
        self.assertIn("retention-days: 7", step)


if __name__ == "__main__":
    unittest.main()
