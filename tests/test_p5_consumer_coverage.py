"""Regression tests for Phase P5 consumer/catalogue coverage and profile validation."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
VALIDATOR_TOOL = ROOT / "tools/validate_p5_consumer_coverage.py"
SMOKE_TOOL = ROOT / "tools/run_p52_example_smoke.py"
NOMINAL_SCENARIO = ROOT / "crates/careops-ed/examples/nominal_mvp.json"


class P5ConsumerCoverageTests(unittest.TestCase):
    def test_p51_default_scenarios_pass_coverage(self):
        proc = subprocess.run(
            [sys.executable, str(VALIDATOR_TOOL)],
            cwd=ROOT,
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("Registry-matrix coverage verified", proc.stdout)
        self.assertIn("P5.1 consumer coverage PASS", proc.stdout)

    def test_p51_unregistered_field_is_rejected(self):
        scenario = json.loads(NOMINAL_SCENARIO.read_text(encoding="utf-8"))
        scenario["unregistered_bogus_field"] = "bad_value"

        with tempfile.NamedTemporaryFile(suffix=".json", mode="w", delete=False) as f:
            f.write(json.dumps(scenario))
            tmp_path = Path(f.name)

        try:
            proc = subprocess.run(
                [sys.executable, str(VALIDATOR_TOOL), "--scenario", str(tmp_path)],
                cwd=ROOT,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(proc.returncode, 0)
            self.assertIn("unregistered configuration field 'unregistered_bogus_field' rejected", proc.stderr)
        finally:
            tmp_path.unlink(missing_ok=True)

    def test_p51_unsupported_schema_version_is_rejected(self):
        scenario = json.loads(NOMINAL_SCENARIO.read_text(encoding="utf-8"))
        scenario["schema_version"] = 999

        with tempfile.NamedTemporaryFile(suffix=".json", mode="w", delete=False) as f:
            f.write(json.dumps(scenario))
            tmp_path = Path(f.name)

        try:
            proc = subprocess.run(
                [sys.executable, str(VALIDATOR_TOOL), "--scenario", str(tmp_path)],
                cwd=ROOT,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(proc.returncode, 0)
            self.assertIn("unsupported schema_version '999'", proc.stderr)
        finally:
            tmp_path.unlink(missing_ok=True)

    def test_p52_example_smoke_repetition(self):
        proc = subprocess.run(
            [sys.executable, str(SMOKE_TOOL)],
            cwd=ROOT,
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("P5.2 smoke PASS", proc.stdout)


if __name__ == "__main__":
    unittest.main()
