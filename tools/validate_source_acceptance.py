#!/usr/bin/env python3
"""Check the machine-verifiable gate for an ED primary-source claim."""

import argparse
import json
import sys
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker


SCHEMA_PATH = Path(__file__).resolve().parents[1] / "model-inputs/ed/schema/source-verification-record.schema.json"


def acceptance_errors(record: dict, schema: dict) -> list[str]:
    errors = list(Draft202012Validator(schema, format_checker=FormatChecker()).iter_errors(record))
    if errors:
        return ["source record fails schema validation"]
    if record.get("evidence_type") != "primary_source_claim":
        return ["only a primary-source claim can pass this gate"]
    result = []
    if record["review"]["outcome"] != "accepted":
        result.append("coordinator review is not accepted")
    if record["primary_source"]["verification"] != "verified":
        result.append("primary source is not verified")
    readback = record["independent_readback"]
    if readback["outcome"] != "pass":
        result.append("independent readback did not pass")
    recomputation = readback["source_hash_recomputation"]
    if recomputation["retrieved_bytes_sha256"] != record["primary_source"]["source_sha256"]:
        result.append("readback digest differs from primary source digest")
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("record", type=Path)
    args = parser.parse_args()
    try:
        record = json.loads(args.record.read_text(encoding="utf-8"))
        schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1
    errors = acceptance_errors(record, schema)
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    if errors:
        return 1
    print("PASS: machine-verifiable source acceptance gate; independent source readback still required")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
