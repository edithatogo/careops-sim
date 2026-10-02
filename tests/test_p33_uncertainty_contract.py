import json
from pathlib import Path
import unittest


CONTRACT_PATH = (
    Path(__file__).parents[1]
    / "model-inputs/ed/calibration/p33-uncertainty-contract.synthetic.json"
)


def load_contract():
    return json.loads(CONTRACT_PATH.read_text())


def uncertainty_type(contract, type_id):
    return next(row for row in contract["uncertainty_types"] if row["type"] == type_id)


class P33UncertaintyContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.contract = load_contract()

    def test_unknown_empirical_family_and_bounds_are_preserved(self):
        provenance = self.contract["provenance"]
        self.assertFalse(provenance["empirical_ed_data_used"])
        self.assertFalse(provenance["empirical_family_selected"])
        self.assertFalse(provenance["empirical_numeric_bounds_asserted"])
        self.assertFalse(provenance["fixture_values_are_defaults"])
        self.assertIn("UNKNOWN", self.contract["upstream_status"]["p32"])
        self.assertTrue(all("UNKNOWN" in row["range_status"] or "not established" in row["range_status"]
                            for row in self.contract["uncertainty_types"]))

    def test_within_run_variability_changes_events_under_fixed_parameter(self):
        oracle = self.contract["synthetic_oracles"]["invented_inner_outcomes_by_fixed_parameter"]
        self.assertEqual(len(set(oracle["replication_1_event_keys"])), 2)
        self.assertEqual(len(set(oracle["replication_1_outcomes"])), 2)
        self.assertEqual(uncertainty_type(self.contract, "within_run_process_variability")["held_fixed"],
                         ["declared parameter/profile state", "replication-level uncertain parameter draw"])
        self.assertFalse(self.contract["provenance"]["fixture_values_are_defaults"])

    def test_parameter_uncertainty_is_outer_replication_level(self):
        states = self.contract["synthetic_oracles"]["invented_outer_parameter_states"]
        self.assertEqual(len({row["outer_replication"] for row in states}), 2)
        self.assertEqual(len({row["parameter_state"] for row in states}), 2)
        param = uncertainty_type(self.contract, "parameter_uncertainty")
        self.assertIn("outer replication", param["unit_of_replication"])
        self.assertIn("never independently redrawn per patient", param["synthetic_oracle"])

    def test_data_and_structural_uncertainty_are_separately_typed(self):
        data = uncertainty_type(self.contract, "data_uncertainty")
        structural = uncertainty_type(self.contract, "structural_model_uncertainty")
        self.assertIn("finite synthetic test block only", data["synthetic_oracle"])
        self.assertIn("candidate structures", structural["varies"])
        self.assertNotEqual(data["type"], "within_run_process_variability")
        self.assertNotEqual(structural["type"], "parameter_uncertainty")

    def test_range_classes_have_distinct_meanings_and_provenance(self):
        ranges = {row["type"]: row for row in self.contract["range_classes"]}
        expected = {"hard_domain_limit", "observed_sample_range", "scenario_range",
                    "uncertainty_interval", "calibration_search_bound"}
        self.assertEqual(set(ranges), expected)
        for row in ranges.values():
            self.assertTrue(row["meaning"])
            self.assertTrue(row["required_provenance"])
            self.assertTrue("no " in row["default_status_for_ed"].lower()
                            or "unknown" in row["default_status_for_ed"].lower())
        search = ranges["calibration_search_bound"]["meaning"]
        self.assertIn("not a hard domain limit", search)
        self.assertIn("or plausible/empirical interval", search)

    def test_search_fixture_is_not_promoted_to_any_operating_range(self):
        search = self.contract["synthetic_oracles"]["invented_search_bound"]
        self.assertEqual(search["lower"], 2)
        self.assertEqual(search["upper"], 5)
        self.assertIn("hard_domain_limit", search["not_a"])
        self.assertIn("empirical ED bound", search["not_a"])
        self.assertEqual(search["purpose"], "optimizer traversal fixture only")
        self.assertIn("cannot be reported as plausible operating ranges", " ".join(self.contract["nonclaims"]))


if __name__ == "__main__":
    unittest.main()
