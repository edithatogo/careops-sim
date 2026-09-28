import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import command_receipts


HASH = "a" * 64


def fixture():
    return {
        "schema_version": 1,
        "packet_id": "D1.5.receipts",
        "attempt_id": "attempt-2",
        "target_repo": ".",
        "base_commit": "b" * 40,
        "worker": {
            "agent_id": "worker-1",
            "model": "gpt-6-luna",
            "model_version": None,
            "reasoning_effort": "low",
        },
        "started_at_utc": "2026-09-28T00:00:00Z",
        "finished_at_utc": "2026-09-28T00:00:04Z",
        "inputs_sha256": {"input.json": HASH},
        "commands": [{
            "argv": ["python3", "-m", "unittest"],
            "cwd": ".",
            "started_at_utc": "2026-09-28T00:00:01Z",
            "finished_at_utc": "2026-09-28T00:00:02Z",
            "expected_exit": 0,
            "exit_code": 0,
            "stdout_sha256": HASH,
            "stdout_bytes": 12,
            "stderr_sha256": HASH,
            "stderr_bytes": 0,
            "evidence_source": "worker_reported",
            "tool_versions": {"python": "3.x"},
        }],
        "outputs_sha256": {"result.json": HASH},
        "changed_paths": ["result.json"],
        "status": "ready_for_review",
        "unresolved_issues": [],
    }


class CommandReceiptTests(unittest.TestCase):
    def test_valid_receipt_and_cli(self):
        data = fixture()
        self.assertEqual(command_receipts.validate_receipt(data), [])
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "receipt.json"
            path.write_text(json.dumps(data), encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(Path(command_receipts.__file__)), "validate", str(path)],
                capture_output=True, text=True,
            )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "receipt valid")

    def test_required_additional_properties_and_digest_rules(self):
        cases = []
        data = fixture(); del data["worker"]; cases.append(data)
        data = fixture(); data["unexpected"] = "value"; cases.append(data)
        data = fixture(); data["base_commit"] = "z" * 40; cases.append(data)
        data = fixture(); data["schema_version"] = True; cases.append(data)
        data = fixture(); data["inputs_sha256"] = {"a/../b": HASH}; cases.append(data)
        data = fixture(); data["outputs_sha256"] = {"a//b": HASH}; cases.append(data)
        data = fixture(); data["changed_paths"] = ["folder/"]; cases.append(data)
        data = fixture(); data["commands"][0]["stdout_bytes"] = True; cases.append(data)
        for data in cases:
            with self.subTest(data=data):
                self.assertTrue(command_receipts.validate_receipt(data))

    def test_worker_evidence_source_is_strict(self):
        data = fixture()
        data["commands"][0]["evidence_source"] = "coordinator_captured"
        self.assertTrue(command_receipts.validate_receipt(data))

    def test_utc_chronology_and_nonoverlapping_commands(self):
        data = fixture()
        data["started_at_utc"] = "2026-02-30T00:00:00Z"
        self.assertTrue(command_receipts.validate_receipt(data))

        data = fixture()
        data["commands"].append(copy.deepcopy(data["commands"][0]))
        data["commands"][1]["started_at_utc"] = "2026-09-28T00:00:01.5Z"
        data["commands"][1]["finished_at_utc"] = "2026-09-28T00:00:03Z"
        self.assertTrue(any("overlap" in item for item in command_receipts.validate_receipt(data)))

        data = fixture()
        data["commands"][0]["started_at_utc"] = "2026-09-28T00:00:05Z"
        data["commands"][0]["finished_at_utc"] = "2026-09-28T00:00:06Z"
        self.assertTrue(any("outside the attempt" in item for item in command_receipts.validate_receipt(data)))

    def test_ready_status_consistency_and_output_map(self):
        mutations = []
        data = fixture(); data["unresolved_issues"] = ["pending review"]; mutations.append(data)
        data = fixture(); data["commands"][0]["exit_code"] = 1; mutations.append(data)
        data = fixture(); data["outputs_sha256"] = {}; mutations.append(data)
        data = fixture(); data["outputs_sha256"] = {}; data["changed_paths"] = []; mutations.append(data)
        for data in mutations:
            with self.subTest(data=data):
                self.assertTrue(command_receipts.validate_receipt(data))

        data = fixture()
        data["status"] = "blocked"
        data["unresolved_issues"] = []
        self.assertTrue(any("requires an issue summary" in item for item in command_receipts.validate_receipt(data)))

    def test_common_sensitive_patterns_are_rejected_without_echoing_values(self):
        data = fixture()
        data["commands"][0]["argv"] = ["tool", "api_key=REDACTED"]
        errors = command_receipts.validate_receipt(data)
        self.assertTrue(any("private-data pattern" in item for item in errors))
        self.assertFalse(any("REDACTED" in item for item in errors))

    def test_untrusted_property_names_are_hidden_and_scanned(self):
        data = fixture()
        data["worker"]["UNTRUSTED_PROPERTY_MARKER"] = "value"
        errors = command_receipts.validate_receipt(data)
        self.assertTrue(any("additional property is forbidden" in item for item in errors))
        self.assertFalse(any("UNTRUSTED_PROPERTY_MARKER" in item for item in errors))

        data = fixture()
        data["worker"]["api_key=REDACTED"] = "value"
        errors = command_receipts.validate_receipt(data)
        self.assertTrue(any("private-data pattern" in item for item in errors))
        self.assertFalse(any("api_key" in item or "REDACTED" in item for item in errors))

    def test_split_secret_flags_are_rejected_without_echoing_values(self):
        for flag in ("--password", "--api-key", "--authorization"):
            with self.subTest(flag=flag):
                data = fixture()
                data["commands"][0]["argv"] = ["tool", flag, "VALUE_MUST_NOT_ECHO"]
                errors = command_receipts.validate_receipt(data)
                self.assertTrue(any("private-data pattern" in item for item in errors))
                self.assertFalse(any("VALUE_MUST_NOT_ECHO" in item for item in errors))

    def test_fractional_timestamp_order_preserves_submicrosecond_digits(self):
        data = fixture()
        data["commands"].append(copy.deepcopy(data["commands"][0]))
        data["commands"][0]["started_at_utc"] = "2026-09-28T00:00:01.0000009Z"
        data["commands"][0]["finished_at_utc"] = "2026-09-28T00:00:01.0000009Z"
        data["commands"][1]["started_at_utc"] = "2026-09-28T00:00:01.0000008Z"
        data["commands"][1]["finished_at_utc"] = "2026-09-28T00:00:01.0000010Z"
        errors = command_receipts.validate_receipt(data)
        self.assertTrue(any("overlap or are out of order" in item for item in errors))

    def test_schema_pattern_path_and_integer_edges(self):
        data = fixture()
        data["base_commit"] = "a" * 40 + "\n"
        self.assertTrue(command_receipts.validate_receipt(data))

        data = fixture()
        data["target_repo"] = "path\x00segment"
        self.assertTrue(command_receipts.validate_receipt(data))

        data = fixture()
        data["schema_version"] = 1.0
        data["commands"][0]["expected_exit"] = 0.0
        data["commands"][0]["exit_code"] = 0.0
        data["commands"][0]["stdout_bytes"] = 12.0
        self.assertEqual(command_receipts.validate_receipt(data), [])

        data = fixture()
        data["commands"][0]["expected_exit"] = float("inf")
        self.assertTrue(command_receipts.validate_receipt(data))

        for field, value in (("stdout_bytes", -1.0), ("stderr_bytes", -2.0)):
            with self.subTest(field=field):
                data = fixture()
                data["commands"][0][field] = value
                self.assertTrue(command_receipts.validate_receipt(data))

        for value in (10**400, -(10**400)):
            with self.subTest(value_sign="negative" if value < 0 else "positive"):
                data = fixture()
                data["commands"][0]["stdout_bytes"] = value
                if value < 0:
                    self.assertTrue(command_receipts.validate_receipt(data))
                else:
                    self.assertEqual(command_receipts.validate_receipt(data), [])

    def test_cli_invalid_receipt_returns_one_and_sanitized_error(self):
        data = fixture()
        data["worker"]["extra"] = "ignored"
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "invalid.json"
            path.write_text(json.dumps(data), encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(Path(command_receipts.__file__)), "validate", str(path)],
                capture_output=True, text=True,
            )
        self.assertEqual(result.returncode, 1)
        self.assertIn("additional property is forbidden", result.stderr)
        self.assertNotIn("ignored", result.stderr)


if __name__ == "__main__":
    unittest.main()
