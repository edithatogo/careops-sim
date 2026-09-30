import copy
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = json.loads((ROOT / "model-inputs/ed/schema/source-verification-record.schema.json").read_text(encoding="utf-8"))
FIXTURES = json.loads((ROOT / "model-inputs/ed/schema/source-verification-examples.json").read_text(encoding="utf-8"))
VALIDATOR = Draft202012Validator(SCHEMA, format_checker=FormatChecker())


class SourceVerificationSchemaTests(unittest.TestCase):
    def test_valid_primary_assumption_and_gap_examples_pass(self):
        for case in FIXTURES["cases"]:
            if case["expected_valid"]:
                with self.subTest(case=case["name"]):
                    self.assertEqual(list(VALIDATOR.iter_errors(case["record"])), [])

    def test_missing_provenance_mixed_evidence_and_invalid_units_fail(self):
        for case in FIXTURES["cases"]:
            if not case["expected_valid"]:
                with self.subTest(case=case["name"]):
                    self.assertTrue(list(VALIDATOR.iter_errors(case["record"])))

    def test_report_hash_and_lineage_are_not_primary_source_fields(self):
        record = copy.deepcopy(next(c["record"] for c in FIXTURES["cases"] if c["name"] == "primary_source_shape_only"))
        record["primary_source"]["report_sha256"] = "b" * 64
        record["primary_source"]["report_line"] = 12
        self.assertTrue(list(VALIDATOR.iter_errors(record)))


if __name__ == "__main__":
    unittest.main()
