import importlib.util
import unittest
from datetime import datetime
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "tools" / "quality_flakes.py"
SPEC = importlib.util.spec_from_file_location("quality_flakes", MODULE_PATH)
quality_flakes = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(quality_flakes)


class QualityFlakesTests(unittest.TestCase):
    def setUp(self):
        self.config = {"schema_version": 1, "flakes": []}
        self.entry = {
            "test_id": "crate::module::test_case",
            "owner": "team@example.test",
            "reason": "Intermittent external scheduling dependency",
            "issue_url": "https://example.test/issues/123",
            "expires_on": "2026-10-05",
        }

    def test_empty_inventory_is_valid_but_does_not_claim_no_flakiness(self):
        self.assertEqual(quality_flakes.check(self.config, "2026-10-05"), [])

    def test_entry_is_valid_on_its_expiry_date(self):
        self.config["flakes"] = [self.entry]
        self.assertEqual(quality_flakes.check(self.config, "2026-10-05"), [])

    def test_entry_is_expired_after_its_expiry_date(self):
        self.config["flakes"] = [self.entry]
        errors = quality_flakes.check(self.config, "2026-10-06")
        self.assertTrue(any("expired" in error for error in errors), errors)

    def test_rejects_missing_and_extra_entry_fields(self):
        self.config["flakes"] = [{**self.entry, "extra": "not allowed"}]
        errors = quality_flakes.check(self.config, "2026-10-05")
        self.assertTrue(any("fields" in error for error in errors), errors)

        self.config["flakes"] = [{key: value for key, value in self.entry.items() if key != "owner"}]
        errors = quality_flakes.check(self.config, "2026-10-05")
        self.assertTrue(any("fields" in error for error in errors), errors)

    def test_rejects_blank_owner(self):
        self.config["flakes"] = [{**self.entry, "owner": "  "}]
        errors = quality_flakes.check(self.config, "2026-10-05")
        self.assertTrue(any("owner" in error for error in errors), errors)

    def test_rejects_non_string_fields(self):
        self.config["flakes"] = [{**self.entry, "reason": 7}]
        errors = quality_flakes.check(self.config, "2026-10-05")
        self.assertTrue(any("reason" in error for error in errors), errors)

    def test_rejects_duplicate_test_ids(self):
        self.config["flakes"] = [self.entry, dict(self.entry)]
        errors = quality_flakes.check(self.config, "2026-10-05")
        self.assertTrue(any("duplicate" in error for error in errors), errors)

    def test_rejects_invalid_expiry_dates(self):
        for expiry in ("2026-02-30", "2026-1-05", "not-a-date"):
            with self.subTest(expiry=expiry):
                self.config["flakes"] = [{**self.entry, "expires_on": expiry}]
                errors = quality_flakes.check(self.config, "2026-10-05")
                self.assertTrue(any("expires_on" in error for error in errors), errors)

    def test_rejects_boolean_or_wrong_schema_version(self):
        for version in (True, 1.0, 2, "1"):
            with self.subTest(version=version):
                self.config["schema_version"] = version
                self.assertTrue(quality_flakes.check(self.config, "2026-10-05"))
        self.config["schema_version"] = 1

    def test_rejects_non_list_flakes_and_non_object_entries(self):
        for flakes in (None, {}, "empty", [None, self.entry]):
            with self.subTest(flakes=flakes):
                self.config["flakes"] = flakes
                self.assertTrue(quality_flakes.check(self.config, "2026-10-05"))

    def test_allows_additional_unrelated_root_fields(self):
        self.config["unrelated_policy"] = {"opaque": True}
        self.assertEqual(quality_flakes.check(self.config, "2026-10-05"), [])

    def test_rejects_invalid_today(self):
        for today in ("2026-02-30", "2026-1-05", "today"):
            with self.subTest(today=today):
                self.assertTrue(quality_flakes.check(self.config, today))

    def test_rejects_datetime_today_without_raising(self):
        self.config["flakes"] = [self.entry]
        errors = quality_flakes.check(self.config, datetime(2026, 10, 5))
        self.assertTrue(any("today" in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main()
