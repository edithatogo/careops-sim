import csv
import tempfile
import unittest
from pathlib import Path

from tools.validate_p02_disposition import ACCEPT_ROWS, DEFER_ROWS, validate


ROOT = Path(__file__).resolve().parents[1]
CANDIDATE = ROOT / "model-inputs/ed/schema/p0.2-owner-review-response-candidate.csv"


class P02DispositionTests(unittest.TestCase):
    def setUp(self):
        with CANDIDATE.open(newline="", encoding="utf-8") as handle:
            reader = csv.DictReader(handle)
            self.fields = reader.fieldnames
            self.rows = list(reader)
        for index, row in enumerate(self.rows):
            row_number = index + 1
            if row_number in ACCEPT_ROWS:
                status, detail = "proposed_accept", "retain current proposed mapping"
            elif row_number in DEFER_ROWS:
                status, detail = "proposed_defer", "trigger: resolve named prerequisite before profile use"
            else:
                status, detail = "proposed_revise", "replacement: use revised row-specific contract"
            row["coordinator_resolution"] = f"{status}: {row['parameter_id']} {detail}"
            row["resolved_at"] = ""
            row["reviewed_at"] = "2026-09-30"

    def check_rows(self, rows):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "disposition.csv"
            with output.open("w", newline="", encoding="utf-8") as handle:
                writer = csv.DictWriter(handle, fieldnames=self.fields)
                writer.writeheader()
                writer.writerows(rows)
            return validate(CANDIDATE, output)

    def test_valid_full_registry_response(self):
        self.assertEqual(self.check_rows(self.rows), [])

    def test_missing_row_is_rejected(self):
        errors = self.check_rows(self.rows[:-1])
        self.assertTrue(any("exactly 101" in error for error in errors), errors)

    def test_reviewer_drift_is_rejected(self):
        rows = [dict(row) for row in self.rows]
        rows[0]["E0_rationale"] += " Altered."
        errors = self.check_rows(rows)
        self.assertTrue(any("reviewer field E0_rationale changed" in error for error in errors), errors)

    def test_blank_resolution_is_rejected(self):
        rows = [dict(row) for row in self.rows]
        rows[0]["coordinator_resolution"] = "proposed_accept: "
        errors = self.check_rows(rows)
        self.assertTrue(any("needs a proposed_accept/revise/defer prefix" in error for error in errors), errors)

    def test_false_accepted_status_is_rejected(self):
        rows = [dict(row) for row in self.rows]
        rows[0]["coordinator_resolution"] = "accepted: looks good"
        errors = self.check_rows(rows)
        self.assertTrue(any("needs a proposed_accept/revise/defer prefix" in error for error in errors), errors)

    def test_wrong_status_counts_are_rejected(self):
        rows = [dict(row) for row in self.rows]
        rows[0]["coordinator_resolution"] = (
            f"proposed_revise: {rows[0]['parameter_id']} replacement: revised contract"
        )
        errors = self.check_rows(rows)
        self.assertTrue(any("proposed_accept count must be 30" in error for error in errors), errors)
        self.assertTrue(any("proposed_revise count must be 60" in error for error in errors), errors)

    def test_status_swap_with_same_counts_is_rejected(self):
        rows = [dict(row) for row in self.rows]
        first = rows[0]["coordinator_resolution"].replace("proposed_accept:", "proposed_revise:", 1)
        second = rows[1]["coordinator_resolution"].replace("proposed_revise:", "proposed_accept:", 1)
        rows[0]["coordinator_resolution"] = first + "; replacement: revised contract"
        rows[1]["coordinator_resolution"] = second
        errors = self.check_rows(rows)
        self.assertTrue(any("row 1 (ed.demandcasemix.arrivals) must use proposed_accept" in error for error in errors), errors)
        expected = f"row 2 ({rows[1]['parameter_id']}) must use proposed_revise"
        self.assertTrue(any(expected in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main()
