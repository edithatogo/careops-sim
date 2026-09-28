"""Pure recovery-decision tests over temporary local Git fixtures."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
spec = importlib.util.spec_from_file_location("recovery", ROOT / "tools/recovery.py")
recovery = importlib.util.module_from_spec(spec)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class RecoveryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self._git("init", "-q")
        self._git("config", "user.email", "fixture@example.invalid")
        self._git("config", "user.name", "Fixture")
        self._write("input.txt", b"input")
        self._write("source.txt", b"source")
        self._write("context.md", b"context")
        self._write("protected/secret.txt", b"protected")
        self._git("add", ".")
        self._git("commit", "-qm", "base")
        self.base = self._git("rev-parse", "HEAD").decode().strip()
        self.packet = {
            "schema_version": 1, "packet_id": "D1.5.fixture", "task_id": "D1.5",
            "status": "prepared", "target_repo": ".", "base_commit": self.base,
            "objective": "synthetic", "context_paths": ["context.md"],
            "input_hashes": {"input.txt": sha(b"input")}, "source_hashes": {"source.txt": sha(b"source")},
            "write_paths": ["out", "result.json", "receipt.json"],
            "protected_paths": [".git", "protected"], "leaf_dependencies": [],
            "interface_contract": "synthetic", "steps": ["prepare", "execute", "verify"],
            "verification": [{"argv": ["true"], "cwd": ".", "expected_exit": 0, "oracle": "pass"}],
            "acceptance": ["resume exactly"], "result_path": "result.json", "stop_conditions": ["drift"],
            "coordinator_binding": {"receipt_path": "receipt.json", "prerequisite_receipts": {}},
        }
        self.packet_bytes = (json.dumps(self.packet, sort_keys=True, separators=(",", ":")) + "\n").encode()
        self.packet_hash = sha(self.packet_bytes)
        self.ledger = {"packet_id": "D1.5.fixture", "attempt_id": "attempt-1", "target_repo": ".",
                       "base_commit": self.base, "packet_sha256": self.packet_hash, "state": "running",
                       "result_path": "result.json", "receipt_path": "receipt.json",
                       "reserved_paths": ["out", "result.json", "receipt.json"]}
        self.checkpoint = {"schema_version": 1, "packet_id": "D1.5.fixture", "attempt_id": "attempt-1",
                           "steps": [
                               {"index": 0, "label": "prepare", "state": "accepted",
                                "artifacts_sha256": {"out/prepared.txt": sha(b"prepared")}},
                               {"index": 1, "label": "execute", "state": "not_started", "artifacts_sha256": {}},
                               {"index": 2, "label": "verify", "state": "not_started", "artifacts_sha256": {}},
                           ]}
        self._write("out/prepared.txt", b"prepared")
        self.stopped = {"packet_id": "D1.5.fixture", "attempt_id": "attempt-1", "confirmer": "coordinator",
                        "timestamp_utc": "2026-09-28T10:00:00Z", "basis": "worker process observed stopped"}

    def _git(self, *args):
        return subprocess.run(["git", "-C", str(self.root), *args], check=True, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE).stdout

    def _write(self, path, data):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)

    def call(self, **kwargs):
        return recovery.inspect_attempt(self.ledger, self.packet, self.checkpoint, self.packet_hash, self.root, **kwargs)

    def test_killed_worker_resumes_exact_next_step_and_keeps_accepted_artifact(self):
        decision = self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")
        self.assertEqual(decision["decision"], "retry_proposed")
        retry = decision["retry_plan"]
        self.assertEqual(retry["resume_step"], {"index": 1, "label": "execute"})
        self.assertEqual(retry["accepted_steps"], [{"index": 0, "label": "prepare",
                         "artifacts_sha256": {"out/prepared.txt": sha(b"prepared")}}])
        self.assertEqual((self.root / "out/prepared.txt").read_bytes(), b"prepared")
        self.assertEqual(self.ledger["attempt_id"], "attempt-1")

    def test_changed_or_missing_accepted_artifact_blocks(self):
        self._write("out/prepared.txt", b"changed")
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "blocked")
        (self.root / "out/prepared.txt").unlink()
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "blocked")

    def test_running_without_stop_attestation_is_suspected_stale(self):
        decision = self.call(new_attempt_id="attempt-2")
        self.assertEqual(decision["decision"], "suspected_stale")
        self.assertNotIn("retry_plan", decision)

    def test_started_unknown_is_manual_blocker(self):
        self.checkpoint["steps"][1] = {"index": 1, "label": "execute", "state": "started_unknown",
                                        "artifacts_sha256": {}}
        decision = self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")
        self.assertEqual(decision["decision"], "manual_reconciliation")
        self.assertNotIn("retry_plan", decision)

    def test_packet_hash_mismatch_requires_rebind_without_retry(self):
        decision = recovery.inspect_attempt(self.ledger, self.packet, self.checkpoint, "0" * 64, self.root,
                                           worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")
        self.assertEqual(decision["decision"], "rebind_required")
        self.assertNotIn("retry_plan", decision)

    def test_stale_base_input_and_source_require_rebind(self):
        self.packet["base_commit"] = "a" * 40
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "rebind_required")
        self.packet["base_commit"] = self.base
        self._write("input.txt", b"changed")
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "rebind_required")
        self._write("input.txt", b"input")
        self._write("source.txt", b"changed")
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "rebind_required")

    def test_packet_readiness_base_drift_requires_rebind(self):
        self._git("commit", "--allow-empty", "-qm", "advance")
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "rebind_required")

    def test_ready_for_review_routes_to_acceptance(self):
        self.ledger["state"] = "ready_for_review"
        decision = self.call()
        self.assertEqual(decision["decision"], "route_to_acceptance")
        self.assertNotIn("retry_plan", decision)

    def test_integrated_is_terminal(self):
        self.ledger["state"] = "integrated"
        self.assertEqual(self.call()["decision"], "terminal")

    def test_prepared_dispatches_same_attempt_only_when_checkpoint_fresh(self):
        self.ledger["state"] = "prepared"
        self.checkpoint["steps"] = [{"index": i, "label": label, "state": "not_started", "artifacts_sha256": {}}
                                     for i, label in enumerate(self.packet["steps"])]
        self.assertEqual(self.call()["decision"], "dispatch_existing_attempt")
        self.checkpoint["steps"][0]["state"] = "accepted"
        self.checkpoint["steps"][0]["artifacts_sha256"] = {"out/prepared.txt": sha(b"prepared")}
        self.assertEqual(self.call()["decision"], "blocked")

    def test_failed_states_require_bound_stop_and_issue_assertions(self):
        resolution = {"packet_id": "D1.5.fixture", "attempt_id": "attempt-1", "resolver": "coordinator",
                      "timestamp_utc": "2026-09-28T10:01:00Z", "summary": "Cause reviewed and corrected"}
        for status in ("blocked", "failed", "cancelled"):
            self.ledger["state"] = status
            self.assertEqual(self.call(new_attempt_id="attempt-2")["decision"], "blocked")
            self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                             "blocked")
            proposal = self.call(worker_stopped_attestation=self.stopped, issue_resolution=resolution,
                                 new_attempt_id="attempt-2")
            self.assertEqual(proposal["decision"], "retry_proposed")
            self.assertEqual(proposal["retry_plan"]["supersedes_attempt_id"], "attempt-1")

    def test_new_attempt_id_must_be_fresh_and_valid(self):
        for attempt in (None, "", "attempt-1", "attempt-0"):
            decision = self.call(worker_stopped_attestation=self.stopped, new_attempt_id=attempt,
                                 prior_attempt_ids=["attempt-0"])
            self.assertEqual(decision["decision"], "blocked")

    def test_attestations_must_bind_correct_ids_and_safe_fields(self):
        bad = copy.deepcopy(self.stopped)
        bad["attempt_id"] = "other"
        self.assertEqual(self.call(worker_stopped_attestation=bad, new_attempt_id="attempt-2")["decision"], "blocked")
        bad = copy.deepcopy(self.stopped)
        bad["basis"] = "token=supersecret123456789"
        self.assertEqual(self.call(worker_stopped_attestation=bad, new_attempt_id="attempt-2")["decision"], "blocked")

    def test_mismatched_checkpoint_and_ledger_ids_rejected(self):
        self.checkpoint["packet_id"] = "other"
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "blocked")

    def test_duplicate_order_unknown_step_and_state_rejected(self):
        cases = []
        bad = copy.deepcopy(self.checkpoint); bad["steps"][1]["index"] = 0; cases.append(bad)
        bad = copy.deepcopy(self.checkpoint); bad["steps"][1]["state"] = "mystery"; cases.append(bad)
        bad = copy.deepcopy(self.checkpoint); bad["steps"][2]["state"] = "accepted"; bad["steps"][2]["artifacts_sha256"] = {"out/end": sha(b"end")}; self._write("out/end", b"end"); cases.append(bad)
        for checkpoint in cases:
            self.checkpoint = checkpoint
            self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                             "blocked")

    def test_artifact_path_reservation_and_alias_checks(self):
        for path in ("outside.txt", "input.txt", "protected/secret.txt", "result.json", "receipt.json", "../escape"):
            self.checkpoint["steps"][0]["artifacts_sha256"] = {path: sha(b"prepared")}
            decision = self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")
            self.assertEqual(decision["decision"], "blocked", path)

    def test_artifact_symlink_escape_rejected(self):
        outside = Path(self.temp.name).parent / (Path(self.temp.name).name + "-outside")
        outside.write_text("outside")
        self.addCleanup(lambda: outside.unlink(missing_ok=True))
        (self.root / "out/escape").symlink_to(outside)
        self.checkpoint["steps"][0]["artifacts_sha256"] = {"out/escape": sha(b"outside")}
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "blocked")

    def test_input_and_source_symlink_escapes_require_rebind(self):
        outside = Path(self.temp.name).parent / (Path(self.temp.name).name + "-input-outside")
        outside.write_bytes(b"input")
        self.addCleanup(lambda: outside.unlink(missing_ok=True))
        (self.root / "input-link").symlink_to(outside)
        self.packet["input_hashes"] = {"input-link": sha(b"input")}
        self._refresh_packet_hash()
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "rebind_required")

        (self.root / "source-link").symlink_to(outside)
        self.packet["input_hashes"] = {"input.txt": sha(b"input")}
        self.packet["source_hashes"] = {"source-link": sha(b"input")}
        self._refresh_packet_hash()
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "rebind_required")

    def _refresh_packet_hash(self):
        self.packet_bytes = (json.dumps(self.packet, sort_keys=True, separators=(",", ":")) + "\n").encode()
        self.packet_hash = sha(self.packet_bytes)
        self.ledger["packet_sha256"] = self.packet_hash

    def test_terminal_and_acceptance_states_precede_unknown_checkpoint(self):
        self.checkpoint["steps"][1] = {"index": 1, "label": "execute", "state": "started_unknown",
                                        "artifacts_sha256": {}}
        self.ledger["state"] = "integrated"
        self.assertEqual(self.call()["decision"], "terminal")
        self.ledger["state"] = "ready_for_review"
        self.assertEqual(self.call()["decision"], "route_to_acceptance")
        self.ledger["state"] = "integrated"
        self.assertEqual(recovery.inspect_attempt(self.ledger, self.packet, self.checkpoint, "0" * 64,
                                                  self.root)["decision"], "terminal")

    def test_ledger_base_and_checkpoint_schema_must_match(self):
        self.ledger["base_commit"] = "a" * 40
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "blocked")
        self.ledger["base_commit"] = self.base
        self.checkpoint["schema_version"] = 2
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "blocked")

    def test_all_accepted_routes_to_review_instead_of_empty_retry(self):
        self._write("out/executed.txt", b"executed")
        self._write("out/verified.txt", b"verified")
        self.checkpoint["steps"] = [
            {"index": 0, "label": "prepare", "state": "accepted", "artifacts_sha256": {"out/prepared.txt": sha(b"prepared")}},
            {"index": 1, "label": "execute", "state": "accepted", "artifacts_sha256": {"out/executed.txt": sha(b"executed")}},
            {"index": 2, "label": "verify", "state": "accepted", "artifacts_sha256": {"out/verified.txt": sha(b"verified")}},
        ]
        self.assertEqual(self.call(worker_stopped_attestation=self.stopped, new_attempt_id="attempt-2")["decision"],
                         "route_to_acceptance")

    def test_cli_is_read_only_and_outputs_sanitized_decision(self):
        paths = []
        for name, value in (("ledger.json", self.ledger), ("packet.json", self.packet), ("checkpoint.json", self.checkpoint)):
            path = self.root / name; path.write_text(json.dumps(value), encoding="utf-8"); paths.append(str(path))
        self.ledger["packet_sha256"] = sha((self.root / "packet.json").read_bytes())
        (self.root / "ledger.json").write_text(json.dumps(self.ledger), encoding="utf-8")
        before = {name: (self.root / name).read_bytes() for name in ("ledger.json", "packet.json", "checkpoint.json")}
        result = subprocess.run(["python3", str(ROOT / "tools/recovery.py"), "inspect", *paths, "--repo", str(self.root)],
                                capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn(str(self.root), result.stdout + result.stderr)
        self.assertEqual(before, {name: (self.root / name).read_bytes() for name in before})
        decision = json.loads(result.stdout)
        self.assertEqual(decision["decision"], "suspected_stale")


if spec.loader is not None:
    spec.loader.exec_module(recovery)

if __name__ == "__main__":
    unittest.main()
