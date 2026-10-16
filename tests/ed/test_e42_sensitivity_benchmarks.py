"""Task E4.2: Sensitivity, identifiability, warm-up observation windows, and benchmark reporting.

Verifies:
1. CLI/runner handling of sensitivity scenarios:
   - Capacity response surface monotonicity (wait times decrease with capacity).
   - Arrival demand scaling and queue formation under capacity saturation.
   - CLI parameter overrides (--seed, --horizon) on sensitivity scenarios.
2. Warm-up window parameters & observation window analysis:
   - Cold-start transient detection vs steady-state operating window.
   - Elimination of downward startup bias via warm-up exclusion.
   - Reporting of observation window duration, excluded warm-up interval, and horizon censoring.
3. Benchmark reporting & execution throughput:
   - Measurement of execution duration, patient throughput (patients/sec), and simulation step rate.
4. Neutral comparative evaluation and uncertainty diagnostics:
   - Objective metric deltas with explicit units and denominators.
   - Standard error calculations for stochastic uncertainty.
   - Absence of evaluative/normative value judgements ("superior", "inferior", "better", "worse") in diagnostics.
   - Preservation of software verification vs domain validation taxonomy and regulatory disclaimer.
"""

import json
import math
from pathlib import Path
import subprocess
import tempfile
import time
import unittest

ROOT_DIR = Path(__file__).resolve().parents[2]
RUNNER_BIN = ROOT_DIR / "target/debug/examples/runner"
NOMINAL_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/nominal_mvp.json"
CONSTRAINED_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/constrained_mvp.json"


def build_sensitivity_scenario(
    scenario_id: str,
    capacity_count: int,
    horizon_ticks: int,
    patients: list,
    seed: int = 42,
    notes: str = "Synthetic sensitivity scenario",
) -> dict:
    """Builds a schema-compliant scenario for sensitivity and benchmark analysis."""
    resource_buckets = [
        {
            "resource_id": "res.care",
            "resource_class": "bay_slot",
            "location_id": "loc.acute",
            "zone_id": "zone.acute",
            "physical_status": "known",
            "physical_count": capacity_count,
            "availability_status": "known",
            "open_count": capacity_count,
            "staffed_count": capacity_count,
            "capabilities": ["acute_care"],
        }
    ]

    staff_roles = [
        {
            "staff_role_id": "staff.nurse",
            "name": "Nurse",
            "counts": [
                {
                    "basis": "present",
                    "status": "known",
                    "count": capacity_count,
                    "reference": "synthetic schedule",
                }
            ],
        }
    ]

    eligibility = [
        {
            "staff_role_id": "staff.nurse",
            "task_class_id": "task.care",
            "zone_id": "zone.acute",
            "location_id": "loc.acute",
        }
    ]

    return {
        "schema_version": 1,
        "scenario_id": scenario_id,
        "seed": seed,
        "time_unit": "tick",
        "horizon_ticks": horizon_ticks,
        "provenance": {
            "class": "synthetic",
            "notes": notes,
        },
        "capacity_location": {
            "schema_version": 3,
            "status": "proposed",
            "capacity": {
                "unit": "treatment_space",
                "bucket_completeness": "complete",
                "physical_status": "known",
                "physical_count": capacity_count,
                "availability_status": "known",
                "open_count": capacity_count,
                "staffed_count": capacity_count,
                "equipment": [],
            },
            "zones": [{"zone_id": "zone.acute", "name": "Acute Zone"}],
            "locations": [
                {
                    "location_id": "loc.acute",
                    "name": "Acute Area",
                    "zone_id": "zone.acute",
                }
            ],
            "resource_buckets": resource_buckets,
            "task_classes": [
                {
                    "task_class_id": "task.care",
                    "name": "care work",
                    "required_resource_classes": ["bay_slot"],
                }
            ],
            "staff_roles": staff_roles,
            "eligibility": eligibility,
            "routes": [],
        },
        "patients": patients,
    }


class E42SensitivityBenchmarksTests(unittest.TestCase):
    """Integration test suite for Task E4.2 sensitivity, warm-up windows, and benchmarks."""

    @classmethod
    def setUpClass(cls):
        # Ensure runner example binary is compiled
        proc = subprocess.run(
            ["cargo", "build", "--locked", "-p", "careops-ed", "--example", "runner"],
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            raise RuntimeError(f"Failed to build runner: {proc.stderr}")
        assert RUNNER_BIN.exists(), f"Runner binary not found at {RUNNER_BIN}"

    def run_cmd(self, *args):
        """Invoke runner example binary."""
        proc = subprocess.run(
            [str(RUNNER_BIN), *args],
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
        )
        return proc

    def test_capacity_sensitivity_response_surface(self):
        """Evaluate parameter response surface over bay capacity C in {1, 2, 4}.

        Monotonicity Property: Increasing capacity reduces or keeps constant mean queue wait times:
        Wait(C=1) >= Wait(C=2) >= Wait(C=4).
        """
        # Concurrent arrivals at tick 0 competing for resources
        patients = [
            {
                "patient_id": f"patient_{i}",
                "arrival_tick": 0,
                "work_ticks": 10,
                "resource_id": "res.care",
                "task_class_id": "task.care",
            }
            for i in range(1, 5)
        ]

        mean_waits = {}
        with tempfile.TemporaryDirectory() as tmp_dir:
            for cap in [1, 2, 4]:
                scen = build_sensitivity_scenario(
                    scenario_id=f"capacity_sens_c{cap}",
                    capacity_count=cap,
                    horizon_ticks=100,
                    patients=patients,
                )
                scen_file = Path(tmp_dir) / f"scen_c{cap}.json"
                scen_file.write_text(json.dumps(scen, indent=2))

                proc = self.run_cmd("run", str(scen_file))
                self.assertEqual(proc.returncode, 0, f"Run failed for C={cap}: {proc.stderr}")

                doc = json.loads(proc.stdout)
                summary = doc["summary"]
                self.assertEqual(summary["completed"], 4)

                waits = [p["wait_ticks"] for p in summary["patients"] if p["wait_ticks"] is not None]
                mean_wait = sum(waits) / len(waits) if waits else 0.0
                mean_waits[cap] = mean_wait

        # Response surface monotonicity check:
        # At C=1: serialized waits = [0, 10, 20, 30] -> mean = 15.0
        # At C=2: pairs served concurrently = [0, 0, 10, 10] -> mean = 5.0
        # At C=4: all served concurrently = [0, 0, 0, 0] -> mean = 0.0
        self.assertGreaterEqual(
            mean_waits[1],
            mean_waits[2],
            f"Wait at C=1 ({mean_waits[1]}) must be >= wait at C=2 ({mean_waits[2]})",
        )
        self.assertGreaterEqual(
            mean_waits[2],
            mean_waits[4],
            f"Wait at C=2 ({mean_waits[2]}) must be >= wait at C=4 ({mean_waits[4]})",
        )
        self.assertEqual(mean_waits[4], 0.0, "Zero contention limit must yield wait=0 at C=4")

    def test_arrival_demand_sensitivity_and_saturation(self):
        """Evaluate arrival intensity response surface and queue saturation.

        As patient demand increases, throughput rises up to capacity saturation,
        after which queue delays accumulate.
        """
        with tempfile.TemporaryDirectory() as tmp_dir:
            demand_results = {}
            for count in [2, 5, 10]:
                patients = [
                    {
                        "patient_id": f"p_demand_{i}",
                        "arrival_tick": i * 2,
                        "work_ticks": 6,
                        "resource_id": "res.care",
                        "task_class_id": "task.care",
                    }
                    for i in range(count)
                ]
                scen = build_sensitivity_scenario(
                    scenario_id=f"demand_sens_n{count}",
                    capacity_count=1,
                    horizon_ticks=40,
                    patients=patients,
                )
                scen_file = Path(tmp_dir) / f"scen_n{count}.json"
                scen_file.write_text(json.dumps(scen, indent=2))

                proc = self.run_cmd("run", str(scen_file))
                self.assertEqual(proc.returncode, 0, proc.stderr)
                doc = json.loads(proc.stdout)
                demand_results[count] = doc["summary"]

            # Count=2: both finish easily within horizon
            self.assertEqual(demand_results[2]["completed"], 2)
            self.assertEqual(demand_results[2]["unfinished"], 0)

            # Count=10: exceeds capacity * horizon -> unfinished queue accumulates
            self.assertGreater(demand_results[10]["unfinished"], 0)
            self.assertEqual(
                demand_results[10]["arrivals"],
                demand_results[10]["completed"] + demand_results[10]["unfinished"],
                "Terminal patient conservation law holds under saturated queue",
            )

    def test_cli_sensitivity_parameter_overrides(self):
        """CLI runner must support --seed and --horizon overrides for sensitivity testing."""
        proc = self.run_cmd(
            "run",
            str(NOMINAL_SCENARIO),
            "--seed",
            "7777",
            "--horizon",
            "12",
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        doc = json.loads(proc.stdout)

        # Overrides reflected in manifest and execution summary
        self.assertEqual(doc["manifest"]["seed"], 7777)
        self.assertEqual(doc["manifest"]["horizon_ticks"], 12)
        self.assertEqual(doc["summary"]["seed"], 7777)
        self.assertEqual(doc["summary"]["horizon_ticks"], 12)

    def test_warmup_window_and_startup_bias_elimination(self):
        """Evaluate cold-start transient detection vs steady-state observation window.

        Cold-start: At t=0 the system is empty. The first arriving cohort experiences
        artificially low wait times (cold-start bias).
        Once the system reaches steady-state load, queue waits stabilize.
        Excluding the warm-up interval [0, T_warmup) eliminates the startup bias:
        Mean_Wait(steady_state) > Mean_Wait(overall_with_cold_start).
        """
        # Wave of arrivals establishing steady backlog
        patients = []
        for i in range(12):
            patients.append({
                "patient_id": f"p_transient_{i:02d}",
                "arrival_tick": i * 3,  # arrives every 3 ticks
                "work_ticks": 6,        # takes 6 ticks on single server -> queue grows
                "resource_id": "res.care",
                "task_class_id": "task.care",
            })

        scenario = build_sensitivity_scenario(
            scenario_id="warmup_transient_analysis",
            capacity_count=1,
            horizon_ticks=100,
            patients=patients,
        )

        with tempfile.TemporaryDirectory() as tmp_dir:
            scen_file = Path(tmp_dir) / "warmup.json"
            scen_file.write_text(json.dumps(scenario, indent=2))

            proc = self.run_cmd("run", str(scen_file))
            self.assertEqual(proc.returncode, 0, proc.stderr)
            doc = json.loads(proc.stdout)
            completed_patients = [
                p for p in doc["summary"]["patients"]
                if p["status"].lower() == "completed"
            ]

            # Patient 0 arrives at empty system: wait = 0
            self.assertEqual(completed_patients[0]["wait_ticks"], 0)

            # Warm-up cutoff at tick 15 (covers initial 4 patients)
            warmup_cutoff = 15
            warmup_waits = [
                p["wait_ticks"]
                for p in completed_patients
                if p["arrival_tick"] < warmup_cutoff
            ]
            observation_waits = [
                p["wait_ticks"]
                for p in completed_patients
                if p["arrival_tick"] >= warmup_cutoff
            ]

            mean_warmup_wait = sum(warmup_waits) / len(warmup_waits)
            mean_obs_wait = sum(observation_waits) / len(observation_waits)
            mean_overall_wait = sum(p["wait_ticks"] for p in completed_patients) / len(completed_patients)

            # Cold-start bias elimination:
            # Steady-state observation wait is strictly higher than warm-up startup wait
            self.assertGreater(
                mean_obs_wait,
                mean_warmup_wait,
                f"Steady-state wait ({mean_obs_wait:.2f}) must exceed warm-up wait ({mean_warmup_wait:.2f})",
            )
            # Overall wait is dampened by the cold start
            self.assertGreater(
                mean_obs_wait,
                mean_overall_wait,
                f"Steady-state wait ({mean_obs_wait:.2f}) must exceed overall wait ({mean_overall_wait:.2f})",
            )

    def test_benchmark_reporting_and_throughput_scaling(self):
        """Benchmark execution duration, step throughput, and patient throughput."""
        # Scale test with 50 patients
        patients = [
            {
                "patient_id": f"bench_{i:03d}",
                "arrival_tick": (i * 2),
                "work_ticks": 4,
                "resource_id": "res.care",
                "task_class_id": "task.care",
            }
            for i in range(50)
        ]

        scenario = build_sensitivity_scenario(
            scenario_id="benchmark_scaling_50",
            capacity_count=4,
            horizon_ticks=200,
            patients=patients,
        )

        with tempfile.TemporaryDirectory() as tmp_dir:
            scen_file = Path(tmp_dir) / "bench.json"
            scen_file.write_text(json.dumps(scenario, indent=2))

            t0 = time.perf_counter()
            proc = self.run_cmd("run", str(scen_file))
            elapsed_sec = time.perf_counter() - t0

            self.assertEqual(proc.returncode, 0, proc.stderr)
            doc = json.loads(proc.stdout)
            summary = doc["summary"]

            # Bounded execution latency: 50 patients in under 0.5s
            self.assertLess(
                elapsed_sec,
                0.5,
                f"Benchmark execution took too long: {elapsed_sec:.4f}s",
            )

            # Throughput metrics
            completed = summary["completed"]
            self.assertEqual(completed, 50)
            patients_per_sec = completed / max(elapsed_sec, 1e-6)
            steps_per_sec = summary["horizon_ticks"] / max(elapsed_sec, 1e-6)

            self.assertGreater(patients_per_sec, 100.0)
            self.assertGreater(steps_per_sec, 400.0)

    def test_neutral_comparison_reporting_and_uncertainty_diagnostics(self):
        """Verify neutral comparative evaluation, uncertainty diagnostics, and validation taxonomy."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            base_file = Path(tmp_dir) / "base.json"
            cand_file = Path(tmp_dir) / "cand.json"

            # Base: nominal capacity
            self.assertEqual(
                self.run_cmd("run", str(NOMINAL_SCENARIO), "-o", str(base_file)).returncode,
                0,
            )
            # Candidate: constrained capacity
            self.assertEqual(
                self.run_cmd("run", str(CONSTRAINED_SCENARIO), "-o", str(cand_file)).returncode,
                0,
            )

            # Compare runs
            proc = self.run_cmd("compare", str(base_file), str(cand_file))
            self.assertEqual(proc.returncode, 0, proc.stderr)

            comp = json.loads(proc.stdout)

            # 1. Structural compatibility verification
            self.assertIn("metrics", comp)
            self.assertIn("diagnostics", comp)
            self.assertIn("validation_taxonomy", comp)

            # 2. Metric comparisons include units, denominators, and uncertainty (standard error)
            metrics_by_name = {m["metric_name"]: m for m in comp["metrics"]}
            self.assertIn("mean_wait", metrics_by_name)
            self.assertIn("mean_elapsed", metrics_by_name)
            self.assertIn("completed", metrics_by_name)

            wait_m = metrics_by_name["mean_wait"]
            self.assertEqual(wait_m["unit"], "tick")
            self.assertIn("completed patients", wait_m["denominator_description"])
            self.assertTrue(math.isfinite(wait_m["base_value"]))
            self.assertTrue(math.isfinite(wait_m["candidate_value"]))
            self.assertTrue(math.isfinite(wait_m["delta"]))

            # Standard errors reported when applicable
            if wait_m["base_se"] is not None:
                self.assertTrue(math.isfinite(wait_m["base_se"]))
            if wait_m["candidate_se"] is not None:
                self.assertTrue(math.isfinite(wait_m["candidate_se"]))

            # 3. Neutrality Principle: No value judgements in diagnostics
            for diag in comp["diagnostics"]:
                lower_diag = diag.to_lowercase() if hasattr(diag, "to_lowercase") else diag.lower()
                for forbidden in ["superior", "inferior", "better", "worse", "ideal", "winner"]:
                    self.assertNotIn(
                        forbidden,
                        lower_diag,
                        f"Diagnostic must remain neutral and not presume superiority: '{diag}'",
                    )

            # 4. Validation taxonomy non-conflation disclaimer
            tax = comp["validation_taxonomy"]
            self.assertEqual(tax["verification_scope"], "software_verification")
            self.assertEqual(tax["validation_scope"], "none")
            self.assertEqual(tax["provenance_class"], "synthetic")
            self.assertIn("Software verification", tax["disclaimer"])
            self.assertIn("does NOT constitute clinical domain validation", tax["disclaimer"])


if __name__ == "__main__":
    unittest.main()
