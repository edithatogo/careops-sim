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

    def load_frozen_examples(self):
        examples = json.loads(FIXTURES.read_text(encoding="utf-8"))
        expected = {
            "known-zero": True,
            "explicit-unknown-no-value": True,
            "unknown-as-zero": False,
            "missing-distinct-range-class": False,
            "missing-provenance-and-evidence": False,
        }
        self.assertTrue(
            all(type(case["expected_valid"]) is bool for case in examples["cases"]),
            "expected_valid values must be JSON booleans",
        )
        self.assertEqual(
            {case["id"]: case["expected_valid"] for case in examples["cases"]},
            expected,
        )
        self.assertEqual(len(examples["cases"]), 5)
        return examples

    def invoke_examples(self, examples_path):
        return subprocess.run(
            [sys.executable, str(TOOL), "--schema", str(SCHEMA), "--registry", str(REGISTRY),
             "--examples", str(examples_path)],
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

    def test_examples_cli_validates_the_frozen_five_case_matrix(self):
        self.load_frozen_examples()
        result = self.invoke_examples(FIXTURES)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "validated 5 example cases\n")
        self.assertEqual(result.stderr, "")

    def test_examples_cli_names_the_case_with_a_flipped_expectation(self):
        examples = self.load_frozen_examples()
        known_zero = next(case for case in examples["cases"] if case["id"] == "known-zero")
        known_zero["expected_valid"] = not known_zero["expected_valid"]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "flipped-examples.json"
            path.write_text(json.dumps(examples), encoding="utf-8")
            result = self.invoke_examples(path)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertEqual(
            result.stderr,
            "examples case known-zero: expectation mismatch\n"
            "1 example case(s) failed\n",
        )


if __name__ == "__main__":
    unittest.main()
