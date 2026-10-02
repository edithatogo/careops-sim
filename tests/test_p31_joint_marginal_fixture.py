import json
import unittest
from collections import Counter
from pathlib import Path


FIXTURE_PATH = Path("model-inputs/ed/calibration/p31-joint-marginal-fixture.json")


def count_values(rows):
    return dict(sorted(Counter(rows).items()))


class JointMarginalFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = json.loads(FIXTURE_PATH.read_text())
        comparison = cls.fixture["comparison"]
        cls.reference = [tuple(row) for row in comparison["reference_pairs"]]
        cls.comparison = [tuple(row) for row in comparison["comparison_pairs"]]

    def test_coordinate_marginal_counts_match_but_joint_pair_counts_differ(self):
        for coordinate in (0, 1):
            self.assertEqual(
                count_values([row[coordinate] for row in self.reference]),
                count_values([row[coordinate] for row in self.comparison]),
            )

        self.assertNotEqual(count_values(self.reference), count_values(self.comparison))
        self.assertEqual(count_values(self.reference), {(0, 0): 1, (1, 1): 1})
        self.assertEqual(count_values(self.comparison), {(0, 1): 1, (1, 0): 1})

    def test_fixture_is_synthetic_and_limited_to_hand_computable_oracle(self):
        provenance = self.fixture["provenance"]
        comparison = self.fixture["comparison"]

        self.assertEqual(provenance["class"], "synthetic_only")
        self.assertFalse(provenance["empirical_data_used"])
        self.assertFalse(provenance["promoted_to_ed_parameter_or_default"])
        self.assertFalse(provenance["fitting_performed"])
        self.assertEqual(comparison["value_status"], "synthetic")
        self.assertIn("Separate marginal fit checks cannot establish joint or pathway agreement", comparison["scope_limit"])
        self.assertIn("hand-computable synthetic test oracle", comparison["scope_limit"])
        self.assertTrue(any("report-12.md.txt" in row["path"] for row in self.fixture["source_lineage"]))
        self.assertTrue(any("ed-research-incorporation" in row["path"] for row in self.fixture["source_lineage"]))


if __name__ == "__main__":
    unittest.main()
