import copy
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from tools.validate_abm_catalogue import validate_catalogues, validate_family


ROOT = Path(__file__).resolve().parents[1]


class AbmCatalogueTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.registry_doc = json.loads((ROOT / "model-inputs/ed/schema/parameter-ids.json").read_text())
        cls.matrix_doc = json.loads((ROOT / "model-inputs/ed/schema/parameter-usage-matrix.json").read_text())
        cls.schema = json.loads((ROOT / "model-inputs/ed/schema/parameter-record.schema.json").read_text())
        cls.registry = {entry["parameter_id"]: entry for entry in cls.registry_doc["entries"]}
        cls.matrix = {entry["parameter_id"]: entry for entry in cls.matrix_doc["entries"]}
        cls.validator = Draft202012Validator(cls.schema)
        cls.catalogues = {}
        for family, filename in (("abm_staff_behavior", "staff_behavior.json"), ("spatial_abm", "spatial.json"), ("optional_complexity", "optional_complexity.json")):
            cls.catalogues[family] = json.loads((ROOT / "model-inputs/ed/abm" / filename).read_text())

    def test_all_catalogues_pass_coverage_provenance_and_semantics(self):
        self.assertEqual(validate_catalogues(ROOT), [])

    def _validate(self, family, catalogue):
        return validate_family(ROOT, family, catalogue, self.registry, self.matrix, self.validator)

    def test_missing_family_id_fails(self):
        catalogue = copy.deepcopy(self.catalogues["abm_staff_behavior"])
        catalogue["records"].pop()
        self.assertTrue(any("IDs do not exactly match" in error for error in self._validate("abm_staff_behavior", catalogue)))

    def test_numeric_default_promotion_fails(self):
        catalogue = copy.deepcopy(self.catalogues["spatial_abm"])
        catalogue["records"][0]["reference_default"] = 1.2
        self.assertTrue(any("unsupported value/default/distribution" in error for error in self._validate("spatial_abm", catalogue)))

    def test_usage_matrix_unit_drift_fails(self):
        catalogue = copy.deepcopy(self.catalogues["spatial_abm"])
        catalogue["annotations"]["ed.spatialabm.scaledistance"]["unit_contract"] = "feet"
        self.assertTrue(any("unit contract mismatch" in error for error in self._validate("spatial_abm", catalogue)))

    def test_evidence_reference_scope_and_hash_are_checked(self):
        catalogue = copy.deepcopy(self.catalogues["abm_staff_behavior"])
        entry = catalogue["annotations"]["ed.abmstaffbehavior.agentattributes"]["evidence_refs"][0]
        entry["sha256"] = "0" * 64
        self.assertTrue(any("stale evidence hash" in error for error in self._validate("abm_staff_behavior", catalogue)))

    def test_selected_candidate_or_default_fails(self):
        catalogue = copy.deepcopy(self.catalogues["abm_staff_behavior"])
        catalogue["annotations"]["ed.abmstaffbehavior.taskchoice"]["candidate_status"] = "selected: categorical policy"
        self.assertTrue(any("explicitly unselected" in error for error in self._validate("abm_staff_behavior", catalogue)))


if __name__ == "__main__":
    unittest.main()
