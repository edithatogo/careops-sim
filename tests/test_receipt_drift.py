"""Acceptance and dispatch drift tests using synthetic local Git repositories."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
spec = importlib.util.spec_from_file_location("receipt_drift", ROOT / "tools/receipt_drift.py")
drift = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(drift)
task_spec = importlib.util.spec_from_file_location("tasks", ROOT / "tools/tasks.py")
tasks = importlib.util.module_from_spec(task_spec)
assert task_spec.loader is not None
task_spec.loader.exec_module(tasks)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class DriftVerifierTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self._git("init", "-q")
        self._git("config", "user.email", "fixture@example.invalid")
        self._git("config", "user.name", "Fixture")
        self._write("contract.md", b"contract")
        self._write("source.txt", b"source")
        self._write("prereq.txt", b"accepted evidence")
        self._write("protected/guard.txt", b"protected")
        self._git("add", ".")
        self._git("commit", "-qm", "fixture base")
        self.base = self._git("rev-parse", "HEAD").decode().strip()
        self.packet = {
            "status": "prepared", "packet_id": "D1.5.drift.test", "task_id": "D1.5",
            "target_repo": ".", "base_commit": self.base,
            "objective": "test", "context_paths": ["contract.md"],
            "interface_contract": "test", "steps": ["test"], "acceptance": ["test"],
            "stop_conditions": ["test"],
            "input_hashes": {"contract.md": sha(b"contract")},
            "source_hashes": {"source.txt": sha(b"source")},
            "verification": [
                {"argv": ["python3", "-m", "unittest", "-v"], "cwd": ".", "expected_exit": 0, "oracle": "pass"},
                {"argv": ["python3", "-m", "unittest", "all"], "cwd": ".", "expected_exit": 0, "oracle": "pass"},
                {"argv": ["git", "diff", "--check"], "cwd": ".", "expected_exit": 0, "oracle": "pass"},
            ],
            "write_paths": ["out/code.py", "out/test.py", "control/result.json", "control/receipt.json"],
            "protected_paths": [".git", "protected"],
            "result_path": "control/result.json", "leaf_dependencies": ["D1.5.receipts"],
            "coordinator_binding": {
                "receipt_path": "control/receipt.json", "required_outputs": ["out/code.py", "out/test.py"],
                "prerequisite_receipts": {"D1.5.receipts": {
                    "status": "accepted", "reviewer": "Reviewer", "all_instances_accepted": True,
                    "expected_instances": ["schema-validator"], "accepted_instances": ["schema-validator"],
                    "artifacts": {"prereq.txt": sha(b"accepted evidence")},
                }},
            },
        }
        self._write("out/code.py", b"code")
        self._write("out/test.py", b"test")
        self.receipt = self._receipt()

    def _git(self, *args):
        return subprocess.run(["git", "-C", str(self.root), *args], check=True, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE).stdout

    def _write(self, path, data):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)

    def _receipt(self):
        commands = []
        for index, check in enumerate(self.packet["verification"]):
            stamp = f"2026-09-28T00:00:0{index}Z"
            commands.append({
                "argv": copy.deepcopy(check["argv"]), "cwd": check["cwd"],
                "expected_exit": check["expected_exit"], "started_at_utc": stamp, "finished_at_utc": stamp,
                "exit_code": check["expected_exit"], "stdout_sha256": sha(b""), "stdout_bytes": 0,
                "stderr_sha256": sha(b""), "stderr_bytes": 0, "evidence_source": "worker_reported",
                "tool_versions": {"python": "3"},
            })
        return {
            "schema_version": 1, "packet_id": self.packet["packet_id"], "attempt_id": "attempt-1",
            "target_repo": ".", "base_commit": self.base,
            "worker": {"agent_id": "fixture", "model": "gpt-6-luna", "model_version": None,
                       "reasoning_effort": "low"},
            "started_at_utc": "2026-09-28T00:00:00Z", "finished_at_utc": "2026-09-28T00:00:03Z",
            "inputs_sha256": copy.deepcopy(self.packet["input_hashes"]), "commands": commands,
            "outputs_sha256": {"out/code.py": sha(b"code"), "out/test.py": sha(b"test")},
            "changed_paths": ["out/code.py", "out/test.py"], "status": "ready_for_review",
            "unresolved_issues": [],
        }

    def errors(self, receipt=None, packet=None, path="control/receipt.json"):
        return drift.verify_acceptance(receipt if receipt is not None else self.receipt, path,
                                       packet if packet is not None else self.packet, self.root)

    def test_valid_precommit_acceptance(self):
        self.assertEqual(self.errors(), [])

    def test_live_head_drift_rejected(self):
        self._write("new.txt", b"change")
        self._git("add", "new.txt")
        self._git("commit", "-qm", "advance head")
        self.assertIn("Current Git base has drifted", self.errors())

    def test_packet_input_drift_rejected(self):
        self._write("contract.md", b"changed")
        self.assertIn("Packet input hash drift detected", self.errors())

    def test_source_drift_rejected(self):
        self._write("source.txt", b"changed")
        self.assertIn("Packet source hash drift detected", self.errors())

    def test_dispatch_rejects_stale_base_input_and_source_hashes(self):
        self.assertEqual(tasks.packet_errors(self.root, self.packet), [])
        stale_base = copy.deepcopy(self.packet)
        stale_base["base_commit"] = "a" * 40
        self.assertTrue(any("Base commit drift" in item for item in tasks.packet_errors(self.root, stale_base)))
        self._write("contract.md", b"changed")
        self.assertTrue(any("Input hash drift" in item for item in tasks.packet_errors(self.root, self.packet)))
        self._write("contract.md", b"contract")
        self._write("source.txt", b"changed")
        self.assertTrue(any("source hash drift" in item for item in tasks.packet_errors(self.root, self.packet)))

    def test_receipt_identity_and_input_map_mismatch_rejected(self):
        bad = copy.deepcopy(self.receipt)
        bad["packet_id"] = "other"
        self.assertTrue(self.errors(bad))
        bad = copy.deepcopy(self.receipt)
        bad["inputs_sha256"] = {}
        self.assertTrue(self.errors(bad))

    def test_missing_required_output_rejected(self):
        bad = copy.deepcopy(self.receipt)
        del bad["outputs_sha256"]["out/test.py"]
        bad["changed_paths"].remove("out/test.py")
        self.assertIn("Required output hash is missing", self.errors(bad))

    def test_extraneous_changed_and_output_paths_rejected(self):
        bad = copy.deepcopy(self.receipt)
        self._write("outside.txt", b"x")
        bad["changed_paths"].append("outside.txt")
        bad["outputs_sha256"]["outside.txt"] = sha(b"x")
        self.assertTrue(self.errors(bad))

    def test_altered_command_fields_rejected(self):
        for field, value in (("argv", ["forged"]), ("cwd", "other"), ("expected_exit", 2)):
            bad = copy.deepcopy(self.receipt)
            bad["commands"][0][field] = value
            self.assertTrue(self.errors(bad), field)

    def test_output_hash_mismatch_rejected(self):
        bad = copy.deepcopy(self.receipt)
        bad["outputs_sha256"]["out/code.py"] = "0" * 64
        self.assertIn("Receipt output hash drift detected", self.errors(bad))

    def test_changed_prerequisite_artifact_rejected(self):
        self._write("prereq.txt", b"altered")
        self.assertIn("Prerequisite artifact hash drift detected", self.errors())

    def test_empty_incomplete_or_mismatched_instance_sets_rejected(self):
        for expected, accepted in (([], []), (["one", "two"], ["one"]), (["one"], ["two"])):
            packet = copy.deepcopy(self.packet)
            prerequisite = packet["coordinator_binding"]["prerequisite_receipts"]["D1.5.receipts"]
            prerequisite["expected_instances"] = expected
            prerequisite["accepted_instances"] = accepted
            self.assertIn("Prerequisite instance sets are invalid", self.errors(packet=packet))

    def test_unsafe_output_path_rejected(self):
        bad = copy.deepcopy(self.receipt)
        bad["changed_paths"].append("../escape")
        self.assertTrue(self.errors(bad))

    def test_protected_alias_rejected(self):
        packet = copy.deepcopy(self.packet)
        packet["coordinator_binding"]["required_outputs"] = ["alias/guard.txt"]
        packet["write_paths"].append("alias")
        bad = copy.deepcopy(self.receipt)
        bad["outputs_sha256"]["alias/guard.txt"] = sha(b"protected")
        bad["changed_paths"].append("alias/guard.txt")
        (self.root / "alias").symlink_to(self.root / "protected", target_is_directory=True)
        self.assertTrue(self.errors(bad, packet=packet))

    def test_receipt_path_mismatch_rejected(self):
        self.assertIn("Receipt path does not match the coordinator binding", self.errors(path="other.json"))

    def test_symlink_escape_rejected(self):
        outside = Path(self.temp.name).parent / (Path(self.temp.name).name + "-outside")
        outside.write_text("outside")
        self.addCleanup(lambda: outside.unlink(missing_ok=True))
        (self.root / "escape").symlink_to(outside)
        packet = copy.deepcopy(self.packet)
        packet["coordinator_binding"]["required_outputs"] = ["escape"]
        packet["write_paths"].append("escape")
        bad = copy.deepcopy(self.receipt)
        bad["outputs_sha256"]["escape"] = sha(b"outside")
        bad["changed_paths"].append("escape")
        self.assertTrue(self.errors(bad, packet=packet))

    def test_staged_unstaged_and_untracked_changes_are_counted(self):
        self._write("out/code.py", b"staged")
        self._git("add", "out/code.py")
        self._write("out/test.py", b"unstaged")
        self._write("out/extra.txt", b"untracked")
        self.packet["write_paths"].append("out/extra.txt")
        self.receipt["outputs_sha256"].update({"out/code.py": sha(b"staged"), "out/test.py": sha(b"unstaged"),
                                                "out/extra.txt": sha(b"untracked")})
        self.receipt["changed_paths"].append("out/extra.txt")
        self.assertEqual(self.errors(), [])

    def test_deletions_rejected(self):
        self._git("rm", "protected/guard.txt")
        _, git_errors = drift._git_changed_paths(self.root)
        self.assertIn("Git deletions are not accepted", git_errors)

    def test_renames_and_copies_rejected(self):
        self._git("mv", "protected/guard.txt", "protected/moved.txt")
        _, git_errors = drift._git_changed_paths(self.root)
        self.assertIn("Git renames and copies are not accepted", git_errors)

    def test_sidecar_exclusions_are_exact(self):
        self._write("control/result.json", b"result")
        self._write("control/receipt.json", b"receipt")
        # Control paths are excluded by the verifier, but no other path is.
        self.assertEqual(self.errors(), [])

    def test_output_symlink_alias_to_control_sidecar_rejected(self):
        self._write("control/receipt.json", b"receipt")
        (self.root / "receipt-alias.json").symlink_to(self.root / "control/receipt.json")
        packet = copy.deepcopy(self.packet)
        packet["coordinator_binding"]["required_outputs"] = ["receipt-alias.json"]
        packet["write_paths"].append("receipt-alias.json")
        receipt = copy.deepcopy(self.receipt)
        receipt["outputs_sha256"]["receipt-alias.json"] = sha(b"receipt")
        receipt["changed_paths"].append("receipt-alias.json")
        self.assertIn("Receipt output resolves into a control sidecar",
                      self.errors(receipt, packet=packet))

    def test_malformed_receipt_shapes_fail_closed_without_exception(self):
        bad = copy.deepcopy(self.receipt)
        bad["changed_paths"] = [{"unhashable": True}]
        self.assertTrue(self.errors(bad))

    def test_malformed_packet_path_types_fail_closed_without_exception(self):
        for field, value in (("result_path", {"unhashable": True}),
                             ("receipt_path", ["not", "hashable"])):
            packet = copy.deepcopy(self.packet)
            key = field if field == "result_path" else None
            if key:
                packet[key] = value
            else:
                packet["coordinator_binding"][field] = value
            with self.subTest(field=field):
                errors = self.errors(packet=packet)
                self.assertTrue(errors)

    def test_cli_loads_packet_before_bound_receipt(self):
        packet_path = self.root / "packet.json"
        packet_path.write_text(json.dumps({"status": "bad"}), encoding="utf-8")
        result = subprocess.run(["python3", str(ROOT / "tools/receipt_drift.py"), "verify", "missing.json",
                                 str(packet_path), "--repo", str(self.root)], capture_output=True, text=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn("Coordinator binding", result.stderr)
        self.assertNotIn(str(self.root), result.stderr)


if __name__ == "__main__":
    unittest.main()
