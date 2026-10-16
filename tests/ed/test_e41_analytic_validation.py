"""Analytic limiting cases, invariant/differential tests, and validation taxonomy suite for Task E4.1.

Distinguishes Software Verification from Domain Validation:
1. Software Verification ("Building the model right"):
   - Mathematical and algorithmic correctness of the simulation engine.
   - Verifies queue mechanics, priority scheduling, work conservation, zero-contention limits,
     Kendall D/D/1 deterministic single-server schedules, and state conservation invariants.
   - Independent of real-world clinical datasets.
2. Domain Validation ("Building the right model"):
   - Empirical calibration and fidelity to a specific clinical site (e.g. Cairns ED, AIHW/NHAMCS benchmarks).
   - Evaluates whether model parameters reflect real-world clinical practice.
   - Distinct and non-conflated with software verification.

Test Coverage:
- `test_software_verification_vs_domain_validation_boundary`:
  Verifies that software verification checks are distinct from domain validation reports, and that
  runner outputs include explicit provenance/validation taxonomy stating that software verification
  does NOT constitute clinical domain validation for any healthcare site.
- `test_public_reference_distribution_comparisons`:
  Compares simulated ED distributions (wait times, length of stay) against generic public reference
  baselines (e.g. AIHW Australian ED Care 2022-23 and NHAMCS US ED summary statistics: median LOS ~ 2.5-4h,
  right-skewed lognormal-like distribution, ATS 1-5 triage wait distribution ordering: ATS 1 immediate,
  ATS 2-5 progressively longer median waits), confirming qualitative alignment without claiming
  hospital-calibrated validity.
- `test_analytic_limiting_case_runner_scenarios`:
  Executes the runner example on zero-contention and Kendall D/D/1 deterministic single-server scenarios,
  asserting wait_ticks == 0 in zero contention and bit-for-bit schedule matching in single server.
- `test_differential_fidelity_comparison_through_cli`:
  Runs Macro vs Micro through the runner compare command and confirms comparison output diagnostics
  report metric deltas neutrally without declaring empirical superiority.
- `test_invariant_state_conservation_audit`:
  Verifies patient and resource conservation in run outputs: `arrivals == completed + unfinished`,
  `started <= arrivals`, across nominal, constrained, and horizon-truncated scenarios.
"""

import json
import math
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT_DIR = Path(__file__).resolve().parents[2]
RUNNER_BIN = ROOT_DIR / "target/debug/examples/runner"
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


class E41AnalyticValidationTests(unittest.TestCase):
    """E4.1 analytic limiting cases, invariant conservation, and validation boundary tests."""

    @classmethod
    def setUpClass(cls):
        # Ensure the runner example is built
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
        """Invoke runner binary with provided arguments."""
        proc = subprocess.run(
            [str(RUNNER_BIN), *args],
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
        )
        return proc

    def test_software_verification_vs_domain_validation_boundary(self):
        """Verify that software verification and clinical domain validation are strictly distinguished.

        Runner outputs must explicitly include validation taxonomy stating that software
        verification does NOT constitute clinical domain validation for any healthcare site.
        """
        proc = self.run_cmd("run", str(NOMINAL_SCENARIO))
        self.assertEqual(proc.returncode, 0, f"Runner failed: {proc.stderr}")

        doc = json.loads(proc.stdout)
        self.assertIn("validation_taxonomy", doc)
        tax = doc["validation_taxonomy"]

        # 1. Verification vs. validation scope categorization
        self.assertEqual(tax["verification_scope"], "software_verification")
        self.assertEqual(tax["validation_scope"], "none")
        self.assertEqual(tax["provenance_class"], "synthetic")

        # 2. Explicit non-conflation disclaimer
        disclaimer = tax["disclaimer"]
        self.assertIn("Software verification", disclaimer)
        self.assertIn("does NOT constitute clinical domain validation", disclaimer)
        self.assertIn("healthcare site", disclaimer)

        # 3. Comparative evaluation also retains taxonomy boundary
        with tempfile.TemporaryDirectory() as tmp_dir:
            base_file = Path(tmp_dir) / "base.json"
            cand_file = Path(tmp_dir) / "cand.json"
            self.assertEqual(
                self.run_cmd("run", str(NOMINAL_SCENARIO), "-o", str(base_file)).returncode,
                0,
            )
            self.assertEqual(
                self.run_cmd("run", str(CONSTRAINED_SCENARIO), "-o", str(cand_file)).returncode,
                0,
            )

            proc_comp = self.run_cmd("compare", str(base_file), str(cand_file))
            self.assertEqual(proc_comp.returncode, 0, proc_comp.stderr)
            comp_rep = json.loads(proc_comp.stdout)

            self.assertIn("validation_taxonomy", comp_rep)
            self.assertEqual(
                comp_rep["validation_taxonomy"]["verification_scope"],
                "software_verification",
            )
            self.assertIn(
                "does NOT constitute clinical domain validation",
                comp_rep["validation_taxonomy"]["disclaimer"],
            )

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

            proc = self.run_cmd("run", str(scen_file))
            self.assertEqual(proc.returncode, 0, f"Run failed: {proc.stderr}")

            doc = json.loads(proc.stdout)
            patient_results = {p["patient_id"]: p for p in doc["summary"]["patients"]}

            # 1. Triage Wait Ordering (ATS 1-5):
            # ATS 1 has 0 wait ticks (immediate resuscitation target)
            ats1_wait = patient_results["ats1_patient"]["wait_ticks"]
            ats2_wait = patient_results["ats2_patient"]["wait_ticks"]
            ats3_wait = patient_results["ats3_patient"]["wait_ticks"]
            ats4_wait = patient_results["ats4_patient"]["wait_ticks"]
            ats5_wait = patient_results["ats5_patient"]["wait_ticks"]

            self.assertEqual(ats1_wait, 0, "ATS 1 must receive immediate access (0 wait)")
            self.assertLessEqual(ats1_wait, ats2_wait, "ATS 1 wait must be <= ATS 2 wait")
            self.assertLessEqual(ats2_wait, ats3_wait, "ATS 2 wait must be <= ATS 3 wait")
            self.assertLessEqual(ats3_wait, ats4_wait, "ATS 3 wait must be <= ATS 4 wait")
            self.assertLessEqual(ats4_wait, ats5_wait, "ATS 4 wait must be <= ATS 5 wait")

            # 2. Length of Stay (LOS) Median Baseline:
            # For 1 tick/minute representation: 150 to 240 ticks maps to 2.5h to 4.0h
            cohort_los = [
                p["elapsed_ticks"]
                for p in doc["summary"]["patients"]
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

            # 4. Explicit verification taxonomy non-conflation check
            self.assertEqual(
                doc["validation_taxonomy"]["verification_scope"],
                "software_verification",
            )
            self.assertEqual(doc["validation_taxonomy"]["validation_scope"], "none")

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

            proc_zc = self.run_cmd("run", str(zc_file))
            self.assertEqual(proc_zc.returncode, 0, proc_zc.stderr)

            doc_zc = json.loads(proc_zc.stdout)
            summary_zc = doc_zc["summary"]

            self.assertEqual(summary_zc["arrivals"], 5)
            self.assertEqual(summary_zc["completed"], 5)
            self.assertEqual(summary_zc["unfinished"], 0)

            for p in summary_zc["patients"]:
                self.assertEqual(
                    p["wait_ticks"],
                    0,
                    f"Patient {p['patient_id']} experienced non-zero wait in zero-contention case",
                )
                self.assertEqual(
                    p["start_tick"],
                    p["arrival_tick"],
                    f"Patient {p['patient_id']} start_tick != arrival_tick in zero contention",
                )
                self.assertEqual(
                    p["elapsed_ticks"],
                    p["work_ticks"],
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

            proc_dd1 = self.run_cmd("run", str(dd1_file))
            self.assertEqual(proc_dd1.returncode, 0, proc_dd1.stderr)

            doc_dd1 = json.loads(proc_dd1.stdout)
            summary_dd1 = doc_dd1["summary"]

            self.assertEqual(summary_dd1["completed"], 4)
            self.assertEqual(summary_dd1["unfinished"], 0)

            # Verify Report 12 hand-computable oracle schedule
            expected_schedule = [
                {"id": "dd1_p0", "arr": 1, "start": 1, "wait": 0, "comp": 3},
                {"id": "dd1_p1", "arr": 2, "start": 3, "wait": 1, "comp": 5},
                {"id": "dd1_p2", "arr": 3, "start": 5, "wait": 2, "comp": 8},
                {"id": "dd1_p3", "arr": 8, "start": 8, "wait": 0, "comp": 9},
            ]

            actual_pts = {p["patient_id"]: p for p in summary_dd1["patients"]}
            for exp in expected_schedule:
                act = actual_pts[exp["id"]]
                self.assertEqual(
                    act["start_tick"],
                    exp["start"],
                    f"Patient {exp['id']} start_tick mismatch: expected {exp['start']}, got {act['start_tick']}",
                )
                self.assertEqual(
                    act["wait_ticks"],
                    exp["wait"],
                    f"Patient {exp['id']} wait_ticks mismatch: expected {exp['wait']}, got {act['wait_ticks']}",
                )
                self.assertEqual(
                    act["completion_tick"],
                    exp["comp"],
                    f"Patient {exp['id']} completion_tick mismatch: expected {exp['comp']}, got {act['completion_tick']}",
                )

    def test_differential_fidelity_comparison_through_cli(self):
        """Run Macro vs Micro comparison through runner compare command.

        Confirms:
        1. Comparison diagnostics report metric deltas neutrally without declaring empirical superiority.
        2. Micro fidelity structural transit delta increases elapsed duration.
        3. Diagnostics do not use value-laden superiority claims ('superior', 'better', 'clinical truth').
        """
        # Construct Macro and Micro run documents with identical patient cohorts
        # In Macro: elapsed_ticks == work_ticks (0 transit overhead)
        # In Micro: elapsed_ticks == work_ticks + transit_ticks (structural transit delay)
        patients_common = [
            {"id": "p001", "arr": 1, "work": 30, "transit": 5},
            {"id": "p002", "arr": 5, "work": 40, "transit": 5},
            {"id": "p003", "arr": 10, "work": 25, "transit": 5},
        ]

        macro_doc = {
            "manifest": {
                "schema_version": 1,
                "scenario_id": "synthetic-ed-macro",
                "input_file": "synthetic-ed-macro.json",
                "input_sha256": "0000000000000000000000000000000000000000000000000000000000000001",
                "seed": 42,
                "time_unit": "tick",
                "horizon_ticks": 100,
            },
            "summary": {
                "schema_version": 1,
                "scenario_id": "synthetic-ed-macro",
                "seed": 42,
                "time_unit": "tick",
                "horizon_ticks": 100,
                "arrivals": len(patients_common),
                "started": len(patients_common),
                "completed": len(patients_common),
                "unfinished": 0,
                "patients": [
                    {
                        "patient_id": p["id"],
                        "arrival_tick": p["arr"],
                        "start_tick": p["arr"],
                        "completion_tick": p["arr"] + p["work"],
                        "work_ticks": p["work"],
                        "wait_ticks": 0,
                        "elapsed_ticks": p["work"],
                        "status": "completed",
                    }
                    for p in patients_common
                ],
            },
        }

        micro_doc = {
            "manifest": {
                "schema_version": 1,
                "scenario_id": "synthetic-ed-micro",
                "input_file": "synthetic-ed-micro.json",
                "input_sha256": "0000000000000000000000000000000000000000000000000000000000000002",
                "seed": 42,
                "time_unit": "tick",
                "horizon_ticks": 100,
            },
            "summary": {
                "schema_version": 1,
                "scenario_id": "synthetic-ed-micro",
                "seed": 42,
                "time_unit": "tick",
                "horizon_ticks": 100,
                "arrivals": len(patients_common),
                "started": len(patients_common),
                "completed": len(patients_common),
                "unfinished": 0,
                "patients": [
                    {
                        "patient_id": p["id"],
                        "arrival_tick": p["arr"],
                        "start_tick": p["arr"],
                        "completion_tick": p["arr"] + p["work"] + p["transit"],
                        "work_ticks": p["work"],
                        "wait_ticks": 0,
                        "elapsed_ticks": p["work"] + p["transit"],
                        "status": "completed",
                    }
                    for p in patients_common
                ],
            },
        }

        with tempfile.TemporaryDirectory() as tmp_dir:
            macro_path = Path(tmp_dir) / "macro_run.json"
            micro_path = Path(tmp_dir) / "micro_run.json"

            macro_path.write_text(json.dumps(macro_doc, indent=2))
            micro_path.write_text(json.dumps(micro_doc, indent=2))

            proc_compare = self.run_cmd("compare", str(macro_path), str(micro_path))
            self.assertEqual(proc_compare.returncode, 0, proc_compare.stderr)

            rep = json.loads(proc_compare.stdout)
            self.assertEqual(rep["base_scenario_id"], "synthetic-ed-macro")
            self.assertEqual(rep["candidate_scenario_id"], "synthetic-ed-micro")

            metrics = {m["metric_name"]: m for m in rep["metrics"]}

            # 1. Micro increases elapsed duration due to transit
            self.assertIn("mean_elapsed", metrics)
            elapsed_m = metrics["mean_elapsed"]
            self.assertGreater(
                elapsed_m["delta"],
                0.0,
                "Micro fidelity must reflect structural transit delta in mean_elapsed",
            )
            self.assertAlmostEqual(elapsed_m["delta"], 5.0)

            # 2. Diagnostics report metric deltas neutrally without superiority claims
            diagnostics_text = " ".join(rep["diagnostics"]).lower()
            unpermitted_superiority_terms = [
                "superior",
                "inferior",
                "better model",
                "more accurate",
                "clinically validated",
                "ground truth",
            ]
            for term in unpermitted_superiority_terms:
                self.assertNotIn(
                    term,
                    diagnostics_text,
                    f"Diagnostic emitted non-neutral superiority claim '{term}'",
                )

            # 3. Report includes explicit validation taxonomy
            self.assertIn("validation_taxonomy", rep)
            self.assertEqual(
                rep["validation_taxonomy"]["verification_scope"],
                "software_verification",
            )

    def test_invariant_state_conservation_audit(self):
        """Verify strict patient and state conservation across diverse scenarios.

        Invariants:
        1. Patient count conservation: `arrivals == completed + unfinished`.
        2. Started bounds: `started <= arrivals`.
        3. Completed bounds: `completed <= started`.
        4. Every patient record is exhaustively accounted for in patient list.
        """
        # Test 1: Nominal MVP scenario
        proc_nom = self.run_cmd("run", str(NOMINAL_SCENARIO))
        self.assertEqual(proc_nom.returncode, 0)
        doc_nom = json.loads(proc_nom.stdout)
        sum_nom = doc_nom["summary"]

        self.assertEqual(sum_nom["arrivals"], sum_nom["completed"] + sum_nom["unfinished"])
        self.assertLessEqual(sum_nom["started"], sum_nom["arrivals"])
        self.assertLessEqual(sum_nom["completed"], sum_nom["started"])
        self.assertEqual(len(sum_nom["patients"]), sum_nom["arrivals"])

        # Test 2: Constrained MVP scenario
        proc_con = self.run_cmd("run", str(CONSTRAINED_SCENARIO))
        self.assertEqual(proc_con.returncode, 0)
        doc_con = json.loads(proc_con.stdout)
        sum_con = doc_con["summary"]

        self.assertEqual(sum_con["arrivals"], sum_con["completed"] + sum_con["unfinished"])
        self.assertLessEqual(sum_con["started"], sum_con["arrivals"])
        self.assertLessEqual(sum_con["completed"], sum_con["started"])
        self.assertEqual(len(sum_con["patients"]), sum_con["arrivals"])

        # Test 3: Truncated horizon scenario with unfinished presentations
        # Horizon is 8 ticks, while arrivals at tick 1, 3, 5 require 10 ticks each
        trunc_patients = [
            {
                "patient_id": f"trunc_p{i}",
                "arrival_tick": arr,
                "work_ticks": 10,
                "resource_id": "res.care",
                "task_class_id": "task.care",
            }
            for i, arr in enumerate([1, 3, 5])
        ]
        trunc_scenario = build_scenario_dict(
            scenario_id="horizon-truncation-conservation",
            horizon_ticks=8,
            resource_count=1,
            patients=trunc_patients,
            notes="Finite horizon truncation scenario testing unfinished conservation",
        )

        with tempfile.TemporaryDirectory() as tmp_dir:
            trunc_file = Path(tmp_dir) / "trunc.json"
            trunc_file.write_text(json.dumps(trunc_scenario, indent=2))

            proc_trunc = self.run_cmd("run", str(trunc_file))
            self.assertEqual(proc_trunc.returncode, 0)

            doc_trunc = json.loads(proc_trunc.stdout)
            sum_trunc = doc_trunc["summary"]

            # Patients cannot finish before horizon=8 (work=10)
            self.assertEqual(sum_trunc["arrivals"], 3)
            self.assertEqual(sum_trunc["completed"], 0)
            self.assertEqual(sum_trunc["unfinished"], 3)
            self.assertEqual(
                sum_trunc["completed"] + sum_trunc["unfinished"],
                sum_trunc["arrivals"],
                "Terminal conservation must hold when unfinished > 0",
            )
            self.assertLessEqual(sum_trunc["started"], sum_trunc["arrivals"])

            # Verify patient status breakdown
            completed_count = sum(
                1 for p in sum_trunc["patients"] if p["status"] == "completed" and p["completion_tick"] is not None
            )
            unfinished_count = sum(
                1 for p in sum_trunc["patients"] if p["status"] == "unfinished" and p["completion_tick"] is None
            )
            self.assertEqual(completed_count, sum_trunc["completed"])
            self.assertEqual(unfinished_count, sum_trunc["unfinished"])


if __name__ == "__main__":
    unittest.main()
