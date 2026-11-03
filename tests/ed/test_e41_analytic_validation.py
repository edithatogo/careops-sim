"""E41 software-verification vectors through actual native ED tables.

Retains zero-contention, exact D/D/1 schedules, synthetic qualitative
shape/triage-ordering vectors and conservation. Historical generic reference
labels are not renewed empirical/site validation. Native help states that
software verification does not constitute clinical domain validation.

The former forged Macro/Micro JSON comparison is replaced with an actual native
boundary rejection/neutral-comparison test. The real domain fidelity/transit
oracle remains the C62 Rust holdout comparison, outside this CLI producer.
"""

import json
import math
from pathlib import Path
import tempfile
import unittest

ROOT_DIR = Path(__file__).resolve().parents[2]
from native_cli_support import NativeCliCase, invoke, native_tick, metric_rows
NOMINAL_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/nominal_mvp.json"
CONSTRAINED_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/constrained_mvp.json"


def compute_median(values):
    """Compute exact median of numerical series."""
    sorted_v = sorted(values)
    n = len(sorted_v)
    if n == 0:
        return 0.0
    mid = n // 2
    if n % 2 == 1:
        return float(sorted_v[mid])
    return (sorted_v[mid - 1] + sorted_v[mid]) / 2.0


def compute_skewness(values):
    """Compute sample skewness of numerical series (Fisher-Pearson coefficient)."""
    n = len(values)
    if n < 3:
        return 0.0
    mean = sum(values) / n
    variance = sum((x - mean) ** 2 for x in values) / (n - 1)
    std_dev = math.sqrt(variance)
    if std_dev == 0.0:
        return 0.0
    m3 = sum((x - mean) ** 3 for x in values) / n
    return m3 / (std_dev ** 3)


def build_scenario_dict(
    scenario_id: str,
    horizon_ticks: int = 50,
    resource_count: int = 1,
    patients: list = None,
    notes: str = "Synthetic verification scenario",
):
    """Build a schema-conformant scenario dictionary."""
    if patients is None:
        patients = []

    resource_buckets = [
        {
            "resource_id": "res.care",
            "resource_class": "care_slot",
            "location_id": "loc.care",
            "zone_id": "zone.care",
            "physical_status": "known",
            "physical_count": resource_count,
            "availability_status": "known",
            "open_count": resource_count,
            "staffed_count": resource_count,
            "capabilities": ["care_treatment"],
        }
    ]

    return {
        "schema_version": 1,
        "scenario_id": scenario_id,
        "seed": 42,
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
                "physical_count": resource_count,
                "availability_status": "known",
                "open_count": resource_count,
                "staffed_count": resource_count,
                "equipment": [],
            },
            "zones": [{"zone_id": "zone.care", "name": "Care Zone"}],
            "locations": [
                {
                    "location_id": "loc.care",
                    "name": "Care Bay",
                    "zone_id": "zone.care",
                }
            ],
            "resource_buckets": resource_buckets,
            "task_classes": [
                {
                    "task_class_id": "task.care",
                    "name": "care work",
                    "required_resource_classes": ["care_slot"],
                }
            ],
            "staff_roles": [
                {
                    "staff_role_id": "staff.care",
                    "name": "Care Staff",
                    "counts": [
                        {
                            "basis": "present",
                            "status": "known",
                            "count": resource_count,
                            "reference": "synthetic shift",
                        }
                    ],
                }
            ],
            "eligibility": [
                {
                    "staff_role_id": "staff.care",
                    "task_class_id": "task.care",
                    "zone_id": "zone.care",
                    "location_id": "loc.care",
                }
            ],
            "routes": [],
        },
        "patients": patients,
    }


class E41AnalyticValidationTests(NativeCliCase):
    """E4.1 analytic limiting cases, invariant conservation, and validation boundary tests."""





    def test_software_verification_vs_domain_validation_boundary(self):
        """Native help retains the verification boundary; no clinical acceptance is inferred."""
        proc = invoke('--help')
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn('Software verification', proc.stdout)
        self.assertIn('does NOT constitute clinical domain validation', proc.stdout)
        self.assertIn('healthcare site', proc.stdout)
        manifest, tables = self.run_scenario(NOMINAL_SCENARIO)
        self.assertEqual(manifest['scenario_id'], 'nominal-ed-mvp')
        self.assertEqual(tables['patients'].num_rows, 5)
        self.assertEqual(json.loads(NOMINAL_SCENARIO.read_text())['provenance']['class'], 'synthetic')

    def test_public_reference_distribution_comparisons(self):
        """Compare simulated ED distributions against generic public reference baselines.

        Baselines (AIHW Australian Emergency Department Care 2022-23 and US NHAMCS):
        - Median length of stay (LOS): ~2.5 - 4.0 hours (150 - 240 minutes).
        - Skewness: Right-skewed distribution (skewness > 0, mean > median, tail of complex cases).
        - Triage Wait Ordering (ATS 1-5):
          ATS 1 (Resuscitation): immediate access, median wait == 0 ticks.
          ATS 2-5: progressively longer median waits (wait ATS 1 <= ATS 2 <= ATS 3 <= ATS 4 <= ATS 5).

        Verifies that synthetic outputs qualitatively align with generic public reference shapes
        without claiming real-hospital calibration.
        """
        patients = [
            # ATS 1: Resuscitation (immediate start at tick 0, service 180 ticks -> wait 0, LOS 180)
            {
                "patient_id": "ats1_patient",
                "arrival_tick": 0,
                "work_ticks": 180,
                "resource_id": "res.bay1",
                "task_class_id": "task.care",
            },
            # ATS 2: Emergency (arrives tick 10, bay busy until tick 20 -> wait 10, work 170 -> LOS 180)
            {
                "patient_id": "ats2_blocker",
                "arrival_tick": 0,
                "work_ticks": 20,
                "resource_id": "res.bay2",
                "task_class_id": "task.care",
            },
            {
                "patient_id": "ats2_patient",
                "arrival_tick": 10,
                "work_ticks": 170,
                "resource_id": "res.bay2",
                "task_class_id": "task.care",
            },
            # ATS 3: Urgent (arrives tick 20, bay busy until tick 50 -> wait 30, work 170 -> LOS 200)
            {
                "patient_id": "ats3_blocker",
                "arrival_tick": 0,
                "work_ticks": 50,
                "resource_id": "res.bay3",
                "task_class_id": "task.care",
            },
            {
                "patient_id": "ats3_patient",
                "arrival_tick": 20,
                "work_ticks": 170,
                "resource_id": "res.bay3",
                "task_class_id": "task.care",
            },
            # ATS 4: Semi-urgent (arrives tick 30, bay busy until tick 90 -> wait 60, work 150 -> LOS 210)
            {
                "patient_id": "ats4_blocker",
                "arrival_tick": 0,
                "work_ticks": 90,
                "resource_id": "res.bay4",
                "task_class_id": "task.care",
            },
            {
                "patient_id": "ats4_patient",
                "arrival_tick": 30,
                "work_ticks": 150,
                "resource_id": "res.bay4",
                "task_class_id": "task.care",
            },
            # ATS 5: Non-urgent (arrives tick 40, bay busy until tick 130 -> wait 90, work 140 -> LOS 230)
            {
                "patient_id": "ats5_blocker",
                "arrival_tick": 0,
                "work_ticks": 130,
                "resource_id": "res.bay5",
                "task_class_id": "task.care",
            },
            {
                "patient_id": "ats5_patient",
                "arrival_tick": 40,
                "work_ticks": 140,
                "resource_id": "res.bay5",
                "task_class_id": "task.care",
            },
            # Complex/admitted patient cases generating characteristic prolonged ED stays
            {
                "patient_id": "complex_patient_1",
                "arrival_tick": 50,
                "work_ticks": 360,
                "resource_id": "res.bay6",
                "task_class_id": "task.care",
            },
            {
                "patient_id": "complex_patient_2",
                "arrival_tick": 60,
                "work_ticks": 450,
                "resource_id": "res.bay7",
                "task_class_id": "task.care",
            },
        ]

        resource_buckets = [
            {
                "resource_id": f"res.bay{i}",
                "resource_class": "bay_slot",
                "location_id": "loc.acute",
                "zone_id": "zone.acute",
                "physical_status": "known",
                "physical_count": 1,
                "availability_status": "known",
                "open_count": 1,
                "staffed_count": 1,
                "capabilities": ["acute_care"],
            }
            for i in range(1, 8)
        ]

        scenario = {
            "schema_version": 1,
            "scenario_id": "public-ref-baseline-comparison",
            "seed": 42,
            "time_unit": "tick",
            "horizon_ticks": 1440,
            "provenance": {
                "class": "synthetic",
                "notes": "Qualitative comparison against AIHW and NHAMCS public reference baselines.",
            },
            "capacity_location": {
                "schema_version": 3,
                "status": "proposed",
                "capacity": {
                    "unit": "treatment_space",
                    "bucket_completeness": "complete",
                    "physical_status": "known",
                    "physical_count": 7,
                    "availability_status": "known",
                    "open_count": 7,
                    "staffed_count": 7,
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
                "staff_roles": [
                    {
                        "staff_role_id": "staff.nurse",
                        "name": "Nurse",
                        "counts": [
                            {
                                "basis": "present",
                                "status": "known",
                                "count": 7,
                                "reference": "synthetic schedule",
                            }
                        ],
                    }
                ],
                "eligibility": [
                    {
                        "staff_role_id": "staff.nurse",
                        "task_class_id": "task.care",
                        "zone_id": "zone.acute",
                        "location_id": "loc.acute",
                    }
                ],
                "routes": [],
            },
            "patients": patients,
        }

        with tempfile.TemporaryDirectory() as tmp_dir:
            scen_file = Path(tmp_dir) / "public_ref.json"
            scen_file.write_text(json.dumps(scenario, indent=2))

            _, tables = self.run_scenario(scen_file)
            native_patients = tables['patients'].to_pylist()
            patient_results = {p['patient_id']: p for p in native_patients}

            # 1. Triage Wait Ordering (ATS 1-5):
            # ATS 1 has 0 wait ticks (immediate resuscitation target)
            ats1_wait = native_tick(patient_results["ats1_patient"], "wait_ticks")
            ats2_wait = native_tick(patient_results["ats2_patient"], "wait_ticks")
            ats3_wait = native_tick(patient_results["ats3_patient"], "wait_ticks")
            ats4_wait = native_tick(patient_results["ats4_patient"], "wait_ticks")
            ats5_wait = native_tick(patient_results["ats5_patient"], "wait_ticks")

            self.assertEqual(ats1_wait, 0, "ATS 1 must receive immediate access (0 wait)")
            self.assertLessEqual(ats1_wait, ats2_wait, "ATS 1 wait must be <= ATS 2 wait")
            self.assertLessEqual(ats2_wait, ats3_wait, "ATS 2 wait must be <= ATS 3 wait")
            self.assertLessEqual(ats3_wait, ats4_wait, "ATS 3 wait must be <= ATS 4 wait")
            self.assertLessEqual(ats4_wait, ats5_wait, "ATS 4 wait must be <= ATS 5 wait")

            # 2. Length of Stay (LOS) Median Baseline:
            # For 1 tick/minute representation: 150 to 240 ticks maps to 2.5h to 4.0h
            cohort_los = [
                native_tick(p, 'elapsed_ticks')
                for p in native_patients
                if "blocker" not in p["patient_id"]
            ]
            median_los = compute_median(cohort_los)
            self.assertGreaterEqual(
                median_los,
                150.0,
                f"Median LOS ({median_los} ticks) below 2.5h reference threshold (150 ticks)",
            )
            self.assertLessEqual(
                median_los,
                240.0,
                f"Median LOS ({median_los} ticks) above 4.0h reference threshold (240 ticks)",
            )

            # 3. Distribution Right-Skewness:
            # Real ED length of stay exhibits significant right-skewness (lognormal-like tail)
            skewness = compute_skewness(cohort_los)
            mean_los = sum(cohort_los) / len(cohort_los)
            self.assertGreater(
                skewness,
                0.0,
                f"LOS distribution must be right-skewed (skewness={skewness})",
            )
            self.assertGreater(
                mean_los,
                median_los,
                f"Mean LOS ({mean_los}) must exceed median LOS ({median_los}) for right-skewed distribution",
            )

            # Synthetic qualitative checks do not become hospital validation.
            self.assertEqual(scenario['provenance']['class'], 'synthetic')
            self.assertIn('does NOT constitute clinical domain validation', invoke('--help').stdout)

    def test_analytic_limiting_case_runner_scenarios(self):
        """Execute runner on zero-contention and Kendall D/D/1 single-server analytic scenarios.

        Verifies:
        1. Zero Contention:
           When capacity >= demand, all wait times are identically zero (wait_ticks == 0).
           start_tick == arrival_tick for every presentation.
        2. Kendall D/D/1 Deterministic Single-Server Queue (Report 12 hand-computable oracle):
           Deterministic arrivals [1, 2, 3, 8] and durations [2, 2, 3, 1] on 1 server yield
           exact bit-for-bit schedule:
           - P0: arrival=1, work=2 -> start=1, wait=0, completion=3
           - P1: arrival=2, work=2 -> start=3, wait=1, completion=5
           - P2: arrival=3, work=3 -> start=5, wait=2, completion=8
           - P3: arrival=8, work=1 -> start=8, wait=0, completion=9
        """
        # Case 1: Zero Contention Scenario
        zero_contention_patients = [
            {
                "patient_id": f"zc_patient_{i}",
                "arrival_tick": arr,
                "work_ticks": 4,
                "resource_id": "res.care",
                "task_class_id": "task.care",
            }
            for i, arr in enumerate([1, 1, 3, 5, 5])
        ]
        # Capacity is 10 (strictly exceeds 5 concurrent arrivals)
        zc_scenario = build_scenario_dict(
            scenario_id="zero-contention-limiting-case",
            horizon_ticks=30,
            resource_count=10,
            patients=zero_contention_patients,
            notes="Analytic zero-contention limiting case where C >= A(t)",
        )

        with tempfile.TemporaryDirectory() as tmp_dir:
            zc_file = Path(tmp_dir) / "zero_contention.json"
            zc_file.write_text(json.dumps(zc_scenario, indent=2))

            _, tables_zc = self.run_scenario(zc_file)
            summary_zc = tables_zc['replications'].to_pylist()[0]

            self.assertEqual(summary_zc["arrivals"], 5)
            self.assertEqual(summary_zc["completed"], 5)
            self.assertEqual(summary_zc["unfinished"], 0)

            for p in tables_zc['patients'].to_pylist():
                self.assertEqual(
                    native_tick(p, 'wait_ticks'),
                    0,
                    f"Patient {p['patient_id']} experienced non-zero wait in zero-contention case",
                )
                self.assertEqual(
                    native_tick(p, 'start_tick'),
                    native_tick(p, 'arrival_tick'),
                    f"Patient {p['patient_id']} start_tick != arrival_tick in zero contention",
                )
                self.assertEqual(
                    native_tick(p, 'elapsed_ticks'),
                    native_tick(p, 'work_ticks'),
                    f"Patient {p['patient_id']} elapsed_ticks != work_ticks in zero contention",
                )

        # Case 2: Kendall D/D/1 Single-Server Deterministic Queue
        dd1_patients = [
            {
                "patient_id": "dd1_p0",
                "arrival_tick": 1,
                "work_ticks": 2,
                "resource_id": "res.care",
                "task_class_id": "task.care",
            },
            {
                "patient_id": "dd1_p1",
                "arrival_tick": 2,
                "work_ticks": 2,
                "resource_id": "res.care",
                "task_class_id": "task.care",
            },
            {
                "patient_id": "dd1_p2",
                "arrival_tick": 3,
                "work_ticks": 3,
                "resource_id": "res.care",
                "task_class_id": "task.care",
            },
            {
                "patient_id": "dd1_p3",
                "arrival_tick": 8,
                "work_ticks": 1,
                "resource_id": "res.care",
                "task_class_id": "task.care",
            },
        ]
        # Capacity is exactly 1 server
        dd1_scenario = build_scenario_dict(
            scenario_id="kendall-dd1-single-server",
            horizon_ticks=20,
            resource_count=1,
            patients=dd1_patients,
            notes="Kendall D/D/1 deterministic single-server queue oracle",
        )

        with tempfile.TemporaryDirectory() as tmp_dir:
            dd1_file = Path(tmp_dir) / "dd1_scenario.json"
            dd1_file.write_text(json.dumps(dd1_scenario, indent=2))

            _, tables_dd1 = self.run_scenario(dd1_file)
            summary_dd1 = tables_dd1['replications'].to_pylist()[0]

            self.assertEqual(summary_dd1["completed"], 4)
            self.assertEqual(summary_dd1["unfinished"], 0)

            # Verify Report 12 hand-computable oracle schedule
            expected_schedule = [
                {"id": "dd1_p0", "arr": 1, "start": 1, "wait": 0, "comp": 3},
                {"id": "dd1_p1", "arr": 2, "start": 3, "wait": 1, "comp": 5},
                {"id": "dd1_p2", "arr": 3, "start": 5, "wait": 2, "comp": 8},
                {"id": "dd1_p3", "arr": 8, "start": 8, "wait": 0, "comp": 9},
            ]

            actual_pts = {p["patient_id"]: p for p in tables_dd1['patients'].to_pylist()}
            for exp in expected_schedule:
                act = actual_pts[exp["id"]]
                self.assertEqual(
                    native_tick(act, 'start_tick'),
                    exp["start"],
                    f"Patient {exp['id']} start_tick mismatch: expected {exp['start']}, got {native_tick(act, 'start_tick')}",
                )
                self.assertEqual(
                    native_tick(act, 'wait_ticks'),
                    exp["wait"],
                    f"Patient {exp['id']} wait_ticks mismatch: expected {exp['wait']}, got {native_tick(act, 'wait_ticks')}",
                )
                self.assertEqual(
                    native_tick(act, 'completion_tick'),
                    exp["comp"],
                    f"Patient {exp['id']} completion_tick mismatch: expected {exp['comp']}, got {native_tick(act, 'completion_tick')}",
                )

    def test_native_comparison_rejects_forged_fidelity_documents(self):
        """The CLI cannot confer fidelity provenance on invented alpha JSON.

        Real Macro/Micro transit, preemption and work conservation are owned by
        c62_macro_micro_holdout_comparison::test_macro_micro_holdout_comparison_through_api_runner.
        This boundary test makes no claim to execute that domain producer.
        """
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            forged = directory / 'forged.json'
            forged.write_text(json.dumps({'manifest': {'scenario_id': 'macro'},
                                          'summary': {'patients': [], 'mean_elapsed': 5}}))
            rejected = invoke('compare', forged, forged)
            self.assertEqual(rejected.returncode, 4, rejected.stderr)
            _, left = self.run_ok(directory / 'nominal')
            _, right = self.run_ok(directory / 'constrained', CONSTRAINED_SCENARIO)
            result = invoke('compare', directory / 'nominal/data', directory / 'constrained/data')
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads(result.stdout)
            wait = {row['name']: row for row in report['metrics']}['mean_wait_ticks']
            self.assertAlmostEqual(wait['delta'], 3.8)
            self.assertEqual(metric_rows(left)['mean_wait_ticks']['unit'], 'tick')
            self.assertEqual(metric_rows(right)['mean_wait_ticks']['unit'], 'tick')
            self.assertEqual(report['uncertainty_method'], 'not_estimated')
            for term in ['superior', 'inferior', 'better model', 'more accurate', 'clinically validated', 'ground truth']:
                self.assertNotIn(term, result.stdout.lower())

    def test_invariant_state_conservation_audit(self):
        """Actual native patient rows and per-replication counts conserve state."""
        for scenario in [NOMINAL_SCENARIO, CONSTRAINED_SCENARIO]:
            _, tables = self.run_scenario(scenario)
            summary = tables['replications'].to_pylist()[0]
            self.assertEqual(summary['arrivals'], summary['completed'] + summary['unfinished'])
            self.assertLessEqual(summary['started'], summary['arrivals'])
            self.assertLessEqual(summary['completed'], summary['started'])
            self.assertEqual(tables['patients'].num_rows, summary['arrivals'])
        patients = [dict(patient_id=f'trunc_p{i}', arrival_tick=arrival, work_ticks=10,
                         resource_id='res.care', task_class_id='task.care')
                    for i, arrival in enumerate([1, 3, 5])]
        scenario = build_scenario_dict('horizon-truncation-conservation', 8, 1, patients,
                                       'Finite horizon truncation testing unfinished conservation')
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'trunc.json'; source.write_text(json.dumps(scenario))
            _, tables = self.run_scenario(source)
            summary = tables['replications'].to_pylist()[0]
            self.assertEqual((summary['arrivals'], summary['completed'], summary['unfinished']), (3, 0, 3))
            self.assertEqual(summary['arrivals'], summary['completed'] + summary['unfinished'])
            self.assertLessEqual(summary['started'], summary['arrivals'])
            rows = tables['patients'].to_pylist()
            self.assertEqual(sum(p['status'] == 'completed' and native_tick(p, 'completion_tick') is not None for p in rows), summary['completed'])
            self.assertEqual(sum(p['status'] == 'unfinished' and native_tick(p, 'completion_tick') is None for p in rows), summary['unfinished'])


if __name__ == "__main__":
    unittest.main()
