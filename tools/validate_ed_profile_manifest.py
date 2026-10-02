#!/usr/bin/env python3
"""Verify the exact-byte inventory for the synthetic P4 profile artifacts."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from typing import Any

EXPECTED_PATHS = {
    "model-inputs/ed/profiles/p4-minimal-pack.json",
    "model-inputs/ed/profiles/p4-nominal-pack.json",
    "model-inputs/ed/profiles/p4-surge-pack.json",
    "model-inputs/ed/profiles/p4-validation-cases.json",
    "model-inputs/ed/profiles/p4-capacity-status-cases.json",
}


def validate_manifest(manifest_path: Path, repository_root: Path | None = None) -> list[str]:
    """Return validation errors; file hashes cover exact bytes, not parsed JSON."""
    manifest_path = manifest_path.resolve()
    root = (repository_root or manifest_path.parents[3]).resolve()
    errors: list[str] = []
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return [f"cannot read manifest: {exc}"]

    if manifest.get("manifest_version") != 1:
        errors.append("manifest_version must be 1")
    licensing = manifest.get("repository_licensing", {})
    if licensing.get("status") != "not_established":
        errors.append("repository_licensing.status must be not_established")
    if not licensing.get("evidence_checked") or not licensing.get("redistribution_note"):
        errors.append("repository licensing evidence and disposition are required")

    entries = manifest.get("artifacts")
    if not isinstance(entries, list):
        return errors + ["artifacts must be a list"]
    paths = [entry.get("path") for entry in entries if isinstance(entry, dict)]
    if len(paths) != len(entries) or set(paths) != EXPECTED_PATHS or len(paths) != len(set(paths)):
        errors.append("artifacts must contain each of the five required paths exactly once")

    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            continue
        label = entry.get("path", f"artifacts[{index}]")
        rel = entry.get("path")
        if not isinstance(rel, str) or Path(rel).is_absolute() or ".." in Path(rel).parts:
            errors.append(f"{label}: path must be repository-relative and stay within the repository")
            continue
        artifact = (root / rel).resolve()
        if not artifact.is_relative_to(root):
            errors.append(f"{label}: path resolves outside the repository")
            continue
        try:
            data = artifact.read_bytes()
        except OSError as exc:
            errors.append(f"{label}: cannot read artifact: {exc}")
            continue
        actual = hashlib.sha256(data).hexdigest()
        if entry.get("sha256") != actual:
            errors.append(f"{label}: SHA-256 mismatch (expected {entry.get('sha256')}, got {actual})")
        if entry.get("provenance_class") != "synthetic":
            errors.append(f"{label}: provenance_class must be synthetic")
        if not entry.get("provenance_note"):
            errors.append(f"{label}: provenance_note is required")
        if not entry.get("schema") or not isinstance(entry.get("schema_version"), int):
            errors.append(f"{label}: schema and integer schema_version are required")
        if entry.get("redistribution") != "not_established_for_external_redistribution":
            errors.append(f"{label}: conservative redistribution disposition is required")
    return errors


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(f"usage: {Path(argv[0]).name} MANIFEST.json", file=sys.stderr)
        return 2
    errors = validate_manifest(Path(argv[1]))
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("P4 profile manifest valid: 5 artifacts, exact-byte SHA-256 verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
