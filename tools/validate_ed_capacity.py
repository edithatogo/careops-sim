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

    def unique_ids(collection: Any, field: str, label: str) -> set[str]:
        found: set[str] = set()
        if isinstance(collection, list):
            for index, item in enumerate(collection):
                if not isinstance(item, dict):
                    continue
                value = item.get(field)
                if isinstance(value, str):
                    if value in found:
                        errors.append(f"{label}[{index}].{field} duplicates {value!r}")
                    found.add(value)
        return found

    zones = record.get("zones")
    locations = record.get("locations")
    resources = record.get("resource_buckets")
    tasks = record.get("task_classes")
    roles = record.get("staff_roles")
    zone_ids = unique_ids(zones, "zone_id", "zones")
    location_ids = unique_ids(locations, "location_id", "locations")
    resource_ids = unique_ids(resources, "resource_id", "resource_buckets")
    task_ids = unique_ids(tasks, "task_class_id", "task_classes")
    role_ids = unique_ids(roles, "staff_role_id", "staff_roles")

    location_by_id = ({item.get("location_id"): item for item in locations if isinstance(item, dict)}
                      if isinstance(locations, list) else {})
    if isinstance(locations, list):
        for index, location in enumerate(locations):
            if not isinstance(location, dict):
                continue
            zone_id = location.get("zone_id")
            if isinstance(zone_id, str) and zone_id not in zone_ids:
                errors.append(f"locations[{index}].zone_id references unknown zone {zone_id!r}")

    # Resource IDs identify allocatable buckets. pool_group_id is only a label
    # for related buckets and is intentionally never counted as another bucket.
    if isinstance(resources, list):
        for index, resource in enumerate(resources):
            if not isinstance(resource, dict):
                continue
            location_id = resource.get("location_id")
            zone_id = resource.get("zone_id")
            location = location_by_id.get(location_id)
            if isinstance(location_id, str) and location_id not in location_ids:
                errors.append(f"resource_buckets[{index}].location_id references unknown location {location_id!r}")
            if isinstance(zone_id, str) and zone_id not in zone_ids:
                errors.append(f"resource_buckets[{index}].zone_id references unknown zone {zone_id!r}")
            elif location and zone_id != location.get("zone_id"):
                errors.append(f"resource_buckets[{index}].zone_id does not match its location zone")
            physical = resource.get("physical_count")
            opened = resource.get("open_count")
            staffed = resource.get("staffed_count")
            if (resource.get("availability_status") == "known"
                    and isinstance(staffed, int) and not isinstance(staffed, bool)
                    and isinstance(opened, int) and not isinstance(opened, bool) and staffed > opened):
                errors.append(f"resource_buckets[{index}].staffed_count must not exceed open_count")
            if (resource.get("physical_status") == "known"
                    and isinstance(opened, int) and not isinstance(opened, bool)
                    and isinstance(physical, int) and not isinstance(physical, bool) and opened > physical):
                errors.append(f"resource_buckets[{index}].open_count must not exceed physical_count")

    # The department object is a report constraint, not a pool. A declared
    # complete set with known aggregate values must be fully enumerated/known.
    capacity = record.get("capacity")
    treatment = [r for r in resources if isinstance(r, dict) and r.get("resource_class") == "treatment_space"] if isinstance(resources, list) else []
    if isinstance(capacity, dict) and capacity.get("bucket_completeness") == "complete":
        for field, aggregate_status in (("physical_count", "physical_status"),
                                        ("open_count", "availability_status"),
                                        ("staffed_count", "availability_status")):
            aggregate = capacity.get(field)
            known_aggregate = (capacity.get(aggregate_status) == "known"
                               and isinstance(aggregate, int) and not isinstance(aggregate, bool))
            if not known_aggregate:
                continue
            if not treatment:
                errors.append(f"capacity.{field} is known and complete but treatment_space buckets are absent")
                continue
            values = []
            for index, bucket in enumerate(treatment):
                status = bucket.get(aggregate_status)
                value = bucket.get(field)
                if status != "known" or not isinstance(value, int) or isinstance(value, bool):
                    errors.append(f"treatment_space bucket {bucket.get('resource_id', index)!r} has unknown {field} in complete set")
                else:
                    values.append(value)
            if len(values) == len(treatment) and sum(values) != aggregate:
                errors.append(f"capacity.{field} must equal the complete treatment_space bucket sum")

    task_by_id = {item.get("task_class_id"): item for item in tasks if isinstance(item, dict)} if isinstance(tasks, list) else {}
    role_by_id = {item.get("staff_role_id"): item for item in roles if isinstance(item, dict)} if isinstance(roles, list) else {}
    class_names = {r.get("resource_class") for r in resources if isinstance(r, dict)} if isinstance(resources, list) else set()
    eligibility = record.get("eligibility")
    if isinstance(eligibility, list):
        for index, row in enumerate(eligibility):
            if not isinstance(row, dict):
                continue
            role_id, task_id, zone_id = row.get("staff_role_id"), row.get("task_class_id"), row.get("zone_id")
            if isinstance(role_id, str) and role_id not in role_ids:
                errors.append(f"eligibility[{index}].staff_role_id references unknown role {role_id!r}")
            if isinstance(task_id, str) and task_id not in task_ids:
                errors.append(f"eligibility[{index}].task_class_id references unknown task {task_id!r}")
            if isinstance(zone_id, str) and zone_id not in zone_ids:
                errors.append(f"eligibility[{index}].zone_id references unknown zone {zone_id!r}")
            location_id = row.get("location_id")
            if location_id is not None:
                if isinstance(location_id, str) and location_id not in location_ids:
                    errors.append(f"eligibility[{index}].location_id references unknown location {location_id!r}")
                elif isinstance(location_id, str) and location_by_id.get(location_id, {}).get("zone_id") != zone_id:
                    errors.append(f"eligibility[{index}].location_id is outside its zone")

    for task_id, task in task_by_id.items():
        required = task.get("required_resource_classes", [])
        task_rows = [row for row in eligibility if isinstance(row, dict)
                     and row.get("task_class_id") == task_id] if isinstance(eligibility, list) else []
        if required and not task_rows:
            errors.append(f"task {task_id!r} requires resources but has no eligibility scope")
        for resource_class in required:
            if resource_class not in class_names:
                errors.append(f"task_classes.{task_id} requires unknown resource class {resource_class!r}")
            for row in task_rows:
                zone_id = row.get("zone_id")
                location_id = row.get("location_id")
                matches = [r for r in resources if isinstance(r, dict) and r.get("resource_class") == resource_class
                           and (r.get("location_id") == location_id if location_id is not None
                                else r.get("zone_id") == zone_id)] if isinstance(resources, list) else []
                scope = f"location {location_id!r}" if location_id is not None else f"zone {zone_id!r}"
                if not matches or not any(r.get("availability_status") == "known" and isinstance(r.get("staffed_count"), int)
                                          and not isinstance(r.get("staffed_count"), bool) and r.get("staffed_count") > 0
                                          for r in matches):
                    errors.append(f"task {task_id!r} has no known staffed {resource_class!r} capacity at {scope}")
        for row in task_rows:
            scope = (f"location {row.get('location_id')!r}" if row.get("location_id") is not None
                     else f"zone {row.get('zone_id')!r}")
            if not _has_known_effective_staff(role_by_id.get(row.get("staff_role_id"))):
                errors.append(f"task {task_id!r} has no known present or task_eligible staff at {scope}")

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


def _has_known_effective_staff(role: Any) -> bool:
    """Whether a role declares known effective present/task-eligible capacity."""
    if not isinstance(role, dict):
        return False
    return any(isinstance(count, dict) and count.get("basis") in {"present", "task_eligible"}
               and count.get("status") == "known" and isinstance(count.get("count"), int)
               and not isinstance(count.get("count"), bool) and count["count"] > 0
               for count in role.get("counts", []))


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
