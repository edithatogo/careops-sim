#!/usr/bin/env python3
"""Validate proposed ED capacity/location examples against schema and semantics."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator
from jsonschema.exceptions import SchemaError


def _read_json(path: Path, label: str) -> Any:
    try:
        with path.open(encoding="utf-8") as stream:
            return json.load(stream)
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"cannot read {label} JSON at {path}: {exc}") from None


def _schema_errors(record: Any, validator: Draft202012Validator) -> list[str]:
    if not isinstance(record, dict):
        return ["record must be a JSON object"]
    return [
        f"schema violation at {'.'.join(map(str, error.absolute_path)) or '<root>'} "
        f"({error.validator} constraint)"
        for error in validator.iter_errors(record)
    ]


def _semantic_errors(record: Any) -> list[str]:
    """Check relationships and references that JSON Schema cannot express here."""
    if not isinstance(record, dict):
        return []

    errors: list[str] = []
    capacity = record.get("capacity")
    if isinstance(capacity, dict):
        physical = capacity.get("physical_count")
        opened = capacity.get("open_count")
        staffed = capacity.get("staffed_count")
        if (capacity.get("availability_status") == "known"
                and isinstance(staffed, int) and not isinstance(staffed, bool)
                and isinstance(opened, int) and not isinstance(opened, bool)
                and staffed > opened):
            errors.append("capacity.staffed_count must not exceed capacity.open_count")
        if (capacity.get("physical_status") == "known"
                and isinstance(opened, int) and not isinstance(opened, bool)
                and isinstance(physical, int) and not isinstance(physical, bool)
                and opened > physical):
            errors.append("capacity.open_count must not exceed capacity.physical_count")

    locations = record.get("locations")
    location_ids: set[str] = set()
    if isinstance(locations, list):
        for index, location in enumerate(locations):
            if not isinstance(location, dict):
                continue
            location_id = location.get("location_id")
            if isinstance(location_id, str):
                if location_id in location_ids:
                    errors.append(f"locations[{index}].location_id duplicates {location_id!r}")
                location_ids.add(location_id)

    zones = record.get("zones")
    zone_ids: set[str] = set()
    if isinstance(zones, list):
        for index, zone in enumerate(zones):
            if not isinstance(zone, dict):
                continue
            zone_id = zone.get("zone_id")
            if isinstance(zone_id, str):
                if zone_id in zone_ids:
                    errors.append(f"zones[{index}].zone_id duplicates {zone_id!r}")
                zone_ids.add(zone_id)

    if isinstance(locations, list):
        for index, location in enumerate(locations):
            if not isinstance(location, dict):
                continue
            zone_id = location.get("zone_id")
            if isinstance(zone_id, str) and zone_id not in zone_ids:
                errors.append(f"locations[{index}].zone_id references unknown zone {zone_id!r}")

    routes = record.get("routes")
    if isinstance(routes, list):
        for index, route in enumerate(routes):
            if not isinstance(route, dict):
                continue
            for field in ("from_location_id", "to_location_id"):
                endpoint = route.get(field)
                if isinstance(endpoint, str) and endpoint not in location_ids:
                    errors.append(f"routes[{index}].{field} references unknown location {endpoint!r}")

    return errors


def validate_record(record: Any, validator: Draft202012Validator) -> list[str]:
    return _schema_errors(record, validator) + _semantic_errors(record)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--schema", type=Path, required=True, help="Draft 2020-12 capacity/location schema")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--record", type=Path, help="one capacity/location record JSON")
    mode.add_argument("--examples", type=Path, help="capacity/location example collection JSON")
    return parser


def _validate_examples(examples: Any, validator: Draft202012Validator) -> tuple[list[str], int]:
    if (not isinstance(examples, dict) or examples.get("schema_version") != 1
            or examples.get("status") != "proposed"):
        raise ValueError("examples must be a proposed schema_version 1 collection")
    cases = examples.get("cases")
    if not isinstance(cases, list) or not cases:
        raise ValueError("examples must contain a non-empty cases array")

    failures: list[str] = []
    for index, case in enumerate(cases):
        if (not isinstance(case, dict) or not isinstance(case.get("case_id"), str)
                or not isinstance(case.get("synthetic_only"), bool) or "record" not in case):
            failures.append(f"examples case {index}: requires case_id, synthetic_only and record")
            continue
        errors = validate_record(case["record"], validator)
        failures.extend(f"examples case {case['case_id']}: {error}" for error in errors)
    return failures, len(cases)


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        schema = _read_json(args.schema, "schema")
        Draft202012Validator.check_schema(schema)
        validator = Draft202012Validator(schema)
        if args.record:
            errors = validate_record(_read_json(args.record, "record"), validator)
            for error in errors:
                print(f"{args.record}: {error}", file=sys.stderr)
            return 1 if errors else 0

        failures, count = _validate_examples(_read_json(args.examples, "examples"), validator)
        for failure in failures:
            print(f"{args.examples}: {failure}", file=sys.stderr)
        if failures:
            print(f"{len(failures)} capacity/location validation error(s)", file=sys.stderr)
            return 1
        print(f"validated {count} capacity/location example cases")
        return 0
    except (ValueError, SchemaError) as exc:
        print(f"validation error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
