"""Contract tests for the P1.3 developer-only DES catalogue checker."""

from __future__ import annotations

import copy
import json
import math
import hashlib
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

from validate_des_catalogue import catalogue_errors, semantic_errors  # noqa: E402


FAMILIES = (
    "demand_case_mix",
    "des_pathways",
    "durations",
    "resources",
    "patient_behavior",
    "hospital_interfaces",
)


def baseline():
    return {
        family: json.loads((ROOT / "model-inputs/ed/des" / f"{family}.json").read_text())
        for family in FAMILIES
    }


def record_for(collections, parameter_id):
    for collection in collections.values():
        for record in collection["records"]:
            if record["parameter_id"] == parameter_id:
                return record
    raise AssertionError(f"missing fixture record {parameter_id}")


class CatalogueCoverageTests(unittest.TestCase):
    def test_current_catalogue_passes(self):
        self.assertEqual([], catalogue_errors(baseline(), root=ROOT))

    def test_missing_family_and_missing_record_are_reported(self):
        collections = baseline()
        collections.pop("resources")
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("resources" in error.lower() for error in errors), errors)

        collections = baseline()
        collections["des_pathways"]["records"].pop()
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.despathways.preemption" in error for error in errors), errors)

    def test_duplicate_id_and_annotation_key_mismatch_fail_with_id(self):
        collections = baseline()
        collections["des_pathways"]["records"].append(
            copy.deepcopy(collections["des_pathways"]["records"][0])
        )
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.despathways.registration" in error for error in errors), errors)

    def test_malformed_unhashable_record_id_fails_closed(self):
        collections = baseline()
        collections["durations"]["records"][0]["parameter_id"] = {}
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("durations" in error and "parameter_id" in error for error in errors), errors)

        collections = baseline()
        annotations = collections["des_pathways"]["annotations"]
        annotations.pop("ed.despathways.registration")
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.despathways.registration" in error for error in errors), errors)

    def test_family_unit_and_annotation_contract_mismatches_fail(self):
        collections = baseline()
        record = record_for(collections, "ed.durations.boarding")
        record["unit"] = "minutes"
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.durations.boarding" in error and "unit" in error.lower() for error in errors), errors)

        collections = baseline()
        annotation = collections["des_pathways"]["annotations"]["ed.despathways.routingprobability"]
        annotation["value_role"] = "observed_target"
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.despathways.routingprobability" in error for error in errors), errors)

    def test_matrix_deferred_row_cannot_be_promoted(self):
        collections = baseline()
        record = record_for(collections, "ed.patientbehavior.mobilityassistance")
        record["value_status"] = {"status": "unknown", "reason": "test"}
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.patientbehavior.mobilityassistance" in error for error in errors), errors)


class CatalogueEvidenceTests(unittest.TestCase):
    def test_stale_and_semantically_wrong_evidence_refs_fail(self):
        collections = baseline()
        annotation = collections["des_pathways"]["annotations"]["ed.despathways.routingprobability"]
        annotation["evidence_refs"][0]["sha256"] = "0" * 64
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.despathways.routingprobability" in error for error in errors), errors)

        collections = baseline()
        annotation = collections["des_pathways"]["annotations"]["ed.despathways.routingprobability"]
        source = ROOT / "model-inputs/ed/des/evidence/p1-staff-breaks.json"
        annotation["evidence_refs"] = [{
            "path": "model-inputs/ed/des/evidence/p1-staff-breaks.json",
            "sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        }]
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.despathways.routingprobability" in error and "does not explicitly cover" in error for error in errors), errors)

    def test_review_and_explicit_parameter_membership_are_required(self):
        collections = baseline()
        path = ROOT / "model-inputs/ed/des/evidence/p1-pathway-routing.json"
        with tempfile.TemporaryDirectory() as td:
            temp_root = Path(td)
            shutil.copytree(ROOT / "model-inputs", temp_root / "model-inputs")
            evidence_copy = temp_root / path.relative_to(ROOT)
            changed = json.loads(evidence_copy.read_text())
            changed["review"]["outcome"] = "pending"
            evidence_copy.write_text(json.dumps(changed))
            ann = collections["des_pathways"]["annotations"]["ed.despathways.routingprobability"]
            ann["evidence_refs"] = [{
                "path": "model-inputs/ed/des/evidence/p1-pathway-routing.json",
                "sha256": hashlib.sha256(evidence_copy.read_bytes()).hexdigest(),
            }]
            errors = catalogue_errors(collections, root=temp_root)
            self.assertTrue(any("ed.despathways.routingprobability" in error and "coordinator-accepted" in error for error in errors), errors)

    def test_absolute_and_traversing_evidence_paths_fail(self):
        for bad_path in (
            str(ROOT / "model-inputs/ed/des/evidence/p1-pathway-routing.json"),
            "model-inputs/ed/des/evidence/../../../../tmp/elsewhere.json",
        ):
            collections = baseline()
            ann = collections["des_pathways"]["annotations"]["ed.despathways.routingprobability"]
            ann["evidence_refs"][0]["path"] = bad_path
            errors = catalogue_errors(collections, root=ROOT)
            self.assertTrue(any("ed.despathways.routingprobability" in error for error in errors), errors)

    def test_symlinked_evidence_escape_is_rejected(self):
        collections = baseline()
        with tempfile.TemporaryDirectory() as td:
            temp_root = Path(td) / "repo"
            shutil.copytree(ROOT / "model-inputs", temp_root / "model-inputs")
            outside = Path(td) / "outside.json"
            outside.write_text(json.dumps({"review": {"outcome": "accepted"}, "parameter_ids": ["ed.despathways.routingprobability"]}))
            link = temp_root / "model-inputs/ed/des/evidence/escape.json"
            link.symlink_to(outside)
            ann = collections["des_pathways"]["annotations"]["ed.despathways.routingprobability"]
            ann["evidence_refs"].append({
                "path": "model-inputs/ed/des/evidence/escape.json",
                "sha256": hashlib.sha256(outside.read_bytes()).hexdigest(),
            })
            errors = catalogue_errors(collections, root=temp_root)
            self.assertTrue(any("ed.despathways.routingprobability" in error and "escapes evidence root" in error for error in errors), errors)

    def test_evidence_directory_symlink_escape_is_rejected(self):
        collections = baseline()
        with tempfile.TemporaryDirectory() as td:
            temp_root = Path(td) / "repo"
            shutil.copytree(ROOT / "model-inputs", temp_root / "model-inputs")
            evidence_dir = temp_root / "model-inputs/ed/des/evidence"
            outside = Path(td) / "outside-evidence"
            shutil.move(str(evidence_dir), outside)
            evidence_dir.symlink_to(outside, target_is_directory=True)
            errors = catalogue_errors(collections, root=temp_root)
            self.assertTrue(any("evidence directory escapes repository root" in error for error in errors), errors)


class CurrentCatalogueBoundaryTests(unittest.TestCase):
    def test_unknown_zero_defaults_distributions_and_known_ranges_fail(self):
        mutations = (
            ("value", 0),
            ("reference_default", 0),
            ("distribution", {"family": "gamma"}),
        )
        for key, value in mutations:
            collections = baseline()
            record_for(collections, "ed.despathways.routingprobability")[key] = value
            errors = catalogue_errors(collections, root=ROOT)
            self.assertTrue(any("ed.despathways.routingprobability" in error for error in errors), errors)

        collections = baseline()
        record_for(collections, "ed.despathways.routingprobability")["physical_limits"] = {
            "status": "known", "minimum": 0, "maximum": 1, "unit": "dimensionless",
            "provenance": "test", "rationale": "test"
        }
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.despathways.routingprobability" in error for error in errors), errors)

    def test_boarding_is_output_without_candidate_families(self):
        collections = baseline()
        record_for(collections, "ed.durations.boarding")["value_kind"] = "empirical_sample"
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.durations.boarding" in error for error in errors), errors)

        collections = baseline()
        collections["durations"]["annotations"]["ed.durations.boarding"]["candidate_families"] = ["gamma"]
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.durations.boarding" in error for error in errors), errors)

    def test_candidates_are_unselected_nonnumeric_research_labels(self):
        collections = baseline()
        ann = collections["des_pathways"]["annotations"]["ed.despathways.routingprobability"]
        ann["candidate_families"] = ["gamma"]
        ann["candidate_status"] = "selected"
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.despathways.routingprobability" in error for error in errors), errors)


class FutureShapeSemanticHelperTests(unittest.TestCase):
    """Developer-only shape checks; passing these fixtures approves no runtime profile."""

    def record(self, **changes):
        record = copy.deepcopy(record_for(baseline(), "ed.despathways.routingprobability"))
        record.update(changes)
        return record

    def test_probability_scalar_and_frozen_choice_table_shape(self):
        self.assertEqual([], semantic_errors(self.record(value_status={"status": "known"}, value=0.25)))
        table = {
            "probabilities": [0.25, 0.75],
            "alternatives": ["a", "b"],
            "denominator": 20,
            "decision_point": "fixture only",
            "conditioning": {"stratum": "fixture"},
        }
        self.assertEqual([], semantic_errors(self.record(value_status={"status": "known"}, value=table)))
        invalid = copy.deepcopy(table)
        invalid["probabilities"] = [0.2, 0.7]
        self.assertTrue(semantic_errors(self.record(value_status={"status": "known"}, value=invalid)))

    def test_probability_rejects_nonfinite_out_of_range_and_unbound_list(self):
        for value in (math.nan, math.inf, -0.1, 1.1, [0.5, 0.5]):
            errors = semantic_errors(self.record(value_status={"status": "known"}, value=value))
            self.assertTrue(errors, value)

    def test_resource_shape_checks_physical_staffed_open_order(self):
        base = copy.deepcopy(record_for(baseline(), "ed.resources.beds"))
        base["value_status"] = {"status": "known"}
        base["value"] = {"physical": 12, "staffed": 10, "open": 8}
        self.assertEqual([], semantic_errors(base))
        base["value"] = {"physical": 9, "staffed": 10, "open": 8}
        self.assertTrue(semantic_errors(base))

    def test_known_ranges_require_finite_ordered_nonnegative_values_and_matching_units(self):
        record = self.record(value_status={"status": "known"}, value=0.25)
        record["type"] = "time"
        record["unit"] = "s"
        record["physical_limits"] = {
            "status": "known", "minimum": 10, "maximum": 5, "unit": "s",
            "provenance": "fixture", "rationale": "fixture"
        }
        self.assertTrue(semantic_errors(record))
        record["physical_limits"]["minimum"] = math.nan
        self.assertTrue(semantic_errors(record))

    def test_malformed_status_and_overflowing_integer_fail_closed(self):
        record = self.record(value_status={"status": {}})
        self.assertTrue(semantic_errors(record))

        collections = baseline()
        record_for(collections, "ed.despathways.routingprobability")["physical_limits"]["status"] = {}
        errors = catalogue_errors(collections, root=ROOT)
        self.assertTrue(any("ed.despathways.routingprobability" in error for error in errors), errors)

        record = self.record(value_status={"status": "known"}, value=10**400)
        self.assertTrue(semantic_errors(record))

        record = self.record()
        record["physical_limits"]["status"] = {}
        self.assertTrue(semantic_errors(record))

        record = self.record(value_status={"status": "known"}, value=0.5, value_kind="parametric_distribution")
        self.assertTrue(semantic_errors(record))


if __name__ == "__main__":
    unittest.main()
