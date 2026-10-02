import json
import unittest
from pathlib import Path


FIXTURE_PATH = Path("model-inputs/ed/calibration/p31-identifiability-fixtures.json")


def feasible_points(fixture, case):
    grid = fixture["synthetic_grid"]
    return [
        {"walk": walk, "work": work}
        for walk in grid["walk_values"]
        for work in grid["work_values"]
        if walk + work == case["observed_total"]
        and (case.get("observed_walk") is None or walk == case["observed_walk"])
    ]


def identification_status(candidates):
    return "identified_on_finite_synthetic_grid" if len(candidates) == 1 else "non_identifiable"


class IdentifiabilityFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = json.loads(FIXTURE_PATH.read_text())

    def test_total_only_enumeration_retains_all_three_decompositions(self):
        case = self.fixture["observations"]["total_only"]
        candidates = feasible_points(self.fixture, case)

        self.assertEqual(
            candidates,
            [
                {"walk": 1, "work": 4},
                {"walk": 2, "work": 3},
                {"walk": 3, "work": 2},
            ],
        )
        self.assertEqual(identification_status(candidates), case["identification_status"])
        self.assertIsNone(case["selected_decomposition"])

    def test_independent_walk_observation_identifies_one_point_only_on_this_grid(self):
        case = self.fixture["observations"]["total_plus_independent_walk"]
        candidates = feasible_points(self.fixture, case)

        self.assertEqual(candidates, [{"walk": 2, "work": 3}])
        self.assertEqual(identification_status(candidates), case["identification_status"])
        self.assertEqual(case["selected_decomposition"], candidates[0])
        self.assertIn("finite synthetic grid", case["scope_limit"])
        self.assertIn("not an empirical or universal identifiability claim", case["scope_limit"])

    def test_every_numeric_fixture_value_is_explicitly_synthetic(self):
        self.assertEqual(self.fixture["provenance"]["class"], "synthetic_only")
        self.assertFalse(self.fixture["provenance"]["empirical_data_used"])
        self.assertFalse(self.fixture["provenance"]["promoted_to_ed_parameter_or_default"])
        self.assertFalse(self.fixture["provenance"]["fitting_performed"])
        self.assertEqual(self.fixture["synthetic_grid"]["value_status"], "synthetic")


if __name__ == "__main__":
    unittest.main()
