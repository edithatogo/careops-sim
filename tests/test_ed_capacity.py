import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools" / "validate_ed_capacity.py"
SCHEMA = ROOT / "model-inputs/ed/schema/capacity-location.schema.json"
EXAMPLES = ROOT / "model-inputs/ed/schema/capacity-location-examples.json"


class EDCapacityValidationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.examples = json.loads(EXAMPLES.read_text(encoding="utf-8"))
        cls.valid_records = [case["record"] for case in cls.examples["cases"]]

    def invoke(self, record):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "record.json"
            path.write_text(json.dumps(record), encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(TOOL), "--schema", str(SCHEMA), "--record", str(path)],
                cwd=ROOT, text=True, capture_output=True, check=False,
            )

    def test_existing_synthetic_examples_pass(self):
        result = subprocess.run(
            [sys.executable, str(TOOL), "--schema", str(SCHEMA), "--examples", str(EXAMPLES)],
            cwd=ROOT, text=True, capture_output=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("validated 2 capacity/location example cases", result.stdout)

    def test_staffed_capacity_cannot_exceed_open_capacity(self):
        record = copy.deepcopy(self.valid_records[0])
        record["capacity"]["staffed_count"] = 11
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("staffed_count must not exceed", result.stderr)

    def test_open_capacity_cannot_exceed_known_physical_capacity(self):
        record = copy.deepcopy(self.valid_records[0])
        record["capacity"]["open_count"] = 13
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("open_count must not exceed", result.stderr)

    def test_duplicate_location_ids_fail(self):
        record = copy.deepcopy(self.valid_records[0])
        record["locations"].append(copy.deepcopy(record["locations"][0]))
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("duplicates 'loc-triage'", result.stderr)

    def test_route_to_unknown_location_fails(self):
        record = copy.deepcopy(self.valid_records[0])
        record["routes"][0]["to_location_id"] = "loc-missing"
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("references unknown location 'loc-missing'", result.stderr)

    def test_unknown_capacity_remains_unknown_and_is_not_zero(self):
        record = copy.deepcopy(self.valid_records[1])
        result = self.invoke(record)
        self.assertEqual(result.returncode, 0, result.stderr)

        record["capacity"]["open_count"] = 0
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("schema violation", result.stderr)


if __name__ == "__main__":
    unittest.main()
