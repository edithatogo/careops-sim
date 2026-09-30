#!/usr/bin/env python3
"""Validate P0.3 preparation-only DES/ABM source packet manifests."""

import argparse
import json
import re
import sys
from pathlib import Path


REQUIRED = (
    "packet_id", "phase_task", "family", "parameter_ids", "source_report_lead",
    "source_status", "named_gap", "search_scope", "output_path",
    "extraction_fields", "worker_oracle", "independent_readback_oracle",
    "stop_conditions", "dependency_gate",
)


def validate(plan: dict, matrix: dict) -> list[str]:
    errors = []
    track = plan.get("track")
    if plan.get("schema_version") != 1 or plan.get("status") != "preparation_only" or track not in {"P1", "P2"}:
        return ["manifest must be a v1 preparation-only P1 or P2 plan"]
    rows = plan.get("packets")
    if not isinstance(rows, list) or not rows:
        return ["packets must be a nonempty array"]
    accepted = {entry["parameter_id"]: entry for entry in matrix["entries"]}
    seen_ids = set()
    seen_outputs = set()
    folder = "des" if track == "P1" else "abm"
    for number, row in enumerate(rows, 1):
        label = f"packet {number}"
        if not isinstance(row, dict) or any(key not in row for key in REQUIRED):
            errors.append(f"{label} lacks required fields")
            continue
        packet_id = row["packet_id"]
        if not isinstance(packet_id, str) or not re.fullmatch(rf"{track.lower()}-[a-z0-9-]+", packet_id):
            errors.append(f"{label} has invalid packet_id")
        elif packet_id in seen_ids:
            errors.append(f"{label} duplicate packet_id")
        seen_ids.add(packet_id)
        if row["phase_task"] not in ({"P1.1", "P1.2"} if track == "P1" else {"P2.1", "P2.2"}):
            errors.append(f"{label} invalid phase_task")
        ids = row["parameter_ids"]
        if not isinstance(ids, list) or not ids or len(ids) != len(set(ids)) or any(x not in accepted for x in ids):
            errors.append(f"{label} has unknown/duplicate parameter IDs")
        elif any(accepted[x]["profile_stage"] == "deferred_or_post_v1" for x in ids):
            errors.append(f"{label} includes a deferred parameter without separate re-entry decision")
        elif any(accepted[x]["family"] != row["family"] for x in ids):
            errors.append(f"{label} mixes parameter families")
        lead = row["source_report_lead"]
        if not isinstance(lead, dict) or not all(isinstance(lead.get(k), str) and lead[k].strip() for k in ("report_id", "locator")):
            errors.append(f"{label} needs one named report/locator lead")
        if row["source_status"] != "unverified_lead":
            errors.append(f"{label} falsely promotes source verification")
        for key in ("family", "named_gap", "search_scope", "worker_oracle", "independent_readback_oracle", "dependency_gate"):
            if not isinstance(row[key], str) or not row[key].strip():
                errors.append(f"{label} missing {key}")
        for key in ("extraction_fields", "stop_conditions"):
            value = row[key]
            if not isinstance(value, list) or not value or any(not isinstance(x, str) or not x.strip() for x in value):
                errors.append(f"{label} missing {key}")
        output = row["output_path"]
        if not isinstance(output, str) or not re.fullmatch(rf"model-inputs/ed/{folder}/evidence/[a-z0-9-]+\.json", output):
            errors.append(f"{label} has unsafe/out-of-owner output path")
        else:
            if output != f"model-inputs/ed/{folder}/evidence/{packet_id}.json":
                errors.append(f"{label} output path does not match packet ID")
            if output in seen_outputs:
                errors.append(f"{label} duplicate output path")
            seen_outputs.add(output)
        if any(key in row for key in ("accepted_value", "fitted_distribution", "empirical_default")):
            errors.append(f"{label} contains a premature empirical/model decision")
    if plan.get("packet_count") != len(rows):
        errors.append("packet_count mismatch")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--file", type=Path, required=True)
    parser.add_argument("--matrix", type=Path, default=Path("model-inputs/ed/schema/parameter-usage-matrix.json"))
    args = parser.parse_args()
    try:
        plan = json.loads(args.file.read_text())
        matrix = json.loads(args.matrix.read_text())
    except (OSError, json.JSONDecodeError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1
    errors = validate(plan, matrix)
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    if errors:
        return 1
    print(f"PASS: {plan['packet_count']} {plan['track']} source/family preparation packets; no empirical promotion")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
