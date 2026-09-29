#!/usr/bin/env python3
"""Check proposed consumer/unit/profile coverage against registered ED IDs."""

import argparse
import json
import sys
from pathlib import Path


def validate(registry_path: Path, matrix_path: Path) -> list[str]:
    try:
        registry = json.loads(registry_path.read_text(encoding="utf-8"))
        matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return [f"cannot read JSON input: {exc.__class__.__name__}"]

    registered = registry.get("entries")
    rows = matrix.get("entries")
    if not isinstance(registered, list) or not isinstance(rows, list):
        return ["registry and matrix must each contain an entries array"]

    registry_by_id = {
        entry.get("parameter_id"): entry
        for entry in registered
        if isinstance(entry, dict) and isinstance(entry.get("parameter_id"), str)
    }
    errors: list[str] = []
    seen: set[str] = set()
    for index, row in enumerate(rows):
        label = f"entries[{index}]"
        if not isinstance(row, dict):
            errors.append(f"{label} must be an object")
            continue
        parameter_id = row.get("parameter_id")
        if not isinstance(parameter_id, str) or parameter_id not in registry_by_id:
            errors.append(f"{label} has an unregistered parameter_id")
            continue
        if parameter_id in seen:
            errors.append(f"duplicate matrix row for {parameter_id}")
        seen.add(parameter_id)
        source = registry_by_id[parameter_id]
        if row.get("consumer") != source.get("consumer"):
            errors.append(f"consumer drift for {parameter_id}")
        if not isinstance(row.get("proposed_config_key"), str) or not row["proposed_config_key"]:
            errors.append(f"missing proposed_config_key for {parameter_id}")
        unit = row.get("unit_contract")
        if not isinstance(unit, dict) or not all(
            isinstance(unit.get(key), str) and unit[key]
            for key in ("class", "proposal")
        ):
            errors.append(f"missing unit contract for {parameter_id}")
        profile = row.get("profile_coverage")
        if not isinstance(profile, dict) or not all(
            isinstance(profile.get(key), str) and profile[key]
            for key in ("stage", "profile_use")
        ):
            errors.append(f"missing profile coverage for {parameter_id}")
        if row.get("mapping_status") != "proposed_requires_E0_C0_owner_review":
            errors.append(f"unreviewed mapping status missing for {parameter_id}")
        if row.get("evidence_status") != "identity_only_no_value_or_distribution_asserted":
            errors.append(f"evidence boundary missing for {parameter_id}")

    missing = sorted(set(registry_by_id) - seen)
    errors.extend(f"missing matrix row for {parameter_id}" for parameter_id in missing)
    if matrix.get("row_count") != len(rows):
        errors.append("row_count does not match entries length")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registry", type=Path, required=True)
    parser.add_argument("--matrix", type=Path, required=True)
    args = parser.parse_args()
    errors = validate(args.registry, args.matrix)
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    matrix = json.loads(args.matrix.read_text(encoding="utf-8"))
    print(f"PASS: {matrix['row_count']} registered IDs have proposed consumer, unit and profile mappings")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
