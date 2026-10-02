"""Behavior tests for the P4 exact-byte manifest verifier."""

import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from tools.validate_ed_profile_manifest import EXPECTED_PATHS, validate_manifest

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "model-inputs/ed/profiles/p4-pack-manifest.json"


class ProfileManifestTests(unittest.TestCase):
    def test_all_five_artifacts_have_reproducible_hash_and_required_metadata(self):
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.assertEqual({entry["path"] for entry in manifest["artifacts"]}, EXPECTED_PATHS)
        self.assertEqual(manifest["repository_licensing"]["status"], "not_established")
        for entry in manifest["artifacts"]:
            data = (ROOT / entry["path"]).read_bytes()
            self.assertEqual(entry["sha256"], hashlib.sha256(data).hexdigest())
            self.assertEqual(entry["provenance_class"], "synthetic")
            self.assertTrue(entry["provenance_note"])
            self.assertEqual(entry["redistribution"], "not_established_for_external_redistribution")
            self.assertIsInstance(entry["schema_version"], int)
        self.assertEqual(validate_manifest(MANIFEST, ROOT), [])

    def test_exact_byte_drift_is_rejected(self):
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for entry in manifest["artifacts"]:
                source = ROOT / entry["path"]
                target = root / entry["path"]
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, target)
            target = root / manifest["artifacts"][0]["path"]
            target.write_bytes(target.read_bytes() + b" ")
            manifest_path = root / "manifest.json"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            errors = validate_manifest(manifest_path, root)
            self.assertTrue(any("SHA-256 mismatch" in error for error in errors), errors)

    def test_incomplete_inventory_is_rejected(self):
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        manifest["artifacts"].pop()
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "manifest.json"
            path.write_text(json.dumps(manifest), encoding="utf-8")
            errors = validate_manifest(path, ROOT)
        self.assertTrue(any("five required paths" in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main()
