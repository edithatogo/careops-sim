import copy
import csv
import json
import unittest
from pathlib import Path

from tools.validate_p02_matrix_drafts import CHUNKS, validate, validate_final


ROOT = Path(__file__).resolve().parents[1]


def inputs():
    folder = ROOT / "conductor/evidence/p0.2-matrix-drafts"
    chunks = [json.loads((folder / name).read_text()) for _, _, name in CHUNKS]
    registry = json.loads((ROOT / "model-inputs/ed/schema/parameter-ids.json").read_text())
    proposed = json.loads((ROOT / "model-inputs/ed/schema/parameter-usage-matrix-proposal.json").read_text())
    with (ROOT / "model-inputs/ed/schema/p0.2-owner-review-disposition.csv").open(newline="") as file:
        disposition = list(csv.DictReader(file))
    return registry, proposed, disposition, chunks


class StructuredMatrixDraftTests(unittest.TestCase):
    def setUp(self):
        self.data = inputs()

    def test_current_draft_has_full_ordered_coverage(self):
        self.assertEqual([], validate(*self.data))

    def test_missing_row_or_swapped_id_fails(self):
        data = copy.deepcopy(self.data)
        data[3][0]["entries"].pop()
        self.assertTrue(any("wrong row count" in error for error in validate(*data)))
        data = copy.deepcopy(self.data)
        data[3][0]["entries"][0]["parameter_id"] = "ed.wrong"
        self.assertTrue(any("ID/order mismatch" in error for error in validate(*data)))

    def test_observed_boarding_cannot_be_relabelled_service_input(self):
        data = copy.deepcopy(self.data)
        data[3][1]["entries"][1]["value_role"] = "generative"
        self.assertTrue(any("observed boarding boundary" in error for error in validate(*data)))

    def test_seed_and_location_cannot_be_deferred_past_mvp(self):
        for chunk_index, entry_index in ((1, 28), (2, 20)):
            data = copy.deepcopy(self.data)
            data[3][chunk_index]["entries"][entry_index]["profile_stage"] = "hardened_v1"
            self.assertTrue(validate(*data))

    def test_proposal_boilerplate_and_active_deferred_row_fail(self):
        data = copy.deepcopy(self.data)
        data[3][0]["entries"][0]["profile_use"] = "candidate input for deterministic/minimal profiles"
        self.assertTrue(any("proposal boilerplate" in error for error in validate(*data)))
        data = copy.deepcopy(self.data)
        data[3][0]["entries"][7]["profile_stage"] = "MVP_conditional"
        self.assertTrue(any("deferred input appears active" in error for error in validate(*data)))

    def test_conditional_use_cannot_be_minimum_or_share_generic_gate(self):
        data = copy.deepcopy(self.data)
        data[3][0]["entries"][3]["profile_stage"] = "MVP_minimum"
        self.assertTrue(any("minimum stage conflicts" in error for error in validate(*data)))
        data = copy.deepcopy(self.data)
        data[3][1]["entries"][4]["open_gate"] = data[3][1]["entries"][3]["open_gate"]
        self.assertTrue(any("duplicates another E2 open gate" in error for error in validate(*data)))

    def test_observation_window_is_metadata(self):
        data = copy.deepcopy(self.data)
        data[3][2]["entries"][92 - 69]["value_role"] = "observed_target"
        self.assertTrue(any("observation-window metadata role" in error for error in validate(*data)))

    def test_config_key_must_be_a_unique_stable_path(self):
        data = copy.deepcopy(self.data)
        data[3][2]["entries"][90 - 69]["config_key"] = "time units (old switch)"
        self.assertTrue(any("stable dotted key" in error for error in validate(*data)))
        data = copy.deepcopy(self.data)
        data[3][0]["entries"][1]["config_key"] = data[3][0]["entries"][0]["config_key"]
        self.assertTrue(any("duplicates a config_key" in error for error in validate(*data)))

    def test_final_matrix_and_resolved_review_cannot_drift(self):
        folder = ROOT / "model-inputs/ed/schema"
        joined = json.loads((folder / "parameter-usage-matrix-v2-candidate.json").read_text())
        final = json.loads((folder / "parameter-usage-matrix.json").read_text())
        disposition = self.data[2]
        with (folder / "p0.2-owner-review-resolved.csv").open(newline="") as file:
            resolved = list(csv.DictReader(file))
        self.assertEqual([], validate_final(joined, final, disposition, resolved))
        changed = copy.deepcopy(final)
        changed["entries"][74]["value_role"] = "observed_target"
        self.assertTrue(any("accepted matrix differs" in error for error in validate_final(joined, changed, disposition, resolved)))
        changed_rows = copy.deepcopy(resolved)
        changed_rows[0]["E0_disposition"] = "defer"
        self.assertTrue(any("reviewer field drift" in error for error in validate_final(joined, final, disposition, changed_rows)))


if __name__ == "__main__":
    unittest.main()
