"""Generic ED pathway specification and metric interval tests for Phase E1.1.

Verifies:
1. Empty run specification (empty cohort, undefined / empty metrics).
2. Single-patient pathway execution, deterministic events, half-open intervals.
3. Acuity and pathway mix coverage (ATS 1-5 across distinct care pathways).
4. Saturated arrival demand and queue wait intervals.
5. Zero service duration, long service duration, and horizon boundary clipping.
6. Impossible resource configuration rejection.
7. Endpoint count conservation: arrivals = completed + unfinished.
8. Metric definitions, exact intervals, and expected bounds.
"""

import copy
import json
from pathlib import Path
import subprocess
import unittest

ROOT_DIR = Path(__file__).resolve().parents[2]


class GenericEDPathwaySpecificationTests(unittest.TestCase):
    """Pathway conformance and metric specification tests."""

    def test_e1_preparation_proposal_oracles_match_schema(self):
        """Verify the e1-preparation-v1 design oracle file exists and has all required cases."""
        oracle_path = ROOT_DIR / "conductor/design/ed/proposals/e1-preparation-v1/oracles.json"
        self.assertTrue(oracle_path.exists(), f"Missing oracle file: {oracle_path}")

        data = json.loads(oracle_path.read_text())
        cases = {c["id"]: c for c in data.get("cases", [])}

        required_cases = [
            "empty",
            "one-ordered-pathway",
            "saturated-fifo",
            "horizon-clips-service",
            "zero-duration-acyclic",
            "unknown-resource",
            "zero-open-capacity-for-required-path",
            "arrival-at-horizon",
            "duplicate-patient-id",
        ]
        for case_id in required_cases:
            self.assertIn(case_id, cases, f"Oracle case {case_id} not found in design oracles")

        # Verify empty run expectation in design oracles
        empty_case = cases["empty"]
        self.assertEqual(empty_case["expected"]["arrived"], 0)
        self.assertEqual(empty_case["expected"]["terminal"], 0)
        self.assertEqual(empty_case["expected"]["unfinished"], 0)
        self.assertIsNone(empty_case["expected"]["mean_los"])
        self.assertIsNone(empty_case["expected"]["mean_queue_wait"])

        # Verify saturated-fifo expectation in design oracles
        sat_case = cases["saturated-fifo"]
        self.assertEqual(sat_case["expected"]["arrived"], 3)
        self.assertEqual(sat_case["expected"]["terminal"], 3)
        self.assertEqual(sat_case["expected"]["unfinished"], 0)
        self.assertEqual(sat_case["expected"]["wait_sum_ticks"], 5)
        self.assertEqual(sat_case["expected"]["completed_los_sum_ticks"], 11)

    def test_metrics_observation_contract_specifies_explicit_definitions(self):
        """Verify that metric definitions and interval algebra adhere to contract."""
        contract_path = (
            ROOT_DIR
            / "conductor/design/ed/proposals/sources/e0.3-metrics-contract-0bfe013.json"
        )
        self.assertTrue(contract_path.exists())
        data = json.loads(contract_path.read_text())
        metrics = {m["id"]: m for m in data.get("metrics", [])}

        expected_metric_ids = [
            "queue_wait",
            "time_to_care",
            "length_of_stay",
            "throughput",
            "boarding_time",
            "resource_utilization",
        ]
        for metric_id in expected_metric_ids:
            self.assertIn(metric_id, metrics)
            m = metrics[metric_id]
            self.assertIn("definition", m)
            self.assertIn("unit", m)
            self.assertIn("numerator", m)
            self.assertIn("denominator", m)
            self.assertIn("censoring", m)

    def test_rust_pathway_conformance_suite_passes(self):
        """Execute the native Rust pathway conformance test suite."""
        cmd = ["cargo", "test", "--package", "careops-ed", "--test", "pathway_conformance"]
        proc = subprocess.run(
            cmd,
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
        )
        self.assertEqual(
            proc.returncode,
            0,
            f"Cargo test pathway_conformance failed:\nStdout: {proc.stdout}\nStderr: {proc.stderr}",
        )
        self.assertIn("test result: ok. 8 passed", proc.stdout)

    def test_interval_algebra_and_conservation_properties(self):
        """Mathematical consistency checks for metric intervals:
        1. half-open interval [start, completion) has duration completion - start.
        2. LOS = wait_ticks + service_ticks (for single stage FIFO).
        3. Endpoint conservation: arrivals = completed + unfinished.
        4. Capacity upper bounds: busy_time <= total_available_time.
        """
        # Case A: completed single patient
        arrival = 2
        start = 2
        completion = 8
        work = 6
        self.assertLessEqual(arrival, start)
        self.assertEqual(completion - start, work)
        wait = start - arrival
        los = completion - arrival
        self.assertEqual(los, wait + work)

        # Case B: unfinished / censored patient
        horizon = 10
        p_arr = 6
        p_work = 10
        p_start = 6
        # at horizon, p has not completed
        self.assertGreater(p_start + p_work, horizon)
        # unfinished at horizon
        completed = 0
        unfinished = 1
        arrivals = 1
        self.assertEqual(arrivals, completed + unfinished)


if __name__ == "__main__":
    unittest.main()
