import copy
import json
import unittest
from pathlib import Path

from tools.validate_p03_packet_plans import validate


ROOT = Path(__file__).resolve().parents[1]
MATRIX = json.loads((ROOT / "model-inputs/ed/schema/parameter-usage-matrix.json").read_text())


def sample():
    row = {
        "packet_id": "p1-source-arrivals",
        "phase_task": "P1.1",
        "family": "demand_case_mix",
        "parameter_ids": ["ed.demandcasemix.arrivals"],
        "source_report_lead": {"report_id": "R3", "locator": "arrival table lead; exact source bound at dispatch"},
        "source_status": "unverified_lead",
        "named_gap": "Exact primary denominator and inclusion definition",
        "search_scope": "Find the named original table only",
        "output_path": "model-inputs/ed/des/evidence/p1-source-arrivals.json",
        "extraction_fields": ["population", "period", "unit", "denominator"],
        "worker_oracle": "Read source table and record exact fields or unresolved gap",
        "independent_readback_oracle": "Reviewer recomputes source hash and denominator",
        "stop_conditions": ["Source unavailable", "Different population"],
        "dependency_gate": "P0.4 accepted before dispatch",
    }
    return {"schema_version": 1, "status": "preparation_only", "track": "P1", "packet_count": 1, "packets": [row]}


class P03PacketPlanTests(unittest.TestCase):
    def test_valid_preparation_only_packet(self):
        self.assertEqual([], validate(sample(), MATRIX))

    def test_unverified_source_and_unknown_or_deferred_id_are_not_promoted(self):
        plan = sample()
        plan["packets"][0]["source_status"] = "verified"
        self.assertTrue(any("falsely promotes" in x for x in validate(plan, MATRIX)))
        plan = sample()
        plan["packets"][0]["parameter_ids"] = ["ed.invalid"]
        self.assertTrue(any("unknown/duplicate" in x for x in validate(plan, MATRIX)))
        plan = sample()
        plan["packets"][0]["parameter_ids"] = ["ed.demandcasemix.jointcasemix"]
        self.assertTrue(any("deferred parameter" in x for x in validate(plan, MATRIX)))

    def test_one_writer_and_no_premature_distribution(self):
        plan = sample()
        second = copy.deepcopy(plan["packets"][0])
        second["packet_id"] = "p1-source-other"
        plan["packets"].append(second)
        plan["packet_count"] = 2
        self.assertTrue(any("duplicate output path" in x for x in validate(plan, MATRIX)))
        plan = sample()
        plan["packets"][0]["fitted_distribution"] = "exponential"
        self.assertTrue(any("premature" in x for x in validate(plan, MATRIX)))


if __name__ == "__main__":
    unittest.main()
