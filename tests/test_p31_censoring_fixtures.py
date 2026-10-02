import json
import unittest
from pathlib import Path


FIXTURE_PATH = (
    Path(__file__).resolve().parents[1]
    / "model-inputs/ed/calibration/p31-censoring-dependence-fixtures.json"
)


def classify_window_event(event_time, start, end, status):
    if status == "ambiguous":
        return "unresolved_ambiguous_status"
    if status != "observed_event":
        return "not_observed_event"
    return "included_observed_event" if start <= event_time < end else "outside_window"


def features_available_at(records, cutoff):
    return [record for record in records if record["available_at"] <= cutoff]


class CensoringDependenceFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = json.loads(FIXTURE_PATH.read_text())
        cls.cases = cls.fixture["cases"]

    def test_ambiguous_event_censor_status_stays_unresolved(self):
        case = self.cases["ambiguous_status"]
        interval = case["interval"]
        self.assertIsNone(interval["event_indicator"])
        self.assertIsNone(interval["realised_event_type"])
        self.assertIsNone(interval["censor_reason"])
        self.assertEqual(
            classify_window_event(interval["last_observed_time"], 3, 4, interval["status"]),
            case["expected_classification"],
        )

    def test_provenance_is_synthetic_and_scope_excludes_fitting(self):
        self.assertEqual(self.fixture["provenance"]["class"], "synthetic_only")
        self.assertFalse(self.fixture["provenance"]["empirical_data_used"])
        self.assertFalse(self.fixture["provenance"]["promoted_to_ed_parameter_or_default"])
        self.assertFalse(self.fixture["provenance"]["fitting_performed"])
        self.assertIn("P3.2", " ".join(self.fixture["limits"]))

    def test_fx001_counts_events_without_converting_censor_to_completion(self):
        case = self.cases["FX-001"]
        intervals = case["intervals"]
        self.assertEqual([x["event_indicator"] for x in intervals], [1, 1, 0])
        self.assertEqual(sum(x["event_indicator"] for x in intervals), case["expected"]["observed_event_count"])
        self.assertEqual(sum(x["elapsed_time"] for x in intervals), case["expected"]["total_follow_up"])
        self.assertEqual(intervals[-1]["status"], "right_censored")
        self.assertEqual(len(intervals), case["expected"]["naive_completed_event_count"])
        self.assertTrue(case["expected"]["naive_count_is_invalid"])

    def test_fx014_future_completion_and_unavailable_features_do_not_enter_training(self):
        case = self.cases["FX-014"]
        view = case["expected_training_view"]
        self.assertEqual(case["training_cutoff"] - case["risk_start"], view["follow_up"])
        self.assertEqual(view["status"], "right_censored")
        self.assertEqual(view["event_indicator"], 0)
        self.assertFalse(view["completion_visible"])
        usable = features_available_at(case["features"], case["training_cutoff"])
        self.assertEqual({item["name"] for item in usable}, {"triage_category"})
        for feature in case["features"]:
            self.assertEqual(
                feature in usable,
                feature["available_at"] <= case["training_cutoff"],
            )
        late_diagnosis = case["features"][-1]
        self.assertLess(late_diagnosis["event_time"], case["training_cutoff"])
        self.assertGreater(late_diagnosis["available_at"], case["training_cutoff"])

    def test_half_open_window_includes_start_and_excludes_end(self):
        contract = self.fixture["time_contract"]
        self.assertEqual(contract["observation_window"], "[start,end)")
        events = self.cases["window_boundaries"]["events"]
        self.assertEqual(events[0]["expected"], "included_observed_event")
        self.assertEqual(events[1]["expected"], "excluded_administratively_censored")
        self.assertEqual(classify_window_event(3, 3, 4, "observed_event"), "included_observed_event")
        self.assertEqual(classify_window_event(4, 3, 4, "observed_event"), "outside_window")
        self.assertEqual(events[0]["event_time"], self.cases["window_boundaries"]["window"]["start"])
        self.assertEqual(events[1]["event_time"], self.cases["window_boundaries"]["window"]["end"])

    def test_fx003_day_cluster_variance_and_rowwise_leakage(self):
        case = self.cases["FX-003"]
        self.assertEqual([day["values"] for day in case["days"]], [[0, 0], [10, 10]])
        expected = case["expected"]
        self.assertEqual(expected["day_bootstrap_possible_means"], [0, 5, 10])
        self.assertEqual(expected["day_bootstrap_probabilities"], [0.25, 0.5, 0.25])
        self.assertEqual(expected["day_bootstrap_mean_variance"], 12.5)
        self.assertEqual(expected["iid_row_bootstrap_mean_variance"], 6.25)
        self.assertGreater(expected["day_bootstrap_mean_variance"], expected["iid_row_bootstrap_mean_variance"])
        day_split = case["split_examples"]["day_block"]
        patient_split = case["split_examples"]["patient_group"]
        self.assertTrue(set(day_split["train_days"]).isdisjoint(day_split["holdout_days"]))
        self.assertTrue(set(day_split["train_patient_ids"]).isdisjoint(day_split["holdout_patient_ids"]))
        self.assertTrue(set(patient_split["train_patients"]).isdisjoint(patient_split["holdout_patients"]))
        self.assertTrue(set(patient_split["train_day_ids"]).isdisjoint(patient_split["holdout_day_ids"]))
        leak = case["split_examples"]["day_block"]["rowwise_leakage_counterexample"]
        self.assertEqual(leak["leaks_day_id"], "synthetic-day-a")
        self.assertTrue(leak["train_rows"] and leak["holdout_rows"])

    def test_group_policies_are_distinct_and_crossing_is_explicit(self):
        case = self.cases["FX-003"]
        policies = case["split_policies"]
        self.assertIn("day", policies["day_block"].lower())
        self.assertIn("patient", policies["patient_group"].lower())
        crossed = case["split_examples"]["crossed_group_case"]
        self.assertEqual(crossed["expected"], "crossed_group_constraint_requires_explicit_split_policy")
        self.assertIn("cannot separate", crossed["constraint"])
        self.assertIn("require a declared policy", policies["crossed_groups"].lower())

    def test_source_lineage_is_explicit_and_fixture_extensions_are_labeled(self):
        lineage = {(item["source"], item["lines"], item["case"]) for item in self.fixture["source_lineage"]}
        expected = {
            ("conductor/research/supplied/20260927/report-12.md.txt", "62", "FX-014"),
            ("conductor/research/supplied/20260927/report-12.md.txt", "78-85", "FX-001"),
            ("conductor/research/supplied/20260927/report-12.md.txt", "89-95", "FX-003"),
            ("conductor/research/ed-research-incorporation-9-12-20260927.md", "130", "FX-001"),
            ("conductor/research/ed-research-incorporation-9-12-20260927.md", "134", "FX-003"),
            ("conductor/research/ed-research-incorporation-9-12-20260927.md", "136", "FX-014"),
        }
        self.assertTrue(expected.issubset(lineage))
        extensions = self.fixture["synthetic_fixture_extensions"]
        self.assertIn("synthetic", " ".join(extensions.values()).lower())

    def test_time_fields_keep_occurrence_and_knowledge_distinct(self):
        fields = self.fixture["time_contract"]["time_fields"]
        self.assertIn("occurred", fields["event_time"])
        self.assertIn("available", fields["available_at"])
        self.assertIn("delayed", fields["recorded_at"])


if __name__ == "__main__":
    unittest.main()
