"""Independent analytic oracles for the frozen E1 extremes fixture family.

These tests validate fixture specifications only. They do not execute or accept
an ED runtime; every case remains runtime_status='unbound' for E1.2 binding.
"""

import copy
import json
import unittest
from pathlib import Path


FIXTURE_PATH = Path(__file__).parents[1] / "conductor/design/ed/e1-extremes.json"
CASE_IDS = {
    "extremes.saturated",
    "extremes.saturated_permuted",
    "extremes.long_censored",
    "extremes.long_exact_horizon",
    "extremes.queued_censored",
    "extremes.zero_service_rejected",
}
COUNTS = ("arrivals", "started", "completed", "unfinished")


def read_fixture():
    return json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))


def analytic_projection(inputs):
    """Single-server FCFS projection, independently computed from raw inputs."""
    horizon = inputs["horizon_ticks"]
    capacity = inputs["capacity"]
    patients = inputs["patients"]
    if horizon <= 0 or capacity <= 0:
        raise ValueError("invalid horizon or capacity")
    if any(patient["service_ticks"] <= 0 for patient in patients):
        return {"validation": "reject", "diagnostic_category": "zero_service", "scheduled_events": 0}
    if any(not 0 <= patient["arrival_tick"] < horizon for patient in patients):
        raise ValueError("arrival outside [0,horizon)")
    ids = [patient["patient_id"] for patient in patients]
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate patient ID")

    # The frozen family uses one server. A list keeps the oracle explicit if
    # another case adds capacity, without relying on a simulation scheduler.
    available_at = [0] * capacity
    projected = []
    for patient in sorted(patients, key=lambda item: (item["arrival_tick"], item["patient_id"])):
        server = min(range(capacity), key=lambda index: (available_at[index], index))
        start = max(patient["arrival_tick"], available_at[server])
        if start > horizon:
            start = None
            interval = None
            completion = None
            observed_work = 0
        else:
            planned_completion = start + patient["service_ticks"]
            end = min(planned_completion, horizon)
            interval = [start, end]
            completion = planned_completion if planned_completion <= horizon else None
            observed_work = end - start
            available_at[server] = planned_completion
        completed = completion is not None
        projected.append(
            {
                "patient_id": patient["patient_id"],
                "start_tick": start,
                "service_interval": interval,
                "completion_tick": completion,
                "status": "completed" if completed else "unfinished",
                "first_care_wait_ticks": start - patient["arrival_tick"] if start is not None else None,
                "observed_work_ticks": observed_work,
                "observed_elapsed_ticks": (completion if completed else horizon) - patient["arrival_tick"],
                "completed_los_ticks": completion - patient["arrival_tick"] if completed else None,
            }
        )
    arrivals = len(patients)
    started = sum(patient["start_tick"] is not None for patient in projected)
    completed = sum(patient["completion_tick"] is not None for patient in projected)
    return {
        "counts": {
            "arrivals": arrivals,
            "started": started,
            "completed": completed,
            "unfinished": arrivals - completed,
        },
        "patients": projected,
    }


def validate_family(fixture):
    if set(fixture) != {"fixture_version", "family", "provenance", "time_unit", "runtime_status", "cases"}:
        raise ValueError("unexpected or missing family fields")
    if fixture["fixture_version"] != 1 or fixture["family"] != "extremes":
        raise ValueError("wrong fixture version or family")
    if fixture["provenance"] != "synthetic_test_only" or fixture["time_unit"] != "tick":
        raise ValueError("wrong provenance or time unit")
    if fixture["runtime_status"] != "unbound":
        raise ValueError("fixture cannot claim runtime binding")
    cases = fixture["cases"]
    if {case.get("case_id") for case in cases} != CASE_IDS or len(cases) != len(CASE_IDS):
        raise ValueError("required case identity/count mismatch")

    by_id = {case["case_id"]: case for case in cases}
    saturated = by_id["extremes.saturated"]["inputs"]
    if saturated != {
        "horizon_ticks": 7,
        "capacity": 1,
        "patients": [
            {"patient_id": "p0", "arrival_tick": 1, "service_ticks": 2},
            {"patient_id": "p1", "arrival_tick": 1, "service_ticks": 2},
            {"patient_id": "p2", "arrival_tick": 1, "service_ticks": 2},
            {"patient_id": "p3", "arrival_tick": 1, "service_ticks": 2},
        ],
    }:
        raise ValueError("saturated case inputs changed")
    permuted = by_id["extremes.saturated_permuted"]["inputs"]
    if permuted["horizon_ticks"] != 7 or permuted["capacity"] != 1:
        raise ValueError("permuted case endpoints or capacity changed")
    if [p["patient_id"] for p in permuted["patients"]] != ["p3", "p1", "p0", "p2"]:
        raise ValueError("permuted case identity/order changed")
    if sorted(permuted["patients"], key=lambda p: p["patient_id"]) != sorted(
        saturated["patients"], key=lambda p: p["patient_id"]
    ):
        raise ValueError("permuted case is not the same patient input set")
    anchors = {
        "extremes.long_censored": (7, "long", 1, 8),
        "extremes.long_exact_horizon": (9, "long", 1, 8),
    }
    for case_id, (horizon, patient_id, arrival, service) in anchors.items():
        inputs = by_id[case_id]["inputs"]
        if inputs != {
            "horizon_ticks": horizon,
            "capacity": 1,
            "patients": [{"patient_id": patient_id, "arrival_tick": arrival, "service_ticks": service}],
        }:
            raise ValueError(f"{case_id} input or endpoint changed")
    queued = by_id["extremes.queued_censored"]["inputs"]
    if queued["horizon_ticks"] != 2 or queued["capacity"] != 1 or len(queued["patients"]) != 4:
        raise ValueError("queued-censor endpoint or patient count changed")
    if any(p != {"patient_id": f"p{i}", "arrival_tick": 1, "service_ticks": 2} for i, p in enumerate(queued["patients"])
    ):
        raise ValueError("queued-censor patient identities or durations changed")
    zero = by_id["extremes.zero_service_rejected"]["inputs"]
    if zero != {"horizon_ticks": 7, "capacity": 1, "patients": [{"patient_id": "zero", "arrival_tick": 1, "service_ticks": 0}]}:
        raise ValueError("zero-service rejection payload changed")

    for case in cases:
        if set(case) != {"case_id", "inputs", "expected", "runtime_owner", "purpose"}:
            raise ValueError(f"{case['case_id']} has unexpected fields")
        if case["runtime_owner"] != "E1.2" or not case["purpose"]:
            raise ValueError(f"{case['case_id']} has invalid owner or purpose")
        computed = analytic_projection(case["inputs"])
        expected = case["expected"]
        if expected != computed:
            raise ValueError(f"{case['case_id']} expected output differs from independent oracle")
        if "counts" in computed:
            counts = computed["counts"]
            if set(counts) != set(COUNTS) or counts["completed"] + counts["unfinished"] != counts["arrivals"]:
                raise ValueError(f"{case['case_id']} violates endpoint conservation")
            if counts["started"] > counts["arrivals"]:
                raise ValueError(f"{case['case_id']} starts more patients than arrived")
            patient_ids = [patient["patient_id"] for patient in computed["patients"]]
            if len(patient_ids) != len(set(patient_ids)) or set(patient_ids) != {
                patient["patient_id"] for patient in case["inputs"]["patients"]
            }:
                raise ValueError(f"{case['case_id']} patient identity conservation failed")
    if by_id["extremes.saturated"]["expected"] != by_id["extremes.saturated_permuted"]["expected"]:
        raise ValueError("input-order permutation changed the observable result")


class ExtremeFixtureTests(unittest.TestCase):
    def test_complete_fixture_matches_independent_arithmetic(self):
        validate_family(read_fixture())

    def test_saturated_exact_intervals_and_endpoint_conservation(self):
        fixture = read_fixture()
        saturated = next(case for case in fixture["cases"] if case["case_id"] == "extremes.saturated")
        self.assertEqual(saturated["expected"]["counts"], {"arrivals": 4, "started": 4, "completed": 3, "unfinished": 1})
        self.assertEqual([p["service_interval"] for p in saturated["expected"]["patients"]], [[1, 3], [3, 5], [5, 7], [7, 7]])
        self.assertEqual([p["completion_tick"] for p in saturated["expected"]["patients"]], [3, 5, 7, None])

    def test_zero_service_is_rejected_before_scheduling(self):
        fixture = read_fixture()
        zero = next(case for case in fixture["cases"] if case["case_id"] == "extremes.zero_service_rejected")
        self.assertEqual(analytic_projection(zero["inputs"]), {"validation": "reject", "diagnostic_category": "zero_service", "scheduled_events": 0})

    def test_independent_mutations_are_detected(self):
        mutations = {
            "duration": lambda f: f["cases"][0]["inputs"]["patients"][0].update(service_ticks=3),
            "endpoint": lambda f: f["cases"][0]["inputs"].update(horizon_ticks=8),
            "count": lambda f: f["cases"][0]["inputs"]["patients"].pop(),
            "identity": lambda f: f["cases"][0]["inputs"]["patients"][0].update(patient_id="renamed"),
            "expected interval": lambda f: f["cases"][0]["expected"]["patients"][0].update(completion_tick=4),
            "provenance": lambda f: f.update(provenance="observed"),
            "runtime binding": lambda f: f.update(runtime_status="passed"),
            "zero-service validation": lambda f: f["cases"][-1]["inputs"]["patients"][0].update(service_ticks=1),
        }
        for label, mutate in mutations.items():
            with self.subTest(mutation=label):
                fixture = copy.deepcopy(read_fixture())
                mutate(fixture)
                with self.assertRaises(ValueError):
                    validate_family(fixture)


if __name__ == "__main__":
    unittest.main()
