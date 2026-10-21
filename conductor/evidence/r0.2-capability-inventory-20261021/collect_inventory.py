"""Read-only, exact-revision R0.2 source inventory. Never updates refs or pins."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib

CRATES = ('kairo-ecs-core', 'kairo-ecs-types', 'kairo-ecs-state', 'kairo-ecs-rng',
          'kairo-ecs-des', 'kairo-ecs-abm', 'kairo-ecs-arrow',
          'kairo-ecs-arrow-io', 'kairo-ecs-calibration')
SYMBOLS = ('FlowRuntime', 'FlowAllocationMode', 'FidelityAdapter',
           'FlowAgentBehavior', 'SchedulerCheckpointV1', 'FlowCheckpointV4',
           'read_ipc_file', 'write_parquet', 'ConditionWait',
           'claim_atomic_multi_resource')
def digest(data):
    return hashlib.sha256(data).hexdigest()
def collect(parent, repo, pinned, candidate):
    def git(*args, optional=False):
        p = subprocess.run(['git', '-C', str(repo), *args], capture_output=True)
        if p.returncode and not optional:
            raise RuntimeError(p.stderr.decode())
        return None if p.returncode else p.stdout
    for ref in (pinned, candidate):
        if not re.fullmatch(r'[0-9a-f]{40}', ref):
            raise ValueError('Only immutable 40-character commit IDs are accepted')
        git('cat-file', '-e', ref + '^{commit}')
    contract_path = parent / 'conductor/evidence/d2.4-kairos-contract.json'
    contract_bytes = contract_path.read_bytes()
    contract = json.loads(contract_bytes)
    if contract['kairos_commit'] != pinned:
        raise ValueError('Pinned revision differs from reviewed contract')
    result = {'scope': 'Source and historical receipt inventory only; no fresh runtime acceptance',
              'parent_head': subprocess.check_output(['git', '-C', str(parent), 'rev-parse', 'HEAD'], text=True).strip(),
              'contract_sha256': digest(contract_bytes),
              'revisions': {'pinned': pinned, 'candidate': candidate},
              'merge_base': git('merge-base', pinned, candidate).decode().strip(),
              'reviewed_paths': [], 'inventories': {}}
    for path, expected in sorted(contract['source_sha256'].items()):
        row = {'path': path, 'reviewed_sha256': expected}
        for label, ref in result['revisions'].items():
            data = git('show', ref + ':' + path, optional=True)
            row[label] = {'present': data is not None,
                          'sha256': digest(data) if data is not None else None}
        result['reviewed_paths'].append(row)
    for label, ref in result['revisions'].items():
        paths = git('ls-tree', '-r', '--name-only', ref, '--', 'crates').decode().splitlines()
        selected = [p for p in paths if any(p.startswith('crates/' + c + '/') for c in CRATES)]
        inv = {'manifests': {}, 'rust_files': [], 'selected_symbols': {s: [] for s in SYMBOLS}}
        for crate in CRATES:
            path = f'crates/{crate}/Cargo.toml'
            data = git('show', ref + ':' + path, optional=True)
            entry = {'present': data is not None}
            if data is not None:
                doc = tomllib.loads(data.decode())
                entry.update({'sha256': digest(data), 'features': doc.get('features', {}),
                              'dependencies': doc.get('dependencies', {})})
            inv['manifests'][crate] = entry
        for path in selected:
            if not path.endswith('.rs'):
                continue
            data = git('show', ref + ':' + path)
            text = data.decode()
            inv['rust_files'].append({'path': path, 'sha256': digest(data), 'bytes': len(data),
                'public_items': re.findall(r'(?m)^pub\s+(?:struct|enum|trait|type|fn)\s+(\w+)', text),
                'test_functions': re.findall(r'(?m)^\s*fn\s+(\w+)\s*\(', text) if '/tests/' in path else []})
            if '/src/' in path:
                for n, line in enumerate(text.splitlines(), 1):
                    for symbol in SYMBOLS:
                        if re.search(r'\b' + re.escape(symbol) + r'\b', line):
                            inv['selected_symbols'][symbol].append({'path': path, 'line': n, 'text': line.strip()[:250]})
        result['inventories'][label] = inv
    rows = result['reviewed_paths']
    result['summary'] = {'reviewed_paths': len(rows),
        'pinned_hash_mismatches': [r['path'] for r in rows if r['pinned']['sha256'] != r['reviewed_sha256']],
        'candidate_missing': sum(not r['candidate']['present'] for r in rows),
        'candidate_changed': sum(r['candidate']['present'] and r['candidate']['sha256'] != r['reviewed_sha256'] for r in rows)}
    return result

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--parent', type=Path, required=True)
    parser.add_argument('--kairos', type=Path, required=True)
    parser.add_argument('--pinned', required=True)
    parser.add_argument('--candidate', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = collect(args.parent, args.kairos, args.pinned, args.candidate)
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result['summary'], indent=2))
    return bool(result['summary']['pinned_hash_mismatches'])
if __name__ == '__main__':
    sys.exit(main())
