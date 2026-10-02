import json
from pathlib import Path
import unittest


CONTRACT_PATH = (
    Path(__file__).parents[1]
    / "model-inputs/ed/calibration/p33-des-abm-join.synthetic.json"
)
ALLOWED_KINDS = {"active_work", "setup", "transit", "queue_or_wait", "interruption", "handover"}


def load_contract():
    return json.loads(CONTRACT_PATH.read_text())


def account_timeline(timeline):
    if timeline.get("status") in {"right_censored", "left_censored", "interval_censored"}:
        raise ValueError("censored timeline has no completed macro duration")
    start, end = timeline["start"], timeline["end"]
    segments = sorted(timeline["segments"], key=lambda row: (row["start"], row["end"]))
    if end < start:
        raise ValueError("macro interval has negative duration")
    cursor = start
    totals = {kind: 0 for kind in ALLOWED_KINDS}
    for segment in segments:
        left, right, kind = segment["start"], segment["end"], segment["kind"]
        if kind not in ALLOWED_KINDS:
            raise ValueError("unknown interval type")
        if right < left:
            raise ValueError("micro interval has negative duration")
        if left != cursor:
            raise ValueError("micro intervals overlap or leave an unclassified gap")
        totals[kind] += right - left
        cursor = right
    if cursor != end:
        raise ValueError("micro intervals do not reconcile to macro boundary")
    return {"macro_elapsed": end - start, "components": totals,
            "intrinsic_active_work": totals["active_work"],
            "task_elapsed_including_setup": totals["active_work"] + totals["setup"]}


def observed_accounting(timeline):
    if timeline.get("status") in {"right_censored", "left_censored", "interval_censored"}:
        return {"status": timeline["status"], "completed_duration": None,
                "last_known_time": timeline["last_known_time"]}
    return {"status": "complete", **account_timeline(timeline)}


def combine_macro_timelines(timelines):
    if not timelines:
        return 0
    entity_ids = {timeline["entity_id"] for timeline in timelines}
    event_ids = {timeline["event_id"] for timeline in timelines}
    timeline_ids = [timeline["timeline_id"] for timeline in timelines]
    if len(timeline_ids) != len(set(timeline_ids)):
        raise ValueError("duplicate timeline representation cannot be summed twice")
    if len(timelines) > 1:
        raise ValueError("separate timeline rows require explicit reconciliation before summing")
    if len(entity_ids) != 1 or len(event_ids) != 1:
        raise ValueError("actor timelines cannot be summed into one macro elapsed interval")
    return sum(account_timeline(timeline)["macro_elapsed"] for timeline in timelines)


class P33DesAbmJoinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.contract = load_contract()

    def test_contract_is_synthetic_and_keeps_empirical_gaps_open(self):
        provenance = self.contract["provenance"]
        self.assertEqual(self.contract["status"], "synthetic_contract_only")
        self.assertFalse(provenance["empirical_ed_data_used"])
        self.assertFalse(provenance["empirical_duration_claim_asserted"])
        self.assertFalse(provenance["fixture_values_are_defaults"])
        self.assertIn("unknown", self.contract["upstream"]["p33_uncertainty"].lower())
        self.assertTrue(any("No ED active-work" in row for row in self.contract["nonclaims"]))

    def test_macro_equals_exclusive_micro_and_work_excludes_transit_wait(self):
        timeline = self.contract["invented_accounting_fixture"]["patient_timeline"]
        result = account_timeline(timeline)
        self.assertEqual(result["macro_elapsed"], 10)
        self.assertEqual(result["macro_elapsed"], sum(result["components"].values()))
        self.assertEqual(result["intrinsic_active_work"], 3)
        self.assertEqual(result["task_elapsed_including_setup"], 4)
        self.assertEqual(result["components"]["transit"], 2)
        self.assertEqual(result["components"]["queue_or_wait"], 4)
        self.assertNotEqual(result["intrinsic_active_work"], result["macro_elapsed"])

    def test_overlapping_gap_negative_and_unknown_components_reject(self):
        timeline = self.contract["invented_accounting_fixture"]["patient_timeline"]
        overlap = json.loads(json.dumps(timeline))
        overlap["segments"][1]["start"] = 2
        with self.assertRaises(ValueError):
            account_timeline(overlap)
        duplicate = json.loads(json.dumps(timeline))
        duplicate["segments"].append(dict(duplicate["segments"][0]))
        with self.assertRaises(ValueError):
            account_timeline(duplicate)
        gap = json.loads(json.dumps(timeline))
        gap["segments"][1]["start"] = 4
        with self.assertRaises(ValueError):
            account_timeline(gap)
        negative = json.loads(json.dumps(timeline))
        negative["segments"][0]["end"] = -1
        with self.assertRaises(ValueError):
            account_timeline(negative)
        unknown = json.loads(json.dumps(timeline))
        unknown["segments"][0]["kind"] = "unknown"
        with self.assertRaises(ValueError):
            account_timeline(unknown)

    def test_concurrent_staff_transit_is_linked_not_double_added_to_patient(self):
        fixture = self.contract["invented_accounting_fixture"]
        patient = account_timeline(fixture["patient_timeline"])
        staff = fixture["concurrent_staff_timeline"]
        staff_move = staff["segments"][0]
        self.assertEqual(staff["linked_patient_event_id"], fixture["patient_timeline"]["event_id"])
        self.assertEqual(staff_move["kind"], "transit")
        self.assertGreaterEqual(staff_move["start"], 6)
        self.assertLessEqual(staff_move["end"], 10)
        self.assertEqual(patient["macro_elapsed"], 10)
        self.assertEqual(patient["intrinsic_active_work"], 3)
        patient_timeline = fixture["patient_timeline"]
        staff_timeline = dict(staff, event_id=staff["linked_patient_event_id"])
        with self.assertRaises(ValueError):
            combine_macro_timelines([patient_timeline, staff_timeline])
        with self.assertRaises(ValueError):
            combine_macro_timelines([patient_timeline, json.loads(json.dumps(patient_timeline))])

    def test_censoring_is_not_converted_to_completed_duration(self):
        censored = self.contract["invented_accounting_fixture"]["censored_timeline"]
        self.assertEqual(censored["status"], "right_censored")
        self.assertIsNone(censored["completed_duration"])
        self.assertIn("censor_reason", censored)
        self.assertIn("censoring", self.contract["upstream"]["p31_censoring_identifiability"].lower())
        observation = observed_accounting(censored)
        self.assertIsNone(observation["completed_duration"])
        with self.assertRaises(ValueError):
            account_timeline(censored)

    def test_sampling_and_cohort_boundaries_are_retained(self):
        gates = " ".join(self.contract["retained_sampling_and_cohort_gates"])
        self.assertIn("unknown/missing mode", gates)
        self.assertIn("valid-record denominator", gates)
        self.assertIn("available_at", gates)
        self.assertIn("future information", gates)
        self.assertIn("accepted in P3.3.sampling", self.contract["sampling_gate_evidence"])


if __name__ == "__main__":
    unittest.main()
