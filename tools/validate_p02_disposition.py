#!/usr/bin/env python3
"""Validate the provisional P0.2 E0/C0 disposition response CSV."""

import argparse
import csv
import re
import sys
from pathlib import Path


EXPECTED_COUNTS = {
    "proposed_accept": 30,
    "proposed_revise": 60,
    "proposed_defer": 11,
}
ACCEPT_ROWS = {1, 4, 7, 12, 13, 14, 15, 16, 20, 23, 26, 28, 38, 61, 62, 65, 71, 77, 78, 84, 85, 86, 87, 88, 93, 96, 97, 98, 100, 101}
DEFER_ROWS = {8, 11, 69, 72, 73, 74, 79, 80, 81, 82, 83}
REVIEWER_PREFIXES = ("E0_", "C0_")
DISPOSITION_RE = re.compile(r"^(proposed_accept|proposed_revise|proposed_defer):\s*(.+)$", re.I)
PROMOTION_RE = re.compile(
    r"\b(?:matrix|mapping|parameter map)\b.{0,40}\b(?:accepted|approved|frozen|finalized)\b"
    r"|\b(?:accepted|approved|frozen|finalized)\b.{0,40}\b(?:matrix|mapping|parameter map)\b"
    r"|\b(?:empirical|observed|measured)\s+(?:value|estimate|rate|count)\s*(?:is|=|of)\s*[-+]?\d",
    re.I,
)


def _read_csv(path: Path) -> tuple[list[str], list[dict[str, str]]] | None:
    try:
        with path.open(newline="", encoding="utf-8-sig") as handle:
            reader = csv.DictReader(handle)
            if reader.fieldnames is None:
                return None
            return reader.fieldnames, list(reader)
    except (OSError, csv.Error, UnicodeError):
        return None


def validate(candidate_path: Path, disposition_path: Path) -> list[str]:
    """Return validation errors for an output compared with its immutable candidate."""
    source = _read_csv(candidate_path)
    output = _read_csv(disposition_path)
    if source is None:
        return ["cannot read candidate CSV"]
    if output is None:
        return ["cannot read disposition CSV"]
    source_header, source_rows = source
    output_header, rows = output
    errors: list[str] = []
    if source_header != output_header:
        return ["disposition CSV header must exactly match candidate CSV"]
    if len(source_rows) != 101:
        errors.append(f"candidate must contain exactly 101 data rows (found {len(source_rows)})")
    if len(rows) != 101:
        errors.append(f"disposition must contain exactly 101 data rows (found {len(rows)})")

    immutable = [name for name in source_header if name.startswith(REVIEWER_PREFIXES)]
    source_ids = [row.get("parameter_id", "") for row in source_rows]
    output_ids = [row.get("parameter_id", "") for row in rows]
    if output_ids != source_ids:
        errors.append("parameter IDs and row order must exactly match candidate")

    counts = {status: 0 for status in EXPECTED_COUNTS}
    resolutions: set[str] = set()
    for index, row in enumerate(rows):
        label = f"row {index + 2}"
        if index >= len(source_rows):
            continue
        original = source_rows[index]
        parameter_id = row.get("parameter_id", "")
        for field in immutable:
            if row.get(field, "") != original.get(field, ""):
                errors.append(f"{label} reviewer field {field} changed for {parameter_id}")
        if row.get("resolved_at", "").strip():
            errors.append(f"{label} resolved_at must be blank for {parameter_id}")

        raw = row.get("coordinator_resolution", "").strip()
        match = DISPOSITION_RE.fullmatch(raw)
        if not match:
            errors.append(f"{label} coordinator_resolution needs a proposed_accept/revise/defer prefix and resolution")
            continue
        status, resolution = match.groups()
        status = status.lower()
        counts[status] += 1
        row_number = index + 1
        expected_status = (
            "proposed_accept" if row_number in ACCEPT_ROWS
            else "proposed_defer" if row_number in DEFER_ROWS
            else "proposed_revise"
        )
        if status != expected_status:
            errors.append(
                f"row {row_number} ({parameter_id}) must use {expected_status} per coordinator map"
            )
        if not resolution.strip() or resolution.strip().lower() in {"tbd", "todo", "pending", "resolution", "n/a"}:
            errors.append(f"{label} needs a nonblank row-specific resolution for {parameter_id}")
        normalized = resolution.strip().casefold()
        if normalized in resolutions:
            errors.append(f"{label} resolution is duplicated and not row-specific for {parameter_id}")
        resolutions.add(normalized)
        if status == "proposed_defer" and not re.search(r"\btrigger\s*:\s*\S", resolution, re.I):
            errors.append(f"{label} proposed_defer needs an explicit trigger for {parameter_id}")
        if status == "proposed_revise" and not re.search(r"\breplacement\s*:\s*\S", resolution, re.I):
            errors.append(f"{label} proposed_revise needs replacement content for {parameter_id}")
        if PROMOTION_RE.search(resolution):
            errors.append(f"{label} resolution must not claim a matrix promotion or empirical value for {parameter_id}")

    for status, expected in EXPECTED_COUNTS.items():
        if counts[status] != expected:
            errors.append(f"{status} count must be {expected} (found {counts[status]})")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, default=Path("model-inputs/ed/schema/p0.2-owner-review-response-candidate.csv"))
    parser.add_argument("--disposition", type=Path, default=Path("model-inputs/ed/schema/p0.2-owner-review-disposition.csv"))
    args = parser.parse_args()
    errors = validate(args.candidate, args.disposition)
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("PASS: 101 P0.2 rows preserve candidate reviewer fields and have provisional dispositions (30/60/11)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
