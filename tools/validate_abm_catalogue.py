#!/usr/bin/env python3
"""Validate the proposed P2.3 staff, spatial and optional-complexity catalogues."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator
from jsonschema.exceptions import SchemaError


FAMILIES = {
    "abm_staff_behavior": "staff_behavior.json",
    "spatial_abm": "spatial.json",
    "optional_complexity": "optional_complexity.json",
}
RANGE_FIELDS = (
    "physical_limits",
    "observed_sample_range",
    "generic_scenario_range",
    "uncertainty_interval",
    "calibration_bounds",
)


def _read(path: Path, label: str) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"cannot read {label} JSON at {path}: {exc}") from None


def _mapping(entries: Any, field: str, label: str) -> dict[str, dict[str, Any]]:
    if not isinstance(entries, list):
        raise ValueError(f"{label} must be an entries array")
    result: dict[str, dict[str, Any]] = {}
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict) or not isinstance(entry.get(field), str):
            raise ValueError(f"{label} entry {index} needs string {field}")
        key = entry[field]
        if key in result:
            raise ValueError(f"{label} contains duplicate {field}: {key}")
        result[key] = entry
    return result


def validate_family(
    root: Path,
    family: str,
    catalogue: Any,
    registry: dict[str, dict[str, Any]],
    matrix: dict[str, dict[str, Any]],
    validator: Draft202012Validator,
) -> list[str]:
    errors: list[str] = []
    expected = {key for key, item in registry.items() if item.get("family") == family}
    if not isinstance(catalogue, dict) or catalogue.get("schema_version") != 1 or catalogue.get("status") != "proposed":
        return [f"{family}: catalogue header must be proposed schema_version 1"]
    if catalogue.get("family") != family:
        errors.append(f"{family}: catalogue family label mismatch")
    records = catalogue.get("records")
    annotations = catalogue.get("annotations")
    if not isinstance(records, list) or not isinstance(annotations, dict):
        return errors + [f"{family}: records array and annotations object are required"]
    ids = [record.get("parameter_id") for record in records if isinstance(record, dict)]
    if len(ids) != len(records):
        errors.append(f"{family}: every record must be an object")
    if len(ids) != len(set(ids)):
        errors.append(f"{family}: duplicate parameter IDs")
    if set(ids) != expected:
        errors.append(f"{family}: IDs do not exactly match registry family (missing={sorted(expected-set(ids))}, extra={sorted(set(ids)-expected)})")
    if set(annotations) != expected:
        errors.append(f"{family}: annotation keys do not exactly match registry family")

    for record in records:
        if not isinstance(record, dict):
            continue
        parameter_id = record.get("parameter_id")
        if parameter_id not in expected:
            continue
        prefix = f"{family}/{parameter_id}"
        for issue in validator.iter_errors(record):
            location = ".".join(map(str, issue.absolute_path)) or "<root>"
            errors.append(f"{prefix}: schema violation at {location} ({issue.validator})")
        registered = registry[parameter_id]
        usage = matrix.get(parameter_id)
        annotation = annotations.get(parameter_id)
        if usage is None or not isinstance(annotation, dict):
            errors.append(f"{prefix}: usage-matrix row or annotation missing")
            continue
        if record.get("semantic_name") != registered.get("semantic_name"):
            errors.append(f"{prefix}: semantic_name differs from registry")
        if record.get("owner") != registered.get("owner"):
            errors.append(f"{prefix}: owner differs from registry")
        if record.get("consumer") != usage.get("consumer"):
            errors.append(f"{prefix}: consumer differs from usage matrix")
        if record.get("unit") != usage.get("unit_contract"):
            errors.append(f"{prefix}: unit differs from accepted unit contract")
        required_annotation = {"unit_contract", "value_role", "evidence_refs", "acquisition_plan", "candidate_families", "candidate_status", "downstream_gates"}
        if not required_annotation <= set(annotation):
            errors.append(f"{prefix}: annotation missing required fields")
            continue
        if annotation.get("unit_contract") != usage.get("unit_contract"):
            errors.append(f"{prefix}: annotation unit contract mismatch")
        if annotation.get("value_role") != usage.get("value_role"):
            errors.append(f"{prefix}: annotation value role mismatch")
        if annotation.get("downstream_gates") != usage.get("open_gate"):
            errors.append(f"{prefix}: downstream gates differ from accepted usage matrix")
        if not isinstance(annotation.get("acquisition_plan"), str) or not annotation["acquisition_plan"].strip():
            errors.append(f"{prefix}: acquisition plan is empty")
        if not isinstance(annotation.get("candidate_status"), str) or not annotation["candidate_status"].strip().lower().startswith("unselected"):
            errors.append(f"{prefix}: candidate status must remain explicitly unselected")
        candidates = annotation.get("candidate_families")
        if not isinstance(candidates, list):
            errors.append(f"{prefix}: candidate_families must be an array")
        else:
            for candidate in candidates:
                status = candidate.get("status") if isinstance(candidate, dict) else candidate
                if isinstance(candidate, dict) and not candidate.get("family"):
                    errors.append(f"{prefix}: candidate family name is empty")
                if not isinstance(status, str) or "unselected" not in status.lower():
                    errors.append(f"{prefix}: a candidate family is presented as selected")

        deferred = usage.get("decision") == "defer" or registered.get("scope_status") == "deferred"
        expected_status = "deferred" if deferred else "unknown"
        if not isinstance(record.get("value_status"), dict) or record["value_status"].get("status") != expected_status:
            errors.append(f"{prefix}: value_status must be {expected_status}")
        for field in RANGE_FIELDS:
            if not isinstance(record.get(field), dict) or record[field].get("status") != expected_status:
                errors.append(f"{prefix}: {field} must be {expected_status}")
        forbidden = {"value", "distribution", "reference_default"} & set(record)
        if forbidden:
            errors.append(f"{prefix}: unsupported value/default/distribution fields present")

        refs = annotation.get("evidence_refs")
        if not isinstance(refs, list):
            errors.append(f"{prefix}: evidence_refs must be an array")
            continue
        for ref in refs:
            if not isinstance(ref, dict) or not isinstance(ref.get("path"), str) or not isinstance(ref.get("sha256"), str):
                errors.append(f"{prefix}: evidence ref needs path and sha256")
                continue
            relative = Path(ref["path"])
            if relative.is_absolute() or ".." in relative.parts:
                errors.append(f"{prefix}: unsafe evidence path")
                continue
            evidence_path = root / relative
            try:
                actual_hash = hashlib.sha256(evidence_path.read_bytes()).hexdigest()
                evidence = _read(evidence_path, "evidence")
            except (OSError, ValueError):
                errors.append(f"{prefix}: evidence file missing or unreadable: {ref['path']}")
                continue
            if actual_hash != ref["sha256"]:
                errors.append(f"{prefix}: stale evidence hash: {ref['path']}")
            if parameter_id not in evidence.get("parameter_ids", []):
                errors.append(f"{prefix}: evidence does not explicitly cover this parameter ID: {ref['path']}")
            if evidence.get("review", {}).get("outcome") != "accepted":
                errors.append(f"{prefix}: evidence record is not accepted: {ref['path']}")
    return errors


def validate_catalogues(root: Path) -> list[str]:
    registry_doc = _read(root / "model-inputs/ed/schema/parameter-ids.json", "registry")
    matrix_doc = _read(root / "model-inputs/ed/schema/parameter-usage-matrix.json", "usage matrix")
    schema = _read(root / "model-inputs/ed/schema/parameter-record.schema.json", "parameter schema")
    Draft202012Validator.check_schema(schema)
    validator = Draft202012Validator(schema)
    registry_entries = _mapping(registry_doc.get("entries"), "parameter_id", "registry")
    matrix_entries = _mapping(matrix_doc.get("entries"), "parameter_id", "usage matrix")
    errors: list[str] = []
    for family, filename in FAMILIES.items():
        try:
            catalogue = _read(root / "model-inputs/ed/abm" / filename, "ABM catalogue")
        except ValueError as exc:
            errors.append(str(exc))
            continue
        errors.extend(validate_family(root, family, catalogue, registry_entries, matrix_entries, validator))
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd(), help="repository root")
    args = parser.parse_args()
    try:
        errors = validate_catalogues(args.root.resolve())
    except (OSError, ValueError, SchemaError, TypeError) as exc:
        print(f"validation error: {exc}", file=sys.stderr)
        return 2
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        print(f"ABM catalogue validation failed with {len(errors)} issue(s)", file=sys.stderr)
        return 1
    total = sum(len(_read(args.root / "model-inputs/ed/abm" / filename, "ABM catalogue")["records"]) for filename in FAMILIES.values())
    print(f"validated {total} proposed ABM parameter records across {len(FAMILIES)} families; no empirical parameter values accepted")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
