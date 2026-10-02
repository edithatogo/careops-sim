"""Integrity checks for E0.1 contract fixtures, not application validation."""

import json
import unittest
from pathlib import Path


FIXTURE_PATH = (
    Path(__file__).resolve().parents[1]
    / "conductor/design/ed/e0.1-scenario-validation-cases.json"
)
REQUIRED_CATEGORIES = {
    "invalid_resource_count",
    "invalid_unit",
    "unsupported_schema_version",
    "missing_required_input",
}
EXPECTED_CASES = {
    "invalid-resource-count": {
        "category": "invalid_resource_count",
        "parameter_id": "ed.resources.beds",
        "input": {"concept_id": "ed.resources.beds", "value": -1},
        "diagnostic": {
            "code": "ED_RESOURCE_COUNT_INVALID",
            "message": "Resource count must be a non-negative integer.",
        },
    },
    "invalid-unit-sentinel": {
        "category": "invalid_unit",
        "parameter_id": "ed.experimentcontrols.timeunits",
        "input": {
            "config_key": "time_fields.source_unit",
            "value": "__unsupported_unit_sentinel__",
        },
        "diagnostic": {
            "code": "ED_TIME_UNIT_UNSUPPORTED",
            "message": "Time field source unit is unsupported.",
        },
    },
    "unsupported-schema-version-sentinel": {
        "category": "unsupported_schema_version",
        "parameter_id": None,
        "input": {"schema_version": "__unsupported_version_sentinel__"},
        "diagnostic": {
            "code": "ED_SCHEMA_VERSION_UNSUPPORTED",
            "message": "Scenario schema version is unsupported.",
        },
    },
    "missing-required-run-seed": {
        "category": "missing_required_input",
        "parameter_id": "ed.experimentcontrols.seedstreams",
        "input": {"config_key": "experiment.run_seed", "present": False},
        "diagnostic": {
            "code": "ED_RUN_SEED_REQUIRED",
            "message": "Required run seed is missing.",
        },
    },
}


class ScenarioValidationFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalogue = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        cls.cases = cls.catalogue["cases"]

    def test_catalogue_declares_fixture_only_scope(self):
        self.assertEqual(self.catalogue["catalogue_id"], "e0.1.scenario_validation")
        self.assertFalse(self.catalogue["runtime_rejection_claimed"])
        self.assertIn("no application schema", self.catalogue["scope"])

    def test_required_cases_have_unique_stable_ids(self):
        ids = [case["id"] for case in self.cases]
        self.assertEqual(len(ids), len(set(ids)))
        self.assertEqual(set(ids), set(EXPECTED_CASES))
        self.assertEqual({case["category"] for case in self.cases}, REQUIRED_CATEGORIES)
        self.assertEqual(len(self.cases), len(REQUIRED_CATEGORIES))

    def test_each_exact_negative_input_maps_to_its_expected_diagnostic(self):
        for case in self.cases:
            with self.subTest(case=case["id"]):
                expected = EXPECTED_CASES[case["id"]]
                self.assertEqual(case["category"], expected["category"])
                self.assertEqual(case["parameter_id"], expected["parameter_id"])
                self.assertEqual(case["input"], expected["input"])
                self.assertEqual(case["expected_diagnostic"], expected["diagnostic"])

    def test_each_case_has_provenance_invariant_and_diagnostic(self):
        for case in self.cases:
            with self.subTest(case=case["id"]):
                self.assertTrue(case["synthetic_test_only"])
                self.assertTrue(case["provenance"].strip())
                self.assertTrue(case["invariant"].strip())
                self.assertEqual(case["schema_path"], None)
                self.assertEqual(case["schema_path_status"], "bind_in_E0.2")
                diagnostic = case["expected_diagnostic"]
                self.assertTrue(diagnostic["code"].strip())
                self.assertTrue(diagnostic["message"].strip())

    def test_bound_p0_identifiers_and_config_keys_are_used(self):
        by_id = {case["id"]: case for case in self.cases}
        self.assertEqual(
            by_id["invalid-resource-count"]["parameter_id"], "ed.resources.beds"
        )
        self.assertEqual(
            by_id["invalid-unit-sentinel"]["config_key"], "time_fields.source_unit"
        )
        self.assertEqual(
            by_id["missing-required-run-seed"]["parameter_id"],
            "ed.experimentcontrols.seedstreams",
        )
        self.assertEqual(
            by_id["missing-required-run-seed"]["config_key"], "experiment.run_seed"
        )

    def test_diagnostic_contracts_are_stable_and_distinct(self):
        codes = [case["expected_diagnostic"]["code"] for case in self.cases]
        messages = [case["expected_diagnostic"]["message"] for case in self.cases]
        self.assertEqual(len(codes), len(set(codes)))
        self.assertEqual(len(messages), len(set(messages)))
        self.assertEqual(
            set(codes),
            {
                "ED_RESOURCE_COUNT_INVALID",
                "ED_TIME_UNIT_UNSUPPORTED",
                "ED_SCHEMA_VERSION_UNSUPPORTED",
                "ED_RUN_SEED_REQUIRED",
            },
        )


if __name__ == "__main__":
    unittest.main()
