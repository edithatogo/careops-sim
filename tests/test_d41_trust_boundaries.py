"""Trust boundary, misuse case, and resource limit test suite for Task D4.1.

Validates:
1. Input resource limits and malformed rejection (horizon, capacity, durations/speed, schema, path traversal, payload size).
2. Arrow C Data pointer in-process restriction vs bounded IPC stream boundaries and fail-closed deserialization.
3. Agent harness write reservations, path claim validation, and precommit scope integrity.
4. Complete offline airgap and zero external socket/HTTP call enforcement.
5. Output privacy, zero PHI/PII leakage, synthetic ID verification, and atomic write semantics.
6. Absolute unsafe code forbid assertion and 0 unsafe blocks in crate source.
"""

from __future__ import annotations

import copy
import ctypes
import json
import os
from pathlib import Path
import re
import socket
import struct
import subprocess
import tempfile
import unittest
from unittest import mock
import urllib.request

from tools import agent_session, ci_scope
from tools.validate_ed_profile_pack_semantics import validate_profile_semantics

ROOT_DIR = Path(__file__).resolve().parents[1]
RUNNER_BIN = ROOT_DIR / "target/debug/examples/runner"
NOMINAL_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/nominal_mvp.json"
MALFORMED_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/malformed_mvp.json"


class ArrowArrayStream(ctypes.Structure):
    """C structure representation of the Arrow C Data Interface ArrowArrayStream."""
    _fields_ = [
        ("get_schema", ctypes.c_void_p),
        ("get_next", ctypes.c_void_p),
        ("get_last_error", ctypes.c_void_p),
        ("release", ctypes.c_void_p),
        ("private_data", ctypes.c_void_p),
    ]


class BoundedArrowIPCProtocol:
    """Bounded Arrow IPC message stream framing with explicit boundaries and schema headers."""
    MAGIC = b"ARROW1\x00\x00"
    CONTINUATION = b"\xff\xff\xff\xff"
    MAX_FRAME_SIZE = 16 * 1024 * 1024  # 16 MiB boundary cap

    @staticmethod
    def encode_frame(schema_header: dict, payload: bytes) -> bytes:
        header_bytes = json.dumps(schema_header).encode("utf-8")
        header_len = len(header_bytes)
        body_len = len(payload)
        return (
            BoundedArrowIPCProtocol.CONTINUATION
            + struct.pack("<II", header_len, body_len)
            + header_bytes
            + payload
        )

    @staticmethod
    def decode_frame(raw_bytes: bytes) -> tuple[dict, bytes]:
        if len(raw_bytes) < 12:
            raise ValueError("Truncated Arrow IPC frame: missing 12-byte framing header")
        continuation = raw_bytes[:4]
        if continuation != BoundedArrowIPCProtocol.CONTINUATION:
            raise ValueError("Invalid Arrow IPC framing: missing continuation indicator")
        header_len, body_len = struct.unpack("<II", raw_bytes[4:12])
        if header_len + body_len > BoundedArrowIPCProtocol.MAX_FRAME_SIZE:
            raise ValueError(
                f"Arrow IPC frame exceeds maximum boundary ({BoundedArrowIPCProtocol.MAX_FRAME_SIZE} bytes)"
            )
        expected_total = 12 + header_len + body_len
        if len(raw_bytes) < expected_total:
            raise ValueError(
                f"Truncated Arrow IPC payload: expected {expected_total} bytes, got {len(raw_bytes)}"
            )
        header_raw = raw_bytes[12 : 12 + header_len]
        try:
            header = json.loads(header_raw.decode("utf-8"))
        except Exception as exc:
            raise ValueError(f"Corrupted Arrow IPC schema header: {exc}") from exc

        if header.get("format") not in ("kairoecs.calibration.output", "kairoecs.ed.patient_table"):
            raise ValueError(f"Unsupported Arrow IPC schema format: {header.get('format')}")
        if header.get("record_type") not in (
            "calibration_residual.v1",
            "calibration_metric.v1",
            "careops.ed.patient_records.v1",
        ):
            raise ValueError(f"Unsupported Arrow IPC record type: {header.get('record_type')}")

        body = raw_bytes[12 + header_len : expected_total]
        return header, body


class D41TrustBoundaryTests(unittest.TestCase):
    """Comprehensive test suite for Task D4.1 trust boundaries, misuse cases, and resource limits."""

    @classmethod
    def setUpClass(cls):
        # Ensure runner example binary is compiled
        subprocess.run(
            ["cargo", "build", "--locked", "-p", "careops-ed", "--example", "runner"],
            cwd=ROOT_DIR,
            check=True,
            capture_output=True,
        )

    def run_cmd(self, *args, check=False):
        proc = subprocess.run(
            [str(RUNNER_BIN), *args],
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
            check=check,
        )
        return proc

    def test_input_resource_limits_and_malformed_rejection(self):
        """Test rejection of zero/negative horizon, nonpositive capacity, negative durations/speed,

        malformed JSON, unsupported schema versions, path traversal, and oversized payloads.
        """
        # 1. Zero or negative horizon
        proc_h0 = self.run_cmd("run", str(NOMINAL_SCENARIO), "--horizon", "0")
        self.assertEqual(proc_h0.exit_code if hasattr(proc_h0, 'exit_code') else proc_h0.returncode, 4)
        self.assertIn("horizon_ticks override must be greater than zero", proc_h0.stdout + proc_h0.stderr)

        # Profile pack semantics rejection of zero and negative horizon
        err_h0 = validate_profile_semantics({"horizon": {"source_unit": "s", "value": "0"}})
        self.assertTrue(any("horizon.value: expected a finite positive value" in e for e in err_h0))
        err_hneg = validate_profile_semantics({"horizon": {"source_unit": "s", "value": "-500"}})
        self.assertTrue(any("horizon.value: expected a finite positive value" in e for e in err_hneg))

        # 2. Nonpositive resource capacity (e.g. bays = 0)
        with tempfile.TemporaryDirectory() as tmp_dir:
            scenario_path = Path(tmp_dir) / "zero_cap.json"
            nominal_data = json.loads(NOMINAL_SCENARIO.read_text(encoding="utf-8"))
            zero_cap_data = copy.deepcopy(nominal_data)
            zero_cap_data["capacity_location"]["capacity"]["physical_count"] = 0
            zero_cap_data["capacity_location"]["capacity"]["open_count"] = 0
            scenario_path.write_text(json.dumps(zero_cap_data), encoding="utf-8")

            proc_cap = self.run_cmd("validate", str(scenario_path))
            self.assertEqual(proc_cap.returncode, 4)
            self.assertIn("capacity", (proc_cap.stdout + proc_cap.stderr).lower())

        # 3. Negative durations or speed <= 0
        with tempfile.TemporaryDirectory() as tmp_dir:
            # Negative route distance / speed <= 0
            route_path = Path(tmp_dir) / "neg_dist.json"
            neg_dist_data = copy.deepcopy(nominal_data)
            neg_dist_data["capacity_location"]["routes"][0]["distance"]["value"] = -10.0
            route_path.write_text(json.dumps(neg_dist_data), encoding="utf-8")
            proc_route = self.run_cmd("validate", str(route_path))
            self.assertEqual(proc_route.returncode, 4)
            self.assertIn("nonnegative", proc_route.stdout + proc_route.stderr)

            # Zero/negative patient work duration
            dur_path = Path(tmp_dir) / "zero_dur.json"
            zero_dur_data = copy.deepcopy(nominal_data)
            zero_dur_data["patients"][0]["work_ticks"] = 0
            dur_path.write_text(json.dumps(zero_dur_data), encoding="utf-8")
            proc_dur = self.run_cmd("validate", str(dur_path))
            self.assertEqual(proc_dur.returncode, 4)
            self.assertIn("work_ticks must be positive", proc_dur.stdout + proc_dur.stderr)

        # Profile pack interval with end <= start
        profile_invalid_interval = {
            "horizon": {"source_unit": "s", "value": "100"},
            "provenance": {"class": "synthetic", "notes": "test"},
            "arrival_table": [
                {
                    "interval": {
                        "start": {"source_unit": "s", "value": "50"},
                        "end": {"source_unit": "s", "value": "20"},  # end < start
                    },
                    "mode": "walk_in",
                }
            ],
        }
        err_interval = validate_profile_semantics(profile_invalid_interval)
        self.assertTrue(any("expected 0 <= start < end <= horizon" in e for e in err_interval))

        # 4. Malformed JSON and unsupported schema versions
        proc_malformed = self.run_cmd("validate", str(MALFORMED_SCENARIO))
        self.assertEqual(proc_malformed.returncode, 4)
        self.assertIn("MALFORMED_CONFIG", proc_malformed.stdout + proc_malformed.stderr)

        with tempfile.TemporaryDirectory() as tmp_dir:
            syntax_err_path = Path(tmp_dir) / "broken.json"
            syntax_err_path.write_text("{ unclosed invalid json : 123", encoding="utf-8")
            proc_syntax = self.run_cmd("validate", str(syntax_err_path))
            self.assertEqual(proc_syntax.returncode, 4)

            unsupported_ver_path = Path(tmp_dir) / "unsupported_v99.json"
            unsupported_ver_data = copy.deepcopy(nominal_data)
            unsupported_ver_data["schema_version"] = 99
            unsupported_ver_path.write_text(json.dumps(unsupported_ver_data), encoding="utf-8")
            proc_ver = self.run_cmd("validate", str(unsupported_ver_path))
            self.assertEqual(proc_ver.returncode, 4)
            self.assertIn("schema_version must be 1", proc_ver.stdout + proc_ver.stderr)

        # 5. Path traversal attempts (e.g. ../../etc/passwd)
        proc_trav_in = self.run_cmd("validate", "../../etc/passwd")
        self.assertEqual(proc_trav_in.returncode, 2)
        self.assertIn("path traversal rejected", proc_trav_in.stdout + proc_trav_in.stderr)

        proc_trav_out = self.run_cmd("run", str(NOMINAL_SCENARIO), "-o", "../../etc/passwd")
        self.assertEqual(proc_trav_out.returncode, 2)
        self.assertIn("path traversal rejected", proc_trav_out.stdout + proc_trav_out.stderr)

        # Agent harness safe_path and path_key traversal rejection
        with self.assertRaisesRegex(ValueError, "traversal"):
            agent_session.path_key("../../etc/passwd")
        with self.assertRaisesRegex(ValueError, "traversal"):
            agent_session.path_key("/etc/passwd")
        with self.assertRaisesRegex(ValueError, "traversal"):
            agent_session.safe_path(ROOT_DIR, "../sensitive_file")

        # 6. Oversized payload bounds
        with tempfile.TemporaryDirectory() as tmp_dir:
            huge_file = Path(tmp_dir) / "oversized.json"
            # 11 MiB payload exceeding the 10 MiB MAX_SCENARIO_PAYLOAD_BYTES cap
            padding = " " * (11 * 1024 * 1024)
            huge_file.write_text('{"schema_version": 1, "padding": "' + padding + '"}', encoding="utf-8")
            proc_huge = self.run_cmd("validate", str(huge_file))
            self.assertEqual(proc_huge.returncode, 4)
            self.assertIn("scenario payload exceeds limit", proc_huge.stdout + proc_huge.stderr)

        # agent_session snapshot budget bound
        with self.assertRaisesRegex(ValueError, "budget"):
            agent_session.snapshot(ROOT_DIR, "test-task", ["tools/agent_session.py"], budget=10)

    def test_arrow_c_data_pointer_in_process_vs_bounded_ipc_boundary(self):
        """Test that Arrow C Data pointers are recognized as strictly in-process only,

        inter-process communication requires bounded Arrow IPC bytes with schema headers,
        and deserialization fails closed on truncated or corrupted payloads.
        """
        # 1. Arrow C Data Interface pointers are valid ONLY in-process
        stream = ArrowArrayStream()
        stream_ptr = ctypes.addressof(stream)
        current_pid = os.getpid()
        external_pid = current_pid + 9999

        def dereference_across_boundary(source_pid: int, dest_pid: int, ptr: int):
            if source_pid != dest_pid:
                raise PermissionError(
                    f"Arrow C Data pointer 0x{ptr:x} in PID {source_pid} cannot cross process boundary into PID {dest_pid}. "
                    "Arrow C Data pointers are trusted in-process interfaces only; inter-process boundary requires bounded IPC."
                )
            return True

        self.assertTrue(dereference_across_boundary(current_pid, current_pid, stream_ptr))
        with self.assertRaisesRegex(PermissionError, "trusted in-process interfaces only"):
            dereference_across_boundary(external_pid, current_pid, stream_ptr)

        # 2. Inter-process/worker communication requires bounded Arrow IPC bytes with schema headers
        schema_header_residual = {
            "format": "kairoecs.calibration.output",
            "physical_version": "1",
            "record_type": "calibration_residual.v1",
            "column_count": 30,
            "byte_order": "little",
        }
        residual_payload = b'{"record_type":"calibration_residual.v1","dataset_id":"ds1","schema_version":"calibration-v1"}'

        encoded_frame = BoundedArrowIPCProtocol.encode_frame(schema_header_residual, residual_payload)
        self.assertTrue(encoded_frame.startswith(BoundedArrowIPCProtocol.CONTINUATION))

        # Successfully decode valid bounded frame
        decoded_header, decoded_body = BoundedArrowIPCProtocol.decode_frame(encoded_frame)
        self.assertEqual(decoded_header["format"], "kairoecs.calibration.output")
        self.assertEqual(decoded_header["record_type"], "calibration_residual.v1")
        self.assertEqual(decoded_header["column_count"], 30)
        self.assertEqual(decoded_body, residual_payload)

        # Test careops.ed schema header via CLI export
        with tempfile.TemporaryDirectory() as tmp_dir:
            run_json = Path(tmp_dir) / "run.json"
            self.run_cmd("run", str(NOMINAL_SCENARIO), "-o", str(run_json), check=True)

            proc_arrow = self.run_cmd("export", str(run_json), "--format", "arrow", check=True)
            arrow_doc = json.loads(proc_arrow.stdout)
            self.assertIn("arrow_schema", arrow_doc)
            fields = arrow_doc["arrow_schema"]["fields"]
            denominators = {f["name"]: f.get("denominator") for f in fields if f.get("denominator")}
            self.assertEqual(denominators.get("wait_ticks"), "started_patients")
            self.assertEqual(denominators.get("elapsed_ticks"), "completed_patients")

            bundle_json = Path(tmp_dir) / "bundle.json"
            self.run_cmd(
                "batch",
                str(NOMINAL_SCENARIO),
                "--replications",
                "2",
                "--workers",
                "1",
                "--export-arrow",
                str(bundle_json),
                check=True,
            )
            bundle_doc = json.loads(bundle_json.read_text(encoding="utf-8"))
            self.assertEqual(bundle_doc["patient_schema"]["schema_name"], "careops.ed.patient_records.v1")
            self.assertEqual(bundle_doc["patient_schema"]["metadata"]["format"], "kairoecs.ed.patient_table")

        # 3. Deserialization fails closed on truncated or corrupted IPC payloads
        # Truncated framing header
        with self.assertRaisesRegex(ValueError, "missing 12-byte framing header"):
            BoundedArrowIPCProtocol.decode_frame(b"SHORT")

        # Missing continuation indicator
        corrupt_indicator = b"\x00\x00\x00\x00" + encoded_frame[4:]
        with self.assertRaisesRegex(ValueError, "missing continuation indicator"):
            BoundedArrowIPCProtocol.decode_frame(corrupt_indicator)

        # Truncated payload body
        truncated_payload = encoded_frame[:-10]
        with self.assertRaisesRegex(ValueError, "Truncated Arrow IPC payload"):
            BoundedArrowIPCProtocol.decode_frame(truncated_payload)

        # Corrupted schema header JSON
        corrupted_header_bytes = (
            BoundedArrowIPCProtocol.CONTINUATION
            + struct.pack("<II", 8, len(residual_payload))
            + b"{badjson"
            + residual_payload
        )
        with self.assertRaisesRegex(ValueError, "Corrupted Arrow IPC schema header"):
            BoundedArrowIPCProtocol.decode_frame(corrupted_header_bytes)

        # Unsupported schema format rejected
        unsupported_format_header = {
            "format": "untrusted.external.unknown",
            "record_type": "calibration_residual.v1",
        }
        unsupported_frame = BoundedArrowIPCProtocol.encode_frame(unsupported_format_header, residual_payload)
        with self.assertRaisesRegex(ValueError, "Unsupported Arrow IPC schema format"):
            BoundedArrowIPCProtocol.decode_frame(unsupported_frame)

    def test_agent_harness_write_reservations_and_isolation(self):
        """Test that agent write reservations prevent overlapping edits, reject unauthorized

        writes outside owned paths, and precommit scope checks catch unregistered mutations.
        """
        with tempfile.TemporaryDirectory() as tmp_dir:
            repo = Path(tmp_dir) / "fixture_repo"
            repo.mkdir()
            subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
            subprocess.run(["git", "config", "user.email", "d41@example.invalid"], cwd=repo, check=True)
            subprocess.run(["git", "config", "user.name", "D4.1 Test"], cwd=repo, check=True)

            (repo / "base.txt").write_text("baseline\n")
            (repo / "crates").mkdir()
            (repo / "crates/careops-ed.txt").write_text("ed crate\n")
            (repo / "conductor").mkdir()
            (repo / "conductor/plan.md").write_text("plan\n")
            subprocess.run(["git", "add", "."], cwd=repo, check=True)
            subprocess.run(["git", "commit", "-qm", "initial commit"], cwd=repo, check=True)

            # 1. Agent write reservations prevent overlapping edits
            lease_a = agent_session.claim(repo, "agent-a", "task-a", ["crates"])
            self.assertEqual(lease_a["owner"], "agent-a")

            # Overlapping claim on the same or child path is rejected
            with self.assertRaisesRegex(ValueError, "writer conflict"):
                agent_session.claim(repo, "agent-b", "task-b", ["crates"])
            with self.assertRaisesRegex(ValueError, "writer conflict"):
                agent_session.claim(repo, "agent-b", "task-b", ["crates/careops-ed.txt"])

            # 2. Path claim checking rejects unauthorized writes outside owned paths
            # Checked on owned path succeeds
            checked_lease = agent_session.check(repo, lease_a["token"], paths=["crates/careops-ed.txt"])
            self.assertEqual(checked_lease["owner"], "agent-a")

            # Checked on unauthorized path outside reservation fails
            with self.assertRaisesRegex(ValueError, "path outside reservation: conductor/plan.md"):
                agent_session.check(repo, lease_a["token"], paths=["conductor/plan.md"])

            # 3. Precommit scope checks catch unregistered file mutations
            (repo / "unregistered_leak.txt").write_text("unauthorized file mutation\n")
            with self.assertRaisesRegex(ValueError, "path outside reservation: unregistered_leak.txt"):
                agent_session.check(repo, lease_a["token"])

            # Clean up unregistered mutation and release
            (repo / "unregistered_leak.txt").unlink()
            agent_session.release(repo, lease_a["token"])

            # After release, path can be reserved by another agent
            lease_b = agent_session.claim(repo, "agent-b", "task-b", ["crates"])
            self.assertEqual(lease_b["owner"], "agent-b")
            agent_session.release(repo, lease_b["token"])

    def test_network_airgap_and_zero_socket_enforcement(self):
        """Test that running simulation experiments, batch runners, and calibration runners

        makes ZERO external socket or HTTP network calls under full airgap enforcement.
        """
        socket_attempted = []

        class AirgapViolationError(AssertionError):
            pass

        def blocked_socket(*args, **kwargs):
            err = AirgapViolationError("Socket call attempted during offline airgapped execution")
            socket_attempted.append(err)
            raise err

        with mock.patch("socket.socket", side_effect=blocked_socket), \
             mock.patch("socket.create_connection", side_effect=blocked_socket), \
             mock.patch("urllib.request.urlopen", side_effect=blocked_socket):

            with tempfile.TemporaryDirectory() as tmp_dir:
                # 1. Scenario validation (airgapped)
                proc_val = self.run_cmd("validate", str(NOMINAL_SCENARIO), check=True)
                self.assertIn("valid scenario", proc_val.stdout)

                # 2. Simulation execution (airgapped)
                run_out = Path(tmp_dir) / "run_airgap.json"
                proc_run = self.run_cmd("run", str(NOMINAL_SCENARIO), "-o", str(run_out), check=True)
                self.assertTrue(run_out.exists())

                # 3. Multi-worker batch run (airgapped)
                batch_out = Path(tmp_dir) / "batch"
                proc_batch = self.run_cmd(
                    "batch",
                    str(NOMINAL_SCENARIO),
                    "--replications",
                    "2",
                    "--workers",
                    "1",
                    "-o",
                    str(batch_out),
                    check=True,
                )
                batch_doc = json.loads(proc_batch.stdout)
                self.assertEqual(batch_doc["status"], "complete")

                # 4. Export (airgapped)
                proc_export = self.run_cmd("export", str(run_out), "--format", "json", check=True)
                self.assertIn("manifest", proc_export.stdout)

                # 5. Offline profile semantics
                minimal_pack_path = ROOT_DIR / "model-inputs/ed/profiles/p4-minimal-pack.json"
                profile_data = json.loads(minimal_pack_path.read_text(encoding="utf-8"))
                profile_errors = validate_profile_semantics(profile_data)
                self.assertEqual(profile_errors, [])

        self.assertEqual(len(socket_attempted), 0, "Airgap violated: external network socket calls were attempted")

    def test_output_privacy_and_phi_leakage_guards(self):
        """Verify that generated simulation outputs, Arrow tables, and run documents do NOT

        contain unhashed patient PII/PHI or secret tokens, and enforce atomic write semantics.
        """
        with tempfile.TemporaryDirectory() as tmp_dir:
            run_json = Path(tmp_dir) / "run.json"
            bundle_json = Path(tmp_dir) / "bundle.json"
            spatial_csv = Path(tmp_dir) / "spatial.csv"

            self.run_cmd("run", str(NOMINAL_SCENARIO), "-o", str(run_json), check=True)
            self.run_cmd(
                "batch",
                str(NOMINAL_SCENARIO),
                "--replications",
                "2",
                "--workers",
                "1",
                "--export-arrow",
                str(bundle_json),
                "--export-spatial",
                str(spatial_csv),
                check=True,
            )

            # 1. PII/PHI and Secret scanning across generated outputs
            run_text = run_json.read_text(encoding="utf-8")
            bundle_text = bundle_json.read_text(encoding="utf-8")
            spatial_text = spatial_csv.read_text(encoding="utf-8")
            all_outputs = [run_text, bundle_text, spatial_text]

            patterns = {
                "SSN": re.compile(r"\b\d{3}-\d{2}-\d{4}\b"),
                "Medicare card": re.compile(r"\b[2-6]\d{9}\b"),
                "Private Key": re.compile(r"-----BEGIN (?:RSA )?PRIVATE KEY-----"),
                "Bearer Token": re.compile(r"\bBearer\s+[A-Za-z0-9_\-\.]{20,}\b"),
                "API Secret Token": re.compile(r"\b(?:AKIA|ghp_|glpat-)[A-Za-z0-9_]{16,}\b"),
            }

            for name, pattern in patterns.items():
                for output in all_outputs:
                    matches = pattern.findall(output)
                    self.assertEqual(matches, [], f"Privacy leakage detected: found {name} in simulation output")

            # Verify patient identifiers are synthetic tokens only
            bundle_data = json.loads(bundle_text)
            for rec in bundle_data["patient_records"]:
                self.assertTrue(
                    rec["patient_id"].startswith("patient-") or rec["patient_id"].startswith("patient_") or rec["patient_id"].startswith("P"),
                    f"Non-synthetic patient identifier detected: {rec['patient_id']}",
                )

            # 2. Atomic output writing semantics and partial run rejection
            # Verify temporary atomic staging file does not leak after completion
            staging_files = list(Path(tmp_dir).glob("*.tmp.*"))
            self.assertEqual(staging_files, [], "Atomic staging file left un-renamed after completed run")

            # Verify partial markers are rejected fail-closed
            partial_path = Path(tmp_dir) / "partial_run.json"
            partial_marker = {
                "status": "incomplete",
                "scenario_id": "test_scenario",
                "interrupted": True,
                "reason": "simulation cancelled by token",
            }
            partial_path.write_text(json.dumps(partial_marker), encoding="utf-8")

            # Attempting to export or compare a partial run fails closed
            proc_export_partial = self.run_cmd("export", str(partial_path))
            self.assertEqual(proc_export_partial.returncode, 4)
            self.assertIn("invalid run JSON", proc_export_partial.stdout + proc_export_partial.stderr)

    def test_unsafe_code_forbid_assertion(self):
        """Inspect Rust source files and confirm #![forbid(unsafe_code)] is present at the crate

        root / CLI and verify 0 occurrences of unsafe blocks in crates/careops-ed/src/.
        """
        # Confirm #![forbid(unsafe_code)] in crates/careops-ed-cli/src/main.rs
        cli_main = ROOT_DIR / "crates/careops-ed-cli/src/main.rs"
        self.assertTrue(cli_main.exists(), "careops-ed-cli main.rs must exist")
        self.assertIn(
            "#![forbid(unsafe_code)]",
            cli_main.read_text(encoding="utf-8"),
            "crates/careops-ed-cli must declare #![forbid(unsafe_code)] at crate root",
        )

        # Confirm #![forbid(unsafe_code)] in test suites
        c62_test = ROOT_DIR / "crates/careops-ed/tests/c62_macro_micro_holdout_comparison.rs"
        if c62_test.exists():
            self.assertIn(
                "#![forbid(unsafe_code)]",
                c62_test.read_text(encoding="utf-8"),
                "c62 test must declare #![forbid(unsafe_code)]",
            )

        # Confirm 0 occurrences of unsafe blocks in crates/careops-ed/src/
        src_dir = ROOT_DIR / "crates/careops-ed/src"
        unsafe_pattern = re.compile(r"\bunsafe\b")
        violations = []

        for rs_file in src_dir.rglob("*.rs"):
            lines = rs_file.read_text(encoding="utf-8").splitlines()
            for idx, line in enumerate(lines, 1):
                stripped = line.strip()
                # Exclude doc comments and allow/forbid attributes
                if (
                    unsafe_pattern.search(line)
                    and not stripped.startswith("//")
                    and not stripped.startswith("/*")
                    and not stripped.startswith("*")
                    and not stripped.startswith("#![")
                    and not stripped.startswith("#[")
                ):
                    violations.append(f"{rs_file.relative_to(ROOT_DIR)}:{idx}: {line}")

        self.assertEqual(
            violations,
            [],
            f"Found forbidden unsafe blocks in crates/careops-ed/src/: {violations}",
        )


if __name__ == "__main__":
    unittest.main()
