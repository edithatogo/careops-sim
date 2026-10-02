from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path

from tools.validate_ed_profile_pack import _read_json, load_validator, schema_errors

ROOT = Path(__file__).resolve().parents[1]
CAPACITY_FIXTURE = ROOT / "model-inputs/ed/profiles/p4-constrained-surge.json"


def valid_pack() -> dict:
    capacity = json.loads(CAPACITY_FIXTURE.read_text(encoding="utf-8"))["capacity_location"]
    return {
        "profile_pack_schema_version": 1,
        "profile_id": "fixture.minimal",
        "seed": 0,
        "horizon": {"value": "3600.00", "source_unit": "s"},
        "provenance": {"class": "synthetic", "notes": "Invented structural test values."},
        "arrival_table": [{
            "interval": {"start": {"value": "0", "source_unit": "s"}, "end": {"value": "60", "source_unit": "s"}},
            "counts_by_mode": {"walk_in": 1, "ambulance": 0},
        }],
        "resource_calendars": [{
            "resource_id": "resource.synthetic-pool",
            "interval": {"start": {"value": "0", "source_unit": "s"}, "end": {"value": "3600", "source_unit": "s"}},
            "open_count": 1, "staffed_count": 1,
        }],
        "staffing_calendars": [{
            "staff_role_id": "staff.synthetic",
            "interval": {"start": {"value": "0", "source_unit": "s"}, "end": {"value": "3600", "source_unit": "s"}},
            "present_count": 1,
        }],
        "capacity_location": capacity,
        "route_graph": {
            "nodes": [{"node_id": "arrival", "location_id": "location.arrival"}, {"node_id": "work", "location_id": "location.work"}],
            "edges": [{"from_node_id": "arrival", "to_node_id": "work", "distance": {"value": 14, "unit": "m"}}],
        },
        "distributions": [{
            "distribution_id": "work.duration", "unit": "s", "provenance_class": "synthetic",
            "outcomes": [{"value": "30.0", "probability": 1}],
        }],
        "initial_state": {"at": {"value": "0", "source_unit": "s"}, "occupants": [], "remaining_work": []},
    }


class ProfilePackSchemaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.validator = load_validator()

    def test_valid_pack_including_nested_capacity_location_schema(self):
        self.assertEqual(schema_errors(valid_pack(), self.validator), [])

    def test_unknown_fields_are_rejected(self):
        record = valid_pack()
        record["runtime_command"] = "careops-ed run"
        self.assertTrue(schema_errors(record, self.validator))

    def test_time_requires_decimal_seconds(self):
        record = valid_pack()
        record["arrival_table"][0]["interval"]["start"]["source_unit"] = "tick"
        self.assertTrue(schema_errors(record, self.validator))

    def test_occupant_without_active_remaining_work_is_valid(self):
        record = valid_pack()
        record["initial_state"]["occupants"] = [{
            "occupant_id": "occ-waiting", "location_id": "location.arrival",
        }]
        self.assertEqual(schema_errors(record, self.validator), [])

    def test_invalid_nested_capacity_location_is_rejected(self):
        record = valid_pack()
        record["capacity_location"]["unexpected"] = True
        self.assertTrue(schema_errors(record, self.validator))

    def test_duplicate_json_keys_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "duplicate.json"
            path.write_text('{"profile_pack_schema_version": 1, "profile_pack_schema_version": 1}', encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "duplicate JSON object key"):
                _read_json(path, "test")


if __name__ == "__main__":
    unittest.main()
