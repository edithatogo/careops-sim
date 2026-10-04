#!/usr/bin/env python3
"""Validate documented flaky-test exceptions without changing test execution.

An empty ``flakes`` inventory means that no quarantine exceptions are
documented. It is not evidence that the suite contains no flaky tests. This
tool does not quarantine tests, retry failures, or alter nextest policy.
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import date, datetime
from pathlib import Path
from typing import Any


ENTRY_FIELDS = frozenset(
    {"test_id", "owner", "reason", "issue_url", "expires_on"}
)


def _parse_iso_date(value: Any) -> date | None:
    if not isinstance(value, str) or len(value) != 10:
        return None
    try:
        parsed = date.fromisoformat(value)
    except ValueError:
        return None
    if parsed.isoformat() != value:
        return None
    return parsed


def check(config: Any, today: str | date | None = None) -> list[str]:
    """Return structural or expiry errors for the configured flake inventory.

    Unrelated root config fields are intentionally opaque: this leaf validates
    only ``schema_version`` and ``flakes`` so it does not become authority for
    other D3.3 quality policy.
    """
    errors: list[str] = []
    if not isinstance(config, dict):
        return ["config must be a JSON object"]

    version = config.get("schema_version")
    if isinstance(version, bool) or not isinstance(version, int) or version != 1:
        errors.append("schema_version must be integer 1")

    flakes = config.get("flakes")
    if not isinstance(flakes, list):
        errors.append("flakes must be a list")
        return errors

    if today is None:
        comparison_date = date.today()
    elif isinstance(today, datetime):
        errors.append("today must be a date, not a datetime")
        return errors
    elif isinstance(today, date):
        comparison_date = today
    else:
        comparison_date = _parse_iso_date(today)
        if comparison_date is None:
            errors.append("today must be a valid YYYY-MM-DD date")
            return errors

    seen_ids: set[str] = set()
    for index, entry in enumerate(flakes):
        label = f"flakes[{index}]"
        if not isinstance(entry, dict):
            errors.append(f"{label} must be an object")
            continue

        actual_fields = set(entry)
        if actual_fields != ENTRY_FIELDS:
            missing = sorted(ENTRY_FIELDS - actual_fields)
            extra = sorted(actual_fields - ENTRY_FIELDS)
            details = []
            if missing:
                details.append("missing " + ", ".join(missing))
            if extra:
                details.append("unknown " + ", ".join(str(field) for field in extra))
            errors.append(f"{label} has invalid fields ({'; '.join(details)})")

        for field in ENTRY_FIELDS:
            value = entry.get(field)
            if not isinstance(value, str) or not value.strip():
                errors.append(f"{label}.{field} must be a nonempty string")

        test_id = entry.get("test_id")
        if isinstance(test_id, str) and test_id.strip():
            if test_id in seen_ids:
                errors.append(f"{label}.test_id duplicates {test_id!r}")
            seen_ids.add(test_id)

        expires_on = entry.get("expires_on")
        expiry = _parse_iso_date(expires_on)
        if expiry is None:
            if isinstance(expires_on, str) and expires_on.strip():
                errors.append(f"{label}.expires_on must be a valid YYYY-MM-DD date")
        elif comparison_date > expiry:
            errors.append(f"{label} is expired on {expiry.isoformat()}")

    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", required=True, type=Path)
    parser.add_argument("--today", help="fixed comparison date in YYYY-MM-DD form")
    args = parser.parse_args(argv)

    try:
        config = json.loads(args.config.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        print(f"quality_flakes: cannot read config: {error}", file=sys.stderr)
        return 2

    errors = check(config, args.today)
    if errors:
        for error in errors:
            print(f"quality_flakes: {error}", file=sys.stderr)
        return 1

    count = len(config["flakes"])
    print(
        f"quality_flakes=ok documented_exceptions={count}; "
        "empty inventory is not a no-flakiness claim"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
