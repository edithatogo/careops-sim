import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("verify_d03", ROOT / "tools/verify_d03.py")
verify_d03 = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(verify_d03)


class D03ProfileTests(unittest.TestCase):
    def test_missing_controlled_comparison_marker_is_rejected(self):
        markers = sorted(verify_d03.EXPECTED_ACCEPTANCE_MARKERS["functional_mvp"])
        markers.remove("controlled comparison showing that the selected bed/staff input actually affects a known fixture")
        acceptance = "; ".join(markers)
        profiles = {
            "functional_mvp": {"contract_acceptance_markers": markers, "acceptance": acceptance},
            "hardened_native_v1": {
                "contract_acceptance_markers": list(verify_d03.EXPECTED_ACCEPTANCE_MARKERS["hardened_native_v1"]),
                "acceptance": "; ".join(verify_d03.EXPECTED_ACCEPTANCE_MARKERS["hardened_native_v1"]),
            },
        }
        with self.assertRaisesRegex(SystemExit, "marker set is incomplete"):
            verify_d03.verify_stage_acceptance(profiles, acceptance + " " + profiles["hardened_native_v1"]["acceptance"])

    def test_missing_profile_module_is_rejected(self):
        profiles = {
            "functional_mvp": {"module_names": sorted(verify_d03.EXPECTED_MODULES["MVP"] - {"kairo-ecs-abm"})},
            "hardened_native_v1": {"module_names": sorted(verify_d03.EXPECTED_MODULES["Native v1"])},
            "post_v1": {"module_names": sorted(verify_d03.EXPECTED_MODULES["Post-v1"])},
        }
        actual = {
            name: {"delivery_stage": stage}
            for stage, names in verify_d03.EXPECTED_MODULES.items()
            for name in names
        }
        with self.assertRaisesRegex(SystemExit, "narrative profile module list differs"):
            verify_d03.verify_module_stages(actual, profiles)

    def test_missing_source_hash_is_rejected(self):
        hashes = {path: "unused" for path in verify_d03.EXPECTED_SOURCE_PATHS}
        hashes.pop("conductor/delivery-contract.md")
        with self.assertRaisesRegex(SystemExit, "hash set is incomplete"):
            verify_d03.verify_source_hashes(hashes, ROOT)


if __name__ == "__main__":
    unittest.main()
