#!/usr/bin/env python3
"""Verify the bounded D0.2.inventory canary output against local Cargo manifests."""
import argparse
import json
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[1]

def expected(root):
    records = []
    for path in sorted((root/'libs/kairos/crates').glob('*/Cargo.toml')):
        data = tomllib.loads(path.read_text())
        records.append({'crate':data['package']['name'], 'manifest':str(path.relative_to(root)),
                        'dependencies':sorted(data.get('dependencies',{})),
                        'features':sorted(data.get('features',{}))})
    return records

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report',type=Path)
    args = parser.parse_args()
    try:
        actual = json.loads(args.report.read_text())
        reference = {'schema_version':1,'modules':expected(ROOT)}
        if actual != reference:
            raise ValueError('Inventory differs from exact manifest-derived schema/content/order')
        print(f'PASS: {len(reference["modules"])} module records match source manifests')
        return 0
    except (OSError,ValueError,KeyError) as error:
        print(f'FAIL: {error}')
        return 1

if __name__ == '__main__':
    raise SystemExit(main())
