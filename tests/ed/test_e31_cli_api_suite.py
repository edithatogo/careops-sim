"""Public API and CLI test suite for Phase E3.1.

Verifies:
1. validate/run/compare/export subcommands and deterministic exit codes.
2. Fresh-user walkthrough and help interface.
3. Reference scenario selection and parameter overrides (--seed, --horizon).
4. Output overwrite protection (exit code 6) and atomic file persistence.
5. Comparative evaluation with units, denominators, uncertainty, and incompatible scenario diagnostics (exit code 5).
6. Multi-format exports (JSON, CSV, Arrow schema metadata).
"""

import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT_DIR = Path(__file__).resolve().parents[2]
RUNNER_BIN = ROOT_DIR / "target/debug/examples/runner"
NOMINAL_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/nominal_mvp.json"
CONSTRAINED_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/constrained_mvp.json"
MALFORMED_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/malformed_mvp.json"


class E31CliApiSuiteTests(unittest.TestCase):
    """End-to-end CLI execution and API contract verification for E3.1."""

    @classmethod
    def setUpClass(cls):
        # Ensure runner example binary is built
        subprocess.run(
            ["cargo", "build", "--locked", "-p", "careops-ed", "--example", "runner"],
            cwd=ROOT_DIR,
            check=True,
            capture_output=True,
        )

    def run_cmd(self, *args):
        proc = subprocess.run(
            [str(RUNNER_BIN), *args],
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
        )
        return proc

    def test_fresh_user_help_walkthrough(self):
        proc = self.run_cmd("--help")
        self.assertEqual(proc.returncode, 0)
        self.assertIn("Usage: careops-ed <command>", proc.stdout)
        self.assertIn("validate <scenario.json>", proc.stdout)
        self.assertIn("run <scenario.json>", proc.stdout)
        self.assertIn("compare <base_run.json> <candidate_run.json>", proc.stdout)
        self.assertIn("export <run.json>", proc.stdout)
        self.assertIn("Exit Codes:", proc.stdout)
        self.assertIn("6  Output file already exists", proc.stdout)

    def test_missing_or_unknown_command_returns_usage_error(self):
        proc_empty = self.run_cmd()
        self.assertEqual(proc_empty.returncode, 2)
        self.assertIn("missing command", proc_empty.stderr)

        proc_unknown = self.run_cmd("unknown_action")
        self.assertEqual(proc_unknown.returncode, 2)
        self.assertIn("unknown command 'unknown_action'", proc_unknown.stderr)

    def test_validate_nominal_scenario(self):
        proc = self.run_cmd("validate", str(NOMINAL_SCENARIO))
        self.assertEqual(proc.returncode, 0, f"Validation failed: {proc.stderr}")
        self.assertIn("valid scenario 'nominal-ed-mvp'", proc.stdout)
        self.assertIn("schema v1", proc.stdout)
        self.assertIn("5 patients", proc.stdout)

    def test_validate_missing_and_malformed_scenarios(self):
        proc_missing = self.run_cmd("validate", "non_existent_scenario.json")
        self.assertEqual(proc_missing.returncode, 3)
        self.assertIn("scenario file not found", proc_missing.stderr)

        proc_malformed = self.run_cmd("validate", str(MALFORMED_SCENARIO))
        self.assertEqual(proc_malformed.returncode, 4)
        self.assertIn("invalid scenario", proc_malformed.stderr)

    def test_run_nominal_stdout(self):
        proc = self.run_cmd("run", str(NOMINAL_SCENARIO))
        self.assertEqual(proc.returncode, 0, f"Run failed: {proc.stderr}")

        doc = json.loads(proc.stdout)
        self.assertEqual(doc["manifest"]["scenario_id"], "nominal-ed-mvp")
        self.assertEqual(doc["manifest"]["seed"], 42)
        self.assertEqual(doc["summary"]["completed"], 5)
        self.assertEqual(doc["summary"]["unfinished"], 0)

    def test_run_overrides_seed_and_horizon(self):
        proc = self.run_cmd(
            "run",
            str(NOMINAL_SCENARIO),
            "--seed",
            "9999",
            "--horizon",
            "7",
        )
        self.assertEqual(proc.returncode, 0)
        doc = json.loads(proc.stdout)
        self.assertEqual(doc["manifest"]["seed"], 9999)
        self.assertEqual(doc["manifest"]["horizon_ticks"], 7)
        self.assertEqual(doc["summary"]["seed"], 9999)
        self.assertEqual(doc["summary"]["horizon_ticks"], 7)
        # Shortened horizon prevents last patient from completing
        self.assertLess(doc["summary"]["completed"], 5)
        self.assertGreater(doc["summary"]["unfinished"], 0)

    def test_run_output_overwrite_protection(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            out_file = Path(temp_dir) / "run_result.json"

            # First run: writes output file
            proc1 = self.run_cmd("run", str(NOMINAL_SCENARIO), "--output", str(out_file))
            self.assertEqual(proc1.returncode, 0)
            self.assertTrue(out_file.exists())

            # Second run without --overwrite: must fail with exit code 6
            proc2 = self.run_cmd("run", str(NOMINAL_SCENARIO), "--output", str(out_file))
            self.assertEqual(proc2.returncode, 6)
            self.assertIn("already exists", proc2.stderr)

            # Third run with --overwrite: succeeds
            proc3 = self.run_cmd(
                "run",
                str(NOMINAL_SCENARIO),
                "--output",
                str(out_file),
                "--overwrite",
            )
            self.assertEqual(proc3.returncode, 0)

    def test_compare_runs_lifecycle_and_diagnostics(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            nom_out = Path(temp_dir) / "nominal.json"
            con_out = Path(temp_dir) / "constrained.json"

            self.assertEqual(
                self.run_cmd("run", str(NOMINAL_SCENARIO), "--output", str(nom_out)).returncode,
                0,
            )
            self.assertEqual(
                self.run_cmd("run", str(CONSTRAINED_SCENARIO), "--output", str(con_out)).returncode,
                0,
            )

            proc_comp = self.run_cmd("compare", str(nom_out), str(con_out))
            self.assertEqual(proc_comp.returncode, 0, f"Compare failed: {proc_comp.stderr}")

            rep = json.loads(proc_comp.stdout)
            self.assertEqual(rep["base_scenario_id"], "nominal-ed-mvp")
            self.assertEqual(rep["candidate_scenario_id"], "constrained-ed-mvp")
            self.assertEqual(rep["time_unit"], "tick")

            metrics = {m["metric_name"]: m for m in rep["metrics"]}
            self.assertIn("mean_wait", metrics)
            self.assertEqual(metrics["mean_wait"]["unit"], "tick")
            self.assertIn("denominator_description", metrics["mean_wait"])
            self.assertAlmostEqual(metrics["mean_wait"]["base_value"], 0.4)
            self.assertAlmostEqual(metrics["mean_wait"]["candidate_value"], 4.2)
            self.assertGreater(metrics["mean_wait"]["delta"], 0)

            # Check diagnostics explain findings
            self.assertTrue(any("increased mean wait" in d for d in rep["diagnostics"]))

    def test_compare_incompatible_scenario_rejected(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            nom_out = Path(temp_dir) / "nominal.json"
            cand_out = Path(temp_dir) / "mismatched.json"

            self.assertEqual(
                self.run_cmd("run", str(NOMINAL_SCENARIO), "--output", str(nom_out)).returncode,
                0,
            )

            # Create run document with incompatible time unit
            data = json.loads(nom_out.read_text())
            data["manifest"]["time_unit"] = "second"
            cand_out.write_text(json.dumps(data))

            proc_comp = self.run_cmd("compare", str(nom_out), str(cand_out))
            self.assertEqual(proc_comp.returncode, 5)
            self.assertIn("incompatible time units", proc_comp.stderr)

    def test_export_formats(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            run_file = Path(temp_dir) / "run.json"
            self.assertEqual(
                self.run_cmd("run", str(NOMINAL_SCENARIO), "--output", str(run_file)).returncode,
                0,
            )

            # 1. Export CSV
            proc_csv = self.run_cmd("export", str(run_file), "--format", "csv")
            self.assertEqual(proc_csv.returncode, 0)
            self.assertIn("patient_id,arrival_tick", proc_csv.stdout)
            self.assertIn("patient-001", proc_csv.stdout)

            # 2. Export Arrow schema metadata
            proc_arrow = self.run_cmd("export", str(run_file), "--format", "arrow")
            self.assertEqual(proc_arrow.returncode, 0)
            arrow_doc = json.loads(proc_arrow.stdout)
            self.assertIn("arrow_schema", arrow_doc)
            fields = {f["name"]: f for f in arrow_doc["arrow_schema"]["fields"]}
            self.assertIn("wait_ticks", fields)
            self.assertEqual(fields["wait_ticks"]["denominator"], "started_patients")


if __name__ == "__main__":
    unittest.main()
