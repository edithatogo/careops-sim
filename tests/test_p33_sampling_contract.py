import json
from pathlib import Path
import unittest


CONTRACT_PATH = (
    Path(__file__).parents[1]
    / "model-inputs/ed/calibration/p33-sampling-contract.synthetic.json"
)


def load_contract():
    return json.loads(CONTRACT_PATH.read_text())


def topological_order(nodes, edges):
    node_set = set(nodes)
    incoming = {node: 0 for node in node_set}
    outgoing = {node: [] for node in node_set}
    for source, target in edges:
        if source not in node_set or target not in node_set:
            raise ValueError("edge endpoint is not a declared node")
        outgoing[source].append(target)
        incoming[target] += 1
    ready = [node for node in nodes if incoming[node] == 0]
    order = []
    while ready:
        node = ready.pop(0)
        order.append(node)
        for target in outgoing[node]:
            incoming[target] -= 1
            if incoming[target] == 0:
                ready.append(target)
    if len(order) != len(node_set):
        raise ValueError("sampling dependency graph contains a cycle")
    return order


def feature_available_at_decision(feature, decision_time, declared_observable_fields):
    return (
        feature.get("available_at", float("inf")) <= decision_time
        and feature.get("feature") in declared_observable_fields
    )


def early_route_decision(observations, cutoff, declared_observable_fields):
    usable = {
        row["feature"]: row["value"]
        for row in observations
        if feature_available_at_decision(row, cutoff, declared_observable_fields)
    }
    # A tiny synthetic oracle: route on mode only; future diagnosis/disposition
    # can never affect the output, even if their values are mutated.
    mode = usable.get("arrival_mode")
    return {"route": {"synthetic-mode-a": "synthetic-route-a",
                      "synthetic-mode-b": "synthetic-route-b"}.get(mode, "deferred")}


def synthetic_priority(known_state, recorded_measurement):
    # Priority uses the eligible agent-known clinical state only. The recorded
    # triage measurement travels through the signature to make leakage obvious.
    del recorded_measurement
    return {"synthetic-acuity-a": 1, "synthetic-acuity-b": 2}.get(known_state)


def select_synthetic_route(mode, choices, proposed=None):
    if mode not in choices:
        return "deferred"
    route = proposed if proposed is not None else choices[mode][0]
    if route not in choices[mode]:
        raise ValueError("route is not eligible for selected mode")
    return route


def cohort_accounting(presentations, cohort_id):
    if cohort_id == "synthetic-emergency-arrivals":
        selected = [row for row in presentations if row["emergency_eligible"]]
    elif cohort_id == "synthetic-all-presentations":
        selected = list(presentations)
    else:
        raise ValueError("unknown cohort")
    return {
        "eligible_denominator": len(selected),
        "category_a": sum(row["triage"] == "synthetic-category-a" for row in selected),
    }


def validate_cohort_result(requested_cohort_id, result_cohort_id):
    if requested_cohort_id != result_cohort_id:
        raise ValueError("cohort result cannot be relabeled")


def cohort_rate(rows, cohort_id, numerator_category):
    row = next(item for item in rows if item["cohort_id"] == cohort_id)
    validate_denominator(row)
    counts = row["initial_recorded_triage_counts"]
    return counts.get(numerator_category, 0) / row["eligible_denominator"]


def validate_denominator(row):
    counts = row.get("initial_recorded_triage_counts", {})
    values = list(counts.values()) + [row.get("unknown_code_count", 0), row.get("missing_record_count", 0)]
    if any(not isinstance(value, int) or value < 0 for value in values):
        raise ValueError("cohort counts must be nonnegative integers")
    reconciled = (
        sum(counts.values())
        + row.get("unknown_code_count", 0)
        + row.get("missing_record_count", 0)
    )
    if reconciled != row["eligible_denominator"]:
        raise ValueError("triage cohort counts do not reconcile to eligible denominator")
    valid = sum(counts.values())
    if valid != row["valid_recorded_denominator"]:
        raise ValueError("valid recorded denominator does not match category counts")


class P33SamplingContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.contract = load_contract()

    def test_contract_keeps_p32_empirical_family_unknown(self):
        self.assertEqual(self.contract["schema_version"], 1)
        self.assertEqual(self.contract["status"], "synthetic_contract_only")
        provenance = self.contract["provenance"]
        self.assertFalse(provenance["empirical_ed_data_used"])
        self.assertFalse(provenance["empirical_family_selected"])
        self.assertFalse(provenance["empirical_probability_or_range_asserted"])
        self.assertFalse(provenance["clinical_policy_approved"])
        self.assertFalse(provenance["fixture_values_are_defaults"])
        self.assertIn("UNKNOWN", self.contract["evidence_boundary"]["p32_status"])

    def test_arrival_mode_precedes_and_conditions_downstream_choice(self):
        oracle = self.contract["synthetic_oracles"]["sampling_dag"]
        order = topological_order(oracle["nodes"], oracle["edges"])
        self.assertLess(order.index("arrival_times"), order.index("arrival_mode"))
        self.assertLess(order.index("arrival_mode"), order.index("initial_state"))
        self.assertLess(order.index("initial_state"), order.index("pathway_choice"))
        self.assertLess(order.index("pathway_choice"), order.index("realized_disposition"))
        choices = oracle["invented_mode_dependent_choice_sets"]
        self.assertEqual(set(choices), set(oracle["invented_arrival_mode_alternatives"][:2]))
        for mode, alternatives in choices.items():
            self.assertTrue(alternatives)
            self.assertTrue(all(value.startswith("synthetic-route-") for value in alternatives))
            selected = alternatives[0]
            self.assertIn(selected, choices[mode])
            self.assertNotIn(selected, choices["synthetic-mode-b" if mode == "synthetic-mode-a" else "synthetic-mode-a"])
        for mode, expected in (("synthetic-mode-a", "synthetic-route-a"), ("synthetic-mode-b", "synthetic-route-b")):
            self.assertIn(expected, choices[mode])
            self.assertNotIn("synthetic-route-not-eligible", choices[mode])
            self.assertEqual(select_synthetic_route(mode, choices), expected)
            with self.assertRaises(ValueError):
                select_synthetic_route(mode, choices, "synthetic-route-not-eligible")
        for unavailable_mode in ("unknown-or-unrecorded", None):
            self.assertNotIn(unavailable_mode, choices)
            self.assertEqual(select_synthetic_route(unavailable_mode, choices), "deferred")
        with self.assertRaises(ValueError):
            topological_order(oracle["nodes"], oracle["edges"] + [["realized_disposition", "arrival_mode"]])

    def test_future_diagnosis_and_disposition_are_unavailable_early(self):
        canaries = self.contract["synthetic_oracles"]["leakage_canaries"]
        self.assertTrue(canaries)
        for feature in canaries:
            self.assertFalse(feature_available_at_decision(feature, feature["decision_cutoff"], {"arrival_mode"}))
        late_but_early_event = canaries[-1]
        self.assertLess(late_but_early_event["event_time"], late_but_early_event["decision_cutoff"])
        self.assertGreater(late_but_early_event["available_at"], late_but_early_event["decision_cutoff"])
        cutoff = 2
        declared = {"arrival_mode", "final diagnosis", "realized final disposition"}
        base = [
            {"feature": "arrival_mode", "value": "synthetic-mode-a", "available_at": 1},
            {"feature": "final diagnosis", "value": "x", "available_at": 9},
            {"feature": "realized final disposition", "value": "y", "available_at": 12},
        ]
        mutated = [dict(row) for row in base]
        mutated[1]["value"], mutated[2]["value"] = "changed diagnosis", "changed disposition"
        self.assertEqual(early_route_decision(base, cutoff, declared), early_route_decision(mutated, cutoff, declared))
        self.assertFalse(feature_available_at_decision({"feature": "arrival_mode", "available_at": 1}, cutoff, set()))

    def test_cohort_specific_denominators_reconcile_and_stay_distinct(self):
        cohorts = self.contract["synthetic_oracles"]["cohort_denominators"]
        self.assertEqual(len(cohorts), 2)
        emergency, all_presentations = cohorts
        validate_denominator(emergency)
        self.assertEqual(emergency["eligible_denominator"], 4)
        self.assertEqual(emergency["valid_recorded_denominator"], 2)
        self.assertNotEqual(
            emergency["eligible_denominator"], all_presentations["eligible_denominator"]
        )
        self.assertEqual(cohort_rate(cohorts, "synthetic-emergency-arrivals", "synthetic-category-a"), 0.25)
        self.assertEqual(cohort_rate(cohorts, "synthetic-all-presentations", "synthetic-category-a"), 0.2)
        presentations = [
            {"emergency_eligible": True, "triage": "synthetic-category-a"},
            {"emergency_eligible": True, "triage": "synthetic-category-b"},
            {"emergency_eligible": True, "triage": "unknown"},
            {"emergency_eligible": True, "triage": "missing"},
            {"emergency_eligible": False, "triage": "synthetic-category-a"},
        ]
        emergency_before = cohort_accounting(presentations, "synthetic-emergency-arrivals")
        all_before = cohort_accounting(presentations, "synthetic-all-presentations")
        presentations.append({"emergency_eligible": False, "triage": "synthetic-category-b"})
        self.assertEqual(cohort_accounting(presentations, "synthetic-emergency-arrivals"), emergency_before)
        self.assertEqual(cohort_accounting(presentations, "synthetic-all-presentations")["eligible_denominator"], all_before["eligible_denominator"] + 1)
        with self.assertRaises(ValueError):
            validate_cohort_result("synthetic-emergency-arrivals", "synthetic-all-presentations")
        broken = dict(emergency, missing_record_count=0)
        with self.assertRaises(ValueError):
            validate_denominator(broken)
        missing_valid = dict(emergency, valid_recorded_denominator=4)
        with self.assertRaises(ValueError):
            validate_denominator(missing_valid)
        with self.assertRaises(ValueError):
            validate_denominator(dict(emergency, unknown_code_count=-1))

    def test_missing_record_is_not_clinical_priority_or_latent_state(self):
        canary = self.contract["synthetic_oracles"]["missingness_canary"]
        self.assertEqual(canary["latent_state"], canary["agent_known_state"])
        self.assertEqual(canary["recorded_triage"], "missing")
        recorded = {"status": "valid", "value": "synthetic-acuity-b"}
        missing = {"status": "missing", "value": None}
        unknown = {"status": "unknown_code", "value": "synthetic-unmapped-code"}
        priority_before = synthetic_priority(canary["agent_known_state"], recorded)
        self.assertEqual(priority_before, synthetic_priority(canary["agent_known_state"], missing))
        self.assertEqual(priority_before, synthetic_priority(canary["agent_known_state"], unknown))
        self.assertEqual({recorded["status"], missing["status"], unknown["status"]}, {"valid", "missing", "unknown_code"})
        self.assertIsNone(synthetic_priority(None, missing))
        rule = self.contract["decision_time_rule"]
        self.assertIn("recorded measurement", rule["state_separation"])
        oracle = self.contract["synthetic_oracles"]["sampling_dag"]
        self.assertNotIn(["observation", "pathway_choice"], oracle["edges"])
        order = topological_order(oracle["nodes"], oracle["edges"])
        self.assertGreater(order.index("observation"), order.index("realized_disposition"))

    def test_seed_names_are_logical_and_do_not_claim_kairos_stream_support(self):
        purposes = self.contract["logical_seed_purposes"]
        self.assertEqual(len(purposes), len(set(purposes)))
        self.assertIn("arrival_process", purposes)
        self.assertIn("observation_and_missingness", purposes)
        boundary = self.contract["seed_boundary"]
        self.assertIn("do not specify a derivation algorithm", boundary["purpose_names_are"])
        self.assertIn("Kairos 01", boundary["owner"])

    def test_source_triage_discrepancy_is_not_resolved_by_fixture(self):
        claims = " ".join(self.contract["nonclaims_and_deferred_evidence"])
        self.assertIn("3,180-event discrepancy", claims)
        self.assertIn("does not assign that residual", claims)
        self.assertNotIn("9,091,132 / 9,094,312", claims)


if __name__ == "__main__":
    unittest.main()
