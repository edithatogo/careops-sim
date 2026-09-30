#!/usr/bin/env python3
"""Normalize the selected P0.2 E0/C0 panel packets into a review candidate CSV."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
BASE_COMMIT = "16d79a5c0eff9fb60969b9e7829b5c67cdf63e46"
TEMPLATE = Path("model-inputs/ed/schema/p0.2-owner-review-response-template.csv")
SELECTED = Path("conductor/evidence/p0.2-panel/selected")
EXPECTED_SHA256 = {
    str(TEMPLATE): "c60aeb85b5b5d34cd4c2b5904478f67c40eec049f4f5cc623802e8e727c719c7",
    str(SELECTED / "c0-001-035-supplement.json"): "e8acab05a1e2be7324984f5553d9d3e498e508dde7f39f090bfd59f8ba8dbc0e",
    str(SELECTED / "c0-036-070-supplement.json"): "9b4c83167c044b344cb016dd6f556fe3b47f091e5c25255ddfdddca179fb0dd1",
    str(SELECTED / "c0-071-101.json"): "66fa327c9ac5d9573daea635c14f477504938d288ceb3afd0efe749725c71154",
    str(SELECTED / "e0-001-035-supplement.json"): "7b3a8681f7010455f504d33e5aca924f36a9da89f50924936d6fc935891c9223",
    str(SELECTED / "e0-036-070.json"): "c6989b205ed70c3a11f7f23f2ce8c14b70d186ad6d9079ac96057d27a13a020d",
    str(SELECTED / "e0-071-101.json"): "04f51d8ab9eca65a7ea58c491f2ea202fd5a1709a02b7372e16ca899cb85e799",
}


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_inputs(root: Path = ROOT) -> dict[str, str]:
    """Fail closed if the declared source base or any bound input has drifted."""
    subprocess.run(
        ["git", "cat-file", "-e", f"{BASE_COMMIT}^{{commit}}"], cwd=root,
        check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    hashes: dict[str, str] = {}
    for relative, expected in EXPECTED_SHA256.items():
        path = root / relative
        if not path.is_file():
            raise ValueError(f"missing bound input: {relative}")
        actual = _sha256(path)
        if actual != expected:
            raise ValueError(f"SHA-256 mismatch for {relative}: {actual}")
        hashes[relative] = actual
    return hashes


def _first(item: dict[str, Any], *names: str) -> Any:
    for name in names:
        if name in item and item[name] is not None:
            return item[name]
    return None


def _text(value: Any) -> str:
    if value is None:
        return ""
    if isinstance(value, list):
        return "; ".join(str(v) for v in value)
    return str(value)


def _load_role(root: Path, role: str) -> dict[str, dict[str, str]]:
    packets = sorted(
        (root / SELECTED).glob(f"{role.lower()}-*.json"), key=lambda p: p.name
    )
    if len(packets) != 3:
        raise ValueError(f"expected three selected {role} packets, found {len(packets)}")
    result: dict[str, dict[str, str]] = {}
    for path in packets:
        doc = json.loads(path.read_text(encoding="utf-8"))
        if doc.get("base_commit") != BASE_COMMIT:
            raise ValueError(f"{path.name} declares unexpected base_commit")
        row_list = next(
            (value for value in doc.values()
             if isinstance(value, list) and value and isinstance(value[0], dict)),
            None,
        )
        if row_list is None:
            raise ValueError(f"{path.name} has no row recommendations")
        row_range = re.search(r"(\d{3})-(\d{3})", path.name)
        range_start = int(row_range.group(1)) if row_range else None
        reviewer = _text(_first(doc, "reviewer_id"))
        session = _text(_first(doc, "session_id", "reviewer_session_id"))
        for offset, item in enumerate(row_list):
            row_number = _first(item, "row", "data_row", "csv_data_row", "row_number")
            if row_number is None and range_start is not None:
                row_number = range_start + offset
            parameter_id = _text(item.get("parameter_id"))
            if not isinstance(row_number, int) or not parameter_id:
                raise ValueError(f"invalid row identity in {path.name}: {item!r}")
            if parameter_id in result:
                raise ValueError(f"duplicate {role} recommendation for {parameter_id}")
            result[parameter_id] = {
                "row_number": str(row_number),
                "reviewer_id": reviewer,
                "session_id": session,
                "base_commit": BASE_COMMIT,
                "disposition": _text(_first(item, f"{role}_disposition", "disposition")),
                "rationale": _text(_first(item, f"{role}_rationale", "rationale")),
                "evidence_locator": _text(_first(item, f"{role}_evidence_locator", "evidence_locator")),
                "replacement": _text(_first(item, f"{role}_replacement", "replacement")),
            }
    return result


def normalize(root: Path = ROOT) -> tuple[list[str], list[dict[str, str]]]:
    """Return a candidate using template columns and panel suggestions only."""
    validate_inputs(root)
    template_path = root / TEMPLATE
    with template_path.open(newline="", encoding="utf-8") as stream:
        reader = csv.DictReader(stream)
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    if not fields or len(rows) != 101:
        raise ValueError(f"response template must contain 101 data rows; found {len(rows)}")
    template_ids = [row.get("parameter_id", "") for row in rows]
    if not all(template_ids) or len(set(template_ids)) != 101:
        raise ValueError("response template parameter IDs must be 101 unique non-empty IDs")
    expected_order = {parameter_id: i + 1 for i, parameter_id in enumerate(template_ids)}

    # First coverage pass: each role must independently cover all 101 template IDs.
    role_data = {role: _load_role(root, role) for role in ("E0", "C0")}
    for role, data in role_data.items():
        if set(data) != set(template_ids):
            missing = sorted(set(template_ids) - set(data))
            extra = sorted(set(data) - set(template_ids))
            raise ValueError(f"{role} coverage differs from template; missing={missing}, extra={extra}")
        for parameter_id, record in data.items():
            if int(record["row_number"]) != expected_order[parameter_id]:
                raise ValueError(f"{role} row/ID mismatch at data row {record['row_number']}: {parameter_id}")

    required_columns = {
        f"{role}_{suffix}" for role in ("E0", "C0")
        for suffix in ("reviewer_id", "disposition", "rationale", "session_id",
                       "base_commit", "evidence_locator", "replacement")
    }
    if not required_columns.issubset(fields):
        raise ValueError(f"template lacks required response columns: {sorted(required_columns - set(fields))}")
    for template_row in rows:
        parameter_id = template_row["parameter_id"]
        for role in ("E0", "C0"):
            source = role_data[role][parameter_id]
            for column, key in (
                (f"{role}_reviewer_id", "reviewer_id"),
                (f"{role}_disposition", "disposition"),
                (f"{role}_rationale", "rationale"),
                (f"{role}_session_id", "session_id"),
                (f"{role}_base_commit", "base_commit"),
                (f"{role}_evidence_locator", "evidence_locator"),
                (f"{role}_replacement", "replacement"),
            ):
                template_row[column] = source[key]

    # Candidate IDs are checked again against the actual CSV after writing.
    output_ids = [row["parameter_id"] for row in rows]
    if output_ids != template_ids or len(set(output_ids)) != 101:
        raise ValueError("normalized candidate failed pre-write 101-ID coverage pass")
    return fields, rows


def validate_candidate_file(
    path: Path, expected_ids: list[str], expected_rows: list[dict[str, str]] | None = None
) -> list[dict[str, str]]:
    """Read the emitted CSV and validate coverage, provenance, and completeness."""
    with path.open(newline="", encoding="utf-8") as stream:
        reader = csv.DictReader(stream)
        if reader.fieldnames is None:
            raise ValueError("candidate CSV has no header")
        rows = list(reader)
    ids = [row.get("parameter_id", "") for row in rows]
    if len(rows) != 101 or len(set(ids)) != 101 or ids != expected_ids:
        raise ValueError("candidate CSV readback does not contain the expected 101 IDs in order")
    if expected_rows is not None and rows != expected_rows:
        raise ValueError("candidate CSV readback differs from normalized candidate rows")
    for index, row in enumerate(rows, start=1):
        for role in ("E0", "C0"):
            for suffix in ("reviewer_id", "session_id", "disposition", "rationale", "evidence_locator"):
                if not row.get(f"{role}_{suffix}", "").strip():
                    raise ValueError(f"row {index} has empty {role}_{suffix}")
            disposition = row[f"{role}_disposition"]
            if disposition not in {"accept", "revise", "defer", "not_applicable"}:
                raise ValueError(f"row {index} has invalid {role} disposition {disposition!r}")
            if disposition == "revise" and not row.get(f"{role}_replacement", "").strip():
                raise ValueError(f"row {index} has {role} revise disposition without replacement")
    return rows


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="path for the review-candidate CSV")
    args = parser.parse_args(argv)
    try:
        hashes = validate_inputs(ROOT)
        fields, rows = normalize(ROOT)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with args.output.open("w", newline="", encoding="utf-8") as stream:
            writer = csv.DictWriter(stream, fieldnames=fields, lineterminator="\n")
            writer.writeheader()
            writer.writerows(rows)
        validate_candidate_file(args.output, [row["parameter_id"] for row in rows], rows)
    except (OSError, ValueError, json.JSONDecodeError, subprocess.CalledProcessError) as exc:
        print(f"p02_merge_panel: error: {exc}", file=sys.stderr)
        return 2
    print(f"wrote {len(rows)} candidate rows to {args.output}")
    print(f"validated base commit {BASE_COMMIT}; E0 and C0 each cover all 101 IDs")
    print("validated the candidate's 101 IDs and order on CSV readback")
    print(f"validated SHA-256 for {len(hashes)} bound inputs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
