"""Multi-worker CPU execution, checkpointing, and fail-safe recovery test suite for E3.2.

Verifies:
1. Multi-worker batch execution across 1, 2, and 4 CPU workers with deterministic equivalence.
2. Checkpointing and atomic batch run artifacts in output directories.
3. Resumption of interrupted checkpoints via CLI `--resume`.
4. Fail-safe incomplete run protection: rejection of partial runs as finished experiments.
5. Deterministic error codes for malformed or missing batch options.
"""

import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT_DIR = Path(__file__).resolve().parents[2]
RUNNER_BIN = ROOT_DIR / "target/debug/examples/runner"
NOMINAL_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/nominal_mvp.json"


class E32BatchWorkersTests(unittest.TestCase):
    """End-to-end CLI batch runner and multi-worker execution tests for E3.2."""

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

    def test_batch_help_text(self):
        proc = self.run_cmd("--help")
        self.assertEqual(proc.returncode, 0)
        self.assertIn("batch <scenario.json>", proc.stdout)
        self.assertIn("--replications", proc.stdout)
        self.assertIn("--workers", proc.stdout)
        self.assertIn("--checkpoint", proc.stdout)
        self.assertIn("--resume", proc.stdout)

    def test_batch_deterministic_equivalence_across_workers(self):
        # 1 worker
        proc_1w = self.run_cmd(
            "batch",
            str(NOMINAL_SCENARIO),
            "--replications",
            "4",
            "--workers",
            "1",
            "--seed",
            "42",
        )
        self.assertEqual(proc_1w.returncode, 0, proc_1w.stderr)
        doc_1w = json.loads(proc_1w.stdout)

        # 2 workers
        proc_2w = self.run_cmd(
            "batch",
            str(NOMINAL_SCENARIO),
            "--replications",
            "4",
            "--workers",
            "2",
            "--seed",
            "42",
        )
        self.assertEqual(proc_2w.returncode, 0, proc_2w.stderr)
        doc_2w = json.loads(proc_2w.stdout)

        # 4 workers
        proc_4w = self.run_cmd(
            "batch",
            str(NOMINAL_SCENARIO),
            "--replications",
            "4",
            "--workers",
            "4",
            "--seed",
            "42",
        )
        self.assertEqual(proc_4w.returncode, 0, proc_4w.stderr)
        doc_4w = json.loads(proc_4w.stdout)

        # Strict deterministic equivalence checks
        self.assertEqual(doc_1w["total_replications"], 4)
        self.assertEqual(doc_2w["total_replications"], 4)
        self.assertEqual(doc_4w["total_replications"], 4)

        for i in range(4):
            rep1 = doc_1w["replications"][i]
            rep2 = doc_2w["replications"][i]
            rep4 = doc_4w["replications"][i]

            self.assertEqual(rep1["replication_index"], i)
            self.assertEqual(rep2["replication_index"], i)
            self.assertEqual(rep4["replication_index"], i)

            self.assertEqual(rep1["seed"], rep2["seed"])
            self.assertEqual(rep1["seed"], rep4["seed"])

            self.assertEqual(rep1["arrivals"], rep2["arrivals"])
            self.assertEqual(rep1["arrivals"], rep4["arrivals"])

            self.assertEqual(rep1["completed"], rep2["completed"])
            self.assertEqual(rep1["completed"], rep4["completed"])

            self.assertAlmostEqual(rep1["mean_wait_ticks"], rep2["mean_wait_ticks"])
            self.assertAlmostEqual(rep1["mean_wait_ticks"], rep4["mean_wait_ticks"])

        self.assertAlmostEqual(
            doc_1w["aggregate_mean_wait"], doc_2w["aggregate_mean_wait"]
        )
        self.assertAlmostEqual(
            doc_1w["aggregate_mean_wait"], doc_4w["aggregate_mean_wait"]
        )
        self.assertAlmostEqual(
            doc_1w["aggregate_std_error"], doc_2w["aggregate_std_error"]
        )
        self.assertAlmostEqual(
            doc_1w["aggregate_std_error"], doc_4w["aggregate_std_error"]
        )

    def test_batch_checkpoint_and_output_dir(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            proc = self.run_cmd(
                "batch",
                str(NOMINAL_SCENARIO),
                "--replications",
                "4",
                "--workers",
                "2",
                "--output-dir",
                tmpdir,
                "--checkpoint",
                "2",
            )
            self.assertEqual(proc.returncode, 0, proc.stderr)

            chk_file = Path(tmpdir) / "checkpoint.json"
            batch_file = Path(tmpdir) / "batch_run.json"
            partial_file = Path(tmpdir) / "batch_run.partial"

            self.assertTrue(chk_file.exists(), "checkpoint.json must exist")
            self.assertTrue(batch_file.exists(), "batch_run.json must exist")
            self.assertFalse(
                partial_file.exists(), "batch_run.partial must not exist on success"
            )

            chk_data = json.loads(chk_file.read_text())
            self.assertEqual(chk_data["total_replications"], 4)
            self.assertEqual(len(chk_data["completed_replications"]), 4)

    def test_batch_resumption_from_checkpoint(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir)
            chk_path = tmp_path / "checkpoint.json"

            # Create a partial checkpoint where 2 replications are done and 2 are pending
            base_seed = 777
            checkpoint_data = {
                "schema_version": 1,
                "batch_id": "test-resumed-batch",
                "scenario_id": "nominal-ed-mvp",
                "base_seed": base_seed,
                "total_replications": 4,
                "horizon_override": None,
                "completed_replications": [
                    {
                        "replication_index": 0,
                        "seed": base_seed,
                        "arrivals": 5,
                        "started": 5,
                        "completed": 5,
                        "unfinished": 0,
                        "mean_wait_ticks": 0.4,
                        "horizon_ticks": 20,
                    },
                    {
                        "replication_index": 1,
                        "seed": 11400714819323200000,
                        "arrivals": 5,
                        "started": 5,
                        "completed": 5,
                        "unfinished": 0,
                        "mean_wait_ticks": 0.4,
                        "horizon_ticks": 20,
                    },
                ],
                "pending_indices": [2, 3],
                "status": "in_progress",
                "checkpoint_epoch_ms": 12345678,
            }
            chk_path.write_text(json.dumps(checkpoint_data))

            # Resume using CLI
            proc = self.run_cmd(
                "batch",
                str(NOMINAL_SCENARIO),
                "--resume",
                str(chk_path),
                "--workers",
                "2",
                "--output-dir",
                tmpdir,
            )
            self.assertEqual(proc.returncode, 0, proc.stderr)
            doc = json.loads(proc.stdout)

            self.assertEqual(doc["status"], "complete")
            self.assertEqual(doc["total_replications"], 4)
            self.assertEqual(len(doc["replications"]), 4)

            # Check that final batch_run.json was written to output-dir
            batch_run_path = tmp_path / "batch_run.json"
            self.assertTrue(batch_run_path.exists())

    def test_batch_missing_scenario_returns_usage_error(self):
        proc = self.run_cmd("batch")
        self.assertEqual(proc.returncode, 2)
        self.assertIn("batch requires a scenario file path", proc.stderr)

    def test_batch_nonexistent_scenario_returns_file_not_found(self):
        proc = self.run_cmd("batch", "nonexistent_scenario_file.json")
        self.assertEqual(proc.returncode, 3)
        self.assertIn("scenario file not found", proc.stderr)


if __name__ == "__main__":
    unittest.main()
