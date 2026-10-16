"""Delivery Contract Fresh User Walkthrough and Troubleshooting Suite for Task E4.3.

Qualifies hardened native v1 against the delivery contract:
A fresh downstream user installs/builds the CLI, inspects and validates documented
reference scenarios, edits a scenario to model an ED surge condition, runs experiments
with atomic writes and overwrite protection, executes comparative evaluations with
signed deltas and neutral reporting, exports to multi-format schemas (CSV, Arrow, Parquet),
troubleshoots intentional input errors with deterministic exit codes and actionable
diagnostics, and rehearses batch checkpoint recovery and configuration rollback.

Verifies:
1. Installation & help interface verification.
2. Nominal scenario inspection & validation.
3. Scenario editing (creating surge condition).
4. Execution of both scenarios (`run`).
5. Comparative analysis (`compare`) asserting signed deltas and neutral reporting.
6. Multi-format export (`export`).
7. Intentional error troubleshooting (zero capacity, nonpositive horizon, unknown zone, malformed JSON, mismatched comparison, missing file).
8. Rehearse rollback & checkpoint recovery verification.
"""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT_DIR = Path(__file__).resolve().parents[2]
RUNNER_BIN = ROOT_DIR / "target/debug/examples/runner"
CLI_BIN = ROOT_DIR / "target/debug/careops-ed"
NOMINAL_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/nominal_mvp.json"
CONSTRAINED_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/constrained_mvp.json"


class E43DeliveryContractWalkthroughTests(unittest.TestCase):
    """End-to-end fresh user delivery contract walkthrough and troubleshooting suite."""

    @classmethod
    def setUpClass(cls) -> None:
        # Build both the headless runner example and careops-ed-cli binary
        subprocess.run(
            ["cargo", "build", "--locked", "-p", "careops-ed", "--example", "runner"],
            cwd=ROOT_DIR,
            check=True,
            capture_output=True,
        )
        subprocess.run(
            ["cargo", "build", "--locked", "-p", "careops-ed-cli"],
            cwd=ROOT_DIR,
            check=True,
            capture_output=True,
        )

    def run_cmd(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [str(RUNNER_BIN), *args],
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
        )

    # -------------------------------------------------------------------------
    # 1. Installation & Help Interface Verification
    # -------------------------------------------------------------------------
    def test_01_installation_and_help_interface(self) -> None:
        """Verify binary installation, usage documentation, and deterministic exit code table."""
        self.assertTrue(RUNNER_BIN.exists(), f"runner binary must exist at {RUNNER_BIN}")
        self.assertTrue(CLI_BIN.exists(), f"careops-ed cli binary must exist at {CLI_BIN}")

        # Help flag verification
        for flag in ("--help", "-h"):
            proc = self.run_cmd(flag)
            self.assertEqual(proc.returncode, 0)
            self.assertIn("Usage: careops-ed <command> [options]", proc.stdout)
            self.assertIn("validate <scenario.json>", proc.stdout)
            self.assertIn("run <scenario.json> [options]", proc.stdout)
            self.assertIn("batch <scenario.json> [options]", proc.stdout)
            self.assertIn("compare <base_run.json> <candidate_run.json>", proc.stdout)
            self.assertIn("export <run.json> [options]", proc.stdout)
            self.assertIn("Exit Codes:", proc.stdout)
            self.assertIn("0  Success", proc.stdout)
            self.assertIn("2  Usage error / invalid arguments", proc.stdout)
            self.assertIn("3  File not found", proc.stdout)
            self.assertIn("4  Malformed scenario configuration", proc.stdout)
            self.assertIn("5  Incompatible scenarios for comparison", proc.stdout)
            self.assertIn("6  Output file already exists", proc.stdout)
            self.assertIn("7  Output write failed", proc.stdout)
            self.assertIn("8  Simulation failed", proc.stdout)
            self.assertIn("9  Interrupted / cancelled execution", proc.stdout)

        # Missing command produces usage error (code 2)
        proc_empty = self.run_cmd()
        self.assertEqual(proc_empty.returncode, 2)
        self.assertIn("missing command", proc_empty.stderr)

        # Unknown command produces usage error (code 2)
        proc_unknown = self.run_cmd("unknown-command")
        self.assertEqual(proc_unknown.returncode, 2)
        self.assertIn("unknown command 'unknown-command'", proc_unknown.stderr)

    # -------------------------------------------------------------------------
    # 2. Nominal Scenario Inspection & Validation
    # -------------------------------------------------------------------------
    def test_02_nominal_scenario_inspection_and_validation(self) -> None:
        """Verify nominal reference scenario inspection and schema validation."""
        self.assertTrue(NOMINAL_SCENARIO.exists())
        proc = self.run_cmd("validate", str(NOMINAL_SCENARIO))
        self.assertEqual(proc.returncode, 0, f"validation failed: {proc.stderr}")
        self.assertIn("valid scenario 'nominal-ed-mvp'", proc.stdout)
        self.assertIn("schema v1", proc.stdout)
        self.assertIn("20 ticks", proc.stdout)
        self.assertIn("5 patients", proc.stdout)
        self.assertIn("5 physical / 5 open beds", proc.stdout)

    # -------------------------------------------------------------------------
    # 3. Scenario Editing (Creating Surge Condition)
    # -------------------------------------------------------------------------
    def test_03_scenario_editing_surge_condition(self) -> None:
        """Fresh user edits documented scenario to simulate emergency department surge."""
        nominal_data = json.loads(NOMINAL_SCENARIO.read_text(encoding="utf-8"))

        # Create surge scenario by doubling patient volume with high-work presentations
        surge_data = dict(nominal_data)
        surge_data["scenario_id"] = "surge-ed-mvp"
        surge_data["provenance"] = {
            "class": "synthetic",
            "notes": "User-edited surge scenario: double arrivals under fixed bay capacity.",
        }

        # Add 5 additional arriving patients to induce queue contention
        surge_patients = list(nominal_data["patients"])
        for i in range(1, 6):
            surge_patients.append({
                "patient_id": f"surge-patient-{i:03d}",
                "arrival_tick": 2 * i,
                "work_ticks": 4,
                "resource_id": "res.acute",
                "task_class_id": "task.acute_treatment",
            })
        surge_data["patients"] = surge_patients

        with tempfile.TemporaryDirectory() as tmp_dir:
            surge_file = Path(tmp_dir) / "surge_mvp.json"
            surge_file.write_text(json.dumps(surge_data, indent=2), encoding="utf-8")

            # Validate edited surge scenario
            proc = self.run_cmd("validate", str(surge_file))
            self.assertEqual(proc.returncode, 0, f"surge validation failed: {proc.stderr}")
            self.assertIn("valid scenario 'surge-ed-mvp'", proc.stdout)
            self.assertIn("10 patients", proc.stdout)

    # -------------------------------------------------------------------------
    # 4. Execution of Both Scenarios (run)
    # -------------------------------------------------------------------------
    def test_04_execution_atomic_writes_and_overwrite_protection(self) -> None:
        """Run experiments, verify JSON document output, atomic write, and overwrite protection."""
        nominal_data = json.loads(NOMINAL_SCENARIO.read_text(encoding="utf-8"))
        surge_data = dict(nominal_data)
        surge_data["scenario_id"] = "surge-ed-mvp"
        surge_patients = list(nominal_data["patients"])
        for i in range(1, 6):
            surge_patients.append({
                "patient_id": f"surge-patient-{i:03d}",
                "arrival_tick": 2 + i,
                "work_ticks": 4,
                "resource_id": "res.acute",
                "task_class_id": "task.acute_treatment",
            })
        surge_data["patients"] = surge_patients

        with tempfile.TemporaryDirectory() as tmp_dir:
            tmp_path = Path(tmp_dir)
            surge_file = tmp_path / "surge_mvp.json"
            surge_file.write_text(json.dumps(surge_data, indent=2), encoding="utf-8")

            nom_out = tmp_path / "nominal_run.json"
            surge_out = tmp_path / "surge_run.json"

            # 1. Run nominal scenario
            proc_nom = self.run_cmd("run", str(NOMINAL_SCENARIO), "--output", str(nom_out))
            self.assertEqual(proc_nom.returncode, 0, proc_nom.stderr)
            self.assertTrue(nom_out.exists())

            doc_nom = json.loads(nom_out.read_text(encoding="utf-8"))
            self.assertEqual(doc_nom["manifest"]["scenario_id"], "nominal-ed-mvp")
            self.assertEqual(doc_nom["manifest"]["schema_version"], 1)
            self.assertEqual(doc_nom["summary"]["arrivals"], 5)
            self.assertEqual(doc_nom["summary"]["completed"], 5)
            self.assertEqual(doc_nom["summary"]["unfinished"], 0)
            self.assertIn("verification_scope", doc_nom["validation_taxonomy"])

            # 2. Run surge scenario
            proc_surge = self.run_cmd("run", str(surge_file), "--output", str(surge_out))
            self.assertEqual(proc_surge.returncode, 0, proc_surge.stderr)
            self.assertTrue(surge_out.exists())

            doc_surge = json.loads(surge_out.read_text(encoding="utf-8"))
            self.assertEqual(doc_surge["manifest"]["scenario_id"], "surge-ed-mvp")
            self.assertEqual(doc_surge["summary"]["arrivals"], 10)

            # 3. Overwrite protection: running without --overwrite fails with exit code 6
            proc_clobber = self.run_cmd("run", str(NOMINAL_SCENARIO), "--output", str(nom_out))
            self.assertEqual(proc_clobber.returncode, 6)
            self.assertIn("already exists", proc_clobber.stderr)

            # 4. Overwrite allowed: running with --overwrite succeeds
            proc_overwrite = self.run_cmd(
                "run",
                str(NOMINAL_SCENARIO),
                "--output",
                str(nom_out),
                "--overwrite",
            )
            self.assertEqual(proc_overwrite.returncode, 0)

    # -------------------------------------------------------------------------
    # 5. Comparative Analysis (compare) Asserting Signed Deltas & Neutral Reporting
    # -------------------------------------------------------------------------
    def test_05_comparative_analysis_signed_deltas_and_neutral_reporting(self) -> None:
        """Compare nominal and constrained runs, asserting signed deltas, units, and neutral reporting."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            tmp_path = Path(tmp_dir)
            nom_out = tmp_path / "nominal_run.json"
            con_out = tmp_path / "constrained_run.json"

            self.assertEqual(
                self.run_cmd("run", str(NOMINAL_SCENARIO), "--output", str(nom_out)).returncode,
                0,
            )
            self.assertEqual(
                self.run_cmd("run", str(CONSTRAINED_SCENARIO), "--output", str(con_out)).returncode,
                0,
            )

            # Compare command
            proc_comp = self.run_cmd("compare", str(nom_out), str(con_out))
            self.assertEqual(proc_comp.returncode, 0, proc_comp.stderr)

            report = json.loads(proc_comp.stdout)
            self.assertEqual(report["base_scenario_id"], "nominal-ed-mvp")
            self.assertEqual(report["candidate_scenario_id"], "constrained-ed-mvp")
            self.assertEqual(report["time_unit"], "tick")

            mean_wait = next(m for m in report["metrics"] if m["metric_name"] == "mean_wait")
            self.assertEqual(mean_wait["unit"], "tick")
            self.assertEqual(
                mean_wait["denominator_description"],
                "completed patients with wait intervals",
            )
            self.assertGreater(mean_wait["delta"], 0.0)
            self.assertAlmostEqual(mean_wait["delta"], 3.8)
            self.assertGreater(mean_wait["percent_change"], 0.0)

            arrivals = next(m for m in report["metrics"] if m["metric_name"] == "arrivals")
            self.assertEqual(arrivals["unit"], "patient")
            self.assertEqual(arrivals["denominator_description"], "total presentation count")
            self.assertEqual(arrivals["delta"], 0.0)

            # Neutral reporting & clinical disclaimer
            self.assertIn(
                "does NOT constitute clinical domain validation",
                report["validation_taxonomy"]["disclaimer"],
            )
            self.assertTrue(
                any("increased mean wait" in d for d in report["diagnostics"]),
                f"expected increased wait diagnostic: {report['diagnostics']}",
            )

    # -------------------------------------------------------------------------
    # 6. Multi-Format Export (export)
    # -------------------------------------------------------------------------
    def test_06_multiformat_export_csv_arrow_parquet(self) -> None:
        """Export run artifact to CSV, Arrow schema, and Parquet schema formats."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            tmp_path = Path(tmp_dir)
            nom_run = tmp_path / "nominal_run.json"
            self.assertEqual(
                self.run_cmd("run", str(NOMINAL_SCENARIO), "--output", str(nom_run)).returncode,
                0,
            )

            # 1. Export CSV
            csv_out = tmp_path / "patients.csv"
            proc_csv = self.run_cmd(
                "export",
                str(nom_run),
                "--format",
                "csv",
                "--output",
                str(csv_out),
            )
            self.assertEqual(proc_csv.returncode, 0, proc_csv.stderr)
            self.assertTrue(csv_out.exists())
            csv_lines = csv_out.read_text(encoding="utf-8").strip().splitlines()
            self.assertIn("patient_id,arrival_tick", csv_lines[0])
            self.assertEqual(len(csv_lines), 6)  # header + 5 patients

            # 2. Export Arrow schema
            arrow_out = tmp_path / "schema_arrow.json"
            proc_arrow = self.run_cmd(
                "export",
                str(nom_run),
                "--format",
                "arrow",
                "--output",
                str(arrow_out),
            )
            self.assertEqual(proc_arrow.returncode, 0, proc_arrow.stderr)
            self.assertTrue(arrow_out.exists())
            arrow_doc = json.loads(arrow_out.read_text(encoding="utf-8"))
            self.assertIn("arrow_schema", arrow_doc)
            field_names = [f["name"] for f in arrow_doc["arrow_schema"]["fields"]]
            self.assertIn("patient_id", field_names)
            self.assertIn("wait_ticks", field_names)
            self.assertIn("elapsed_ticks", field_names)

            # 3. Export Parquet schema DDL
            proc_parquet = self.run_cmd("export", str(nom_run), "--format", "parquet")
            self.assertEqual(proc_parquet.returncode, 0, proc_parquet.stderr)
            self.assertIn("message careops_ed_patient_record", proc_parquet.stdout)

    # -------------------------------------------------------------------------
    # 7. Intentional Error Troubleshooting
    # -------------------------------------------------------------------------
    def test_07_intentional_error_troubleshooting(self) -> None:
        """Inject intentional errors and assert deterministic error codes and actionable diagnostics."""
        nominal_text = NOMINAL_SCENARIO.read_text(encoding="utf-8")

        with tempfile.TemporaryDirectory() as tmp_dir:
            tmp_path = Path(tmp_dir)

            # Case A: Zero capacity (both physical and open counts 0)
            zero_cap_text = (
                nominal_text.replace('"physical_count": 5,', '"physical_count": 0,')
                .replace('"open_count": 5,', '"open_count": 0,')
                .replace('"staffed_count": 5,', '"staffed_count": 0,')
            )
            zero_cap_file = tmp_path / "zero_cap.json"
            zero_cap_file.write_text(zero_cap_text, encoding="utf-8")
            proc_a = self.run_cmd("validate", str(zero_cap_file))
            self.assertEqual(proc_a.returncode, 4, "zero capacity must exit with 4 (MalformedConfig)")
            self.assertIn("capacity physical and open counts cannot both be zero", proc_a.stderr)

            # Case B: Nonpositive horizon (horizon_ticks = 0)
            zero_hor_text = nominal_text.replace('"horizon_ticks": 20,', '"horizon_ticks": 0,')
            zero_hor_file = tmp_path / "zero_horizon.json"
            zero_hor_file.write_text(zero_hor_text, encoding="utf-8")
            proc_b = self.run_cmd("validate", str(zero_hor_file))
            self.assertEqual(proc_b.returncode, 4, "zero horizon must exit with 4 (MalformedConfig)")
            self.assertIn("horizon_ticks", proc_b.stderr)
            self.assertTrue(
                "positive" in proc_b.stderr or "greater than zero" in proc_b.stderr,
                f"expected positive horizon diagnostic: {proc_b.stderr}",
            )

            # Case C: Unknown zone reference
            bad_zone_text = nominal_text.replace(
                '"location_id": "loc.triage.1", "name": "Triage Desk 1", "zone_id": "zone.triage"',
                '"location_id": "loc.triage.1", "name": "Triage Desk 1", "zone_id": "zone.unregistered"',
            )
            bad_zone_file = tmp_path / "bad_zone.json"
            bad_zone_file.write_text(bad_zone_text, encoding="utf-8")
            proc_c = self.run_cmd("validate", str(bad_zone_file))
            self.assertEqual(proc_c.returncode, 4, "unknown zone must exit with 4 (MalformedConfig)")
            self.assertTrue(
                "unknown zone" in proc_c.stderr or "unresolved" in proc_c.stderr,
                f"expected unknown zone diagnostic: {proc_c.stderr}",
            )

            # Case D: Malformed JSON syntax
            broken_file = tmp_path / "broken.json"
            broken_file.write_text('{ "scenario_id": "broken", invalid json syntax }', encoding="utf-8")
            proc_d = self.run_cmd("validate", str(broken_file))
            self.assertEqual(proc_d.returncode, 4, "malformed JSON syntax must exit with 4")
            self.assertIn("invalid scenario", proc_d.stderr)

            # Case E: Incompatible scenario comparison (mismatched time units)
            nom_run = tmp_path / "nom_run.json"
            self.run_cmd("run", str(NOMINAL_SCENARIO), "--output", str(nom_run))

            bad_unit_data = json.loads(nom_run.read_text(encoding="utf-8"))
            bad_unit_data["manifest"]["time_unit"] = "minute"
            bad_unit_file = tmp_path / "bad_unit_run.json"
            bad_unit_file.write_text(json.dumps(bad_unit_data), encoding="utf-8")

            proc_e = self.run_cmd("compare", str(nom_run), str(bad_unit_file))
            self.assertEqual(proc_e.returncode, 5, "incompatible comparison must exit with 5")
            self.assertIn("incompatible time units", proc_e.stderr)

            # Case F: Missing input file
            proc_f = self.run_cmd("validate", str(tmp_path / "non_existent.json"))
            self.assertEqual(proc_f.returncode, 3, "missing file must exit with 3 (FileNotFound)")
            self.assertIn("scenario file not found", proc_f.stderr)

    # -------------------------------------------------------------------------
    # 8. Rehearse Rollback & Checkpoint Recovery Verification
    # -------------------------------------------------------------------------
    def test_08_rollback_and_checkpoint_recovery_rehearsal(self) -> None:
        """Verify checkpoint generation, clean resumption, corrupted checkpoint rejection, and scenario rollback."""
        with tempfile.TemporaryDirectory() as tmp_dir:
            tmp_path = Path(tmp_dir)
            batch_dir = tmp_path / "batch_initial"
            batch_dir.mkdir()

            # 1. Run batch execution with checkpointing
            proc_batch = self.run_cmd(
                "batch",
                str(NOMINAL_SCENARIO),
                "--replications",
                "4",
                "--workers",
                "2",
                "--output-dir",
                str(batch_dir),
                "--checkpoint",
                "2",
            )
            self.assertEqual(proc_batch.returncode, 0, proc_batch.stderr)

            chk_file = batch_dir / "checkpoint.json"
            batch_file = batch_dir / "batch_run.json"
            self.assertTrue(chk_file.exists(), "checkpoint.json must exist")
            self.assertTrue(batch_file.exists(), "batch_run.json must exist")

            # 2. Clean resumption from checkpoint
            resume_dir = tmp_path / "batch_resumed"
            resume_dir.mkdir()
            proc_resume = self.run_cmd(
                "batch",
                str(NOMINAL_SCENARIO),
                "--resume",
                str(chk_file),
                "--output-dir",
                str(resume_dir),
            )
            self.assertEqual(proc_resume.returncode, 0, proc_resume.stderr)

            # Resumed results match original batch run
            doc_orig = json.loads(batch_file.read_text(encoding="utf-8"))
            doc_res = json.loads((resume_dir / "batch_run.json").read_text(encoding="utf-8"))
            self.assertEqual(doc_orig["total_replications"], doc_res["total_replications"])
            self.assertAlmostEqual(doc_orig["aggregate_mean_wait"], doc_res["aggregate_mean_wait"])

            # 3. Corrupted checkpoint safely rejected without corrupting prior outputs
            corrupted_chk = tmp_path / "corrupted_checkpoint.json"
            corrupted_chk.write_text("{ broken checkpoint JSON payload", encoding="utf-8")
            proc_corrupt = self.run_cmd(
                "batch",
                str(NOMINAL_SCENARIO),
                "--resume",
                str(corrupted_chk),
            )
            self.assertNotEqual(proc_corrupt.returncode, 0, "corrupted checkpoint must be rejected")
            # Ensure prior outputs remain intact
            self.assertTrue(batch_file.exists())
            self.assertEqual(json.loads(batch_file.read_text())["total_replications"], 4)

            # 4. User scenario rollback drill:
            # User introduces an edit error, troubleshoots it, rolls back to nominal, and verifies clean pass.
            work_file = tmp_path / "active_scenario.json"
            work_file.write_text(NOMINAL_SCENARIO.read_text(encoding="utf-8"), encoding="utf-8")

            # Erroneous edit: set horizon to zero
            work_file.write_text(
                work_file.read_text().replace('"horizon_ticks": 20,', '"horizon_ticks": 0,'),
                encoding="utf-8",
            )
            proc_bad = self.run_cmd("validate", str(work_file))
            self.assertEqual(proc_bad.returncode, 4)

            # Rollback drill: restore LKG nominal content
            work_file.write_text(NOMINAL_SCENARIO.read_text(encoding="utf-8"), encoding="utf-8")
            proc_restored = self.run_cmd("validate", str(work_file))
            self.assertEqual(proc_restored.returncode, 0, "restored scenario must validate cleanly")
            proc_restored_run = self.run_cmd("run", str(work_file))
            self.assertEqual(proc_restored_run.returncode, 0, "restored scenario must execute cleanly")


if __name__ == "__main__":
    unittest.main()
