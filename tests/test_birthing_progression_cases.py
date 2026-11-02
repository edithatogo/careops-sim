"""B1.1 synthetic expected-trace contract for future birthing runtime tests."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
FIXTURE_PATH = ROOT / "model-inputs/birthing/fixtures/b1-progression-cases.synthetic.json"
REQUIRED_PATHWAYS = {"nominal_spontaneous", "induction", "protracted"}
ALLOWED_KINDS = {
    "admission",
    "labour_progression",
    "service_reservation",
    "transfer_departure",
    "transfer_arrival",
    "intervention_induction_start",
    "intervention_decision",
    "deterioration",
}


def load_fixture() -> dict[str, Any]:
    value = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("fixture root must be an object")
    return value


def contract_errors(fixture: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if type(fixture.get("schema_version")) is not int or fixture["schema_version"] != 1:
        errors.append("schema_version must be integer 1")
    if fixture.get("status") != "synthetic_expected_trace_contract":
        errors.append("fixture must remain a synthetic expected-trace contract")
    provenance = fixture.get("provenance")
    if not isinstance(provenance, dict) or provenance.get("empirical_data_used") is not False:
        errors.append("fixture must declare synthetic-only provenance")
    if isinstance(provenance, dict) and provenance.get("clinical_thresholds_used") is not False:
        errors.append("fixture must declare that no clinical thresholds are used")
    if fixture.get("time_unit") != "simulation_tick":
        errors.append("time_unit must identify synthetic simulation ticks")
    if fixture.get("runtime_status") != "expected_oracle_not_yet_bound_to_birthing_runtime":
        errors.append("runtime status must not claim executed birthing behavior")

    cases = fixture.get("cases")
    if not isinstance(cases, list):
        errors.append("cases must be an array")
        cases = []
    ids: set[str] = set()
    pathways: set[str] = set()
    for case_index, case in enumerate(cases):
        prefix = f"cases[{case_index}]"
        if not isinstance(case, dict):
            errors.append(f"{prefix} must be an object")
            continue
        case_id = case.get("case_id")
        if not isinstance(case_id, str) or not case_id or case_id in ids:
            errors.append(f"{prefix}.case_id must be unique and nonempty")
        else:
            ids.add(case_id)
        pathway = case.get("pathway")
        if not isinstance(pathway, str):
            errors.append(f"{prefix}.pathway must be a string")
        else:
            pathways.add(pathway)
        if not isinstance(case.get("patient_id"), str) or not case["patient_id"].startswith("synthetic-"):
            errors.append(f"{prefix}.patient_id must be synthetic")
        expectations = case.get("expectations")
        if not isinstance(expectations, list) or not expectations or any(not isinstance(value, str) for value in expectations):
            errors.append(f"{prefix}.expectations must name observable contract assertions")

        events = case.get("events")
        if not isinstance(events, list) or not events:
            errors.append(f"{prefix}.events must be a nonempty array")
            continue
        event_ids: set[str] = set()
        ordering: list[tuple[int, int, str, str]] = []
        for event_index, event in enumerate(events):
            event_prefix = f"{prefix}.events[{event_index}]"
            if not isinstance(event, dict):
                errors.append(f"{event_prefix} must be an object")
                continue
            tick, priority = event.get("tick"), event.get("priority")
            if type(tick) is not int or tick < 0:
                errors.append(f"{event_prefix}.tick must be a nonnegative integer")
            if type(priority) is not int or priority < 0:
                errors.append(f"{event_prefix}.priority must be a nonnegative integer")
            event_id = event.get("event_id")
            if not isinstance(event_id, str) or not event_id or event_id in event_ids:
                errors.append(f"{event_prefix}.event_id must be unique and nonempty within its case")
            else:
                event_ids.add(event_id)
            if event.get("kind") not in ALLOWED_KINDS:
                errors.append(f"{event_prefix}.kind is not a declared event kind")
            for field in ("patient_state", "service_state"):
                if not isinstance(event.get(field), str) or not event[field]:
                    errors.append(f"{event_prefix}.{field} must be explicit")
            if type(tick) is int and type(priority) is int and isinstance(event_id, str):
                ordering.append((tick, priority, event_id, str(case.get("patient_id", ""))))
        if ordering != sorted(ordering):
            errors.append(f"{prefix}.events must follow the declared deterministic order key")
        cancelled = case.get("cancelled_event_ids")
        if not isinstance(cancelled, list) or any(not isinstance(value, str) or not value for value in cancelled):
            errors.append(f"{prefix}.cancelled_event_ids must be an array of event identifiers")
        if isinstance(cancelled, list) and event_ids.intersection(cancelled):
            errors.append(f"{prefix}.cancelled_event_ids cannot also appear in expected events")

    if pathways != REQUIRED_PATHWAYS:
        errors.append("fixture must cover nominal spontaneous, induction and protracted pathways")
    if "hazard_rates" in fixture or "probabilities" in fixture:
        errors.append("fixture cannot specify clinical rates or probabilities")
    return errors


class BirthingProgressionCaseContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.fixture = load_fixture()

    def test_fixture_cases_are_structural_and_synthetic_only(self) -> None:
        self.assertEqual(contract_errors(self.fixture), [])
        self.assertEqual(self.fixture["event_order_key"], ["tick", "priority", "event_id", "patient_id"])

    def test_progression_and_deterioration_are_observable_during_wait_or_transit(self) -> None:
        events = [event for case in self.fixture["cases"] for event in case["events"]]
        self.assertTrue(any(event["kind"] == "labour_progression" and event["service_state"].startswith("waiting") for event in events))
        self.assertTrue(any(event["kind"] == "labour_progression" and event["service_state"] == "in_transit" for event in events))
        self.assertTrue(any(event["kind"] == "deterioration" and event["service_state"] == "in_transit" for event in events))

    def test_induction_and_decision_events_are_distinct_from_progression(self) -> None:
        induction = next(case for case in self.fixture["cases"] if case["pathway"] == "induction")
        kinds = [event["kind"] for event in induction["events"]]
        self.assertIn("intervention_induction_start", kinds)
        self.assertIn("labour_progression", kinds)
        self.assertIn("intervention_decision", kinds)
        self.assertIn("deterioration", kinds)
        self.assertLess(kinds.index("intervention_induction_start"), kinds.index("labour_progression"))

    def test_protracted_case_records_deterioration_during_transit_and_stale_event(self) -> None:
        protracted = next(case for case in self.fixture["cases"] if case["pathway"] == "protracted")
        deterioration = next(event for event in protracted["events"] if event["kind"] == "deterioration")
        self.assertEqual(deterioration["service_state"], "in_transit")
        self.assertEqual(protracted["cancelled_event_ids"], ["protracted-stale-progression-after-deterioration"])

    def test_negative_tick_is_rejected(self) -> None:
        mutated = copy.deepcopy(self.fixture)
        mutated["cases"][0]["events"][0]["tick"] = -1
        self.assertIn("cases[0].events[0].tick must be a nonnegative integer", contract_errors(mutated))

    def test_duplicate_event_id_is_rejected(self) -> None:
        mutated = copy.deepcopy(self.fixture)
        mutated["cases"][0]["events"][1]["event_id"] = mutated["cases"][0]["events"][0]["event_id"]
        self.assertTrue(any("event_id must be unique" in error for error in contract_errors(mutated)))

    def test_out_of_order_event_trace_is_rejected(self) -> None:
        mutated = copy.deepcopy(self.fixture)
        mutated["cases"][0]["events"][1]["tick"] = 99
        self.assertIn("cases[0].events must follow the declared deterministic order key", contract_errors(mutated))

    def test_clinical_hazard_or_probability_field_is_rejected(self) -> None:
        mutated = copy.deepcopy(self.fixture)
        mutated["hazard_rates"] = {"labour_progression": 0.2}
        self.assertIn("fixture cannot specify clinical rates or probabilities", contract_errors(mutated))


if __name__ == "__main__":
    unittest.main()
