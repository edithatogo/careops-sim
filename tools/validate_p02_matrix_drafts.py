#!/usr/bin/env python3
"""Check the three P0.2 structured matrix drafts before coordinator review."""

import argparse
import csv
import hashlib
import json
import re
import sys
from pathlib import Path


CHUNKS = ((1, 34, "rows-001-034.json"), (35, 68, "rows-035-068.json"), (69, 101, "rows-069-101.json"))
STAGES = {"MVP_minimum", "MVP_conditional", "hardened_v1", "deferred_or_post_v1"}
ROLES = {"generative", "observed_target", "source_metadata", "control", "mixed", "deferred"}
DECISIONS = {"accept", "revise", "defer"}
REQUIRED = ("parameter_id", "family", "decision", "config_key", "consumer", "unit_class", "unit_contract", "profile_stage", "profile_use", "value_role", "resolution")


def validate(registry: dict, proposed: dict, disposition: list[dict], chunks: list[dict]) -> list[str]:
    errors = []
    ids = [entry["parameter_id"] for entry in registry["entries"]]
    proposal = proposed["entries"]
    if len(ids) != 101 or len(set(ids)) != 101 or len(proposal) != 101 or len(disposition) != 101:
        return ["source registry, proposal and disposition must each have 101 unique ordered IDs"]
    if [entry["parameter_id"] for entry in proposal] != ids or [entry["parameter_id"] for entry in disposition] != ids:
        return ["source ID order differs"]

    all_rows = []
    for chunk, (first, last, _) in zip(chunks, CHUNKS):
        if not isinstance(chunk, dict) or chunk.get("schema_version") != 1 or chunk.get("status") != "draft_for_coordinator_review":
            errors.append(f"chunk {first}-{last} is not a v1 coordinator draft")
            continue
        if chunk.get("first_row") != first or chunk.get("last_row") != last:
            errors.append(f"chunk {first}-{last} has wrong bounds")
        rows = chunk.get("entries")
        if not isinstance(rows, list) or len(rows) != last - first + 1:
            errors.append(f"chunk {first}-{last} has wrong row count")
            continue
        all_rows.extend(rows)
    if len(all_rows) != 101:
        return errors + ["joined draft must have 101 rows"]

    gate_texts: set[str] = set()
    config_keys: set[str] = set()
    for index, row in enumerate(all_rows):
        number = index + 1
        label = f"row {number}"
        if not isinstance(row, dict):
            errors.append(f"{label} is not an object")
            continue
        if row.get("row") != number or row.get("parameter_id") != ids[index]:
            errors.append(f"{label} ID/order mismatch")
        if row.get("family") != proposal[index].get("family"):
            errors.append(f"{label} family drift")
        if any(not isinstance(row.get(key), str) or not row[key].strip() for key in REQUIRED):
            errors.append(f"{label} has missing/non-string required text")
            continue
        expected = disposition[index]["coordinator_resolution"].split(":", 1)[0].replace("proposed_", "")
        if number == 62:
            expected = "revise"  # reviewed graph-edge-distance clarification
        if row["decision"] not in DECISIONS or row["decision"] != expected:
            errors.append(f"{label} decision differs from reviewed provisional disposition")
        if not re.fullmatch(r"[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*", row["config_key"]):
            errors.append(f"{label} config_key is not a stable dotted key")
        if row["config_key"] in config_keys:
            errors.append(f"{label} duplicates a config_key")
        config_keys.add(row["config_key"])
        if row["profile_stage"] not in STAGES or row["value_role"] not in ROLES:
            errors.append(f"{label} has unknown stage/value role")
        if row["decision"] == "defer" and (row["profile_stage"] != "deferred_or_post_v1" or row["value_role"] != "deferred"):
            errors.append(f"{label} deferred input appears active")
        if "candidate input for deterministic/minimal" in row["profile_use"].lower() or row["resolution"].lower().startswith(("proposed_", "e0:", "c0:")):
            errors.append(f"{label} retains proposal boilerplate")
        if "the resolved row" in row["resolution"].lower() or "resolved replacement:" in row["resolution"].lower() or len(row["resolution"]) > 450:
            errors.append(f"{label} resolution is duplicated or not concise")
        gates = row.get("open_gate")
        if not isinstance(gates, list) or any(not isinstance(gate, str) or not gate.strip() for gate in gates):
            errors.append(f"{label} open_gate must be an array of nonempty strings")
        if row["profile_stage"] in {"hardened_v1", "deferred_or_post_v1"} and gates:
            errors.append(f"{label} later-stage row has an E2 open gate")
        if row["profile_stage"] == "MVP_minimum" and re.search(r"\bonly when\b|\bonly in\b|\bselected profiles\b", row["profile_use"], re.I):
            errors.append(f"{label} minimum stage conflicts with conditional use")
        if isinstance(gates, list):
            for gate in gates:
                if isinstance(gate, str):
                    normalized = gate.strip().casefold()
                    if normalized in gate_texts:
                        errors.append(f"{label} duplicates another E2 open gate")
                    gate_texts.add(normalized)
        if row["decision"] == "defer" and not any(word in (row["profile_use"] + row["resolution"]).lower() for word in ("until", "when", "trigger", "requires")):
            errors.append(f"{label} has no explicit deferral trigger")

    by = {row["row"]: row for row in all_rows if isinstance(row, dict) and isinstance(row.get("row"), int)}
    checks = (
        (35, lambda x: x["decision"] == "revise" and x["profile_stage"] == "MVP_conditional" and "active" in x["resolution"].lower(), "optional active disposition work"),
        (36, lambda x: x["value_role"] == "observed_target" and "boarding" in x["resolution"].lower(), "observed boarding boundary"),
        (37, lambda x: "macro" in x["resolution"].lower() and "micro" in x["resolution"].lower(), "Macro/Micro travel split"),
        (62, lambda x: x["decision"] == "revise" and "graph-edge" in (x["config_key"] + x["unit_contract"] + x["resolution"]).lower(), "graph-edge metre meaning"),
        (63, lambda x: x["profile_stage"] == "MVP_minimum" and "location" in x["resolution"].lower(), "MVP named locations"),
        (75, lambda x: "external" in x["resolution"].lower() and "competing" in x["resolution"].lower(), "external competing demand"),
        (89, lambda x: x["profile_stage"] == "MVP_minimum" and x["config_key"] == "experiment.run_seed" and "seed" in x["resolution"].lower(), "MVP run seed"),
        (90, lambda x: x["profile_stage"] == "MVP_minimum" and x["config_key"] == "time_fields.source_unit" and "global" in x["resolution"].lower() and "field" in x["resolution"].lower(), "per-field clock units"),
        (92, lambda x: x["value_role"] in {"source_metadata", "control"}, "observation-window metadata role"),
    )
    for number, predicate, description in checks:
        if number not in by or not predicate(by[number]):
            errors.append(f"row {number} missing {description}")
    return errors


def validate_final(joined: dict, final: dict, disposition: list[dict], resolved: list[dict]) -> list[str]:
    errors = []
    if (final.get("schema_version") != 2 or final.get("status") != "design_interface_accepted"
            or final.get("accepted_at") != "2026-09-30" or final.get("row_count") != 101
            or final.get("entries") != joined.get("entries")
            or final.get("source_chunks_sha256") != joined.get("source_chunks_sha256")):
        errors.append("accepted matrix differs from reviewed candidate")
    if len(resolved) != 101 or len(disposition) != 101:
        return errors + ["resolved E0/C0 CSV must have 101 rows"]
    for index, (source, row, entry) in enumerate(zip(disposition, resolved, joined["entries"]), 1):
        if row.get("parameter_id") != entry["parameter_id"] or row.get("parameter_id") != source.get("parameter_id"):
            errors.append(f"row {index} resolved ID/order mismatch")
        for key in source:
            if key.startswith(("E0_", "C0_")) and row.get(key) != source.get(key):
                errors.append(f"row {index} reviewer field drift: {key}")
        if row.get("coordinator_id") != "coordinator/root" or row.get("resolved_at") != "2026-09-30":
            errors.append(f"row {index} lacks dated coordinator resolution")
        decision = entry["decision"]
        expected_parts = (f"{decision}:", f"key={entry['config_key']};", f"consumer={entry['consumer']};",
                          f"unit={entry['unit_contract']};", f"stage={entry['profile_stage']};",
                          f"use={entry['profile_use']};", f"role={entry['value_role']};", entry["resolution"])
        text = row.get("coordinator_resolution", "")
        if not all(part in text for part in expected_parts):
            errors.append(f"row {index} coordinator resolution differs from accepted matrix")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("."))
    args = parser.parse_args()
    root = args.root
    folder = root / "conductor/evidence/p0.2-matrix-drafts"
    try:
        chunk_paths = [folder / name for _, _, name in CHUNKS]
        chunks = [json.loads(path.read_text()) for path in chunk_paths]
        registry = json.loads((root / "model-inputs/ed/schema/parameter-ids.json").read_text())
        proposed = json.loads((root / "model-inputs/ed/schema/parameter-usage-matrix-proposal.json").read_text())
        joined = json.loads((root / "model-inputs/ed/schema/parameter-usage-matrix-v2-candidate.json").read_text())
        final = json.loads((root / "model-inputs/ed/schema/parameter-usage-matrix.json").read_text())
        with (root / "model-inputs/ed/schema/p0.2-owner-review-disposition.csv").open(newline="") as file:
            disposition = list(csv.DictReader(file))
        with (root / "model-inputs/ed/schema/p0.2-owner-review-resolved.csv").open(newline="") as file:
            resolved = list(csv.DictReader(file))
    except (OSError, ValueError, csv.Error) as error:
        print(f"ERROR: cannot load P0.2 input: {error}", file=sys.stderr)
        return 1
    errors = validate(registry, proposed, disposition, chunks)
    expected_rows = [row for chunk in chunks for row in chunk["entries"]]
    expected_hashes = {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest() for path in chunk_paths}
    if (joined.get("schema_version") != 2 or joined.get("status") != "coordinator_candidate_unaccepted"
            or joined.get("row_count") != 101 or joined.get("entries") != expected_rows
            or joined.get("source_chunks_sha256") != expected_hashes):
        errors.append("joined v2 candidate differs from reviewed chunks or falsely claims acceptance")
    errors.extend(validate_final(joined, final, disposition, resolved))
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    if errors:
        return 1
    print("PASS: 101 ordered P0.2 design-interface rows, resolved E0/C0 responses and critical MVP boundary checks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
