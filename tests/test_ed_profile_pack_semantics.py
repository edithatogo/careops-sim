from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.validate_ed_profile_pack_semantics import validate_profile_semantics


ROOT = Path(__file__).resolve().parents[1]
PROFILES = [
    "model-inputs/ed/profiles/p4-minimal-pack.json",
    "model-inputs/ed/profiles/p4-nominal-pack.json",
    "model-inputs/ed/profiles/p4-surge-pack.json",
]


def load_profile(index: int = 0) -> dict:
    return json.loads((ROOT / PROFILES[index]).read_text(encoding="utf-8"))


class ProfileSemanticValidationTests(unittest.TestCase):
    def test_all_accepted_profiles_are_semantically_valid(self) -> None:
        for path in PROFILES:
            with self.subTest(path=path):
                profile = json.loads((ROOT / path).read_text(encoding="utf-8"))
                self.assertEqual(validate_profile_semantics(profile), [])

    def assert_issue(self, profile: dict, expected: str) -> None:
        errors = validate_profile_semantics(profile)
        self.assertTrue(any(expected in error for error in errors), errors)

    def test_interval_order_and_coverage(self) -> None:
        profile = load_profile()
        profile["arrival_table"][1]["interval"]["start"]["value"] = "1000"
        self.assert_issue(profile, "arrival_table[1].interval.start: intervals are not ordered and disjoint")

    def test_case_mix_matches_each_mode_and_declared_categories(self) -> None:
        profile = load_profile()
        profile["case_mix_table"][0]["counts_by_category"]["ATS4"] = 0
        self.assert_issue(profile, "category counts do not sum to arrival count")
        profile = load_profile()
        profile["case_mix_table"][0]["counts_by_category"]["OTHER"] = 1
        self.assert_issue(profile, "category outside declared scale")

    def test_distribution_support_and_probability_mass(self) -> None:
        profile = load_profile()
        profile["distributions"][0]["outcomes"][0]["probability"] = 0.5
        self.assert_issue(profile, "probabilities must be finite and sum exactly to 1")
        profile = load_profile()
        profile["distributions"][0]["outcomes"][0]["value"] = "-1"
        self.assert_issue(profile, "expected finite nonnegative support")

    def test_resource_capacity_and_calendar_feasibility(self) -> None:
        profile = load_profile()
        profile["resource_calendars"][0]["staffed_count"] = 2
        self.assert_issue(profile, "resource_calendars[0].staffed_count: exceeds open_count")
        profile = load_profile()
        profile["capacity_location"]["resource_buckets"][0]["open_count"] = 2
        self.assert_issue(profile, "resource_buckets[0].open_count: exceeds physical_count")

    def test_declared_resource_requires_calendar_coverage(self) -> None:
        profile = load_profile()
        profile["resource_calendars"] = []
        self.assert_issue(profile, "resource_calendars: missing full-horizon coverage for resource_id=resource.synthetic")

    def test_malformed_bucket_and_aggregate_capacity_counts_are_reported(self) -> None:
        profile = load_profile()
        profile["capacity_location"]["resource_buckets"][0]["physical_count"] = "one"
        self.assert_issue(profile, "resource_buckets[0].physical_count: expected nonnegative integer")
        profile = load_profile()
        profile["capacity_location"]["capacity"]["staffed_count"] = None
        self.assert_issue(profile, "capacity_location.capacity.staffed_count: expected nonnegative integer")

    def test_staffing_calendars_cover_horizon_and_declared_roles(self) -> None:
        profile = load_profile()
        profile["staffing_calendars"][0]["interval"]["end"]["value"] = "1800"
        self.assert_issue(profile, "staff_role_id=staff.synthetic does not cover [0, horizon]")

    def test_initial_state_references_and_resource_feasibility(self) -> None:
        profile = load_profile()
        profile["initial_state"]["occupants"] = [
            {"occupant_id": "p1", "location_id": "unknown"}
        ]
        self.assert_issue(profile, "initial_state.occupants[0].location_id: unknown location")
        profile = load_profile()
        profile["initial_state"]["occupants"] = [
            {"occupant_id": "p1", "location_id": "location.work"},
            {"occupant_id": "p2", "location_id": "location.work"},
        ]
        self.assert_issue(profile, "exceed physical resource capacity")
        profile = load_profile()
        profile["initial_state"]["occupants"] = [
            {"occupant_id": "p1", "location_id": "location.work"}
        ]
        profile["resource_calendars"][0]["open_count"] = 0
        profile["resource_calendars"][0]["staffed_count"] = 0
        self.assert_issue(profile, "exceed open resource capacity 0")

    def test_synthetic_provenance_is_explicit(self) -> None:
        profile = load_profile()
        profile["provenance"]["class"] = "observed"
        self.assert_issue(profile, "provenance: expected explicit synthetic class")
        profile = load_profile()
        profile["distributions"][0]["provenance_class"] = "unknown"
        self.assert_issue(profile, "distributions[0].provenance_class: expected synthetic")

    def test_unhashable_capacity_identifiers_return_diagnostics(self) -> None:
        profile = load_profile()
        profile["capacity_location"]["staff_roles"][0]["staff_role_id"] = []
        self.assert_issue(profile, "staffing_calendars: unknown staff_role_id=staff.synthetic")

        profile = load_profile()
        profile["initial_state"]["remaining_work"] = [{
            "occupant_id": [],
            "task_class_id": "task.synthetic",
            "remaining_duration": {"value": "1", "source_unit": "s"},
        }]
        self.assert_issue(profile, "initial_state.remaining_work[0].occupant_id: occupant is not present")

        profile = load_profile()
        profile["capacity_location"]["locations"][0]["location_id"] = []
        profile["initial_state"]["occupants"] = [{"occupant_id": "p1", "location_id": []}]
        self.assert_issue(profile, "initial_state.occupants[0].location_id: unknown location")

        profile = load_profile()
        profile["capacity_location"]["task_classes"][0]["task_class_id"] = []
        profile["initial_state"]["occupants"] = [{"occupant_id": "p1", "location_id": "location.work"}]
        profile["initial_state"]["remaining_work"] = [{
            "occupant_id": "p1",
            "task_class_id": [],
            "remaining_duration": {"value": "1", "source_unit": "s"},
        }]
        self.assert_issue(profile, "initial_state.remaining_work[0].task_class_id: unknown task class")

    def test_non_object_has_root_diagnostic(self) -> None:
        self.assertEqual(validate_profile_semantics([]), ["<root>: profile pack must be a JSON object"])


if __name__ == "__main__":
    unittest.main()
