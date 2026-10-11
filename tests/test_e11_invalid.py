"""Independent specification checks for E1 invalid configuration fixtures.

Inputs use integer ticks, one-stage synthetic pathways, resource records with
physical/open/staffed counts, declared task eligibility, and finite stage
records. Rejection is specified before scheduling; tests do not execute an ED
validator or prove runtime transactional behavior.
"""
import copy
import json
import unittest
from pathlib import Path

FIXTURE = Path(__file__).resolve().parents[1] / "conductor/design/ed/e1-invalid.json"

BASE_INPUTS = {
    "horizon": 10, "patient_id": "p1", "patient_ids": ["p1"],
    "arrival_tick": 1, "acuity": 3,
    "resources": {"triage": {"physical_count": 1, "open_count": 1, "staffed_count": 1, "class": "nurse", "zone": "ed"}},
    "tasks": {"triage_task": {"required_class": "nurse", "zone": "ed", "eligible_resources": ["triage"]}},
    "stages": [{"name": "triage", "resource": "triage", "task": "triage_task", "required_class": "nurse", "zone": "ed", "duration": 2}],
}
EXPECTED = {
    "capacity_zero": ({"resources": {"triage": {"physical_count": 0, "open_count": 0, "staffed_count": 0, "class": "nurse", "zone": "ed"}}}, "resources.triage.physical_count", "capacity_invalid", "Resource physical, open and staffed counts must be positive integers."),
    "capacity_unknown": ({"resources": {"triage": {"physical_count": None, "open_count": 1, "staffed_count": 1, "class": "nurse", "zone": "ed"}}}, "resources.triage.physical_count", "capacity_invalid", "Resource physical, open and staffed counts must be positive integers."),
    "capacity_open_zero": ({"resources": {"triage": {"physical_count": 1, "open_count": 0, "staffed_count": 0, "class": "nurse", "zone": "ed"}}}, "resources.triage.open_count", "capacity_invalid", "Resource physical, open and staffed counts must be positive integers."),
    "capacity_staffed_zero": ({"resources": {"triage": {"physical_count": 1, "open_count": 1, "staffed_count": 0, "class": "nurse", "zone": "ed"}}}, "resources.triage.staffed_count", "capacity_invalid", "Resource physical, open and staffed counts must be positive integers."),
    "capacity_staffed_over_open": ({"resources": {"triage": {"physical_count": 3, "open_count": 2, "staffed_count": 3, "class": "nurse", "zone": "ed"}}}, "resources.triage.staffed_count", "capacity_inconsistent", "Staffed count cannot exceed open count, and open count cannot exceed physical count."),
    "capacity_inconsistent": ({"resources": {"triage": {"physical_count": 2, "open_count": 3, "staffed_count": 1, "class": "nurse", "zone": "ed"}}}, "resources.triage.open_count", "capacity_inconsistent", "Staffed count cannot exceed open count, and open count cannot exceed physical count."),
    "resource_unresolved": ({"stages": [{"name": "triage", "resource": "missing", "task": "triage_task", "required_class": "nurse", "zone": "ed", "duration": 2}]}, "stages[0].resource", "resource_unresolved", "Every stage resource reference must resolve."),
    "task_unresolved": ({"stages": [{"name": "triage", "resource": "triage", "task": "missing", "required_class": "nurse", "zone": "ed", "duration": 2}]}, "stages[0].task", "task_unresolved", "Every stage task reference must resolve."),
    "resource_class_incompatible": ({"resources": {"triage": {"physical_count": 1, "open_count": 1, "staffed_count": 1, "class": "porter", "zone": "ed"}}, "stages": [{"name": "triage", "resource": "triage", "task": "triage_task", "required_class": "nurse", "zone": "ed", "duration": 2}]}, "stages[0].required_class", "resource_class_incompatible", "The selected resource class must satisfy the stage requirement."),
    "eligibility_missing": ({"tasks": {"triage_task": {"required_class": "nurse", "zone": "ed", "eligible_resources": []}}}, "stages[0].resource", "eligibility_missing", "The task must declare an eligible resource matching its required class and zone."),
    "acuity_invalid": ({"acuity": 6}, "acuity", "acuity_invalid", "Synthetic acuity must be an integer from 1 through 5."),
    "stage_repeated": ({"stages": [{"name": "triage", "resource": "triage", "task": "triage_task", "required_class": "nurse", "zone": "ed", "duration": 2}, {"name": "triage", "resource": "triage", "task": "triage_task", "required_class": "nurse", "zone": "ed", "duration": 2}]}, "stages", "stage_sequence_invalid", "Finite pathway stages cannot repeat."),
    "stage_disordered": ({"stages": [{"name": "assessment", "resource": "triage", "task": "triage_task", "required_class": "nurse", "zone": "ed", "duration": 2}, {"name": "triage", "resource": "triage", "task": "triage_task", "required_class": "nurse", "zone": "ed", "duration": 2}]}, "stages", "stage_sequence_invalid", "Finite pathway stages must follow triage, assessment, optional diagnostics, treatment order."),
    "patient_id_duplicate": ({"patient_ids": ["p1", "p1"]}, "patient_ids", "patient_identity_invalid", "Patient identifiers must be unique and stable."),
    "horizon_nonpositive": ({"horizon": 0}, "horizon", "time_domain_invalid", "Horizon must be a positive integer."),
    "arrival_at_horizon": ({"arrival_tick": 10}, "arrival_tick", "arrival_out_of_range", "Arrivals must be in the half-open interval [0, horizon)."),
    "duration_nonpositive": ({"stages": [{"name": "triage", "resource": "triage", "task": "triage_task", "required_class": "nurse", "zone": "ed", "duration": 0}]}, "stages[0].duration", "duration_invalid", "Stage work durations must be positive."),
}
REQUIRED_CASES = {"invalid." + key for key in EXPECTED}


def merged_inputs(patch):
    inputs = copy.deepcopy(BASE_INPUTS)
    for key, value in patch.items():
        inputs[key] = copy.deepcopy(value)
    return inputs


def semantic_diagnostic(inputs):
    """Return the first independently modeled rejection, or None if valid."""
    horizon = inputs["horizon"]
    if type(horizon) is not int or horizon <= 0:
        return "time_domain_invalid", "horizon"
    for rid, resource in inputs["resources"].items():
        physical, opened, staffed = (resource.get(k) for k in ("physical_count", "open_count", "staffed_count"))
        if type(physical) is not int or physical <= 0:
            return "capacity_invalid", f"resources.{rid}.physical_count"
        if type(opened) is not int or opened <= 0:
            return "capacity_invalid", f"resources.{rid}.open_count"
        if type(staffed) is not int or staffed <= 0:
            return "capacity_invalid", f"resources.{rid}.staffed_count"
        if opened > physical:
            return "capacity_inconsistent", f"resources.{rid}.open_count"
        if staffed > opened:
            return "capacity_inconsistent", f"resources.{rid}.staffed_count"
    if not inputs["patient_ids"] or len(set(inputs["patient_ids"])) != len(inputs["patient_ids"]):
        return "patient_identity_invalid", "patient_ids"
    if type(inputs["acuity"]) is not int or not 1 <= inputs["acuity"] <= 5:
        return "acuity_invalid", "acuity"
    if type(inputs["arrival_tick"]) is not int or not 0 <= inputs["arrival_tick"] < horizon:
        return "arrival_out_of_range", "arrival_tick"
    names = [stage["name"] for stage in inputs["stages"]]
    if len(set(names)) != len(names) or names != sorted(names, key={"triage": 0, "assessment": 1, "diagnostics": 2, "treatment": 3}.get):
        return "stage_sequence_invalid", "stages"
    for index, stage in enumerate(inputs["stages"]):
        if stage["task"] not in inputs["tasks"]:
            return "task_unresolved", f"stages[{index}].task"
        if stage["resource"] not in inputs["resources"]:
            return "resource_unresolved", f"stages[{index}].resource"
        resource = inputs["resources"][stage["resource"]]
        if stage["required_class"] != resource["class"] or stage["zone"] != resource["zone"]:
            return "resource_class_incompatible", f"stages[{index}].required_class"
        task = inputs["tasks"][stage["task"]]
        if stage["resource"] not in task["eligible_resources"] or task["required_class"] != stage["required_class"] or task["zone"] != stage["zone"]:
            return "eligibility_missing", f"stages[{index}].resource"
        if type(stage["duration"]) is not int or stage["duration"] <= 0:
            return "duration_invalid", f"stages[{index}].duration"
    return None


def find_case(doc, name):
    return next(c for c in doc["cases"] if c["case_id"] == "invalid." + name)


def repaired_inputs(key, inputs):
    """Return a copy with this fixture's single invalid condition corrected."""
    value = copy.deepcopy(inputs)
    resource = value["resources"].get("triage")
    if key == "capacity_zero": resource.update(physical_count=1, open_count=1, staffed_count=1)
    elif key == "capacity_unknown": resource.update(physical_count=1)
    elif key == "capacity_open_zero": resource.update(open_count=1, staffed_count=1)
    elif key == "capacity_staffed_zero": resource.update(staffed_count=1)
    elif key == "capacity_inconsistent": resource.update(open_count=2)
    elif key == "capacity_staffed_over_open": resource.update(staffed_count=2)
    elif key == "resource_unresolved": value["stages"][0]["resource"] = "triage"
    elif key == "task_unresolved": value["stages"][0]["task"] = "triage_task"
    elif key == "resource_class_incompatible": resource["class"] = "nurse"
    elif key == "eligibility_missing": value["tasks"]["triage_task"]["eligible_resources"] = ["triage"]
    elif key == "acuity_invalid": value["acuity"] = 3
    elif key in ("stage_repeated", "stage_disordered"): value["stages"] = [copy.deepcopy(BASE_INPUTS["stages"][0])]
    elif key == "patient_id_duplicate": value["patient_ids"] = ["p1"]
    elif key == "horizon_nonpositive": value["horizon"] = 10
    elif key == "arrival_at_horizon": value["arrival_tick"] = 9
    elif key == "duration_nonpositive": value["stages"][0]["duration"] = 2
    return value


def assert_fixture_valid(testcase, doc):
    testcase.assertEqual(set(doc), {"fixture_version", "family", "provenance", "time_unit", "runtime_status", "cases"})
    testcase.assertEqual((doc["fixture_version"], doc["family"], doc["provenance"], doc["time_unit"], doc["runtime_status"]), (1, "invalid", "synthetic_test_only", "tick", "unbound"))
    testcase.assertEqual({c["case_id"] for c in doc["cases"]}, REQUIRED_CASES)
    testcase.assertEqual(len(doc["cases"]), len(REQUIRED_CASES))
    for case in doc["cases"]:
        testcase.assertEqual(set(case), {"case_id", "inputs", "expected", "runtime_owner", "purpose"})
        key = case["case_id"].removeprefix("invalid.")
        patch, path, category, detail = EXPECTED[key]
        inputs = merged_inputs(patch)
        testcase.assertEqual(case["inputs"], inputs, key)
        testcase.assertEqual(semantic_diagnostic(case["inputs"]), (category, path), key)
        testcase.assertEqual(case["expected"], {"validation": "reject_before_scheduling", "diagnostic_category": category, "offending_path": path, "invariants": ["no_events_scheduled", "no_patient_state_created"], "detail": detail}, key)
        testcase.assertIsNone(semantic_diagnostic(repaired_inputs(key, case["inputs"])), key + " should be invalid only for its stated condition")
        testcase.assertEqual(case["runtime_owner"], "E1.2", key)
        testcase.assertIn(key, case["purpose"], key)
        testcase.assertIn("synthetic invalid-input oracle", case["purpose"].lower(), key)


class InvalidFixtureTests(unittest.TestCase):
    def setUp(self):
        self.doc = json.loads(FIXTURE.read_text())

    def test_complete_invalid_family_matches_independent_semantic_expectations(self):
        assert_fixture_valid(self, self.doc)
        self.assertIsNone(semantic_diagnostic(copy.deepcopy(BASE_INPUTS)), "declared base inputs must be valid")
        for path, value in (("horizon", True), ("arrival_tick", True), ("acuity", True)):
            mutated = copy.deepcopy(BASE_INPUTS); mutated[path] = value
            self.assertIsNotNone(semantic_diagnostic(mutated), path + " booleans are not integer values")
        for field in ("physical_count", "open_count", "staffed_count"):
            mutated = copy.deepcopy(BASE_INPUTS); mutated["resources"]["triage"][field] = True
            self.assertIsNotNone(semantic_diagnostic(mutated), field + " booleans are not integer values")
        mutated = copy.deepcopy(BASE_INPUTS); mutated["stages"][0]["duration"] = True
        self.assertEqual(semantic_diagnostic(mutated), ("duration_invalid", "stages[0].duration"))

    def test_mutations_to_payload_and_validation_inputs_are_rejected(self):
        mutations = [
            lambda d: find_case(d, "capacity_inconsistent")["inputs"]["resources"]["triage"].update(physical_count=2, open_count=2, staffed_count=1),
            lambda d: find_case(d, "capacity_staffed_over_open")["inputs"]["resources"]["triage"].update(staffed_count=2),
            lambda d: find_case(d, "capacity_unknown")["inputs"]["resources"]["triage"].update(physical_count=1),
            lambda d: find_case(d, "capacity_zero")["inputs"]["resources"]["triage"].update(open_count=1),
            lambda d: find_case(d, "capacity_open_zero")["inputs"]["resources"]["triage"].update(open_count=1, staffed_count=1),
            lambda d: find_case(d, "capacity_staffed_zero")["inputs"]["resources"]["triage"].update(staffed_count=1),
            lambda d: find_case(d, "resource_unresolved")["inputs"]["stages"][0].update(resource="triage"),
            lambda d: find_case(d, "task_unresolved")["inputs"]["stages"][0].update(task="triage_task"),
            lambda d: find_case(d, "duration_nonpositive")["inputs"]["stages"][0].update(duration=1),
        ]
        for mutate in mutations:
            changed = copy.deepcopy(self.doc)
            mutate(changed)
            with self.subTest(mutation=mutate):
                with self.assertRaises(AssertionError):
                    assert_fixture_valid(self, changed)

    def test_mutations_to_endpoints_counts_identity_category_invariants_and_binding_are_rejected(self):
        mutations = [
            lambda d: find_case(d, "capacity_inconsistent")["inputs"]["resources"]["triage"].update(open_count=2),
            lambda d: d["cases"].pop(),
            lambda d: find_case(d, "capacity_zero").update(case_id="invalid.changed"),
            lambda d: find_case(d, "capacity_zero")["expected"].update(diagnostic_category="wrong"),
            lambda d: find_case(d, "capacity_zero")["expected"].update(invariants=["no_events_scheduled"]),
            lambda d: find_case(d, "capacity_zero").update(runtime_owner="E1.3"),
            lambda d: d.update(time_unit="hour"),
            lambda d: d.update(provenance="observed"),
            lambda d: d.update(runtime_status="implemented"),
            lambda d: find_case(d, "capacity_open_zero")["inputs"]["resources"]["triage"].update(open_count=1),
        ]
        for mutate in mutations:
            changed = copy.deepcopy(self.doc)
            mutate(changed)
            with self.subTest(mutation=mutate):
                with self.assertRaises(AssertionError):
                    assert_fixture_valid(self, changed)


if __name__ == "__main__":
    unittest.main()
