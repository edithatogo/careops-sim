#!/usr/bin/env python3
"""Validate the structure of a data-only P4 synthetic ED profile pack."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator
from jsonschema.exceptions import SchemaError
from referencing import Registry, Resource

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "model-inputs/ed/schema/example-profile-pack.schema.json"
CAPACITY_SCHEMA_PATH = ROOT / "model-inputs/ed/schema/capacity-location.schema.json"


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON object key: {key!r}")
        result[key] = value
    return result


def _read_json(path: Path, label: str) -> Any:
    try:
        with path.open(encoding="utf-8") as stream:
            return json.load(stream, object_pairs_hook=_unique_object)
    except (OSError, UnicodeError, json.JSONDecodeError, ValueError) as exc:
        raise ValueError(f"cannot read {label} JSON at {path}: {exc}") from None


def load_validator() -> Draft202012Validator:
    schema = _read_json(SCHEMA_PATH, "profile-pack schema")
    capacity_schema = _read_json(CAPACITY_SCHEMA_PATH, "capacity/location schema")
    Draft202012Validator.check_schema(schema)
    Draft202012Validator.check_schema(capacity_schema)
    registry = Registry().with_resource(
        capacity_schema["$id"], Resource.from_contents(capacity_schema)
    )
    return Draft202012Validator(schema, registry=registry)


def schema_errors(record: Any, validator: Draft202012Validator) -> list[str]:
    if not isinstance(record, dict):
        return ["profile pack must be a JSON object"]
    return [
        f"schema violation at {'.'.join(map(str, error.absolute_path)) or '<root>'} "
        f"({error.validator} constraint)"
        for error in validator.iter_errors(record)
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("profile", nargs="?", type=Path, help="profile pack JSON to validate")
    parser.add_argument(
        "--schema-only", action="store_true",
        help="validate the profile-pack and referenced capacity/location schemas",
    )
    args = parser.parse_args()
    if args.schema_only == bool(args.profile):
        parser.error("provide exactly one of PROFILE or --schema-only")
    try:
        validator = load_validator()
        if args.schema_only:
            print("profile-pack and capacity/location schemas are valid Draft 2020-12 schemas")
            return 0
        record = _read_json(args.profile, "profile pack")
        errors = schema_errors(record, validator)
        if errors:
            for error in errors:
                print(error, file=sys.stderr)
            return 1
        print(f"valid data-only profile pack: {args.profile}")
        return 0
    except (ValueError, SchemaError, KeyError) as exc:
        print(str(exc), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
