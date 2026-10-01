#!/usr/bin/env python3
"""Validate a typed source record against its exact preparation packet mapping."""

import argparse
import json
import sys
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker


ROOT = Path(__file__).resolve().parents[1]


def record_errors(record, packet, record_path, root, schema):
    if list(Draft202012Validator(schema, format_checker=FormatChecker()).iter_errors(record)):
        return ['source record fails schema validation']
    errors = []
    if set(record['parameter_ids']) != set(packet['parameter_ids']):
        errors.append('record parameter IDs differ from packet')
    if record_path.resolve() != (root / packet['output_path']).resolve():
        errors.append('record path differs from packet output')
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('record', type=Path)
    parser.add_argument('--packet-id', required=True)
    parser.add_argument('--plan', type=Path, default=ROOT / 'model-inputs/ed/schema/p1-source-packet-plan.json')
    args = parser.parse_args()
    try:
        plan = json.loads(args.plan.read_text())
        matches = [row for row in plan['packets'] if row['packet_id'] == args.packet_id]
        if len(matches) != 1:
            raise ValueError('packet ID must match exactly one plan entry')
        record = json.loads(args.record.read_text())
        schema = json.loads((ROOT / 'model-inputs/ed/schema/source-verification-record.schema.json').read_text())
        errors = record_errors(record, matches[0], args.record, ROOT, schema)
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print(f'ERROR: {exc}', file=sys.stderr)
        return 1
    for error in errors:
        print(f'ERROR: {error}', file=sys.stderr)
    if errors:
        return 1
    print('PASS: typed source record and exact packet mapping; no source or empirical acceptance implied')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
