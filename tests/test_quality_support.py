import hashlib
import importlib.util
import json
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("quality_support", ROOT / "tools" / "quality_support.py")
quality_support = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(quality_support)


class QualitySupportTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        fixture = b'{"kind":"synthetic-arrow-fixture","version":2}\n'
        self.fixture_sha = hashlib.sha256(fixture).hexdigest()
        self.fixture_path = "libs/kairos/crates/kairo-ecs-arrow-io/tests/fixtures/calibration_physical_v2/manifest.json"
        self.write(self.fixture_path, fixture)
        self.config = {
            "schema_version": 1,
            "canonical_rust": "1.99.0",
            "default_msrv": "1.76",
            "packages": [
                {"manifest": "Cargo.toml", "features": {}},
                {"manifest": "crates/careops-ed/Cargo.toml", "features": {}},
                {"manifest": "crates/careops-ed-cli/Cargo.toml", "features": {}},
            ],
            "semver": {
                "state": "unreleased_no_approved_release_baseline",
                "owner": "Track25 / D4",
                "release_allowed": False,
            },
            "schema_fixture": {
                "path": self.fixture_path,
                "sha256": self.fixture_sha,
                "scope": "synthetic fixture",
            },
            "flakes": [],
            "coverage": {"file": "crates/careops-ed/src/lib.rs", "minimum_line_percent": 80,
                         "scope": "dev", "basis": "test baseline"},
            "mutation": {"package": "careops-ed", "file": "crates/careops-ed/src/lib.rs",
                         "policy": "separate evidence"},
        }
        self.write("rust-toolchain.toml", '[toolchain]\nchannel = "1.99.0"\n')
        self.write("Cargo.toml", '[workspace]\nmembers = ["crates/careops-ed", "crates/careops-ed-cli"]\n\n'
                   '[workspace.package]\nrust-version = "1.76"\n\n[package]\nname = "careops-sim"\n')
        self.write("crates/careops-ed/Cargo.toml", '[package]\nname = "careops-ed"\nrust-version.workspace = true\n')
        self.write("crates/careops-ed-cli/Cargo.toml", '[package]\nname = "careops-ed-cli"\nrust-version.workspace = true\n')

    def tearDown(self):
        self.temp.cleanup()

    def write(self, relative, content):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content if isinstance(content, bytes) else content.encode())
        return path

    def errors(self, **kwargs):
        with patch.object(quality_support, "EXPECTED_FIXTURE_SHA", self.fixture_sha):
            return quality_support.check(self.root, self.config, **kwargs)

    def test_matching_contract_passes_development_without_semver_claim(self):
        errors = self.errors()
        self.assertEqual(errors, [])
        self.assertFalse(any("semver comparison passed" in error.lower() for error in errors))

    def test_release_fails_clearly_without_owner_approved_baseline(self):
        errors = self.errors(release=True)
        self.assertTrue(any("no owner-approved stable API release baseline" in error for error in errors), errors)

    def test_actual_toolchain_channel_must_be_canonical(self):
        self.write("rust-toolchain.toml", '[toolchain]\nchannel = "stable"\n')
        self.assertTrue(any("canonical" in error.lower() for error in self.errors()))

    def test_workspace_msrv_and_inherited_package_msrv_must_match(self):
        self.write("Cargo.toml", '[workspace]\nmembers = ["crates/careops-ed", "crates/careops-ed-cli"]\n\n'
                   '[workspace.package]\nrust-version = "1.77"\n\n[package]\nname = "careops-sim"\n')
        errors = self.errors()
        self.assertTrue(any("MSRV" in error for error in errors), errors)

    def test_unreviewed_feature_declaration_is_rejected(self):
        self.write("crates/careops-ed/Cargo.toml", '[package]\nname = "careops-ed"\nrust-version.workspace = true\n\n'
                   '[features]\ndefault = []\nfast = []\n')
        errors = self.errors()
        self.assertTrue(any("feature" in error.lower() for error in errors), errors)

    def test_schema_fixture_content_drift_is_rejected(self):
        self.write(self.fixture_path, b'{"kind":"different"}\n')
        errors = self.errors()
        self.assertTrue(any("SHA-256" in error or "sha256" in error.lower() for error in errors), errors)

    def test_fixture_path_and_hash_cannot_co_drift_with_candidate_config(self):
        replacement = b'{"kind":"candidate-fixture"}\n'
        replacement_path = "fixtures/candidate/manifest.json"
        self.write(replacement_path, replacement)
        self.config["schema_fixture"]["path"] = replacement_path
        self.config["schema_fixture"]["sha256"] = hashlib.sha256(replacement).hexdigest()
        errors = self.errors()
        self.assertTrue(any("schema_fixture.path" in error for error in errors), errors)
        self.assertTrue(any("schema_fixture.sha256" in error for error in errors), errors)

    def test_unpatched_schema_fixture_constants_match_reviewed_contract(self):
        self.assertEqual(
            quality_support.EXPECTED_FIXTURE_PATH,
            "libs/kairos/crates/kairo-ecs-arrow-io/tests/fixtures/calibration_physical_v2/manifest.json",
        )
        self.assertEqual(
            quality_support.EXPECTED_FIXTURE_SHA,
            "e37ade3d61550e58273989fd12c46091ddfdd1f01eaf55a8eb76af72eb1d4082",
        )

    def test_matching_config_and_manifest_cannot_add_unapproved_feature(self):
        self.config["packages"][1]["features"] = {"experimental": []}
        self.write("crates/careops-ed/Cargo.toml", '[package]\nname = "careops-ed"\nrust-version.workspace = true\n\n'
                   '[features]\nexperimental = []\n')
        errors = self.errors()
        self.assertTrue(any("approved feature" in error.lower() for error in errors), errors)

    def test_coverage_and_mutation_targets_cannot_be_rebound(self):
        self.config["coverage"]["file"] = "crates/careops-ed/src/other.rs"
        self.config["coverage"]["minimum_line_percent"] = 79
        self.config["mutation"]["package"] = "careops-ed-cli"
        self.config["mutation"]["file"] = "crates/careops-ed-cli/src/main.rs"
        errors = self.errors()
        self.assertTrue(any("coverage.file" in error for error in errors), errors)
        self.assertTrue(any("minimum_line_percent" in error for error in errors), errors)
        self.assertTrue(any("mutation package" in error.lower() for error in errors), errors)
        self.assertTrue(any("mutation.file" in error for error in errors), errors)

    def test_escaped_fixture_path_is_rejected(self):
        self.config["schema_fixture"]["path"] = "../outside.json"
        errors = self.errors()
        self.assertTrue(any("schema_fixture.path" in error for error in errors), errors)

    def test_symlink_fixture_path_is_rejected(self):
        external = Path(self.temp.name).parent / (Path(self.temp.name).name + "-external.json")
        external.write_text("external", encoding="utf-8")
        link = self.root / self.fixture_path
        link.unlink()
        link.symlink_to(external)
        try:
            errors = self.errors()
            self.assertTrue(any("symlink" in error.lower() or "safe" in error.lower() for error in errors), errors)
        finally:
            external.unlink(missing_ok=True)

    def test_config_shape_rejects_unknown_keys(self):
        self.config["unexpected"] = True
        errors = self.errors()
        self.assertTrue(any("config" in error.lower() and "key" in error.lower() for error in errors), errors)

    def test_absolute_symlink_config_path_is_rejected(self):
        target = self.write("valid-config.json", json.dumps(self.config))
        link = self.root / "config-link.json"
        link.symlink_to(target)
        errors = quality_support.check(self.root, link)
        self.assertTrue(any("symlink" in error.lower() for error in errors), errors)

    def test_absolute_config_symlink_parent_is_rejected(self):
        self.write("real/config.json", json.dumps(self.config))
        (self.root / "alias").symlink_to(self.root / "real", target_is_directory=True)
        errors = quality_support.check(self.root, self.root / "alias/config.json")
        self.assertTrue(any("symlink" in e for e in errors), errors)

    def test_extreme_coverage_floor_is_rejected_without_exception(self):
        self.config["coverage"]["minimum_line_percent"] = 10 ** 1000
        self.assertTrue(any("minimum_line_percent" in e for e in self.errors()))

    def test_cli_exposes_semver_limit_and_fails_release_mode(self):
        config_path = self.write("config.json", json.dumps(self.config))
        stdout, stderr = StringIO(), StringIO()
        with patch.object(quality_support, "EXPECTED_FIXTURE_SHA", self.fixture_sha):
            with redirect_stdout(stdout), redirect_stderr(stderr):
                development_status = quality_support.main([
                    "--root", str(self.root), "--config", str(config_path),
                ])
            self.assertEqual(development_status, 0, stderr.getvalue())
            self.assertIn("semver comparison was not performed", stdout.getvalue())

            stdout, stderr = StringIO(), StringIO()
            with redirect_stdout(stdout), redirect_stderr(stderr):
                release_status = quality_support.main([
                    "--root", str(self.root), "--config", str(config_path), "--release",
                ])
        self.assertEqual(release_status, 1)
        self.assertIn("no owner-approved stable API release baseline", stderr.getvalue())

    def test_nul_in_schema_fixture_path_returns_validation_error(self):
        self.config["schema_fixture"]["path"] = "fixtures/\x00/manifest.json"
        errors = self.errors()
        self.assertTrue(any("NUL" in error for error in errors), errors)

    def test_flakes_must_be_a_list(self):
        self.config["flakes"] = {"test_id": "invalid-container"}
        errors = self.errors()
        self.assertTrue(any("flakes must be a list" in error for error in errors), errors)

    def test_nonempty_flake_entries_are_left_to_the_flake_validator(self):
        self.config["flakes"] = [{"unreviewed": "entry shape belongs to quality_flakes"}]
        errors = self.errors()
        self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()
