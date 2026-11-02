"""B0.3 contract tests for the reference-only Queensland birthing profile.

These checks validate the input artifact's shape and internal accounting. They
do not validate clinical rules, site applicability, or a runnable model.
"""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
PROFILE_PATH = ROOT / "model-inputs/birthing/profiles/qld-public-reference-2022-v1.json"


def load_profile() -> dict[str, Any]:
    value = json.loads(PROFILE_PATH.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("profile root must be an object")
    return value


def contract_errors(profile: dict[str, Any]) -> list[str]:
    """Return structural, provenance, units, range, and reconciliation errors."""
    errors: list[str] = []

    def object_at(parent: Any, key: str, path: str) -> dict[str, Any] | None:
        value = parent.get(key) if isinstance(parent, dict) else None
        if not isinstance(value, dict):
            errors.append(f"{path} must be an object")
            return None
        return value

    def list_at(parent: Any, key: str, path: str) -> list[Any] | None:
        value = parent.get(key) if isinstance(parent, dict) else None
        if not isinstance(value, list):
            errors.append(f"{path} must be an array")
            return None
        return value

    def count(value: Any, path: str) -> int | None:
        if type(value) is not int or value < 0:
            errors.append(f"{path} must be a nonnegative integer count")
            return None
        return value

    if type(profile.get("schema_version")) is not int or profile["schema_version"] != 1:
        errors.append("schema_version must be integer 1")
    if profile.get("profile_status") != "reference_only":
        errors.append("profile_status must remain reference_only")
    if not isinstance(profile.get("profile_id"), str) or not profile["profile_id"].strip():
        errors.append("profile_id must be a nonempty string")
    if type(profile.get("data_year")) is not int or profile["data_year"] <= 0:
        errors.append("data_year must be a positive integer")

    units = object_at(profile, "units", "units")
    if units is not None:
        expected_units = {
            "population_count": "person or baby as named by source table",
            "route_count": "births/babies as named by source table",
            "labour_onset_count": "mothers",
        }
        for name, expected in expected_units.items():
            if units.get(name) != expected:
                errors.append(f"units.{name} must preserve source denominator wording")

    sources = object_at(profile, "sources", "sources")
    source_ids: set[str] = set()
    if sources is not None:
        observed = object_at(sources, "observed_aggregates", "sources.observed_aggregates")
        if observed is not None:
            for field in ("source_id", "authority", "title", "edition", "url", "collection", "accessed"):
                if not isinstance(observed.get(field), str) or not observed[field].strip():
                    errors.append(f"sources.observed_aggregates.{field} must be nonempty provenance")
            if not str(observed.get("url", "")).startswith("https://"):
                errors.append("observed aggregate URL must use HTTPS")
            refs = list_at(observed, "table_refs", "sources.observed_aggregates.table_refs")
            if refs is not None and (not refs or any(not isinstance(ref, str) or not ref.strip() for ref in refs)):
                errors.append("table_refs must contain named source tables")
        policies = list_at(sources, "policy_registry", "sources.policy_registry")
        if policies is not None:
            for index, policy in enumerate(policies):
                path = f"sources.policy_registry[{index}]"
                if not isinstance(policy, dict):
                    errors.append(f"{path} must be an object")
                    continue
                source_id = policy.get("source_id")
                if not isinstance(source_id, str) or not source_id.strip():
                    errors.append(f"{path}.source_id must be nonempty")
                elif source_id in source_ids:
                    errors.append(f"{path}.source_id must be unique")
                else:
                    source_ids.add(source_id)
                for field in ("title", "url", "scope_note"):
                    if not isinstance(policy.get(field), str) or not policy[field].strip():
                        errors.append(f"{path}.{field} must preserve source scope")
                if not str(policy.get("url", "")).startswith("https://"):
                    errors.append(f"{path}.url must use HTTPS")
                if policy.get("review_dates", {}).get("status") == "conflicting_unresolved":
                    dates = policy["review_dates"]
                    if not dates.get("pdf") or not dates.get("publication_index") or dates["pdf"] == dates["publication_index"]:
                        errors.append(f"{path}.review_dates must retain both distinct source values")

    population = object_at(profile, "population", "population")
    mothers = babies = None
    if population is not None:
        mother_record = object_at(population, "mothers", "population.mothers")
        baby_record = object_at(population, "babies", "population.babies")
        if mother_record is not None:
            mothers = count(mother_record.get("count"), "population.mothers.count")
            if not mother_record.get("source_table_measure"):
                errors.append("population.mothers.source_table_measure is required")
        if baby_record is not None:
            babies = count(baby_record.get("count"), "population.babies.count")
            if not baby_record.get("source_table_measure"):
                errors.append("population.babies.source_table_measure is required")
        for key in ("previous_pregnancies", "plurality", "maternal_age"):
            item = object_at(population, key, f"population.{key}")
            if item is None:
                continue
            if item.get("count_unit") != "mothers":
                errors.append(f"population.{key}.count_unit must be mothers")
            rows = list_at(item, "categories", f"population.{key}.categories")
            if rows is not None:
                names: set[str] = set()
                values = []
                for row_index, row in enumerate(rows):
                    if not isinstance(row, dict):
                        errors.append(f"population.{key}.categories[{row_index}] must be an object")
                        continue
                    label = row.get("category")
                    if not isinstance(label, str) or not label or label in names:
                        errors.append(f"population.{key}.categories must have unique nonempty labels")
                    else:
                        names.add(label)
                    value = count(row.get("count"), f"population.{key}.categories[{row_index}].count")
                    if value is not None:
                        values.append(value)
                if mothers is not None and sum(values) != mothers:
                    errors.append(f"population.{key}.categories must reconcile to mothers")
            if key == "previous_pregnancies":
                not_stated = count(item.get("not_stated_count"), "population.previous_pregnancies.not_stated_count")
                if rows is not None and mothers is not None and not_stated is not None:
                    values_sum = sum(count(row.get("count"), "previous_pregnancies.count") or 0 for row in rows if isinstance(row, dict))
                    if values_sum + not_stated != mothers:
                        errors.append("previous-pregnancy categories plus not-stated must reconcile to mothers")

    onset = object_at(profile, "labour_onset", "labour_onset")
    if onset is not None:
        if onset.get("count_unit") != "mothers":
            errors.append("labour_onset.count_unit must be mothers")
        rows = list_at(onset, "mothers_by_category", "labour_onset.mothers_by_category")
        if rows is not None:
            values = []
            for index, row in enumerate(rows):
                if not isinstance(row, dict):
                    errors.append(f"labour_onset.mothers_by_category[{index}] must be an object")
                    continue
                value = count(row.get("count"), f"labour_onset.mothers_by_category[{index}].count")
                if value is not None:
                    values.append(value)
            if mothers is not None and sum(values) != mothers:
                errors.append("labour_onset categories must reconcile to mothers")

    routes = object_at(profile, "routes", "routes")
    if routes is not None:
        if routes.get("count_unit") != "births/babies, source Table 6.05":
            errors.append("routes.count_unit must retain the Table 6.05 baby denominator")
        rows = list_at(routes, "births_by_method", "routes.births_by_method")
        if rows is not None:
            values = []
            for index, row in enumerate(rows):
                if not isinstance(row, dict):
                    errors.append(f"routes.births_by_method[{index}] must be an object")
                    continue
                value = count(row.get("count"), f"routes.births_by_method[{index}].count")
                if value is not None:
                    values.append(value)
            if babies is not None and sum(values) != babies:
                errors.append("birth-method counts must reconcile to babies")
        notes = list_at(routes, "cross_tab_notes", "routes.cross_tab_notes")
        if notes is not None and not notes:
            errors.append("cross_tab_notes must retain denominator and linkage limits")

    for key, expected_status in (
        ("staffing", "unavailable_as_public_parameter"),
        ("eligibility", "not_encoded_without_site_scope"),
        ("operational_timing_targets", "unconfigured_without_site_scope"),
    ):
        item = object_at(profile, key, key)
        if item is None:
            continue
        invalid = item.get("status") != expected_status
        if key != "operational_timing_targets":
            invalid = invalid or item.get("values") is not None
        if invalid:
            errors.append(f"{key} must remain explicitly unconfigured for the generic reference profile")
    timing = profile.get("operational_timing_targets", {})
    if timing.get("time_unit") != "minutes when a future source-qualified target is selected":
        errors.append("operational_timing_targets.time_unit must state the prospective unit without adding a target")
    targets = profile.get("operational_timing_targets", {}).get("targets")
    if targets != []:
        errors.append("operational_timing_targets.targets must remain empty without site scope")

    limits = object_at(profile, "inference_limits", "inference_limits")
    if limits is not None:
        for field in ("individual_hazard_model_supported", "person_level_records_present", "complete_event_history_present", "full_joint_distribution_present"):
            if limits.get(field) is not False:
                errors.append(f"inference_limits.{field} must remain false")
        unknown = list_at(limits, "unknown_or_unresolved", "inference_limits.unknown_or_unresolved")
        correlations = list_at(limits, "known_correlations", "inference_limits.known_correlations")
        if unknown is not None and not unknown:
            errors.append("unknown_or_unresolved must retain evidence gaps")
        if correlations is not None and correlations:
            errors.append("known_correlations cannot invent linkage absent joint evidence")

    assumptions = object_at(profile, "synthetic_assumptions", "synthetic_assumptions")
    if assumptions is not None and assumptions.get("status") != "none_in_this_reference_profile":
        errors.append("synthetic assumptions must remain separate from observed aggregates")
    model_use = object_at(profile, "model_use", "model_use")
    if model_use is not None:
        for field in ("runnable_default", "accepted_parameter_pack", "clinical_validation", "operational_acceptance"):
            if model_use.get(field) is not False:
                errors.append(f"model_use.{field} must remain false")

    eligibility = profile.get("eligibility")
    if isinstance(eligibility, dict):
        refs = eligibility.get("policy_sources")
        if not isinstance(refs, list) or not set(refs) <= source_ids:
            errors.append("eligibility.policy_sources must resolve to versioned policy records")
    return errors


class BirthingReferenceProfileContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile = load_profile()

    def test_reference_profile_schema_and_provenance(self) -> None:
        self.assertEqual(contract_errors(self.profile), [])
        observed = self.profile["sources"]["observed_aggregates"]
        self.assertEqual(observed["collection"], "Queensland Perinatal Data Collection")
        self.assertEqual(observed["accessed"], self.profile["profile_revision_date"])
        self.assertEqual(len({row["source_id"] for row in self.profile["sources"]["policy_registry"]}),
                         len(self.profile["sources"]["policy_registry"]))

    def test_count_ranges_and_mother_baby_denominators_reconcile(self) -> None:
        population = self.profile["population"]
        mothers = population["mothers"]["count"]
        babies = population["babies"]["count"]
        for field in ("previous_pregnancies", "plurality", "maternal_age"):
            rows = population[field]["categories"]
            self.assertTrue(all(type(row["count"]) is int and row["count"] >= 0 for row in rows))
            self.assertEqual(sum(row["count"] for row in rows), mothers, field)
        self.assertEqual(sum(row["count"] for row in self.profile["labour_onset"]["mothers_by_category"]), mothers)
        self.assertEqual(sum(row["count"] for row in self.profile["routes"]["births_by_method"]), babies)
        self.assertNotEqual(mothers, babies)

    def test_unknown_evidence_and_unresolved_correlations_are_retained(self) -> None:
        limits = self.profile["inference_limits"]
        self.assertFalse(limits["individual_hazard_model_supported"])
        self.assertFalse(limits["full_joint_distribution_present"])
        self.assertEqual(limits["known_correlations"], [])
        self.assertGreater(len(limits["unknown_or_unresolved"]), 0)
        self.assertEqual(self.profile["synthetic_assumptions"]["status"], "none_in_this_reference_profile")
        self.assertEqual(self.profile["operational_timing_targets"]["time_unit"],
                         "minutes when a future source-qualified target is selected")

    def test_policy_reviews_keep_source_versions_separate_from_data_year(self) -> None:
        policies = self.profile["sources"]["policy_registry"]
        self.assertTrue(all("url" in row and "last_amended" in row for row in policies))
        current_fetal = next(row for row in policies if row["source_id"] == "qcg_intrapartum_fetal_surveillance_mn25_15_v12_r30")
        self.assertIn("2026-09-25", current_fetal["version_note"])
        instrumental = next(row for row in policies if row["source_id"] == "qcg_instrumental_vaginal_birth_current_2026_10")
        self.assertEqual(instrumental["review_dates"], {
            "pdf": "2029-12", "publication_index": "2029-04", "status": "conflicting_unresolved",
        })
        self.assertNotEqual(self.profile["data_year"], int(current_fetal["last_amended"][:4]))

    def test_negative_count_is_rejected(self) -> None:
        mutated = copy.deepcopy(self.profile)
        mutated["routes"]["births_by_method"][0]["count"] = -1
        errors = contract_errors(mutated)
        self.assertTrue(any("nonnegative integer count" in error for error in errors))

    def test_denominator_corruption_is_rejected(self) -> None:
        mutated = copy.deepcopy(self.profile)
        mutated["routes"]["births_by_method"][0]["count"] += 1
        self.assertIn("birth-method counts must reconcile to babies", contract_errors(mutated))

    def test_non_object_labour_onset_row_is_rejected(self) -> None:
        mutated = copy.deepcopy(self.profile)
        mutated["labour_onset"]["mothers_by_category"].append("not-an-object")
        self.assertIn("labour_onset.mothers_by_category[4] must be an object", contract_errors(mutated))

    def test_non_object_birth_route_row_is_rejected(self) -> None:
        mutated = copy.deepcopy(self.profile)
        mutated["routes"]["births_by_method"].append("not-an-object")
        self.assertIn("routes.births_by_method[5] must be an object", contract_errors(mutated))

    def test_unit_corruption_is_rejected(self) -> None:
        mutated = copy.deepcopy(self.profile)
        mutated["labour_onset"]["count_unit"] = "babies"
        self.assertIn("labour_onset.count_unit must be mothers", contract_errors(mutated))

    def test_invented_site_staffing_is_rejected(self) -> None:
        mutated = copy.deepcopy(self.profile)
        mutated["staffing"]["values"] = {"midwives_per_patient": 1}
        self.assertIn("staffing must remain explicitly unconfigured for the generic reference profile", contract_errors(mutated))

    def test_missing_correlation_limit_is_rejected(self) -> None:
        mutated = copy.deepcopy(self.profile)
        mutated["inference_limits"]["unknown_or_unresolved"] = []
        self.assertIn("unknown_or_unresolved must retain evidence gaps", contract_errors(mutated))


if __name__ == "__main__":
    unittest.main()
