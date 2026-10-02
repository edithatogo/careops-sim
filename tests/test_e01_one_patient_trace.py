import copy
import json
from pathlib import Path
import unittest


FIXTURE = Path(__file__).parents[1] / "conductor/design/ed/e0.1-one-patient-trace.json"
EXPECTED_EVENTS = [
    ("arrival", 0),
    ("triage_start", 0),
    ("triage_end", 2),
    ("assessment_start", 2),
    ("assessment_end", 5),
    ("treatment_start", 5),
    ("treatment_end", 9),
    ("discharge", 9),
]
EXPECTED_INPUTS = {
    "arrival_schedule_seconds": [0],
    "service_duration_seconds": {"triage": 2, "assessment": 3, "treatment": 4},
    "resource_capacity": {"triage": 1, "assessment": 1, "treatment": 1},
}
EXPECTED_STAGE_EVENTS = {
    "triage": ("triage_start", "triage_end"),
    "assessment": ("assessment_start", "assessment_end"),
    "treatment": ("treatment_start", "treatment_end"),
}


def validate_trace(trace):
    if trace["time_unit"] != "second":
        raise ValueError("time unit mismatch")
    if trace["inputs"] != EXPECTED_INPUTS:
        raise ValueError("fixed inputs mismatch")
    patient = trace["patient"]
    events = patient["events"]
    if [(event["event"], event["time_seconds"]) for event in events] != EXPECTED_EVENTS:
        raise ValueError("event order or timestamp mismatch")
    if [event["sequence"] for event in events] != list(range(1, 9)):
        raise ValueError("event sequence mismatch")

    stages = patient["stages"]
    expected_names = ["triage", "assessment", "treatment"]
    if [stage["name"] for stage in stages] != expected_names:
        raise ValueError("stage order mismatch")
    event_times = {event["event"]: event["time_seconds"] for event in events}
    active = {name: [] for name in expected_names}
    total_work = 0
    previous_end = event_times["arrival"]
    for stage in stages:
        start_event, end_event = EXPECTED_STAGE_EVENTS[stage["name"]]
        if stage["start_seconds"] != event_times[start_event] or stage["end_seconds"] != event_times[end_event]:
            raise ValueError("stage interval does not match event trace")
        if stage["start_seconds"] != previous_end:
            raise ValueError("unexpected queue gap or overlapping stages")
        duration = stage["end_seconds"] - stage["start_seconds"]
        if duration != trace["inputs"]["service_duration_seconds"][stage["name"]]:
            raise ValueError("stage duration does not match fixed input")
        if duration <= 0 or duration != stage["work_seconds"]:
            raise ValueError("stage duration/work mismatch")
        if stage["resource"] != stage["name"]:
            raise ValueError("stage resource mismatch")
        active[stage["resource"]].append((stage["start_seconds"], stage["end_seconds"]))
        total_work += duration
        previous_end = stage["end_seconds"]

    if total_work != patient["total_work_seconds"]:
        raise ValueError("total work mismatch")
    if events[-1]["time_seconds"] - events[0]["time_seconds"] != patient["length_of_stay_seconds"]:
        raise ValueError("length of stay mismatch")
    if patient["queue_wait_seconds"] != 0 or total_work + patient["queue_wait_seconds"] != patient["length_of_stay_seconds"]:
        raise ValueError("work/wait/LOS mismatch")
    if patient["terminal_status"] != "discharged" or events[-1]["event"] != "discharge":
        raise ValueError("terminal outcome mismatch")

    checks = trace["checks"]
    counts = checks["patient_counts"]
    if counts != {"arrived": 1, "in_system": 0, "terminal": 1}:
        raise ValueError("patient counts mismatch")
    if counts["arrived"] != counts["in_system"] + counts["terminal"]:
        raise ValueError("patient conservation failed")
    terminals = checks["terminal_counts"]
    if terminals != {"discharged": 1, "admitted": 0, "transferred": 0, "left_without_completion": 0}:
        raise ValueError("terminal counts mismatch")
    if counts["terminal"] != sum(terminals.values()):
        raise ValueError("terminal conservation failed")
    if active != {name: [tuple(pair) for pair in intervals] for name, intervals in checks["resource_active_intervals_seconds"].items()}:
        raise ValueError("resource active intervals mismatch")
    capacities = trace["inputs"]["resource_capacity"]
    maxima = checks["resource_capacity_max_concurrent"]
    for resource, intervals in active.items():
        points = sorted((time, delta) for start, end in intervals for time, delta in ((start, 1), (end, -1)))
        current = maximum = 0
        for _, delta in points:
            current += delta
            maximum = max(maximum, current)
        if maximum != maxima[resource] or maximum > capacities[resource]:
            raise ValueError("resource capacity exceeded or maximum mismatch")
    if checks["resource_capacity_satisfied"] is not True:
        raise ValueError("resource capacity flag mismatch")


class OnePatientTraceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.trace = json.loads(FIXTURE.read_text())

    def test_exact_trace_and_recomputed_totals(self):
        trace = self.trace
        self.assertEqual(trace["provenance"]["classification"], "synthetic_test_only")
        self.assertIs(trace["provenance"]["runtime_claim"], False)
        self.assertEqual(trace["experiment"]["run_seed"], 20261002)
        self.assertEqual(trace["interval_semantics"], "half_open_[start,end)")
        validate_trace(trace)

    def test_rejects_altered_event_order(self):
        altered = copy.deepcopy(self.trace)
        altered["patient"]["events"][2], altered["patient"]["events"][3] = altered["patient"]["events"][3], altered["patient"]["events"][2]
        with self.assertRaises(ValueError):
            validate_trace(altered)

    def test_rejects_altered_duration(self):
        altered = copy.deepcopy(self.trace)
        altered["patient"]["stages"][1]["end_seconds"] = 6
        with self.assertRaises(ValueError):
            validate_trace(altered)

    def test_rejects_altered_outcome(self):
        altered = copy.deepcopy(self.trace)
        altered["patient"]["terminal_status"] = "admitted"
        with self.assertRaises(ValueError):
            validate_trace(altered)

    def test_rejects_altered_counts(self):
        altered = copy.deepcopy(self.trace)
        altered["checks"]["patient_counts"]["terminal"] = 0
        with self.assertRaises(ValueError):
            validate_trace(altered)

    def test_rejects_capacity_overrun(self):
        altered = copy.deepcopy(self.trace)
        altered["inputs"]["resource_capacity"]["assessment"] = 0
        with self.assertRaises(ValueError):
            validate_trace(altered)


if __name__ == "__main__":
    unittest.main()
