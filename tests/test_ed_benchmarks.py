import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import ed_benchmarks as bench


class BenchmarkOracleTests(unittest.TestCase):
    def setUp(self):
        self.template = json.loads(Path("crates/careops-ed/examples/one_patient.json").read_text())
        self.raw = bench.make_input(self.template, 3)
        self.expected = {"count": 3, "scenario_id": "d34-fifo-3"}
        shared = {"schema_version": 1, "scenario_id": "d34-fifo-3", "seed": 7,
                  "time_unit": "tick", "horizon_ticks": 8}
        self.doc = {"manifest": {**shared, "input_sha256": bench.sha256(self.raw),
                                 "input_file": "input.json"},
                    "summary": {**shared,
                                "arrivals": 3, "started": 3, "completed": 3,
                                "unfinished": 0,
                                "patients": [
                                    {"patient_id": f"synthetic-patient-{i + 1:05d}",
                                     "arrival_tick": 1, "start_tick": 1 + i * 2,
                                     "completion_tick": 3 + i * 2, "work_ticks": 2,
                                     "wait_ticks": i * 2, "elapsed_ticks": 2 + i * 2,
                                     "status": "completed"} for i in range(3)]}}

    def validate(self, document=None, input_hash=None):
        return bench.validate_document(document or self.doc, self.expected,
                                       input_hash or bench.sha256(self.raw), "input.json")

    def test_independent_oracle_accepts_all_rows_and_conservation(self):
        got = self.validate()
        self.assertEqual(got["completed"], 3)
        self.assertEqual(got["patients"][-1]["wait_ticks"], 4)

    def test_rejects_corrupt_patient_row(self):
        bad = json.loads(json.dumps(self.doc))
        bad["summary"]["patients"][1]["completion_tick"] += 1
        with self.assertRaises(bench.BenchmarkError):
            self.validate(bad)

    def test_rejects_bad_manifest_hash_and_counts(self):
        bad_hash = json.loads(json.dumps(self.doc))
        bad_hash["manifest"]["input_sha256"] = "0" * 64
        with self.assertRaises(bench.BenchmarkError):
            self.validate(bad_hash)
        bad_count = json.loads(json.dumps(self.doc))
        bad_count["summary"]["completed"] = 2
        with self.assertRaises(bench.BenchmarkError):
            self.validate(bad_count)

    def test_rejects_nonconserving_row_count(self):
        bad = json.loads(json.dumps(self.doc))
        bad["summary"]["patients"].pop()
        with self.assertRaises(bench.BenchmarkError):
            self.validate(bad)

    def test_bool_count_and_wrong_units_cannot_pass_numeric_oracle(self):
        for section, key, value in (("summary", "unfinished", False),
                                    ("manifest", "time_unit", "seconds")):
            bad = json.loads(json.dumps(self.doc))
            bad[section][key] = value
            with self.assertRaises(bench.BenchmarkError):
                self.validate(bad)

    def test_oversized_input_is_refused_before_dispatch(self):
        with self.assertRaises(bench.BenchmarkError):
            bench.preflight_input(b"x" * (bench.MAX_INPUT + 1))


@unittest.skipUnless(hasattr(os, "wait4") and os.name == "posix", "requires POSIX wait4")
class BoundedProcessTests(unittest.TestCase):
    def test_slow_consumer_drains_large_stdout_with_bounded_capture(self):
        code = "import sys; [sys.stdout.buffer.write(b'x' * 8192) for _ in range(128)]; sys.stdout.flush()"
        result = bench.run_bounded([sys.executable, "-c", code], stdout_limit=2 * 1024 * 1024,
                                   slow_consumer_delay=0.002)
        self.assertEqual(result["exit_code"], 0)
        self.assertFalse(result["stdout_overflow"])
        self.assertEqual(len(result["stdout"]), 128 * 8192)
        self.assertGreater(result["stdout_read_count"], 1)
        self.assertGreater(result["peak_rss_bytes"], 0)

    def test_cancel_kills_and_reaps_child_while_it_writes(self):
        code = "import sys,time; sys.stdout.buffer.write(b'x'*10000000); sys.stdout.flush(); time.sleep(30)"
        proc = subprocess.Popen([sys.executable, "-c", code], stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE, start_new_session=True)
        try:
            time.sleep(0.08)
            self.assertIsNone(proc.poll())
            os.killpg(proc.pid, signal.SIGTERM)
            _, status, usage = os.wait4(proc.pid, 0)
            proc.returncode = os.waitstatus_to_exitcode(status)
            self.assertNotEqual(proc.returncode, 0)
            self.assertGreater(bench._rss_bytes(usage), 0)
        finally:
            proc.stdout.close()
            proc.stderr.close()

    def test_timeout_escalates_and_reaps_child_ignoring_sigterm(self):
        code = "import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(30)"
        result = bench.run_bounded([sys.executable, "-c", code], timeout=0.1)
        self.assertTrue(result["timed_out"])
        self.assertNotEqual(result["exit_code"], 0)
        self.assertGreater(result["peak_rss_bytes"], 0)

    def test_timeout_escalates_while_child_continuously_writes_stderr(self):
        code = "import os,signal; signal.signal(signal.SIGTERM,signal.SIG_IGN); exec('while True: os.write(2,b\\\"x\\\"*8192)')"
        result = bench.run_bounded([sys.executable, "-c", code], timeout=0.1)
        self.assertTrue(result["timed_out"])
        self.assertNotEqual(result["exit_code"], 0)
        self.assertEqual(len(result["stderr"]), bench.MAX_STDERR)
        self.assertLess(result["elapsed_seconds"], 2)

    def test_timeout_kills_descendant_holding_output_pipe(self):
        code = "import subprocess,sys,time; subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); print('parent done'); time.sleep(30)"
        result = bench.run_bounded([sys.executable, "-c", code], timeout=0.2)
        self.assertTrue(result["timed_out"])
        self.assertLess(result["elapsed_seconds"], 2)

    def test_pipe_setup_failure_reaps_spawned_child(self):
        original_popen = subprocess.Popen
        captured = []

        def capture(*args, **kwargs):
            proc = original_popen(*args, **kwargs)
            captured.append(proc)
            return proc

        with mock.patch.object(subprocess, "Popen", side_effect=capture), \
             mock.patch("os.set_blocking", side_effect=RuntimeError("injected setup failure")):
            with self.assertRaisesRegex(RuntimeError, "injected setup failure"):
                bench.run_bounded([sys.executable, "-c", "import time; time.sleep(30)"])
        self.assertEqual(len(captured), 1)
        with self.assertRaises(ChildProcessError):
            os.waitpid(captured[0].pid, os.WNOHANG)

    def test_blocked_probe_reaps_child_when_select_raises(self):
        original_popen = subprocess.Popen
        captured = []

        def capture(*args, **kwargs):
            proc = original_popen(*args, **kwargs)
            captured.append(proc)
            return proc

        with tempfile.TemporaryDirectory() as temporary:
            script = Path(temporary) / "sleeping-cli"
            script.write_text("#!/usr/bin/env python3\nimport time\ntime.sleep(30)\n")
            script.chmod(0o755)
            template = json.loads(Path("crates/careops-ed/examples/one_patient.json").read_text())
            with mock.patch.object(subprocess, "Popen", side_effect=capture), \
                 mock.patch("select.select", side_effect=RuntimeError("injected select failure")):
                with self.assertRaisesRegex(RuntimeError, "injected select failure"):
                    bench._blocked_child_termination_probe(
                        script, Path(temporary), template)
        self.assertEqual(len(captured), 1)
        with self.assertRaises(ChildProcessError):
            os.waitpid(captured[0].pid, os.WNOHANG)

    def test_explicit_external_termination_unblocks_unread_full_pipe(self):
        code = "import sys,time; sys.stdout.buffer.write(b'x' * 10000000); sys.stdout.flush(); time.sleep(30)"
        proc = subprocess.Popen([sys.executable, "-c", code], stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE, start_new_session=True)
        try:
            time.sleep(0.1)  # deliberately do not read stdout; child blocks on pipe capacity
            self.assertIsNone(proc.poll())
            os.killpg(proc.pid, signal.SIGTERM)
            _, status, usage = os.wait4(proc.pid, 0)
            proc.returncode = os.waitstatus_to_exitcode(status)
            self.assertNotEqual(proc.returncode, 0)
            self.assertGreater(bench._rss_bytes(usage), 0)
        finally:
            proc.stdout.close()
            proc.stderr.close()


if __name__ == "__main__":
    unittest.main()
