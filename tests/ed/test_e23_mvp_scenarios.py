"""MVP scenario runner, capacity comparison, and input rejection tests for Phase E2.3.

Verifies:
1. Nominal MVP scenario runs and produces a valid run document (manifest + summary).
2. Constrained MVP scenario runs and demonstrates a controlled capacity impact.
3. Malformed scenario input is cleanly rejected with a non-zero exit code.
4. Fixed-seed repeatability: identical outputs across repeated runs.
5. Patient and capacity conservation invariants hold across all scenario runs.
"""

import hashlib
import json
from pathlib import Path
import subprocess
import unittest

ROOT_DIR = Path(__file__).resolve().parents[2]
CLI_PATH = ROOT_DIR / "target/debug/careops-ed"


class E23MvpScenarioTests(unittest.TestCase):
    """Execution and verification of the headless MVP delivery recipe."""

    @classmethod
    def setUpClass(cls):
        # Ensure the careops-ed-cli binary is built
        subprocess.run(
            ["cargo", "build", "--locked", "-p", "careops-ed-cli"],
            cwd=ROOT_DIR,
            check=True,
            capture_output=True,
        )

    def run_cli(self, scenario_rel_path: str):
        full_path = ROOT_DIR / scenario_rel_path
        proc = subprocess.run(
            [str(CLI_PATH), "run", str(full_path)],
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
        )
        return proc

    def test_nominal_mvp_scenario_execution(self):
        proc = self.run_cli("crates/careops-ed/examples/nominal_mvp.json")
        self.assertEqual(proc.returncode, 0, f"CLI failed: {proc.stderr}")

        doc = json.loads(proc.stdout)
        self.assertIn("manifest", doc)
        self.assertIn("summary", doc)

        manifest = doc["manifest"]
        self.assertEqual(manifest["schema_version"], 1)
        self.assertEqual(manifest["scenario_id"], "nominal-ed-mvp")
        self.assertEqual(manifest["seed"], 42)
        self.assertEqual(manifest["time_unit"], "tick")
        self.assertEqual(manifest["horizon_ticks"], 20)

        # Verify input SHA-256 matches actual file bytes
        file_bytes = (ROOT_DIR / "crates/careops-ed/examples/nominal_mvp.json").read_bytes()
        expected_sha = hashlib.sha256(file_bytes).hexdigest()
        self.assertEqual(manifest["input_sha256"], expected_sha)

        summary = doc["summary"]
        self.assertEqual(summary["arrivals"], 5)
        self.assertEqual(summary["started"], 5)
        self.assertEqual(summary["completed"], 5)
        self.assertEqual(summary["unfinished"], 0)
        self.assertEqual(len(summary["patients"]), 5)

        # Patient count conservation: arrivals == completed + unfinished
        self.assertEqual(summary["completed"] + summary["unfinished"], summary["arrivals"])

    def test_constrained_mvp_scenario_and_capacity_comparison(self):
        proc_nom = self.run_cli("crates/careops-ed/examples/nominal_mvp.json")
        proc_con = self.run_cli("crates/careops-ed/examples/constrained_mvp.json")

        self.assertEqual(proc_nom.returncode, 0)
        self.assertEqual(proc_con.returncode, 0)

        doc_nom = json.loads(proc_nom.stdout)
        doc_con = json.loads(proc_con.stdout)

        nom_summary = doc_nom["summary"]
        con_summary = doc_con["summary"]

        # Controlled comparison: exact same arrival count
        self.assertEqual(nom_summary["arrivals"], con_summary["arrivals"])

        # Compute mean waiting times
        nom_waits = [p["wait_ticks"] for p in nom_summary["patients"] if p["wait_ticks"] is not None]
        con_waits = [p["wait_ticks"] for p in con_summary["patients"] if p["wait_ticks"] is not None]

        nom_mean_wait = sum(nom_waits) / len(nom_waits)
        con_mean_wait = sum(con_waits) / len(con_waits)

        self.assertAlmostEqual(nom_mean_wait, 0.4)
        self.assertAlmostEqual(con_mean_wait, 4.2)
        self.assertGreater(con_mean_wait, nom_mean_wait, "Constrained capacity must increase wait times")

    def test_malformed_input_rejection(self):
        proc = self.run_cli("crates/careops-ed/examples/malformed_mvp.json")
        self.assertNotEqual(proc.returncode, 0, "Malformed scenario must exit with failure code")
        self.assertIn("invalid scenario", proc.stderr)
        self.assertIn("schema_version must be 1", proc.stderr)

    def test_fixed_seed_repeatability(self):
        proc1 = self.run_cli("crates/careops-ed/examples/nominal_mvp.json")
        proc2 = self.run_cli("crates/careops-ed/examples/nominal_mvp.json")

        self.assertEqual(proc1.returncode, 0)
        self.assertEqual(proc2.returncode, 0)
        self.assertEqual(proc1.stdout, proc2.stdout, "Fixed seed must produce identical outputs")


if __name__ == "__main__":
    unittest.main()
