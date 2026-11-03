"""Trust boundary, misuse case, and resource limit test suite for Task D4.1.

Validates:
1. Input resource limits and malformed rejection (horizon, capacity, durations/speed, schema, path traversal, payload size).
2. Arrow C Data pointer in-process restriction vs bounded IPC stream boundaries and fail-closed deserialization.
3. Agent harness write reservations, path claim validation, and precommit scope integrity.
4. Offline native workflows and zero Python harness socket/HTTP calls; no child-process confinement claim.
5. Output privacy, zero PHI/PII leakage, synthetic ID verification, and atomic write semantics.
6. Absolute unsafe code forbid assertion and 0 unsafe blocks in crate source.
"""

from __future__ import annotations

import copy
import pickle
import json
import os
from pathlib import Path
import re
import socket
import sys
import subprocess
import tempfile
import unittest
from unittest import mock
import urllib.request

from tools import agent_session, ci_scope
from tools.validate_ed_profile_pack_semantics import validate_profile_semantics

ROOT_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT_DIR / 'tests/ed'))
from native_cli_support import NativeCliCase, invoke, read_run, read_dataset, sha
NOMINAL_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/nominal_mvp.json"
MALFORMED_SCENARIO = ROOT_DIR / "crates/careops-ed/examples/malformed_mvp.json"








class D41TrustBoundaryTests(NativeCliCase):
    """Comprehensive test suite for Task D4.1 trust boundaries, misuse cases, and resource limits."""



    def run_cmd(self, *args, check=False):
        result = invoke(*args)
        if check:
            result.check_returncode()
        return result

    def test_input_resource_limits_and_malformed_rejection(self):
        """Test rejection of zero/negative horizon, nonpositive capacity, negative durations/speed,

        malformed JSON, unsupported schema versions, path traversal, and oversized payloads.
        """
        # 1. Zero or negative horizon
        proc_h0 = self.run_cmd("run", str(NOMINAL_SCENARIO), "--horizon", "0")
        self.assertEqual(proc_h0.exit_code if hasattr(proc_h0, 'exit_code') else proc_h0.returncode, 4)
        self.assertIn("horizon must be positive", proc_h0.stdout + proc_h0.stderr)

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

        proc_trav_out = self.run_cmd("run", str(NOMINAL_SCENARIO), "--output-dir", "../../etc/passwd")
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
            self.assertIn("bounded", proc_huge.stdout + proc_huge.stderr)

        # agent_session snapshot budget bound
        with self.assertRaisesRegex(ValueError, "budget"):
            agent_session.snapshot(ROOT_DIR, "test-task", ["tools/agent_session.py"], budget=10)

    def test_arrow_c_data_pointer_in_process_vs_bounded_ipc_boundary(self):
        """Real in-process capsule vs serialized IPC; production reader rejects invalid files.

        Parent populated FFI lifetime/release acceptance is a separate Rust lane.
        This Python test does not fabricate a pointer or claim parent FFI proof.
        """
        import pyarrow as pa
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory); root = directory / 'run'
            _, tables = self.run_ok(root, NOMINAL_SCENARIO, 'run', '--format', 'ipc')
            patients = tables['patients']
            capsule = patients.__arrow_c_stream__()
            with self.assertRaises(TypeError):
                pickle.dumps(capsule)
            manifest_path = root / 'data/manifest.json'
            original_manifest = manifest_path.read_bytes()
            manifest = json.loads(original_manifest)
            part = manifest['tables'][0]['parts'][0]
            path = root / 'data' / part['file']
            original = path.read_bytes()
            stream = pa.ipc.open_stream(pa.BufferReader(original))
            self.assertEqual(stream.schema.metadata[b'careops.schema'], b'careops.ed.patients.v2')
            self.assertEqual(stream.schema.field('arrival_tick_hi').type, pa.uint64())
            self.assertEqual(stream.schema.field('arrival_tick_lo').type, pa.uint64())
            self.assertTrue(stream.read_all().equals(patients, check_metadata=False))
            child = subprocess.run([sys.executable, '-c',
                "import sys,json,pyarrow as pa; raw=sys.stdin.buffer.read(4*1024*1024+1); "
                "assert len(raw)<=4*1024*1024; r=pa.ipc.open_stream(pa.BufferReader(raw)); "
                "assert r.schema.metadata[b'careops.schema']==b'careops.ed.patients.v2'; "
                "t=r.read_all(); assert t.schema.field('arrival_tick_lo').type==pa.uint64(); "
                "print(json.dumps({'rows':t.num_rows,'columns':t.column_names}))"],
                input=original, capture_output=True, timeout=30)
            self.assertEqual(child.returncode, 0, child.stderr.decode())
            self.assertEqual(json.loads(child.stdout)['rows'], 5)
            self.assertEqual(json.loads(child.stdout)['columns'], patients.column_names)

            def encoded(table):
                sink = pa.BufferOutputStream()
                with pa.ipc.new_stream(sink, table.schema) as writer:
                    writer.write_table(table)
                return sink.getvalue().to_pybytes()

            wrong_metadata = patients.replace_schema_metadata({**patients.schema.metadata,
                                                              b'careops.schema': b'unknown.v99'})
            wrong_type = patients.set_column(patients.schema.get_field_index('arrival_tick_lo'),
                pa.field('arrival_tick_lo', pa.float64(), nullable=False),
                patients['arrival_tick_lo'].cast(pa.float64()))
            oversized = original + b'0' * (4*1024*1024 + 1 - len(original))
            self.assertEqual(pa.ipc.open_stream(pa.BufferReader(oversized)).read_all().num_rows, 5)
            negatives = {
                'truncated': original[:len(original)//2],
                'invalid_framing': b'not an Arrow stream',
                'unsupported_schema': encoded(wrong_metadata),
                'wrong_physical_type': encoded(wrong_type),
                'oversized': oversized,
            }
            for name, raw in negatives.items():
                try:
                    changed = json.loads(original_manifest)
                    path.write_bytes(raw)
                    changed_part = changed['tables'][0]['parts'][0]
                    changed_part.update(bytes=len(raw), sha256=sha(path))
                    manifest_path.write_text(json.dumps(changed))
                    destination = directory / name
                    result = invoke('export', root / 'data', '--output-dir', destination)
                    self.assertEqual(result.returncode, 4, (name, result.stderr))
                    self.assertFalse(destination.exists())
                finally:
                    path.write_bytes(original)
                    manifest_path.write_bytes(original_manifest)
            read_dataset(root / 'data')

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
        """Offline native workflow and no Python harness network-client calls.

        Monkeypatching this Python process does not monitor child native sockets;
        this test makes no operating-system-wide network confinement claim.
        """
        attempted = []
        def blocked_socket(*args, **kwargs):
            attempted.append((args, kwargs))
            raise AssertionError('Python harness network call during offline workflow')
        with mock.patch('socket.socket', side_effect=blocked_socket), \
             mock.patch('socket.create_connection', side_effect=blocked_socket), \
             mock.patch('urllib.request.urlopen', side_effect=blocked_socket):
            with tempfile.TemporaryDirectory() as directory:
                directory = Path(directory)
                validation = self.run_cmd('validate', NOMINAL_SCENARIO, check=True)
                self.assertEqual(json.loads(validation.stdout)['scenario_id'], 'nominal-ed-mvp')
                self.run_ok(directory / 'run')
                self.run_ok(directory / 'batch', NOMINAL_SCENARIO, 'batch', '--replications', 2, '--workers', 1)
                result = self.run_cmd('export', directory / 'run/data', '--output-dir', directory / 'ipc', '--format', 'ipc', check=True)
                self.assertEqual(json.loads(result.stdout)['status'], 'complete')
                read_dataset(directory / 'ipc')
                profile = json.loads((ROOT_DIR / 'model-inputs/ed/profiles/p4-minimal-pack.json').read_text())
                self.assertEqual(validate_profile_semantics(profile), [])
        self.assertEqual(attempted, [])

    def test_output_privacy_and_phi_leakage_guards(self):
        """Scan actual native string values/control files and reject incomplete output."""
        import pyarrow as pa
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            _, run = self.run_ok(directory / 'run')
            _, batch = self.run_ok(directory / 'batch', NOMINAL_SCENARIO, 'batch', '--replications', 2, '--workers', 1)
            outputs = [p.read_text() for p in directory.rglob('*.json')]
            for tables in [run, batch]:
                for table in tables.values():
                    for field in table.schema:
                        if pa.types.is_string(field.type) or pa.types.is_large_string(field.type) or pa.types.is_string_view(field.type):
                            outputs.extend(value for value in table[field.name].to_pylist() if value is not None)
                for identity in tables['patients']['patient_id'].to_pylist():
                    self.assertTrue(identity.startswith(('patient-', 'patient_', 'P')), identity)
            patterns = {
                'SSN': r'\b\d{3}-\d{2}-\d{4}\b',
                'Medicare card': r'\b[2-6]\d{9}\b',
                'Private Key': r'-----BEGIN (?:RSA )?PRIVATE KEY-----',
                'Bearer Token': r'\bBearer\s+[A-Za-z0-9_\-\.]{20,}\b',
                'API Secret Token': r'\b(?:AKIA|ghp_|glpat-)[A-Za-z0-9_]{16,}\b',
            }
            for name, pattern in patterns.items():
                for output in outputs:
                    self.assertEqual(re.findall(pattern, output), [], name)
            self.assertEqual(list(directory.rglob('*.tmp.*')), [])
            dataset = directory / 'run/data'; manifest = dataset / 'manifest.json'
            original = manifest.read_bytes(); incomplete = json.loads(original); incomplete['status'] = 'incomplete'
            manifest.write_text(json.dumps(incomplete))
            try:
                result = self.run_cmd('export', dataset, '--output-dir', directory / 'partial-export')
                self.assertEqual(result.returncode, 4, result.stderr)
                self.assertFalse((directory / 'partial-export').exists())
            finally:
                manifest.write_bytes(original)
            read_dataset(dataset)

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
