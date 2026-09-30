from __future__ import annotations

import csv
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import p02_merge_panel as merge


class P02MergePanelTests(unittest.TestCase):
    def test_bound_inputs_and_each_role_cover_template_ids(self) -> None:
        hashes = merge.validate_inputs()
        self.assertEqual(len(hashes), 7)
        fields, rows = merge.normalize()
        self.assertEqual(len(rows), 101)
        self.assertEqual(len({row["parameter_id"] for row in rows}), 101)
        self.assertIn("E0_replacement", fields)
        self.assertIn("C0_evidence_locator", fields)
        for role in ("E0", "C0"):
            self.assertTrue(all(row[f"{role}_disposition"] for row in rows))
            self.assertTrue(all(row[f"{role}_rationale"] for row in rows))
            self.assertTrue(all(row[f"{role}_evidence_locator"] for row in rows))

    def test_supplement_recommendations_win_and_replacement_is_preserved(self) -> None:
        _, rows = merge.normalize()
        by_id = {row["parameter_id"]: row for row in rows}
        self.assertEqual(
            by_id["ed.demandcasemix.calendarprofile"]["E0_disposition"], "revise"
        )
        self.assertIn(
            "optional modifier",
            by_id["ed.demandcasemix.calendarprofile"]["E0_replacement"],
        )
        self.assertEqual(
            by_id["ed.durations.boarding"]["C0_disposition"], "revise"
        )
        self.assertIn(
            "residual boundary delay",
            by_id["ed.durations.boarding"]["C0_replacement"],
        )

    def test_csv_output_is_template_shaped_and_does_not_resolve_coordinator_fields(self) -> None:
        fields, rows = merge.normalize()
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "candidate.csv"
            with path.open("w", newline="", encoding="utf-8") as stream:
                writer = csv.DictWriter(stream, fieldnames=fields, lineterminator="\n")
                writer.writeheader()
                writer.writerows(rows)
            with path.open(newline="", encoding="utf-8") as stream:
                written = list(csv.DictReader(stream))
            checked = merge.validate_candidate_file(
                path, [row["parameter_id"] for row in rows], rows
            )
        self.assertEqual(len(written), 101)
        self.assertEqual([row["parameter_id"] for row in checked], [row["parameter_id"] for row in rows])
        self.assertTrue(all(not row["coordinator_resolution"] for row in written))
        self.assertTrue(all(not row["coordinator_id"] for row in written))
        self.assertEqual(written[0]["E0_base_commit"], merge.BASE_COMMIT)
        self.assertEqual(written[0]["C0_base_commit"], merge.BASE_COMMIT)

    def test_csv_readback_rejects_reordered_ids_and_incomplete_revisions(self) -> None:
        fields, rows = merge.normalize()
        expected_ids = [row["parameter_id"] for row in rows]
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "candidate.csv"
            reordered = [rows[1], rows[0], *rows[2:]]
            with path.open("w", newline="", encoding="utf-8") as stream:
                writer = csv.DictWriter(stream, fieldnames=fields, lineterminator="\n")
                writer.writeheader()
                writer.writerows(reordered)
            with self.assertRaisesRegex(ValueError, "101 IDs in order"):
                merge.validate_candidate_file(path, expected_ids)

            incomplete = [dict(row) for row in rows]
            revise_row = next(row for row in incomplete if row["E0_disposition"] == "revise")
            revise_row["E0_replacement"] = ""
            with path.open("w", newline="", encoding="utf-8") as stream:
                writer = csv.DictWriter(stream, fieldnames=fields, lineterminator="\n")
                writer.writeheader()
                writer.writerows(incomplete)
            with self.assertRaisesRegex(ValueError, "revise disposition without replacement"):
                merge.validate_candidate_file(path, expected_ids)


if __name__ == "__main__":
    unittest.main()
