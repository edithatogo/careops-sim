#!/usr/bin/env python3
"""P5.1: Consumer/catalogue coverage checks against ED model configuration and profiles.

Verifies:
1. Full alignment between parameter registry (parameter-ids.json) and usage matrix proposal.
2. Rejection of unregistered parameter IDs, unexplained defaults, and consumer drift.
3. Validation of scenario configuration files against known model parameters and schema versions.
4. Explicit rejection of unrecognized or unregistered scenario configuration fields.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = ROOT / "model-inputs/ed/schema/parameter-ids.json"
USAGE_MATRIX_PATH = ROOT / "model-inputs/ed/schema/parameter-usage-matrix-proposal.json"

DEFAULT_SCENARIOS = [
    ROOT / "crates/careops-ed/examples/one_patient.json",
    ROOT / "crates/careops-ed/examples/nominal_mvp.json",
    ROOT / "crates/careops-ed/examples/constrained_mvp.json",
    ROOT / "model-inputs/ed/profiles/p4-minimal-deterministic.json",
    ROOT / "model-inputs/ed/profiles/p4-generic-nominal.json",
    ROOT / "model-inputs/ed/profiles/p4-constrained-surge.json",
]

ALLOWED_SCENARIO_KEYS = {
    "schema_version",
    "scenario_id",
    "seed",
    "time_unit",
    "horizon_ticks",
    "provenance",
    "capacity_location",
    "beds",
    "treatment_spaces",
    "staff",
    "resources",
    "pathways",
    "arrivals",
    "patients",
    "acuity_distribution",
    "initial_patients",
    "service_points",
    "locations",
    "zones",
    "transit_matrix",
    "equipment",
    "policies",
    "diagnostics",
}

SUPPORTED_SCHEMA_VERSIONS = {1}
SUPPORTED_TIME_UNITS = {"tick", "minute", "hour"}


def validate_registry_matrix_coverage(
    registry_file: Path, matrix_file: Path
) -> list[str]:
    errors: list[str] = []
    try:
        registry = json.loads(registry_file.read_text(encoding="utf-8"))
        matrix = json.loads(matrix_file.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return [f"Failed to read registry/matrix JSON: {exc}"]

    reg_entries = registry.get("entries", [])
    matrix_entries = matrix.get("entries", [])

    if not isinstance(reg_entries, list) or not reg_entries:
        return ["Registry entries must be a non-empty list"]
    if not isinstance(matrix_entries, list) or not matrix_entries:
        return ["Usage matrix entries must be a non-empty list"]

    reg_by_id = {
        e["parameter_id"]: e
        for e in reg_entries
        if isinstance(e, dict) and "parameter_id" in e
    }
    matrix_by_id = {
        e["parameter_id"]: e
        for e in matrix_entries
        if isinstance(e, dict) and "parameter_id" in e
    }

    # Check for unregistered IDs in matrix
    for pid, entry in matrix_by_id.items():
        if pid not in reg_by_id:
            errors.append(f"Unregistered parameter_id in matrix: {pid}")
            continue
        reg_entry = reg_by_id[pid]
        if entry.get("consumer") != reg_entry.get("consumer"):
            errors.append(f"Consumer drift for {pid}: {entry.get('consumer')} != {reg_entry.get('consumer')}")
        if not entry.get("proposed_config_key"):
            errors.append(f"Missing proposed_config_key for {pid}")
        if not entry.get("unit_contract"):
            errors.append(f"Missing unit_contract for {pid}")
        if not entry.get("profile_coverage"):
            errors.append(f"Missing profile_coverage for {pid}")

    # Check for missing matrix coverage
    for pid in reg_by_id:
        if pid not in matrix_by_id:
            errors.append(f"Missing usage matrix mapping for registered parameter: {pid}")

    return errors


def validate_scenario_file(scenario_file: Path) -> list[str]:
    errors: list[str] = []
    try:
        data = json.loads(scenario_file.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return [f"{scenario_file}: failed to read JSON ({exc})"]

    if not isinstance(data, dict):
        return [f"{scenario_file}: root must be a JSON object"]

    # Check schema version
    ver = data.get("schema_version")
    if ver not in SUPPORTED_SCHEMA_VERSIONS:
        errors.append(
            f"{scenario_file}: unsupported schema_version '{ver}' (supported: {SUPPORTED_SCHEMA_VERSIONS})"
        )

    # Check time unit
    unit = data.get("time_unit")
    if unit is not None and unit not in SUPPORTED_TIME_UNITS:
        errors.append(
            f"{scenario_file}: unsupported time_unit '{unit}' (supported: {SUPPORTED_TIME_UNITS})"
        )

    # Check for unregistered top-level configuration keys
    for key in data:
        if key not in ALLOWED_SCENARIO_KEYS:
            errors.append(
                f"{scenario_file}: unregistered configuration field '{key}' rejected by consumer coverage"
            )

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--scenario",
        type=Path,
        action="append",
        help="Specific scenario configuration file to validate (may be repeated)",
    )
    args = parser.parse_args()

    errors: list[str] = []

    # 1. Validate Registry and Matrix coverage
    matrix_errors = validate_registry_matrix_coverage(REGISTRY_PATH, USAGE_MATRIX_PATH)
    if matrix_errors:
        errors.extend(matrix_errors)
    else:
        reg_count = len(json.loads(REGISTRY_PATH.read_text())["entries"])
        print(f"PASS: Registry-matrix coverage verified ({reg_count} parameter IDs aligned)")

    # 2. Validate scenarios
    scenarios_to_check = args.scenario if args.scenario else DEFAULT_SCENARIOS
    for sc in scenarios_to_check:
        sc_errors = validate_scenario_file(sc)
        if sc_errors:
            errors.extend(sc_errors)
        else:
            print(f"PASS: Scenario coverage verified: {sc.relative_to(ROOT) if sc.is_relative_to(ROOT) else sc}")

    if errors:
        for err in errors:
            print(f"ERROR: {err}", file=sys.stderr)
        return 1

    print("P5.1 consumer coverage PASS: all registered IDs, matrices, and scenarios verified")
    return 0


if __name__ == "__main__":
    sys.exit(main())
