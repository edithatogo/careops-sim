#!/usr/bin/env python3
"""Validate the frozen D3.3 support contract without claiming release compatibility."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import sys
import tomllib
from typing import Any


CANONICAL_RUST = "1.99.0"
# Schema v1 retains the field name; this now enforces the sole declared floor.
DEFAULT_MSRV = "1.99"
EXPECTED_FIXTURE_PATH = "libs/kairos/crates/kairo-ecs-arrow-io/tests/fixtures/calibration_physical_v2/manifest.json"
EXPECTED_FIXTURE_SHA = "e37ade3d61550e58273989fd12c46091ddfdd1f01eaf55a8eb76af72eb1d4082"
EXPECTED_COVERAGE_FILE = "crates/careops-ed/src/lib.rs"
EXPECTED_MUTATION_PACKAGE = "careops-ed"
EXPECTED_MANIFESTS = (
    "Cargo.toml",
    "crates/careops-ed/Cargo.toml",
    "crates/careops-ed-cli/Cargo.toml",
)
TOP_KEYS = {
    "schema_version", "canonical_rust", "default_msrv", "packages", "semver",
    "schema_fixture", "flakes", "coverage", "mutation",
}


def _keys(value: Any, expected: set[str], where: str, errors: list[str]) -> bool:
    if not isinstance(value, dict):
        errors.append(f"{where} must be an object")
        return False
    actual = set(value)
    missing, extra = expected - actual, actual - expected
    if missing:
        errors.append(f"{where} missing keys: {', '.join(sorted(missing))}")
    if extra:
        errors.append(f"{where} has unreviewed keys: {', '.join(sorted(extra))}")
    return not missing and not extra


def _safe_file(root: Path, relative: Any, label: str, errors: list[str]) -> Path | None:
    if not isinstance(relative, str) or not relative:
        errors.append(f"{label} path must be a nonempty relative path")
        return None
    if "\x00" in relative:
        errors.append(f"{label} path contains NUL")
        return None
    candidate = Path(relative)
    if candidate.is_absolute() or any(part in ("", ".", "..") for part in candidate.parts):
        errors.append(f"{label} path is unsafe or escapes the repository: {relative!r}")
        return None
    current = root
    for part in candidate.parts:
        current = current / part
        if current.is_symlink():
            errors.append(f"{label} path contains a symlink: {relative!r}")
            return None
    try:
        resolved = current.resolve(strict=True)
        resolved.relative_to(root)
    except (OSError, ValueError):
        errors.append(f"{label} path is missing or escapes the repository: {relative!r}")
        return None
    if not resolved.is_file():
        errors.append(f"{label} path is not a regular file: {relative!r}")
        return None
    return resolved


def _load_config(config: Any, root: Path, errors: list[str]) -> dict[str, Any] | None:
    if isinstance(config, dict):
        return config
    path = Path(config) if config is not None else Path(".config/d33-quality-gates.json")
    if path.is_absolute():
        if any(part.is_symlink() and part.resolve().is_relative_to(root)
               for part in (path, *path.parents)):
            errors.append("config path contains a symlink")
            return None
        try:
            relative_parent = path.parent.resolve(strict=True).relative_to(root)
            relpath = (relative_parent / path.name).as_posix()
        except (OSError, ValueError):
            errors.append("config path must resolve inside the repository")
            return None
    else:
        relpath = path.as_posix()
    safe = _safe_file(root, relpath, "config", errors)
    if safe is None:
        return None
    try:
        value = json.loads(safe.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        errors.append(f"config cannot be read as UTF-8 JSON: {exc}")
        return None
    if not isinstance(value, dict):
        errors.append("config must be a JSON object")
        return None
    return value


def _validate_config(config: dict[str, Any], errors: list[str]) -> None:
    if not _keys(config, TOP_KEYS, "config", errors):
        return
    if type(config["schema_version"]) is not int or config["schema_version"] != 1:
        errors.append("config schema_version must be integer 1")
    if config["canonical_rust"] != CANONICAL_RUST:
        errors.append(f"config canonical_rust must be {CANONICAL_RUST}")
    if config["default_msrv"] != DEFAULT_MSRV:
        errors.append(f"config default_msrv must be {DEFAULT_MSRV}")

    packages = config["packages"]
    if not isinstance(packages, list) or len(packages) != len(EXPECTED_MANIFESTS):
        errors.append("config packages must list exactly the three reviewed manifests")
    else:
        seen = []
        for index, item in enumerate(packages):
            if not _keys(item, {"manifest", "features"}, f"config packages[{index}]", errors):
                continue
            manifest = item["manifest"]
            if not isinstance(manifest, str):
                errors.append(f"config packages[{index}].manifest must be a string")
            else:
                seen.append(manifest)
            if not isinstance(item["features"], dict) or any(
                not isinstance(name, str) or not name or not isinstance(values, list)
                or any(not isinstance(entry, str) or not entry for entry in values)
                for name, values in item["features"].items()
            ):
                errors.append(f"config packages[{index}].features must map names to string lists")
            elif item["features"] != {}:
                errors.append(f"config packages[{index}] has unapproved feature maps; contract review is required")
        if seen != list(EXPECTED_MANIFESTS):
            errors.append("config packages must list the exact reviewed manifests in order")

    semver = config["semver"]
    if _keys(semver, {"state", "owner", "release_allowed"}, "config semver", errors):
        if (semver["state"] != "unreleased_no_approved_release_baseline"
                or semver["owner"] != "Track25 / D4" or semver["release_allowed"] is not False):
            errors.append("config semver must record the unreleased, unapproved baseline")

    fixture = config["schema_fixture"]
    if _keys(fixture, {"path", "sha256", "scope"}, "config schema_fixture", errors):
        if isinstance(fixture["path"], str) and "\x00" in fixture["path"]:
            errors.append("config schema_fixture.path contains NUL")
        if fixture["path"] != EXPECTED_FIXTURE_PATH:
            errors.append(f"config schema_fixture.path must be exactly {EXPECTED_FIXTURE_PATH}")
        if fixture["sha256"] != EXPECTED_FIXTURE_SHA:
            errors.append("config schema_fixture.sha256 differs from the reviewed fixture pin")
        if not isinstance(fixture["sha256"], str) or not re.fullmatch(r"[0-9a-f]{64}", fixture["sha256"]):
            errors.append("config schema_fixture.sha256 must be 64 lowercase hexadecimal characters")
        if not isinstance(fixture["scope"], str) or not fixture["scope"].strip():
            errors.append("config schema_fixture.scope must be nonempty text")

    if not isinstance(config["flakes"], list):
        errors.append("config flakes must be a list; entry validation belongs to quality_flakes")
    coverage = config["coverage"]
    if _keys(coverage, {"file", "minimum_line_percent", "scope", "basis"}, "config coverage", errors):
        if coverage["file"] != EXPECTED_COVERAGE_FILE:
            errors.append(f"config coverage.file must be exactly {EXPECTED_COVERAGE_FILE}")
        minimum = coverage["minimum_line_percent"]
        if (isinstance(minimum, bool) or not isinstance(minimum, (int, float))
                or not 80 <= minimum <= 100 or not math.isfinite(minimum)):
            errors.append("config coverage.minimum_line_percent must be between 80 and 100")
        for key in ("scope", "basis"):
            if not isinstance(coverage[key], str) or not coverage[key].strip():
                errors.append(f"config coverage.{key} must be nonempty text")
    mutation = config["mutation"]
    if _keys(mutation, {"package", "file", "policy"}, "config mutation", errors):
        if mutation["package"] != EXPECTED_MUTATION_PACKAGE:
            errors.append(f"config mutation package must be exactly {EXPECTED_MUTATION_PACKAGE}")
        if mutation["file"] != EXPECTED_COVERAGE_FILE:
            errors.append(f"config mutation.file must be exactly {EXPECTED_COVERAGE_FILE}")
        for key in ("package", "file", "policy"):
            if not isinstance(mutation[key], str) or not mutation[key].strip():
                errors.append(f"config mutation.{key} must be nonempty text")


def _read_toml(path: Path, label: str, errors: list[str]) -> dict[str, Any] | None:
    try:
        with path.open("rb") as stream:
            value = tomllib.load(stream)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        errors.append(f"{label} is not readable valid TOML: {exc}")
        return None
    return value


def _check_toolchain(root: Path, errors: list[str]) -> None:
    path = _safe_file(root, "rust-toolchain.toml", "rust-toolchain", errors)
    if path is None:
        return
    document = _read_toml(path, "rust-toolchain.toml", errors)
    if document is None:
        return
    toolchain = document.get("toolchain")
    if not isinstance(toolchain, dict) or toolchain.get("channel") != CANONICAL_RUST:
        errors.append(f"rust-toolchain.toml channel must be canonical {CANONICAL_RUST}")


def _check_manifests(root: Path, config: dict[str, Any], errors: list[str]) -> None:
    packages = config.get("packages")
    if not isinstance(packages, list):
        return
    expected_msrv = config.get("default_msrv")
    workspace_rust_version = None
    for index, spec in enumerate(packages):
        if not isinstance(spec, dict) or not isinstance(spec.get("manifest"), str):
            continue
        manifest_path = _safe_file(root, spec["manifest"], f"manifest[{index}]", errors)
        if manifest_path is None:
            continue
        document = _read_toml(manifest_path, spec["manifest"], errors)
        if document is None:
            continue
        actual_features = document.get("features", {})
        if not isinstance(actual_features, dict):
            errors.append(f"{spec['manifest']} [features] must be a table")
            actual_features = None
        if actual_features is not None and actual_features != spec.get("features"):
            errors.append(f"{spec['manifest']} declared feature map differs from reviewed config")
        if spec.get("features") != {}:
            errors.append(f"{spec['manifest']} config contains unapproved feature maps; contract review is required")
        if index == 0:
            workspace = document.get("workspace")
            workspace_package = workspace.get("package") if isinstance(workspace, dict) else None
            workspace_rust_version = (workspace_package.get("rust-version")
                                      if isinstance(workspace_package, dict) else None)
            if workspace_rust_version != expected_msrv:
                errors.append(f"workspace.package.rust-version must equal configured default MSRV {expected_msrv}")
            package = document.get("package")
            root_version = package.get("rust-version") if isinstance(package, dict) else None
            if root_version not in (None, expected_msrv, {"workspace": True}):
                errors.append("root package rust-version must inherit or equal configured default MSRV")
        else:
            package = document.get("package")
            actual_msrv = package.get("rust-version") if isinstance(package, dict) else None
            if actual_msrv != {"workspace": True}:
                errors.append(f"{spec['manifest']} must inherit rust-version from workspace.package")
            if workspace_rust_version != expected_msrv:
                errors.append(f"{spec['manifest']} inherited MSRV does not resolve to {expected_msrv}")


def _check_fixture(root: Path, config: dict[str, Any], errors: list[str]) -> None:
    fixture = config.get("schema_fixture")
    if not isinstance(fixture, dict):
        return
    path = _safe_file(root, EXPECTED_FIXTURE_PATH, "schema fixture", errors)
    if path is None:
        return
    try:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as exc:
        errors.append(f"schema fixture cannot be read: {exc}")
        return
    if digest != EXPECTED_FIXTURE_SHA:
        errors.append(f"schema fixture SHA-256 mismatch: expected {EXPECTED_FIXTURE_SHA}, got {digest}")


def check(root: str | Path, config: Any, today: Any = None, release: bool = False) -> list[str]:
    """Return contract errors; development mode never claims a semver comparison."""
    del today  # Reserved for compatible future checks; this leaf has no dated policies.
    errors: list[str] = []
    root_path = Path(root).resolve()
    if not root_path.is_dir():
        return [f"repository root is not a directory: {root}"]
    value = _load_config(config, root_path, errors)
    if value is not None:
        _validate_config(value, errors)
        _check_toolchain(root_path, errors)
        _check_manifests(root_path, value, errors)
        _check_fixture(root_path, value, errors)
        if release:
            errors.append("release blocked: no owner-approved stable API release baseline exists; semver comparison is unavailable")
    elif release:
        errors.append("release blocked: no owner-approved stable API release baseline exists; semver comparison is unavailable")
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--config", type=Path)
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args(argv)
    errors = check(args.root, args.config, release=args.release)
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("Development support checks passed; semver comparison was not performed because no owner-approved stable API baseline exists.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
