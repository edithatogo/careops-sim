from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tools.validate_ed_profile_pack import load_validator, schema_errors
from tools.validate_ed_profile_pack_semantics import validate_profile_semantics


ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / "model-inputs/ed/profiles/p4-capacity-status-cases.json"
SURGE_PROFILE = ROOT / "model-inputs/ed/profiles/p4-surge-pack.json"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def profile_with_capacity_case(case: dict) -> dict:
    profile = copy.deepcopy(read_json(SURGE_PROFILE))
    for bucket in profile["capacity_location"]["resource_buckets"]:
        if bucket["resource_id"] == case["resource_id"]:
            for field in (
                "physical_status",
                "physical_count",
                "availability_status",
                "open_count",
                "staffed_count",
            ):
                bucket[field] = case[field]
            if "availability_reason" in case:
                bucket["availability_reason"] = case["availability_reason"]
            else:
                bucket.pop("availability_reason", None)
            if "calendar_open_count" in case:
                for row in profile["resource_calendars"]:
                    if row["resource_id"] == case["resource_id"]:
                        row["open_count"] = case["calendar_open_count"]
                        row["staffed_count"] = case["calendar_staffed_count"]
            else:
                profile["resource_calendars"] = [
                    row for row in profile["resource_calendars"]
                    if row["resource_id"] != case["resource_id"]
                ]
            return profile
    raise AssertionError(f"unknown resource in case: {case['resource_id']}")


class ProfileCapacityStatusCaseTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.corpus = read_json(CORPUS)
        cls.validator = load_validator()

    def test_physical_open_and_staffed_counts_are_distinct(self) -> None:
        case = self.corpus["known_capacity_cases"][0]
        profile = profile_with_capacity_case(case)
        bucket = next(
            row for row in profile["capacity_location"]["resource_buckets"]
            if row["resource_id"] == case["resource_id"]
        )
        self.assertEqual((bucket["physical_count"], bucket["open_count"], bucket["staffed_count"]), (2, 2, 1))
        self.assertEqual(schema_errors(profile, self.validator), [])
        self.assertEqual(validate_profile_semantics(profile), [])

    def test_known_closed_bay_remains_physical_but_not_open(self) -> None:
        case = self.corpus["known_capacity_cases"][1]
        profile = profile_with_capacity_case(case)
        bucket = next(
            row for row in profile["capacity_location"]["resource_buckets"]
            if row["resource_id"] == case["resource_id"]
        )
        self.assertEqual(bucket["physical_count"], 2)
        self.assertEqual((bucket["open_count"], bucket["staffed_count"]), (1, 1))
        self.assertEqual(schema_errors(profile, self.validator), [])
        self.assertEqual(validate_profile_semantics(profile), [])

    def test_surge_calendar_changes_known_open_and_staffed_capacity(self) -> None:
        case = self.corpus["surge_calendar_case"]
        profile = copy.deepcopy(read_json(SURGE_PROFILE))
        actual = [
            row for row in profile["resource_calendars"]
            if row["resource_id"] == case["resource_id"]
        ]
        to_expected = lambda row: {
            "start_s": int(row["interval"]["start"]["value"]),
            "end_s": int(row["interval"]["end"]["value"]),
            "open_count": row["open_count"],
            "staffed_count": row["staffed_count"],
        }
        self.assertEqual([to_expected(row) for row in actual], case["expected_intervals"])
        self.assertEqual(schema_errors(profile, self.validator), [])
        self.assertEqual(validate_profile_semantics(profile), [])

    def test_unknown_availability_is_null_reasoned_and_not_runnable(self) -> None:
        case = self.corpus["unknown_status_case"]
        profile = profile_with_capacity_case(case)
        self.assertEqual(schema_errors(profile, self.validator), [])
        bucket = next(
            row for row in profile["capacity_location"]["resource_buckets"]
            if row["resource_id"] == case["resource_id"]
        )
        self.assertEqual(bucket["availability_status"], "unknown")
        self.assertIsNone(bucket["open_count"])
        self.assertIsNone(bucket["staffed_count"])
        self.assertEqual(bucket["availability_reason"], "roster_status_not_confirmed")

        # The structural schema permits unknown evidence, while the semantic
        # profile validator correctly excludes null availability from a runnable
        # simulation profile, whose calendars require numeric capacity.
        semantic_errors = validate_profile_semantics(profile)
        self.assertTrue(any("resource_buckets[3].open_count" in e for e in semantic_errors), semantic_errors)
        self.assertTrue(any("resource_buckets[3].staffed_count" in e for e in semantic_errors), semantic_errors)

        promoted = copy.deepcopy(profile)
        promoted_bucket = next(
            row for row in promoted["capacity_location"]["resource_buckets"]
            if row["resource_id"] == case["resource_id"]
        )
        promoted_bucket["open_count"] = 1
        promoted_bucket["staffed_count"] = 1
        promoted_errors = schema_errors(promoted, self.validator)
        self.assertTrue(any("open_count" in error for error in promoted_errors), promoted_errors)
        self.assertTrue(any("staffed_count" in error for error in promoted_errors), promoted_errors)


if __name__ == "__main__":
    unittest.main()
