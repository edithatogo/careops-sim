#!/usr/bin/env python3
"""Validate joined D0.2 evidence against the checked-out sources and task DAG."""
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def read_json(relative):
    return json.loads((ROOT / relative).read_text())


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def fail(message):
    raise ValueError(message)


def main():
    manifest_path = 'conductor/evidence/d0.2-kairos-manifest-inventory-20260927.json'
    capability_path = 'conductor/evidence/d0.2-kairos-capability-inventory-20260927.json'
    gates_path = 'conductor/evidence/d0.2-prerequisite-identity-audit-20260927.json'

    manifest = read_json(manifest_path)
    if manifest.get('schema_version') != 1 or len(manifest.get('modules', [])) != 24:
        fail('manifest inventory must contain the 24 checked-in Kairos crate manifests')
    manifests = sorted(p.relative_to(ROOT).as_posix()
                       for p in (ROOT / 'libs/kairos/crates').glob('*/Cargo.toml'))
    if [row['manifest'] for row in manifest['modules']] != manifests:
        fail('manifest inventory paths/order do not match current Kairos source')

    capabilities = read_json(capability_path)
    rows = capabilities.get('capabilities', [])
    readiness = (ROOT / 'conductor/module-readiness.md').read_text().splitlines()
    start = next(i for i, line in enumerate(readiness) if line.startswith('| Module(s) |'))
    expected = []
    for line in readiness[start + 2:]:
        if not line.startswith('|'):
            break
        match = re.match(r'\|\s*([^|]+?)\s*\|', line)
        if match:
            expected.append(match[1])
    if [row.get('module') for row in rows] != expected or len(rows) != 25:
        fail('capability rows do not exactly cover the 25 module-readiness rows')
    for row in rows:
        if row.get('current_state') not in {'implemented', 'partial', 'missing', 'deferred'}:
            fail(f"invalid capability state for {row.get('module')}")
        if not row.get('owner') or not row.get('delivery_gate') or not row.get('evidence'):
            fail(f"missing owner, gate or evidence for {row.get('module')}")
        for evidence in row['evidence']:
            source = ROOT / evidence['path']
            lines = source.read_text().splitlines()
            for span in evidence['line_range'].split(','):
                first, last = (map(int, span.split('-')) if '-' in span
                               else (int(span), int(span)))
                if not 1 <= first <= last <= len(lines):
                    fail(f"invalid evidence range for {row['module']}: {span}")

    gates = read_json(gates_path)
    baseline = gates['baseline']
    parent_pin = git('ls-tree', baseline['parent_commit'], 'libs/kairos').split()[2]
    current_state = read_json('conductor/current-state.json')
    if baseline['kairos_pin'] != baseline['kairos_head'] or parent_pin != baseline['kairos_pin']:
        fail('D0.2 audit pin does not match both parent gitlink and Kairos checkout')
    if current_state['submodule_pins']['libs/kairos'] != baseline['kairos_pin']:
        fail('current-state Kairos pin disagrees with the D0.2 audit')
    if len(gates.get('identity_claims', [])) != 10:
        fail('D0.2 identity audit must retain its ten authority-scoped claims')
    if not gates['licensing'].get('discrepancies') or not gates.get('unresolved'):
        fail('licensing discrepancy and unresolved claims must remain explicit')

    tasks = read_json('conductor/execution/tasks.json')['tasks']
    by_id = {task['id']: task for task in tasks}
    if 'D1.6' not in by_id['Q0.1']['dependencies']:
        fail('Q0.1 no longer depends on D1 closeout')
    if not {'D2.5', 'Q0.4'} <= set(by_id['Q1.1']['dependencies']):
        fail('Q1.1 no longer depends on D2 and Q0 closeout')
    if 'E0.4' not in by_id['D2.1']['dependencies']:
        fail('D2 no longer waits for E0 closeout')

    if not re.search(r'- \[x\] D0\.2\b',
                     (ROOT / 'conductor/tracks/development_readiness_20260925/plan.md').read_text()):
        fail('D0.2 parent task is not marked complete')
    for relative in (
        'conductor/evidence/d0.2-research-audit-reports-10-11-20260927.md',
        'conductor/evidence/d0.2-research-audit-reports-26-28-20260927.md',
    ):
        report = (ROOT / relative).read_text().lower()
        if 'contradicted' not in report or 'unresolved' not in report:
            fail(f'missing verified/contradicted/unresolved classifications: {relative}')

    result = subprocess.run(
        [sys.executable, 'tools/verify_inventory.py', manifest_path],
        cwd=ROOT, text=True, capture_output=True)
    if result.returncode:
        fail(f'manifest verifier failed: {result.stdout}{result.stderr}')
    print('PASS: D0.2 inventory, capability/source coverage, pin, licensing, identity, research and cross-track gates')
    print(result.stdout.strip())


if __name__ == '__main__':
    try:
        main()
    except (OSError, KeyError, ValueError, IndexError, StopIteration) as exc:
        print(f'FAIL: {exc}', file=sys.stderr)
        raise SystemExit(1)
