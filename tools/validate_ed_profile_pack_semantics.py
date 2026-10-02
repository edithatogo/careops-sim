#!/usr/bin/env python3
"""Validate semantic consistency of a data-only P4 ED profile pack."""

from __future__ import annotations

import argparse
import json
import sys
from decimal import Decimal, InvalidOperation
from pathlib import Path
from typing import Any


def _number(value: Any) -> Decimal | None:
    if isinstance(value, bool) or not isinstance(value, (str, int, float)):
        return None
    try:
        result = Decimal(str(value))
    except (InvalidOperation, ValueError):
        return None
    return result if result.is_finite() else None


def _time(record: Any) -> Decimal | None:
    if not isinstance(record, dict) or record.get("source_unit") != "s":
        return None
    return _number(record.get("value"))


def _interval(record: Any) -> tuple[Decimal, Decimal] | None:
    if not isinstance(record, dict):
        return None
    start, end = _time(record.get("start")), _time(record.get("end"))
    if start is None or end is None:
        return None
    return start, end


def _items(value: Any) -> list[Any]:
    return value if isinstance(value, list) else []


def _object(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def _coverage_errors(rows: list[Any], id_key: str, list_path: str,
                     horizon: Decimal, errors: list[str]) -> dict[str, list[tuple[Decimal, Decimal, int]]]:
    grouped: dict[str, list[tuple[Decimal, Decimal, int]]] = {}
    for index, row in enumerate(rows):
        item = _object(row)
        interval = _interval(item.get("interval"))
        if interval is None:
            errors.append(f"{list_path}[{index}].interval: expected finite second offsets")
            continue
        start, end = interval
        if start < 0 or end <= start or end > horizon:
            errors.append(f"{list_path}[{index}].interval: expected 0 <= start < end <= horizon")
            continue
        identifier = item.get(id_key)
        if not isinstance(identifier, str) or not identifier:
            errors.append(f"{list_path}[{index}].{id_key}: expected non-empty identifier")
            continue
        grouped.setdefault(identifier, []).append((start, end, index))
    for identifier, entries in sorted(grouped.items()):
        entries.sort(key=lambda entry: (entry[0], entry[1], entry[2]))
        expected = Decimal(0)
        for start, end, index in entries:
            if start < expected:
                errors.append(f"{list_path}[{index}].interval.start: overlaps prior {id_key}={identifier}")
            elif start > expected:
                errors.append(f"{list_path}[{index}].interval.start: gap before {id_key}={identifier}")
            expected = max(expected, end)
        if expected != horizon:
            errors.append(f"{list_path}: {id_key}={identifier} does not cover [0, horizon]")
    return grouped


def validate_profile_semantics(profile: Any) -> list[str]:
    """Return stable path-oriented semantic errors for a parsed profile object."""
    errors: list[str] = []
    if not isinstance(profile, dict):
        return ["<root>: profile pack must be a JSON object"]

    horizon = _time(profile.get("horizon"))
    if horizon is None or horizon <= 0:
        errors.append("horizon.value: expected a finite positive value in seconds")
        horizon = Decimal(0)

    provenance = _object(profile.get("provenance"))
    if provenance.get("class") != "synthetic" or not isinstance(provenance.get("notes"), str) or not provenance["notes"].strip():
        errors.append("provenance: expected explicit synthetic class and non-empty notes")

    arrivals: dict[tuple[Decimal, Decimal], dict[str, int]] = {}
    arrival_rows = _items(profile.get("arrival_table"))
    previous_end: Decimal | None = None
    for index, row in enumerate(arrival_rows):
        item = _object(row)
        interval = _interval(item.get("interval"))
        if interval is None:
            errors.append(f"arrival_table[{index}].interval: expected finite second offsets")
            continue
        start, end = interval
        if start < 0 or end <= start or end > horizon:
            errors.append(f"arrival_table[{index}].interval: expected 0 <= start < end <= horizon")
        if previous_end is not None:
            if start < previous_end:
                errors.append(f"arrival_table[{index}].interval.start: intervals are not ordered and disjoint")
            elif start > previous_end:
                errors.append(f"arrival_table[{index}].interval.start: gap in interval coverage")
        elif start != 0:
            errors.append(f"arrival_table[{index}].interval.start: coverage must begin at zero")
        previous_end = end
        counts = _object(item.get("counts_by_mode"))
        parsed: dict[str, int] = {}
        for mode, count in sorted(counts.items()):
            if not isinstance(count, int) or isinstance(count, bool) or count < 0:
                errors.append(f"arrival_table[{index}].counts_by_mode.{mode}: expected nonnegative integer")
            else:
                parsed[mode] = count
        arrivals[(start, end)] = parsed
    if arrival_rows and previous_end != horizon:
        errors.append("arrival_table: intervals must cover through horizon")

    scales: dict[str, set[str]] = {}
    for index, raw in enumerate(_items(profile.get("acuity_scales"))):
        row = _object(raw)
        scale_id, cats = row.get("scale_id"), row.get("categories")
        if row.get("provenance_class") != "synthetic":
            errors.append(f"acuity_scales[{index}].provenance_class: expected synthetic")
        if isinstance(scale_id, str) and isinstance(cats, list):
            scales[scale_id] = {c for c in cats if isinstance(c, str)}

    case_rows: dict[tuple[Decimal, Decimal, str], list[int]] = {}
    for index, raw in enumerate(_items(profile.get("case_mix_table"))):
        row = _object(raw)
        interval = _interval(row.get("interval"))
        if interval is None:
            errors.append(f"case_mix_table[{index}].interval: expected finite second offsets")
            continue
        key = (interval[0], interval[1], str(row.get("arrival_mode", "")))
        scale_id = row.get("acuity_scale_id")
        categories = scales.get(scale_id) if isinstance(scale_id, str) else None
        if categories is None:
            errors.append(f"case_mix_table[{index}].acuity_scale_id: unknown acuity scale")
            categories = set()
        counts = _object(row.get("counts_by_category"))
        for category in sorted(counts):
            if category not in categories:
                errors.append(f"case_mix_table[{index}].counts_by_category.{category}: category outside declared scale")
        parsed_counts: list[int] = []
        for category in sorted(categories):
            value = counts.get(category)
            if not isinstance(value, int) or isinstance(value, bool) or value < 0:
                errors.append(f"case_mix_table[{index}].counts_by_category.{category}: expected nonnegative integer")
            else:
                parsed_counts.append(value)
        case_rows.setdefault(key, []).append(sum(parsed_counts))
    for (start, end), modes in sorted(arrivals.items()):
        for mode, expected in sorted(modes.items()):
            key = (start, end, mode)
            sums = case_rows.get(key, [])
            if len(sums) != 1:
                errors.append(f"case_mix_table: expected exactly one row for interval [{start}, {end}) and arrival_mode={mode}")
            elif sums[0] != expected:
                errors.append(f"case_mix_table[{mode}]: category counts do not sum to arrival count {expected} for [{start}, {end})")
    for key in sorted(case_rows):
        if key[:2] not in arrivals or key[2] not in arrivals.get(key[:2], {}):
            errors.append(f"case_mix_table: row has no matching arrival_table interval/mode {key}")

    for index, raw in enumerate(_items(profile.get("distributions"))):
        row = _object(raw)
        if row.get("provenance_class") != "synthetic":
            errors.append(f"distributions[{index}].provenance_class: expected synthetic")
        outcomes = _items(row.get("outcomes"))
        probabilities: list[Decimal] = []
        support_values: set[Any] = set()
        for j, outcome_raw in enumerate(outcomes):
            outcome = _object(outcome_raw)
            raw_value = outcome.get("value")
            value, probability = _number(raw_value), _number(outcome.get("probability"))
            unit = row.get("unit")
            if unit == "category":
                if not isinstance(raw_value, str) or not raw_value:
                    errors.append(f"distributions[{index}].outcomes[{j}].value: expected non-empty categorical support")
            elif value is None or value < 0:
                errors.append(f"distributions[{index}].outcomes[{j}].value: expected finite nonnegative support")
            try:
                support_key = (type(raw_value).__name__, raw_value)
                if support_key in support_values:
                    errors.append(f"distributions[{index}].outcomes[{j}].value: duplicate support value")
                support_values.add(support_key)
            except TypeError:
                errors.append(f"distributions[{index}].outcomes[{j}].value: support value must be scalar")
            if probability is None or probability < 0:
                errors.append(f"distributions[{index}].outcomes[{j}].probability: expected finite nonnegative probability")
            else:
                probabilities.append(probability)
        if not outcomes or len(probabilities) != len(outcomes) or sum(probabilities, Decimal(0)) != Decimal(1):
            errors.append(f"distributions[{index}].outcomes: probabilities must be finite and sum exactly to 1")

    capacity = _object(profile.get("capacity_location"))
    buckets = _items(capacity.get("resource_buckets"))
    bucket_by_id: dict[str, dict[str, Any]] = {}
    valid_bucket_counts: dict[str, tuple[int, int, int]] = {}
    for i, raw in enumerate(buckets):
        row = _object(raw)
        rid, rclass = row.get("resource_id"), row.get("resource_class")
        if not isinstance(rid, str) or not rid:
            continue
        bucket_by_id[rid] = row
        physical = row.get("physical_count")
        opened, staffed = row.get("open_count"), row.get("staffed_count")
        names = ("physical_count", "open_count", "staffed_count")
        values = (physical, opened, staffed)
        valid = []
        for name, value in zip(names, values):
            if not isinstance(value, int) or isinstance(value, bool) or value < 0:
                errors.append(f"capacity_location.resource_buckets[{i}].{name}: expected nonnegative integer")
                valid.append(False)
            else:
                valid.append(True)
        if all(valid):
            valid_bucket_counts[rid] = (physical, opened, staffed)
            if opened > physical:
                errors.append(f"capacity_location.resource_buckets[{i}].open_count: exceeds physical_count")
            if staffed > opened:
                errors.append(f"capacity_location.resource_buckets[{i}].staffed_count: exceeds open_count")
    cap = _object(capacity.get("capacity"))
    physical, opened, staffed = cap.get("physical_count"), cap.get("open_count"), cap.get("staffed_count")
    valid_capacity = []
    for name, value in (("physical_count", physical), ("open_count", opened), ("staffed_count", staffed)):
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            errors.append(f"capacity_location.capacity.{name}: expected nonnegative integer")
            valid_capacity.append(False)
        else:
            valid_capacity.append(True)
    if all(valid_capacity):
        if opened > physical:
            errors.append("capacity_location.capacity.open_count: exceeds physical_count")
        if staffed > opened:
            errors.append("capacity_location.capacity.staffed_count: exceeds open_count")

    resource_groups = _coverage_errors(_items(profile.get("resource_calendars")), "resource_id", "resource_calendars", horizon, errors)
    for rid in sorted(set(bucket_by_id) - set(resource_groups)):
        errors.append(f"resource_calendars: missing full-horizon coverage for resource_id={rid}")
    for rid in sorted(resource_groups):
        bucket = bucket_by_id.get(rid)
        if bucket is None:
            errors.append(f"resource_calendars: unknown resource_id={rid}")
            continue
        max_physical = valid_bucket_counts.get(rid, (None, None, None))[0]
        for _, _, idx in resource_groups[rid]:
            row = _object(_items(profile.get("resource_calendars"))[idx])
            oc, sc = row.get("open_count"), row.get("staffed_count")
            if not isinstance(oc, int) or isinstance(oc, bool) or oc < 0:
                errors.append(f"resource_calendars[{idx}].open_count: expected nonnegative integer")
            if not isinstance(sc, int) or isinstance(sc, bool) or sc < 0:
                errors.append(f"resource_calendars[{idx}].staffed_count: expected nonnegative integer")
            if isinstance(oc, int) and not isinstance(oc, bool) and isinstance(max_physical, int) and oc > max_physical:
                errors.append(f"resource_calendars[{idx}].open_count: exceeds resource physical_count")
            if isinstance(sc, int) and not isinstance(sc, bool) and isinstance(oc, int) and sc > oc:
                errors.append(f"resource_calendars[{idx}].staffed_count: exceeds open_count")

    staff_rows = _items(profile.get("staffing_calendars"))
    staff_groups = _coverage_errors(staff_rows, "staff_role_id", "staffing_calendars", horizon, errors)
    declared_roles = {
        r.get("staff_role_id") for r in _items(capacity.get("staff_roles"))
        if isinstance(r, dict) and isinstance(r.get("staff_role_id"), str)
    }
    for role in sorted(set(staff_groups) - declared_roles):
        errors.append(f"staffing_calendars: unknown staff_role_id={role}")
    for role in sorted(declared_roles - set(staff_groups)):
        errors.append(f"staffing_calendars: missing full-horizon coverage for staff_role_id={role}")
    for i, row_raw in enumerate(staff_rows):
        row = _object(row_raw)
        count = row.get("present_count")
        if not isinstance(count, int) or isinstance(count, bool) or count < 0:
            errors.append(f"staffing_calendars[{i}].present_count: expected nonnegative integer")

    initial = _object(profile.get("initial_state"))
    at = _time(initial.get("at"))
    if at is None or at < 0 or at > horizon:
        errors.append("initial_state.at.value: expected a second offset within [0, horizon]")
        at = Decimal(0)
    occupants = _items(initial.get("occupants"))
    known_locations = {
        r.get("location_id") for r in _items(capacity.get("locations"))
        if isinstance(r, dict) and isinstance(r.get("location_id"), str)
    }
    occupant_ids: set[str] = set()
    counts_by_location: dict[str, int] = {}
    for i, raw in enumerate(occupants):
        row = _object(raw)
        oid, loc = row.get("occupant_id"), row.get("location_id")
        if not isinstance(oid, str) or not oid or oid in occupant_ids:
            errors.append(f"initial_state.occupants[{i}].occupant_id: expected unique non-empty identifier")
        else:
            occupant_ids.add(oid)
        if not isinstance(loc, str) or loc not in known_locations:
            errors.append(f"initial_state.occupants[{i}].location_id: unknown location")
        else:
            counts_by_location[loc] = counts_by_location.get(loc, 0) + 1
    for loc, count in sorted(counts_by_location.items()):
        matching = [b for b in buckets if _object(b).get("location_id") == loc]
        physical = sum(int(_object(b).get("physical_count", 0)) for b in matching if isinstance(_object(b).get("physical_count"), int))
        if physical and count > physical:
            errors.append(f"initial_state.occupants: {count} occupants at {loc} exceed physical resource capacity {physical}")
        if matching:
            open_at_time = 0
            for calendar_raw in _items(profile.get("resource_calendars")):
                calendar = _object(calendar_raw)
                bucket = bucket_by_id.get(calendar.get("resource_id"))
                interval = _interval(calendar.get("interval"))
                if bucket is not None and bucket.get("location_id") == loc and interval and interval[0] <= at < interval[1]:
                    value = calendar.get("open_count")
                    if isinstance(value, int) and not isinstance(value, bool) and value >= 0:
                        open_at_time += value
            if count > open_at_time:
                errors.append(f"initial_state.occupants: {count} occupants at {loc} exceed open resource capacity {open_at_time} at initial_state.at")
    task_ids = {
        r.get("task_class_id") for r in _items(capacity.get("task_classes"))
        if isinstance(r, dict) and isinstance(r.get("task_class_id"), str)
    }
    work_seen: set[tuple[str, str]] = set()
    for i, raw in enumerate(_items(initial.get("remaining_work"))):
        row = _object(raw)
        oid, task = row.get("occupant_id"), row.get("task_class_id")
        if not isinstance(oid, str) or oid not in occupant_ids:
            errors.append(f"initial_state.remaining_work[{i}].occupant_id: occupant is not present")
        if not isinstance(task, str) or task not in task_ids:
            errors.append(f"initial_state.remaining_work[{i}].task_class_id: unknown task class")
        if isinstance(oid, str) and isinstance(task, str):
            pair = (oid, task)
            if pair in work_seen:
                errors.append(f"initial_state.remaining_work[{i}]: duplicate occupant/task assignment")
            work_seen.add(pair)
        duration = _time(row.get("remaining_duration"))
        if duration is None or duration < 0 or duration > horizon - at:
            errors.append(f"initial_state.remaining_work[{i}].remaining_duration.value: expected nonnegative duration within remaining horizon")

    return sorted(set(errors))


def _load_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as stream:
        return json.load(stream)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("profiles", nargs="+", type=Path, help="profile pack JSON files")
    args = parser.parse_args(argv)
    failed = False
    for path in args.profiles:
        try:
            errors = validate_profile_semantics(_load_json(path))
        except (OSError, UnicodeError, json.JSONDecodeError) as exc:
            print(f"{path}: cannot read JSON: {exc}", file=sys.stderr)
            failed = True
            continue
        if errors:
            failed = True
            for error in errors:
                print(f"{path}: {error}", file=sys.stderr)
        else:
            print(f"semantically valid data-only profile pack: {path}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
