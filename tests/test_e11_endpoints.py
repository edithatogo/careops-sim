"""Independent oracle tests for the frozen E1 endpoints fixture specification.

These tests validate synthetic input/output arithmetic only. They never invoke
careops-ed and must not be read as native pathway-runtime acceptance.
"""
from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path


FIXTURE = Path(__file__).parents[1] / "conductor/design/ed/e1-endpoints.json"
EXPECTED_ACUITIES = set(range(1, 6))
EXPECTED_DISPOSITIONS = {"discharge", "admission"}
EXPECTED_DIAGNOSTICS = {False, True}
STAGE_DURATIONS = {
    "triage": 2,
    "assessment": 3,
    "diagnostics": 4,
    "treatment": 5,
}


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def _calculate(case: dict) -> dict:
    """Recompute counts and metrics from case inputs, without trusting expected."""
    inputs = case["inputs"]
    horizon = inputs["horizon_tick"]
    rows = []
    for patient in inputs["patients"]:
        arrival = patient["arrival_tick"]
        now = arrival
        intervals = []
        active_work = 0
        for stage in patient["stages"]:
            start = now
            end = start + stage["duration_ticks"]
            intervals.append({
                "stage": stage["stage"],
                "resource_id": stage["resource_id"],
                "start_tick": start,
                "end_tick": end,
            })
            now = end
            active_work += stage["duration_ticks"]

        first_wait = intervals[0]["start_tick"] - arrival if intervals else None
        stage_wait = sum(
            interval["start_tick"] - (
                arrival if index == 0 else intervals[index - 1]["end_tick"]
            )
            for index, interval in enumerate(intervals)
        )
        disposition = patient["provisional_disposition"]
        boarding = None
        transfer = None
        outcome = None
        terminal_tick = None
        unfinished_reason = None
        if now > horizon:
            unfinished_reason = "pathway_incomplete_at_horizon"
        elif disposition == "discharge":
            terminal_tick = now
            outcome = {"kind": "discharged", "terminal_tick": terminal_tick}
        else:
            offer = next((
                item for item in inputs["bed_offers"]
                if item["patient_id"] == patient["patient_id"]
            ), None)
            if offer is None:
                boarding = {
                    "start_tick": now,
                    "end_tick": horizon,
                    "duration_ticks": horizon - now,
                }
                unfinished_reason = "boarding_without_compatible_offer"
            else:
                boarding = {
                    "start_tick": now,
                    "end_tick": offer["available_tick"],
                    "duration_ticks": offer["available_tick"] - now,
                }
                transfer = {
                    "start_tick": offer["available_tick"],
                    "end_tick": offer["available_tick"] + offer["transfer_duration_ticks"],
                    "duration_ticks": offer["transfer_duration_ticks"],
                    "offer_id": offer["offer_id"],
                }
                if transfer["end_tick"] <= horizon:
                    terminal_tick = transfer["end_tick"]
                    outcome = {
                        "kind": "transferred",
                        "terminal_tick": terminal_tick,
                        "offer_id": offer["offer_id"],
                        "destination": offer["destination"],
                    }
                else:
                    unfinished_reason = "transfer_incomplete_at_horizon"

        terminal = terminal_tick is not None
        rows.append({
            "patient_id": patient["patient_id"],
            "arrival_tick": arrival,
            "latent_state": patient["latent_state"],
            "agent_knowledge": patient["agent_knowledge"],
            "provisional_disposition": disposition,
            "stage_intervals": intervals,
            "first_care_wait_ticks": first_wait,
            "total_stage_queue_wait_ticks": stage_wait,
            "active_work_ticks": active_work,
            "boarding_interval": boarding,
            "transfer_interval": transfer,
            "endpoint_state": "terminal" if terminal else "unfinished_at_horizon",
            "terminal_tick": terminal_tick,
            "realized_outcome": outcome,
            "unfinished_reason": unfinished_reason,
            "observed_elapsed_ticks": (
                terminal_tick - arrival if terminal else horizon - arrival
            ),
            "completed_los_ticks": terminal_tick - arrival if terminal else None,
        })

    arrivals = len(rows)
    started_rows = [row for row in rows if row["first_care_wait_ticks"] is not None]
    terminal_rows = [row for row in rows if row["terminal_tick"] is not None]
    waits = [row["first_care_wait_ticks"] for row in started_rows]
    stage_waits = [row["total_stage_queue_wait_ticks"] for row in rows if row["stage_intervals"]]
    work = [row["active_work_ticks"] for row in rows if row["stage_intervals"]]
    los = [row["completed_los_ticks"] for row in terminal_rows]
    elapsed = [row["observed_elapsed_ticks"] for row in rows]

    def average(values: list[int]) -> float | None:
        return None if not values else sum(values) / len(values)

    terminal_count = len(terminal_rows)
    return {
        "counts": {
            "arrivals": arrivals,
            "started": len(started_rows),
            "terminal": terminal_count,
            "unfinished": arrivals - terminal_count,
            "throughput_count": terminal_count,
        },
        "denominators": {
            "first_care_wait": len(waits),
            "total_stage_queue_wait": len(stage_waits),
            "active_work": len(work),
            "completed_los": len(los),
            "observed_elapsed": len(elapsed),
        },
        "metrics_ticks": {
            "mean_first_care_wait": average(waits),
            "mean_total_stage_queue_wait": average(stage_waits),
            "mean_active_work": average(work),
            "mean_completed_los": average(los),
            "mean_observed_elapsed": average(elapsed),
        },
        "patients": rows,
    }


def _validate_case(case: dict) -> None:
    _require(set(case) == {"case_id", "runtime_owner", "purpose", "inputs", "expected"},
             f"unexpected case fields: {case.get('case_id')}")
    _require(case["case_id"].startswith("endpoints_"), "case ID must be family scoped")
    _require(case["runtime_owner"] in {"E1.2", "E1.3"}, "unknown runtime owner")
    _require(bool(case["purpose"].strip()), "purpose must be explicit")
    inputs = case["inputs"]
    _require(set(inputs) == {"horizon_tick", "patients", "bed_offers"}, "input schema drift")
    _require(type(inputs["horizon_tick"]) is int and inputs["horizon_tick"] > 0,
             "horizon must be a positive integer")
    patients = inputs["patients"]
    _require(isinstance(patients, list), "patients must be a list")
    ids = [patient["patient_id"] for patient in patients]
    _require(len(ids) == len(set(ids)), "duplicate patient ID")
    for patient in patients:
        _require(set(patient) == {
            "patient_id", "arrival_tick", "latent_state", "agent_knowledge",
            "stages", "provisional_disposition",
        }, "patient input fields drifted")
        _require(bool(patient["patient_id"].strip()), "patient ID is empty")
        _require(type(patient["arrival_tick"]) is int and 0 <= patient["arrival_tick"] < inputs["horizon_tick"],
                 "arrival must be in [0,horizon)")
        _require(set(patient["latent_state"]) == {"acuity"}, "latent state must remain explicit")
        _require(set(patient["agent_knowledge"]) == {"triage_acuity"}, "agent knowledge must remain explicit")
        _require(1 <= patient["latent_state"]["acuity"] <= 5, "synthetic acuity outside 1..5")
        _require(1 <= patient["agent_knowledge"]["triage_acuity"] <= 5, "known acuity outside 1..5")
        _require(patient["provisional_disposition"] in EXPECTED_DISPOSITIONS,
                 "invalid provisional disposition")
        stages = patient["stages"]
        names = [stage["stage"] for stage in stages]
        _require(names in [
            ["triage", "assessment", "treatment"],
            ["triage", "assessment", "diagnostics", "treatment"],
        ], "stages must be ordered with optional diagnostics")
        for stage in stages:
            _require(set(stage) == {"stage", "duration_ticks", "resource_id"}, "stage fields drifted")
            _require(type(stage["duration_ticks"]) is int and stage["duration_ticks"] > 0,
                     "service durations must be positive integer ticks")
            _require(bool(stage["resource_id"].strip()), "resource ID is empty")
        offers = [offer for offer in inputs["bed_offers"] if offer["patient_id"] == patient["patient_id"]]
        _require(len(offers) <= 1, "fixture offers must be single use")
        for offer in offers:
            _require(offer["available_tick"] < offer["withdrawn_tick"], "offer interval must be nonempty")
            _require(offer["transfer_duration_ticks"] > 0, "transfer duration must be positive")
    for offer in inputs["bed_offers"]:
        _require(set(offer) == {
            "offer_id", "patient_id", "destination", "available_tick", "withdrawn_tick",
            "transfer_duration_ticks",
        }, "offer fields drifted")
        _require(offer["patient_id"] in ids, "offer patient is unresolved")
        _require(bool(offer["offer_id"].strip()) and bool(offer["destination"].strip()),
                 "offer identity/destination is empty")

    _require(case["expected"] == _calculate(case), f"independent oracle mismatch: {case['case_id']}")
    actual_ids = [row["patient_id"] for row in case["expected"]["patients"]]
    terminal_ids = [row["patient_id"] for row in case["expected"]["patients"] if row["endpoint_state"] == "terminal"]
    unfinished_ids = [row["patient_id"] for row in case["expected"]["patients"] if row["endpoint_state"] == "unfinished_at_horizon"]
    _require(len(actual_ids) == len(set(actual_ids)), "expected patient identity duplicated")
    _require(set(actual_ids) == set(terminal_ids) | set(unfinished_ids), "endpoint conservation lost a patient")
    _require(not set(terminal_ids) & set(unfinished_ids), "patient appears terminal and unfinished")
    _require(len(terminal_ids) + len(unfinished_ids) == len(actual_ids), "endpoint conservation count mismatch")
    _require(case["expected"]["counts"]["arrivals"] == len(actual_ids), "arrival count mismatch")


def _validate_family(family: dict) -> None:
    _require(set(family) == {"fixture_version", "family", "provenance", "time_unit", "runtime_status", "cases"},
             "family fields drifted")
    _require(family["fixture_version"] == 1 and family["family"] == "endpoints", "wrong fixture identity")
    _require(family["provenance"] == "synthetic_test_only" and family["time_unit"] == "tick",
             "provenance or unit drift")
    _require(family["runtime_status"] == "unbound", "fixture must not imply runtime acceptance")
    cases = family["cases"]
    case_ids = [case["case_id"] for case in cases]
    _require(len(case_ids) == len(set(case_ids)), "duplicate case ID")
    for case in cases:
        _validate_case(case)

    by_id = {case["case_id"]: case for case in cases}
    _require("endpoints_empty" in by_id and "endpoints_one_discharge" in by_id,
             "empty and one-patient fixtures are mandatory")
    _require("endpoints_admission_no_offer" in by_id, "no-offer admission fixture is missing")
    _require(len(cases) == 23, "required case count is empty + one + 20 grid + no-offer")
    grid = [case for case in cases if "_acuity_" in case["case_id"]]
    observed = {
        (p["latent_state"]["acuity"], p["provisional_disposition"],
         any(s["stage"] == "diagnostics" for s in p["stages"]))
        for case in grid for p in case["inputs"]["patients"]
    }
    required = {
        (acuity, disposition, diagnostics)
        for acuity in EXPECTED_ACUITIES
        for disposition in EXPECTED_DISPOSITIONS
        for diagnostics in EXPECTED_DIAGNOSTICS
    }
    _require(len(grid) == 20 and observed == required, "acuity/pathway/diagnostics matrix incomplete")
    for case in grid:
        _require(len(case["inputs"]["patients"]) == 1, "grid case must contain exactly one patient")
        patient = case["inputs"]["patients"][0]
        expected_owner = "E1.3" if patient["provisional_disposition"] == "admission" else "E1.2"
        _require(case["runtime_owner"] == expected_owner, "runtime owner must match terminal pathway")
        names = [stage["stage"] for stage in patient["stages"]]
        _require([s["duration_ticks"] for s in patient["stages"]] == [
            STAGE_DURATIONS[name] for name in names
        ], "frozen stage duration changed")
        _require(patient["arrival_tick"] == 1 and case["inputs"]["horizon_tick"] == 30,
                 "frozen timeline changed")
        _require(patient["patient_id"] == f"patient-{case['case_id']}", "stable patient ID changed")
        has_diagnostics = "diagnostics" in names
        treatment_end = 11 + (4 if has_diagnostics else 0)
        row = case["expected"]["patients"][0]
        if patient["provisional_disposition"] == "discharge":
            expected_kind, expected_terminal = "discharged", treatment_end
        else:
            expected_kind, expected_terminal = "transferred", treatment_end + 4
        _require(row["realized_outcome"]["kind"] == expected_kind,
                 "hand-fixed pathway outcome changed")
        _require(row["terminal_tick"] == expected_terminal,
                 "hand-fixed pathway endpoint changed")
        _require(row["active_work_ticks"] == 10 + (4 if has_diagnostics else 0),
                 "hand-fixed active work changed")
        _require(row["completed_los_ticks"] == expected_terminal - 1,
                 "hand-fixed LOS changed")
        if patient["provisional_disposition"] == "admission":
            offer = case["inputs"]["bed_offers"][0]
            _require(offer["available_tick"] == treatment_end + 2 and
                     offer["withdrawn_tick"] == treatment_end + 10 and
                     offer["transfer_duration_ticks"] == 2,
                     "hand-fixed admission offer/transfer timeline changed")
    empty = by_id["endpoints_empty"]["expected"]
    _require(empty["counts"] == {"arrivals": 0, "started": 0, "terminal": 0, "unfinished": 0, "throughput_count": 0},
             "empty-run counts changed")
    _require(all(value is None for value in empty["metrics_ticks"].values()), "empty undefined means must be null")
    _require(all(value == 0 for value in empty["denominators"].values()), "empty denominator must be zero")
    no_offer_case = by_id["endpoints_admission_no_offer"]
    no_offer = no_offer_case["expected"]["patients"][0]
    no_offer_start = no_offer["stage_intervals"][-1]["end_tick"]
    _require(no_offer_case["runtime_owner"] == "E1.3", "admission endpoint must be owned by E1.3")
    _require(no_offer["provisional_disposition"] == "admission", "no-offer disposition must remain provisional admission")
    _require(no_offer["boarding_interval"] == {
        "start_tick": no_offer_start,
        "end_tick": no_offer_case["inputs"]["horizon_tick"],
        "duration_ticks": no_offer_case["inputs"]["horizon_tick"] - no_offer_start,
    }, "no-offer boarding must remain visible through the horizon")
    _require(no_offer["transfer_interval"] is None, "no-offer case must not start transfer")
    _require(no_offer["realized_outcome"] is None and no_offer["terminal_tick"] is None,
             "no offer must not realize admission or transfer")
    _require(no_offer["endpoint_state"] == "unfinished_at_horizon" and
             no_offer["unfinished_reason"] == "boarding_without_compatible_offer",
             "no-offer boarding endpoint changed")
    for case in cases:
        for row in case["expected"]["patients"]:
            _require(row["terminal_tick"] is None or row["terminal_tick"] <= case["inputs"]["horizon_tick"],
                     "terminal tick must not exceed horizon")
            _require((row["terminal_tick"] is not None) == (row["realized_outcome"] is not None),
                     "realized outcome and terminal endpoint must agree")
            _require((row["terminal_tick"] is not None) != (row["unfinished_reason"] is not None),
                     "patient must be exactly terminal or explicitly unfinished")
            _require(row["latent_state"] != row["agent_knowledge"] or
                     set(row["latent_state"]) != set(row["agent_knowledge"]),
                     "latent state and agent knowledge may not be collapsed")


class EndpointsFixtureSpecificationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.family = json.loads(FIXTURE.read_text(encoding="utf-8"))

    def test_complete_family_and_independent_oracle(self) -> None:
        _validate_family(self.family)

    def test_named_reference_timeline_and_metrics(self) -> None:
        case = next(c for c in self.family["cases"] if c["case_id"] == "endpoints_one_discharge")
        expected = case["expected"]
        self.assertEqual(expected["counts"], {
            "arrivals": 1, "started": 1, "terminal": 1, "unfinished": 0, "throughput_count": 1,
        })
        row = expected["patients"][0]
        self.assertEqual([(s["stage"], s["start_tick"], s["end_tick"]) for s in row["stage_intervals"]],
                         [("triage", 1, 3), ("assessment", 3, 6), ("treatment", 6, 11)])
        self.assertEqual((row["first_care_wait_ticks"], row["total_stage_queue_wait_ticks"],
                          row["active_work_ticks"], row["terminal_tick"], row["completed_los_ticks"]),
                         (0, 0, 10, 11, 10))
        self.assertEqual(row["provisional_disposition"], "discharge")
        self.assertEqual(row["realized_outcome"], {"kind": "discharged", "terminal_tick": 11})
        self.assertEqual(case["runtime_owner"], "E1.2")

    def test_independent_mutations_are_rejected(self) -> None:
        mutations = {
            "stage duration": lambda d: d["cases"][1]["inputs"]["patients"][0]["stages"][0].__setitem__("duration_ticks", 9),
            "endpoint tick": lambda d: d["cases"][1]["expected"]["patients"][0].__setitem__("terminal_tick", 99),
            "summary count": lambda d: d["cases"][1]["expected"]["counts"].__setitem__("terminal", 2),
            "patient identity": lambda d: d["cases"][1]["inputs"]["patients"][0].__setitem__("patient_id", "changed"),
            "required family member": lambda d: d["cases"].pop(2),
            "runtime status": lambda d: d.__setitem__("runtime_status", "passed"),
            "nonpositive work": lambda d: d["cases"][1]["inputs"]["patients"][0]["stages"][0].__setitem__("duration_ticks", 0),
            "agent knowledge": lambda d: d["cases"][1]["inputs"]["patients"][0]["agent_knowledge"].__setitem__("triage_acuity", 5),
            "horizon": lambda d: d["cases"][1]["inputs"].__setitem__("horizon_tick", 5),
        }
        for label, mutate in mutations.items():
            with self.subTest(mutation=label):
                altered = copy.deepcopy(self.family)
                mutate(altered)
                with self.assertRaises(AssertionError):
                    _validate_family(altered)

    def test_empty_metrics_have_null_means_and_no_denominator(self) -> None:
        case = next(c for c in self.family["cases"] if c["case_id"] == "endpoints_empty")
        self.assertEqual(case["expected"]["patients"], [])
        self.assertEqual(case["expected"]["counts"]["arrivals"], 0)
        self.assertTrue(all(v is None for v in case["expected"]["metrics_ticks"].values()))
        self.assertTrue(all(v == 0 for v in case["expected"]["denominators"].values()))
        self.assertEqual(case["runtime_owner"], "E1.2")

    def test_runtime_attachment_is_explicitly_unbound(self) -> None:
        self.assertEqual(self.family["runtime_status"], "unbound")
        self.assertTrue(all(case["expected"] == _calculate(case) for case in self.family["cases"]))


if __name__ == "__main__":
    unittest.main()
