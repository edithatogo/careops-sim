#!/usr/bin/env python3
"""Validate proposed ED parameter records and synthetic example collections."""

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


def _registry_ids(value: Any) -> set[str]:
    if not isinstance(value, dict) or not isinstance(value.get("entries"), list):
        raise ValueError("registry must be an object with an entries array")
    ids: list[str] = []
    for index, entry in enumerate(value["entries"]):
        if not isinstance(entry, dict) or not isinstance(entry.get("parameter_id"), str):
            raise ValueError(f"registry entry {index} must contain a string parameter_id")
        ids.append(entry["parameter_id"])
    if len(ids) != len(set(ids)):
        raise ValueError("registry contains duplicate parameter_id entries")
    return set(ids)


def _validate_record(record: Any, validator: Draft202012Validator, ids: set[str]) -> list[str]:
    if not isinstance(record, dict):
        return ["record must be a JSON object"]
    errors = [
        f"schema violation at {'.'.join(map(str, error.absolute_path)) or '<root>'} "
        f"({error.validator} constraint)"
        for error in validator.iter_errors(record)
    ]
    parameter_id = record.get("parameter_id")
    if isinstance(parameter_id, str) and parameter_id not in ids:
        errors.append("parameter_id is not registered")
    return errors


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--schema", type=Path, required=True, help="Draft 2020-12 record schema")
    parser.add_argument("--registry", type=Path, required=True, help="parameter ID registry JSON")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--record", type=Path, help="one parameter record JSON")
    mode.add_argument("--examples", type=Path, help="expected_valid example collection JSON")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        schema = _read_json(args.schema, "schema")
        Draft202012Validator.check_schema(schema)
        validator = Draft202012Validator(schema)
        ids = _registry_ids(_read_json(args.registry, "registry"))
        if args.record:
            errors = _validate_record(_read_json(args.record, "record"), validator, ids)
            for error in errors:
                print(f"{args.record}: {error}", file=sys.stderr)
            return 1 if errors else 0

        examples = _read_json(args.examples, "examples")
        cases = examples.get("cases") if isinstance(examples, dict) else None
        if not isinstance(cases, list) or not cases:
            raise ValueError("examples must be an object with a non-empty cases array")
        failures = 0
        for index, case in enumerate(cases):
            if (not isinstance(case, dict) or not isinstance(case.get("id"), str)
                    or not isinstance(case.get("expected_valid"), bool) or "record" not in case):
                print(f"examples case {index}: requires id, expected_valid and record", file=sys.stderr)
                failures += 1
                continue
            errors = _validate_record(case["record"], validator, ids)
            passed = not errors if case["expected_valid"] else bool(errors)
            if not passed:
                print(f"examples case {case['id']}: expectation mismatch", file=sys.stderr)
                failures += 1
        if failures:
            print(f"{failures} example case(s) failed", file=sys.stderr)
            return 1
        print(f"validated {len(cases)} example cases")
        return 0
    except (ValueError, SchemaError) as exc:
        # Do not print record contents; diagnostics identify structural locations only.
        print(f"validation error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
