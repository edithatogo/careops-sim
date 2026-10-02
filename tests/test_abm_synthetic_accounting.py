import copy
import json
import unittest
from pathlib import Path

from tools.validate_abm_example import validate


ROOT = Path(__file__).resolve().parents[1]


class SyntheticAbmAccountingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.example = json.loads((ROOT / "model-inputs/ed/abm/p23-synthetic-accounting.json").read_text())

    def test_hand_worked_fixture_passes_as_synthetic_only(self):
        validate(copy.deepcopy(self.example))

    def test_reverse_reachability_is_not_inferred(self):
        case = copy.deepcopy(self.example)
        case["graph"]["edges"].append({"edge_id": "ba", "from": "B", "to": "A", "distance_m": 18.5, "accessible": True, "speed_m_per_s": 1.0})
        with self.assertRaisesRegex(ValueError, "declared unreachable"):
            validate(case)

    def test_inaccessible_edge_cannot_be_used(self):
        case = copy.deepcopy(self.example)
        case["micro_case"]["route"] = {"from": "B", "to": "D", "edge_ids": ["bd-closed"]}
        with self.assertRaisesRegex(ValueError, "inaccessible"):
            validate(case)

    def test_invalid_unit_arithmetic_fails(self):
        case = copy.deepcopy(self.example)
        case["graph"]["edges"][0]["speed_m_per_s"] = 0
        with self.assertRaisesRegex(ValueError, "positive metres per second"):
            validate(case)

    def test_travel_interval_must_match_edge_time(self):
        case = copy.deepcopy(self.example)
        case["micro_case"]["intervals_seconds"]["travel"] = 20.0
        with self.assertRaisesRegex(ValueError, "travel interval must equal"):
            validate(case)

    def test_macro_elapsed_is_not_added_to_micro_intervals(self):
        case = copy.deepcopy(self.example)
        case["macro_case"]["elapsed_seconds"] = 75.0
        with self.assertRaisesRegex(ValueError, "same elapsed window"):
            validate(case)

    def test_macro_does_not_require_a_graph(self):
        case = copy.deepcopy(self.example)
        self.assertIsNone(case["macro_case"]["graph"])
        validate(case)


if __name__ == "__main__":
    unittest.main()
