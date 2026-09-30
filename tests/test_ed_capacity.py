import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools" / "validate_ed_capacity.py"
SCHEMA = ROOT / "model-inputs/ed/schema/capacity-location.schema.json"
EXAMPLES = ROOT / "model-inputs/ed/schema/capacity-location-examples.json"


class EDCapacityValidationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.examples = json.loads(EXAMPLES.read_text(encoding="utf-8"))
        cls.valid_records = [case["record"] for case in cls.examples["cases"]]

    def invoke(self, record):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "record.json"
            path.write_text(json.dumps(record), encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(TOOL), "--schema", str(SCHEMA), "--record", str(path)],
                cwd=ROOT, text=True, capture_output=True, check=False,
            )

    def test_existing_synthetic_examples_pass(self):
        result = subprocess.run(
            [sys.executable, str(TOOL), "--schema", str(SCHEMA), "--examples", str(EXAMPLES)],
            cwd=ROOT, text=True, capture_output=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("validated 3 capacity/location example cases", result.stdout)

    def test_staffed_capacity_cannot_exceed_open_capacity(self):
        record = copy.deepcopy(self.valid_records[0])
        record["capacity"]["staffed_count"] = 11
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("staffed_count must not exceed", result.stderr)

    def test_open_capacity_cannot_exceed_known_physical_capacity(self):
        record = copy.deepcopy(self.valid_records[0])
        record["capacity"]["open_count"] = 13
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("open_count must not exceed", result.stderr)

    def test_duplicate_location_ids_fail(self):
        record = copy.deepcopy(self.valid_records[0])
        record["locations"].append(copy.deepcopy(record["locations"][0]))
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("duplicates 'loc-triage'", result.stderr)

    def test_duplicate_zone_ids_fail(self):
        record = copy.deepcopy(self.valid_records[0])
        record["zones"].append(copy.deepcopy(record["zones"][0]))
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("zones[2].zone_id duplicates 'zone-entry'", result.stderr)

    def test_location_with_unknown_zone_fails(self):
        record = copy.deepcopy(self.valid_records[0])
        record["locations"][0]["zone_id"] = "zone-missing"
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("references unknown zone 'zone-missing'", result.stderr)

    def test_route_to_unknown_location_fails(self):
        record = copy.deepcopy(self.valid_records[2])
        record["routes"][0]["to_location_id"] = "loc-missing"
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("references unknown location 'loc-missing'", result.stderr)

    def test_macro_record_without_routes_is_valid(self):
        record = copy.deepcopy(self.valid_records[0])
        self.assertNotIn("routes", record)
        result = self.invoke(record)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_duplicate_resource_task_and_role_ids_fail(self):
        for collection, key in (("resource_buckets", "resource_id"), ("task_classes", "task_class_id"),
                                ("staff_roles", "staff_role_id")):
            with self.subTest(collection=collection):
                record = copy.deepcopy(self.valid_records[0])
                record[collection].append(copy.deepcopy(record[collection][0]))
                result = self.invoke(record)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("duplicates", result.stderr)
                self.assertIn(key, result.stderr)

    def test_resource_location_zone_must_match(self):
        record = copy.deepcopy(self.valid_records[0])
        record["resource_buckets"][0]["zone_id"] = "zone-entry"
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("does not match its location zone", result.stderr)

    def test_resource_and_eligibility_references_must_resolve(self):
        record = copy.deepcopy(self.valid_records[0])
        record["resource_buckets"][0]["location_id"] = "loc-missing"
        record["eligibility"][0]["staff_role_id"] = "role-missing"
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("references unknown location 'loc-missing'", result.stderr)
        self.assertIn("references unknown role 'role-missing'", result.stderr)

    def test_unknown_eligibility_task_and_out_of_zone_location_fail(self):
        record = copy.deepcopy(self.valid_records[0])
        record["eligibility"][0]["task_class_id"] = "task-missing"
        record["eligibility"][0]["location_id"] = "loc-triage"
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("references unknown task 'task-missing'", result.stderr)
        self.assertIn("location_id is outside its zone", result.stderr)

    def test_resource_capacity_ordering_is_enforced(self):
        record = copy.deepcopy(self.valid_records[0])
        record["resource_buckets"][0]["staffed_count"] = 6
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("staffed_count must not exceed open_count", result.stderr)

    def test_staffed_above_open_fails_when_physical_count_is_unknown(self):
        record = copy.deepcopy(self.valid_records[0])
        bucket = record["resource_buckets"][0]
        bucket.update(physical_status="unknown", physical_count=None,
                      availability_status="known", open_count=3, staffed_count=4)
        bucket.pop("availability_reason", None)
        record["capacity"].update(bucket_completeness="incomplete", physical_status="unknown",
                                  physical_count=None, availability_status="known", open_count=8,
                                  staffed_count=8)
        record["capacity"].pop("availability_reason", None)
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("staffed_count must not exceed open_count", result.stderr)

    def test_complete_known_treatment_buckets_must_match_aggregate(self):
        record = copy.deepcopy(self.valid_records[0])
        record["capacity"]["physical_count"] = 13
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("physical_count must equal the complete treatment_space bucket sum", result.stderr)

    def test_complete_known_aggregate_requires_treatment_buckets_and_known_counts(self):
        record = copy.deepcopy(self.valid_records[0])
        record["resource_buckets"] = [r for r in record["resource_buckets"]
                                      if r["resource_class"] != "treatment_space"]
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("treatment_space buckets are absent", result.stderr)

        record = copy.deepcopy(self.valid_records[0])
        bucket = record["resource_buckets"][0]
        bucket.update(physical_status="unknown", physical_count=None,
                      availability_status="unknown", open_count=None,
                      staffed_count=None, availability_reason="Unknown bucket counts.")
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("has unknown physical_count in complete set", result.stderr)
        self.assertIn("has unknown open_count in complete set", result.stderr)

    def test_non_treatment_buckets_are_excluded_from_aggregate(self):
        record = copy.deepcopy(self.valid_records[0])
        # This is an additional monitor bucket; it must not contribute to the
        # department treatment-space aggregate.
        record["resource_buckets"][2].update(physical_count=20, open_count=20, staffed_count=20)
        result = self.invoke(record)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_unknown_required_resource_capacity_fails_task_sufficiency(self):
        record = copy.deepcopy(self.valid_records[0])
        for bucket in record["resource_buckets"][:2]:
            bucket.update(physical_status="unknown", physical_count=None,
                          availability_status="unknown", open_count=None,
                          staffed_count=None, availability_reason="Unknown required capacity.")
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("no known staffed 'treatment_space' capacity", result.stderr)

    def test_location_scoped_task_requires_resource_at_that_location(self):
        record = copy.deepcopy(self.valid_records[0])
        record["task_classes"][0]["required_resource_classes"] = ["monitor"]
        record["eligibility"][0]["location_id"] = "loc-treatment-b"
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("no known staffed 'monitor' capacity at location 'loc-treatment-b'", result.stderr)

    def test_unknown_required_staff_capacity_fails_task_sufficiency(self):
        record = copy.deepcopy(self.valid_records[0])
        record["staff_roles"][0]["counts"][0].update(status="unknown", count=None,
                                                       reason="Unknown present capacity.")
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("no known present or task_eligible staff", result.stderr)

    def test_required_task_without_eligibility_scope_fails(self):
        record = copy.deepcopy(self.valid_records[0])
        record["eligibility"] = []
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("requires resources but has no eligibility scope", result.stderr)

    def test_known_staff_at_other_location_cannot_mask_unknown_staff(self):
        record = copy.deepcopy(self.valid_records[0])
        record["eligibility"][0]["location_id"] = "loc-treatment-a"
        record["staff_roles"].append({
            "staff_role_id": "role-unknown-b",
            "name": "Unknown location B staff",
            "counts": [{
                "basis": "present", "status": "unknown", "count": None,
                "reference": "synthetic-initial-state", "reason": "Unknown effective count."
            }]
        })
        record["eligibility"].append({
            "staff_role_id": "role-unknown-b", "task_class_id": "task-assessment",
            "zone_id": "zone-treatment", "location_id": "loc-treatment-b"
        })
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("no known present or task_eligible staff at location 'loc-treatment-b'", result.stderr)

    def test_task_reference_to_unknown_resource_class_fails(self):
        record = copy.deepcopy(self.valid_records[0])
        record["task_classes"][0]["required_resource_classes"] = ["imaging_device"]
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("requires unknown resource class 'imaging_device'", result.stderr)

    def test_unknown_capacity_remains_unknown_and_is_not_zero(self):
        record = copy.deepcopy(self.valid_records[1])
        result = self.invoke(record)
        self.assertEqual(result.returncode, 0, result.stderr)

        record["capacity"]["open_count"] = 0
        result = self.invoke(record)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("schema violation", result.stderr)


if __name__ == "__main__":
    unittest.main()
