"""Arrow/Parquet schemas, metric denominators, uncertainty, and spatial CLI tests for E3.3.

Verifies:
1. Export of canonical Arrow schemas and Parquet DDL via CLI `export --format arrow` and `export --format parquet`.
2. Batch execution with `--export-arrow` generating full bundle with metric denominators, statistical uncertainty,
   censored/unfinished counts, and complete provenance.
3. Batch execution with `--export-spatial` generating route traversal CSV summaries.
4. Deterministic equivalence of Arrow bundles across 1 and 4 CPU workers.
5. Trajectory position kind classification and spatial revision persistence.
6. Error handling for missing arguments on new flags.
"""

import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT_DIR = Path(__file__).resolve().parents[2]
RUNNER_BIN = ROOT_DIR / "target/debug/examples/runner"
NOMINAL_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/nominal_mvp.json"


class E33ArrowSpatialCliTests(unittest.TestCase):
    """End-to-end CLI tests for Phase E3.3 Arrow/Parquet and spatial exports."""

    @classmethod
    def setUpClass(cls):
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

    def test_cli_export_arrow_and_parquet_schemas(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            run_json = Path(tmp_dir) / "run.json"
            # 1. Run nominal scenario
            proc_run = self.run_cmd("run", str(NOMINAL_SCENARIO), "-o", str(run_json))
            self.assertEqual(proc_run.returncode, 0, proc_run.stderr)
            self.assertTrue(run_json.exists())

            # 2. Export arrow schema
            proc_arrow = self.run_cmd("export", str(run_json), "--format", "arrow")
            self.assertEqual(proc_arrow.returncode, 0, proc_arrow.stderr)
            arrow_doc = json.loads(proc_arrow.stdout)
            self.assertIn("arrow_schema", arrow_doc)
            fields = arrow_doc["arrow_schema"]["fields"]
            field_names = [f["name"] for f in fields]
            self.assertIn("patient_id", field_names)
            self.assertIn("wait_ticks", field_names)
            self.assertIn("elapsed_ticks", field_names)

            # Check denominator annotation
            wait_field = next(f for f in fields if f["name"] == "wait_ticks")
            self.assertEqual(wait_field.get("denominator"), "started_patients")
            elapsed_field = next(f for f in fields if f["name"] == "elapsed_ticks")
            self.assertEqual(elapsed_field.get("denominator"), "completed_patients")

            # 3. Export parquet schema DDL
            proc_parquet = self.run_cmd("export", str(run_json), "--format", "parquet")
            self.assertEqual(proc_parquet.returncode, 0, proc_parquet.stderr)
            self.assertIn("message careops_ed_patient_record", proc_parquet.stdout)
            self.assertIn("message careops_ed_metric_summary", proc_parquet.stdout)
            self.assertIn("message careops_ed_spatial_transit", proc_parquet.stdout)

    def test_cli_batch_arrow_bundle_export(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            bundle_path = Path(tmp_dir) / "bundle.json"
            proc = self.run_cmd(
                "batch",
                str(NOMINAL_SCENARIO),
                "--replications",
                "4",
                "--workers",
                "2",
                "--seed",
                "12345",
                "--export-arrow",
                str(bundle_path),
                "--spatial-rev",
                "spatial.v1.test-rev",
            )
            self.assertEqual(proc.returncode, 0, proc.stderr)
            self.assertTrue(bundle_path.exists())

            with open(bundle_path, "r", encoding="utf-8") as f:
                bundle = json.load(f)

            # 1. Check schemas
            self.assertEqual(bundle["patient_schema"]["schema_name"], "careops.ed.patient_records.v1")
            self.assertEqual(bundle["metrics_schema"]["schema_name"], "careops.ed.metrics.v1")
            self.assertEqual(bundle["spatial_schema"]["schema_name"], "careops.ed.spatial_transit.v1")
            self.assertIn("careops_ed_patient_record", bundle["parquet_schema"])

            # 2. Check provenance
            prov = bundle["provenance"]
            self.assertEqual(prov["spatial_revision"], "spatial.v1.test-rev")
            self.assertEqual(prov["total_replications"], 4)
            self.assertEqual(len(prov["replication_seeds"]), 4)

            # 3. Check metrics with denominators and uncertainty
            metrics = bundle["metrics"]
            m_names = [m["name"] for m in metrics]
            self.assertIn("mean_wait_ticks", m_names)
            self.assertIn("completion_rate", m_names)
            self.assertIn("throughput_per_horizon_tick", m_names)
            self.assertIn("censored_presentation_rate", m_names)

            wait_m = next(m for m in metrics if m["name"] == "mean_wait_ticks")
            self.assertEqual(wait_m["denominator_name"], "started_patients")
            self.assertIn("variance", wait_m["uncertainty"])
            self.assertIn("standard_error", wait_m["uncertainty"])
            self.assertIn("ci_95_lower", wait_m["uncertainty"])
            self.assertIn("ci_95_upper", wait_m["uncertainty"])
            self.assertEqual(wait_m["uncertainty"]["replications_count"], 4)

            comp_m = next(m for m in metrics if m["name"] == "completion_rate")
            self.assertEqual(comp_m["denominator_name"], "total_arrivals")

            tp_m = next(m for m in metrics if m["name"] == "throughput_per_horizon_tick")
            self.assertEqual(tp_m["denominator_name"], "horizon_ticks")

            # 4. Check patient records
            patient_recs = bundle["patient_records"]
            self.assertGreater(len(patient_recs), 0)
            sample_p = patient_recs[0]
            self.assertEqual(sample_p["spatial_revision"], "spatial.v1.test-rev")
            self.assertIn(sample_p["censor_status"], ["censored", "uncensored"])

            # 5. Check spatial transit records and position kinds
            spatial_recs = bundle["spatial_records"]
            self.assertGreater(len(spatial_recs), 0)
            sample_s = spatial_recs[0]
            self.assertEqual(sample_s["spatial_revision"], "spatial.v1.test-rev")
            self.assertEqual(sample_s["position_kind"], "simulated")
            self.assertGreater(sample_s["distance_metres"], 0.0)

            # 6. Check spatial summary
            summary = bundle["spatial_summary"]
            self.assertEqual(summary["spatial_revision"], "spatial.v1.test-rev")
            self.assertGreater(summary["total_transits"], 0)
            self.assertGreater(len(summary["route_traversal_counts"]), 0)

    def test_cli_batch_spatial_csv_export(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            csv_path = Path(tmp_dir) / "spatial_summary.csv"
            proc = self.run_cmd(
                "batch",
                str(NOMINAL_SCENARIO),
                "--replications",
                "2",
                "--workers",
                "1",
                "--export-spatial",
                str(csv_path),
                "--spatial-rev",
                "spatial.v2.cairns",
            )
            self.assertEqual(proc.returncode, 0, proc.stderr)
            self.assertTrue(csv_path.exists())

            content = csv_path.read_text(encoding="utf-8")
            lines = content.strip().split("\n")
            self.assertGreaterEqual(len(lines), 2)
            header = lines[0]
            self.assertEqual(header, "route,traversals,mean_duration_ticks,spatial_revision")
            self.assertIn("spatial.v2.cairns", lines[1])

    def test_cli_batch_deterministic_bundles_across_workers(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            b1 = Path(tmp_dir) / "b1.json"
            b4 = Path(tmp_dir) / "b4.json"

            proc1 = self.run_cmd(
                "batch",
                str(NOMINAL_SCENARIO),
                "--replications",
                "4",
                "--workers",
                "1",
                "--seed",
                "9999",
                "--export-arrow",
                str(b1),
            )
            self.assertEqual(proc1.returncode, 0, proc1.stderr)

            proc4 = self.run_cmd(
                "batch",
                str(NOMINAL_SCENARIO),
                "--replications",
                "4",
                "--workers",
                "4",
                "--seed",
                "9999",
                "--export-arrow",
                str(b4),
            )
            self.assertEqual(proc4.returncode, 0, proc4.stderr)

            doc1 = json.loads(b1.read_text(encoding="utf-8"))
            doc4 = json.loads(b4.read_text(encoding="utf-8"))

            # Patient records match bit-for-bit
            self.assertEqual(doc1["patient_records"], doc4["patient_records"])
            # Spatial records match bit-for-bit
            self.assertEqual(doc1["spatial_records"], doc4["spatial_records"])
            # Metrics match bit-for-bit
            self.assertEqual(doc1["metrics"], doc4["metrics"])
            # Spatial summary matches bit-for-bit
            self.assertEqual(doc1["spatial_summary"], doc4["spatial_summary"])

    def test_cli_error_codes_for_missing_arguments(self):
        proc_arrow = self.run_cmd("batch", str(NOMINAL_SCENARIO), "--export-arrow")
        self.assertEqual(proc_arrow.returncode, 2)
        self.assertIn("--export-arrow requires a file path", proc_arrow.stderr)

        proc_spatial = self.run_cmd("batch", str(NOMINAL_SCENARIO), "--export-spatial")
        self.assertEqual(proc_spatial.returncode, 2)
        self.assertIn("--export-spatial requires a file path", proc_spatial.stderr)

        proc_rev = self.run_cmd("batch", str(NOMINAL_SCENARIO), "--spatial-rev")
        self.assertEqual(proc_rev.returncode, 2)
        self.assertIn("--spatial-rev requires a revision string", proc_rev.stderr)


if __name__ == "__main__":
    unittest.main()
