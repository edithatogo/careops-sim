import copy
import json
import unittest
from pathlib import Path

from tools.validate_source_acceptance import acceptance_errors


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = json.loads((ROOT / "model-inputs/ed/schema/source-verification-record.schema.json").read_text())
EXAMPLES = json.loads((ROOT / "model-inputs/ed/schema/source-verification-examples.json").read_text())
BASE = next(x["record"] for x in EXAMPLES["cases"] if x["name"] == "primary_source_shape_only")


class SourceAcceptanceTests(unittest.TestCase):
    def eligible_record(self):
        row = copy.deepcopy(BASE)
        row["review"]["outcome"] = "accepted"
        row["primary_source"]["verification"] = "verified"
        return row

    def test_equal_digests_and_all_states_pass_structural_gate(self):
        self.assertEqual(acceptance_errors(self.eligible_record(), SCHEMA), [])

    def test_mismatched_digest_rejected_even_when_match_flag_true(self):
        row = self.eligible_record()
        row["independent_readback"]["source_hash_recomputation"]["retrieved_bytes_sha256"] = "c" * 64
        self.assertIn("readback digest differs from primary source digest", acceptance_errors(row, SCHEMA))

    def test_all_three_acceptance_states_required(self):
        for path, value in (("review", "needs_review"), ("source", "unverified"), ("readback", "partial")):
            with self.subTest(path=path):
                row = self.eligible_record()
                if path == "review":
                    row["review"]["outcome"] = value
                elif path == "source":
                    row["primary_source"]["verification"] = value
                else:
                    row["independent_readback"]["outcome"] = value
                self.assertTrue(acceptance_errors(row, SCHEMA))


if __name__ == "__main__":
    unittest.main()
