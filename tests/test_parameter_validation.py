import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools" / "validate_ed_parameters.py"
SCHEMA = ROOT / "model-inputs/ed/schema/parameter-record.schema.json"
REGISTRY = ROOT / "model-inputs/ed/schema/parameter-ids.json"
FIXTURES = ROOT / "model-inputs/ed/schema/schema-negative-examples.json"


class ParameterValidationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.examples = json.loads(FIXTURES.read_text(encoding="utf-8"))
        cls.valid = next(case["record"] for case in cls.examples["cases"] if case["expected_valid"])
        cls.valid["parameter_id"] = json.loads(REGISTRY.read_text(encoding="utf-8"))["entries"][0]["parameter_id"]

    def invoke(self, record):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "record.json"
            path.write_text(json.dumps(record), encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(TOOL), "--schema", str(SCHEMA), "--registry", str(REGISTRY), "--record", str(path)],
                cwd=ROOT, text=True, capture_output=True, check=False,
            )

    def test_minimal_valid_registered_synthetic_record_passes(self):
        result = self.invoke(copy.deepcopy(self.valid))
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_malformed_unit_type_fails(self):
        record = copy.deepcopy(self.valid)
        record["unit"] = 17
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("schema violation", result.stderr)

    def test_missing_provenance_fails(self):
        record = copy.deepcopy(self.valid)
        del record["provenance"]
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("schema violation", result.stderr)

    def test_plausible_unregistered_id_fails(self):
        record = copy.deepcopy(self.valid)
        record["parameter_id"] = "ed.resources.notregistered"
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("parameter_id is not registered", result.stderr)


if __name__ == "__main__":
    unittest.main()
