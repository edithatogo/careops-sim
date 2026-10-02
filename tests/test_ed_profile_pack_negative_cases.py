from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path
from typing import Any

from tools.validate_ed_profile_pack import load_validator, schema_errors

ROOT = Path(__file__).resolve().parents[1]
CORPUS_PATH = ROOT / "model-inputs/ed/profiles/p4-validation-cases.json"
BASE_PROFILE_PATH = ROOT / "model-inputs/ed/profiles/p4-minimal-pack.json"


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def set_path(record: dict[str, Any], path: list[str | int], value: Any) -> None:
    target: Any = record
    for part in path[:-1]:
        target = target[part]
    target[path[-1]] = value


def apply_mutation(record: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(record)
    mutation = case["mutation"]
    path = mutation["path"]
    if mutation["operation"] == "remove":
        target: Any = result
        for part in path[:-1]:
            target = target[part]
        del target[path[-1]]
    elif mutation["operation"] == "set":
        set_path(result, path, mutation["value"])
    else:
        raise AssertionError(f"unsupported corpus mutation: {mutation['operation']}")
    return result


class ProfilePackNegativeCaseTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.corpus = read_json(CORPUS_PATH)
        cls.base_profile = read_json(BASE_PROFILE_PATH)
        cls.fixture = read_json(ROOT / cls.corpus["observation_semantics"]["source_fixture"])
        cls.validator = load_validator()
        cls.cases = {case["case_id"]: case for case in cls.corpus["structural_cases"]}

    def test_missing_required_field_fails_and_is_distinct_from_zero(self) -> None:
        missing_case = self.cases["missing-required-seed"]
        missing_profile = apply_mutation(self.base_profile, missing_case)
        errors = schema_errors(missing_profile, self.validator)
        self.assertNotIn("seed", missing_profile)
        self.assertFalse(missing_case["expected"]["valid"])
        self.assertTrue(any("<root> (required constraint)" in error for error in errors), errors)

        zero_case = self.cases["explicit-zero-arrival-count"]
        zero_profile = apply_mutation(self.base_profile, zero_case)
        self.assertEqual(zero_profile["arrival_table"][1]["counts_by_mode"]["ambulance"], 0)
        self.assertTrue(zero_case["expected"]["valid"])
        self.assertEqual(schema_errors(zero_profile, self.validator), [])

    def test_invalid_clock_unit_fails_structural_validation(self) -> None:
        case = self.cases["invalid-clock-unit"]
        profile = apply_mutation(self.base_profile, case)
        errors = schema_errors(profile, self.validator)
        self.assertEqual(profile["horizon"]["source_unit"], "minute")
        self.assertFalse(case["expected"]["valid"])
        self.assertTrue(any("horizon.source_unit (const constraint)" in error for error in errors), errors)

    def test_right_censored_follow_up_is_not_a_completed_event(self) -> None:
        semantics = self.corpus["observation_semantics"]
        case = self.fixture["cases"][semantics["right_censor_case_id"]]
        expected = semantics["right_censor_expected"]
        censored = [interval for interval in case["intervals"] if interval["status"] == "right_censored"]
        self.assertEqual(len(censored), 1)
        self.assertEqual(censored[0]["event_indicator"], 0)
        observed_event_count = sum(
            interval["event_indicator"] == 1 and interval["status"] == "observed_event"
            for interval in case["intervals"]
        )
        total_follow_up = sum(interval["elapsed_time"] for interval in case["intervals"])
        naive_completed_event_count = len(case["intervals"])
        self.assertEqual(observed_event_count, expected["observed_event_count"])
        self.assertEqual(total_follow_up, expected["total_follow_up"])
        self.assertEqual(naive_completed_event_count, expected["naive_completed_event_count"])
        self.assertEqual(case["expected"]["observed_event_count"], expected["observed_event_count"])
        self.assertEqual(case["expected"]["total_follow_up"], expected["total_follow_up"])
        self.assertEqual(case["expected"]["naive_completed_event_count"], expected["naive_completed_event_count"])
        self.assertTrue(case["expected"]["naive_count_is_invalid"])
        self.assertEqual(expected["naive_completed_event_count"], 3)
        self.assertNotEqual(expected["observed_event_count"], expected["naive_completed_event_count"])

    def test_ambiguous_status_remains_unresolved(self) -> None:
        semantics = self.corpus["observation_semantics"]
        case = self.fixture["cases"][semantics["ambiguous_status_case_id"]]
        self.assertEqual(case["interval"]["status"], "ambiguous")
        self.assertIsNone(case["interval"]["event_indicator"])
        self.assertEqual(case["expected_classification"], semantics["ambiguous_status_expected_classification"])


if __name__ == "__main__":
    unittest.main()
