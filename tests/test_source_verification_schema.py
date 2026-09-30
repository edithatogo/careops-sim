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

    def test_pass_readback_is_bound_to_hash_locator_and_transform_evidence(self):
        record = copy.deepcopy(next(c["record"] for c in FIXTURES["cases"] if c["name"] == "primary_source_shape_only"))
        readback = record["independent_readback"]
        self.assertEqual(readback["source_hash_recomputation"]["retrieved_bytes_sha256"], record["primary_source"]["source_sha256"])
        self.assertTrue(readback["source_hash_recomputation"]["matches_recorded_hash"])
        self.assertTrue(readback["locator_evidence"]["located"])
        self.assertTrue(readback["transformation_evidence"]["reproduced"])
        self.assertEqual(list(VALIDATOR.iter_errors(record)), [])

    def test_nonpass_readback_remains_structured_and_is_not_acceptance(self):
        record = copy.deepcopy(next(c["record"] for c in FIXTURES["cases"] if c["name"] == "primary_source_shape_only"))
        record["independent_readback"]["outcome"] = "partial"
        record["independent_readback"]["reason"] = "Denominator could not be independently recovered."
        record["independent_readback"]["source_hash_recomputation"]["matches_recorded_hash"] = False
        record["independent_readback"]["locator_evidence"]["located"] = False
        record["independent_readback"]["transformation_evidence"]["reproduced"] = False
        self.assertEqual(list(VALIDATOR.iter_errors(record)), [])
        self.assertNotEqual(record["independent_readback"]["outcome"], "pass")


if __name__ == "__main__":
    unittest.main()
