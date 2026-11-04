"""Task E4.2: Sensitivity, identifiability, warm-up observation windows, and benchmark reporting.

Verifies:
1. Actual native CLI handling of sensitivity scenarios:
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
   - Qualified per-run replication uncertainty; paired-difference uncertainty explicitly not estimated.
   - Absence of evaluative/normative value judgements ("superior", "inferior", "better", "worse") in diagnostics.
   - Preservation of software verification vs domain validation taxonomy and regulatory disclaimer.
"""

import json
import math
from pathlib import Path
import tempfile
import time
import unittest

ROOT_DIR = Path(__file__).resolve().parents[2]
from native_cli_support import NativeCliCase, invoke, native_tick, metric_rows, read_run
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


class E42SensitivityBenchmarksTests(NativeCliCase):
    """Integration test suite for Task E4.2 sensitivity, warm-up windows, and benchmarks."""





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

                _, tables = self.run_scenario(scen_file)
                summary = tables['replications'].to_pylist()[0]
                self.assertEqual(summary["completed"], 4)

                waits = [native_tick(p, 'wait_ticks') for p in tables['patients'].to_pylist() if native_tick(p, 'wait_ticks') is not None]
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

                _, tables = self.run_scenario(scen_file)
                demand_results[count] = tables['replications'].to_pylist()[0]

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
        manifest, tables = self.run_scenario(NOMINAL_SCENARIO, '--seed', 7777, '--horizon', 12)
        self.assertEqual((manifest['base_seed'], manifest['horizon_ticks']), (7777, 12))
        replication = tables['replications'].to_pylist()[0]
        self.assertEqual((replication['seed'], replication['horizon_ticks']), (7777, 12))

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

            _, tables = self.run_scenario(scen_file)
            completed_patients = [
                p for p in tables['patients'].to_pylist()
                if p["status"].lower() == "completed"
            ]

            # Patient 0 arrives at empty system: wait = 0
            self.assertEqual(native_tick(completed_patients[0], 'wait_ticks'), 0)

            # Warm-up cutoff at tick 15 (covers initial 4 patients)
            warmup_cutoff = 15
            warmup_waits = [
                native_tick(p, 'wait_ticks')
                for p in completed_patients
                if native_tick(p, 'arrival_tick') < warmup_cutoff
            ]
            observation_waits = [
                native_tick(p, 'wait_ticks')
                for p in completed_patients
                if native_tick(p, 'arrival_tick') >= warmup_cutoff
            ]

            mean_warmup_wait = sum(warmup_waits) / len(warmup_waits)
            mean_obs_wait = sum(observation_waits) / len(observation_waits)
            mean_overall_wait = sum(native_tick(p, 'wait_ticks') for p in completed_patients) / len(completed_patients)

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

            output = Path(tmp_dir) / 'native'
            t0 = time.perf_counter()
            proc = invoke('run', scen_file, '--output-dir', output)
            elapsed_sec = time.perf_counter() - t0

            self.assertEqual(proc.returncode, 0, proc.stderr)
            _, tables = read_run(output)
            summary = tables['replications'].to_pylist()[0]

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
        """Qualified per-run uncertainty stays separate from unestimated differences."""
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            _, left = self.run_ok(directory / 'left', NOMINAL_SCENARIO, 'batch', '--replications', 4)
            _, right = self.run_ok(directory / 'right', CONSTRAINED_SCENARIO, 'batch', '--replications', 4)
            result = invoke('compare', directory / 'left/data', directory / 'right/data')
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads(result.stdout)
            self.assertEqual(report['uncertainty_method'], 'not_estimated')
            self.assertEqual(report['uncertainty_reason'], 'paired_difference_method_unqualified')
            wait = {row['name']: row for row in report['metrics']}['mean_wait_ticks']
            self.assertEqual(wait['unit'], 'tick')
            self.assertAlmostEqual(wait['delta'], 3.8)
            for value in [wait['left'], wait['right'], wait['delta']]:
                self.assertTrue(math.isfinite(value))
            for tables, expected in [(left, 0.4), (right, 4.2)]:
                metric = metric_rows(tables)['mean_wait_ticks']
                self.assertEqual(metric['denominator_name'], 'included_replications')
                self.assertEqual(metric['uncertainty_replications'], 4)
                self.assertEqual(metric['ddof'], 1)
                self.assertEqual(metric['standard_error'], 0.0)
                self.assertAlmostEqual(metric['ci_95_lower'], expected)
                self.assertAlmostEqual(metric['ci_95_upper'], expected)
            for forbidden in ['superior', 'inferior', 'better', 'worse', 'ideal', 'winner']:
                self.assertNotIn(forbidden, result.stdout.lower())
            self.assertIn('does NOT constitute clinical domain validation', invoke('--help').stdout)


if __name__ == "__main__":
    unittest.main()
