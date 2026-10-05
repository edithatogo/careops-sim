#!/usr/bin/env python3
"""Build/check a plan-derived task DAG and propose serial/parallel preparation work.

Read-only except explicit build (catalog) and bind (local artifact) commands. Never launches agents or grants leases.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
CATALOG = Path('conductor/execution/tasks.json')
# Conservative prospective write reservations; reviewed leaf packets narrow these.
SCOPES = {
    'D': ['tools', 'tests', 'conductor/dependency-policy.md'],
    'Q': ['libs/kairos/crates/kairo-ecs-des', 'libs/kairos/crates/kairo-ecs-state'],
    'C': ['libs/kairos/crates/kairo-ecs-calibration', 'libs/kairos/crates/kairo-ecs-arrow'],
    'E': ['crates/careops-ed', 'tests/ed'],
    'P': ['model-inputs/ed'],
}
COORDINATOR_TASKS = {'C2.0','D1.2','D2.2','D4.1','D5.2','E5.1','E6.1','E7.1','E8.1','P3.2','P3.3'}
OVERRIDES = {
    'D0': ['tools', 'tests', 'conductor/dependency-policy.md',
           'conductor/decisions', 'conductor/evidence',
           'conductor/module-readiness.md', 'conductor/current-state.json',
           'conductor/tracks/development_readiness_20260925/plan.md'],
    'D1': ['tools', 'tests', 'conductor/dependency-policy.md',
           'conductor/evidence'],
    'P0': ['model-inputs/ed/schema'],
    'P1': ['model-inputs/ed/des'],
    'P2': ['model-inputs/ed/abm'],
    'D2': ['.github', 'libs/kairos/.github'],
    'Q0': ['conductor/design/queue', 'libs/kairos/conductor/design/queue'],
    'Q4': ['libs/kairos/crates/kairo-ecs-des', 'libs/kairos/crates/kairo-ecs-abm',
           'libs/kairos/crates/kairo-ecs-arrow', 'libs/kairos/crates/kairo-ecs-cli'],
    'C0': ['conductor/design/calibration'],
    'C1': ['libs/kairos/crates/kairo-ecs-arrow', 'libs/kairos/schemas/arrow'],
    'C2': ['libs/kairos/crates/kairo-ecs-des', 'libs/kairos/crates/kairo-ecs-abm',
           'libs/kairos/crates/kairo-ecs-rng'],
    'C3': ['libs/kairos/crates/kairo-ecs-calibration', 'libs/kairos/crates/kairo-ecs-cli'],
    'C5': ['libs/kairos/crates/kairo-ecs-calibration', 'libs/kairos/crates/kairo-ecs-cli'],
    'E0': ['Cargo.toml', 'crates/careops-ed', 'conductor/design/ed'],
    'E5': ['apps/dashboard', 'libs/kairos/crates/kairo-ecs-wasm',
           'libs/kairos/crates/kairo-ecs-ffi', 'libs/kairos/crates/kairo-ecs-viz'],
    'E6': ['libs/kairos/crates/kairo-ecs-gpu'],
    'E7': ['libs/kairos/crates/kairo-ecs-pdes', 'libs/kairos/crates/kairo-ecs-des'],
    'E8': ['libs/kairos/crates/kairo-ecs-mpi', 'libs/kairos/crates/kairo-ecs-grpc'],
}
TASK_OVERRIDES = {
    'C2.0': ['libs/kairos/conductor/design/calibration',
             'libs/kairos/crates/kairo-ecs-calibration/tests',
             'libs/kairos/crates/kairo-ecs-des/tests',
             'conductor/evidence'],
    'D1.3': ['tools', 'tests', 'conductor/dependency-policy.md',
             'conductor/evidence', '.agents/skills',
             'conductor/tracks/development_readiness_20260925/agent-contract.md'],
    'D1.4': ['tools', 'tests', 'conductor/evidence'],
}


def key(task):
    prefix, phase, number = re.fullmatch(r'([A-Z])(\d+)\.(\d+)', task['id']).groups()
    return ({'D': 0, 'P': 1, 'Q': 2, 'C': 3, 'E': 4}.get(prefix, 9), int(phase), int(number))


def derive(root):
    tasks = []
    phase_tasks = {}
    metas = []
    for folder in sorted((root/'conductor/tracks').iterdir()):
        if not folder.is_dir():
            continue
        meta = json.loads((folder/'metadata.json').read_text())
        metas.append(meta)
        for field in ('task_dependency_overrides', 'leaf_dependency_overrides'):
            overrides = meta.get(field, {})
            if not isinstance(overrides, dict) or any(not isinstance(key, str) for key in overrides) or any(
                not isinstance(dependencies, list) or not all(isinstance(d, str) for d in dependencies)
                for dependencies in overrides.values()
            ):
                raise ValueError(f'Invalid {field}')
        plan = folder/'plan.md'
        lines = plan.read_text().splitlines()
        phase = None
        i = 0
        while i < len(lines):
            header = re.match(r'## ([A-Z]\d+) —', lines[i])
            if header:
                phase = header[1]
            match = re.match(r'- \[([ x])\] ([A-Z]\d+\.\d+) (.*)', lines[i])
            if not match:
                i += 1
                continue
            source_line = i + 1
            objective = [match[3]]
            i += 1
            while i < len(lines) and lines[i].startswith('  '):
                objective.append(lines[i].strip())
                i += 1
            task_id = match[2]
            if task_id.split('.')[0] != phase:
                raise ValueError(f'{task_id}: task is outside its declared phase')
            phase_key = f"{meta['track_id']}:{phase}"
            siblings = phase_tasks.setdefault(phase_key, [])
            overrides = meta.get('task_dependency_overrides', {})
            dependencies = list(overrides.get(task_id, siblings[-1:]))
            siblings.append(task_id)
            review = 'Conductor — review and verify phase' in match[3]
            kind = 'review_integrate' if review else ('contract_decision' if phase.endswith('0') or task_id in COORDINATOR_TASKS else 'bounded_work')
            reservations = (['conductor/tracks'] if review else
                            TASK_OVERRIDES.get(task_id, OVERRIDES.get(phase, SCOPES[phase[0]])))
            context_paths = [str((folder/f).relative_to(root)) for f in
                             ('spec.md', 'plan.md', 'test-matrix.md', 'agent-contract.md')]
            if task_id in ('D1.3', 'D1.4'):
                context_paths.extend(['AGENTS.md', 'conductor/agent-engineering.md'])
            tasks.append({
                'id': task_id, 'track_id': meta['track_id'], 'phase': phase,
                'objective': ' '.join(objective), 'kind': kind,
                'accepted': match[1] == 'x',
                'dependencies': dependencies,
                'source': {'path': str(plan.relative_to(root)), 'line': source_line},
                'context_paths': context_paths,
                'write_reservations': list(reservations),
                'dispatch_gate': 'reviewed_leaf_packet_required',
                'model_route': 'coordinator' if kind != 'bounded_work' else 'gpt-6-luna_candidate_after_packet_review',
                'phase_acceptance_source': str(plan.relative_to(root)) + '#' + phase,
            })
        for leaf_id, dependencies in meta.get('leaf_dependency_overrides', {}).items():
            parent_id = '.'.join(leaf_id.split('.')[:2])
            parent = next((task for task in tasks if task['id'] == parent_id and task['track_id'] == meta['track_id']), None)
            if parent is None or not isinstance(dependencies, list) or not all(isinstance(d, str) for d in dependencies):
                raise ValueError(f'Invalid leaf dependency override: {leaf_id}')
            parent.setdefault('leaf_dependency_overrides', {})[leaf_id] = dependencies
    by_id = {task['id']: task for task in tasks}
    if len(by_id) != len(tasks):
        raise ValueError('Task IDs must be unique across tracks')
    for meta in metas:
        overrides = meta.get('task_dependency_overrides', {})
        if not isinstance(overrides, dict):
            raise ValueError('Task dependency overrides must be a mapping')
        for task_id, dependencies in overrides.items():
            if task_id not in by_id or by_id[task_id]['track_id'] != meta['track_id']:
                raise ValueError(f'Unknown task dependency override: {task_id}')
            if not isinstance(dependencies, list) or not all(isinstance(d, str) for d in dependencies):
                raise ValueError(f'Invalid task dependency override: {task_id}')
        for phase, dependencies in meta['phase_dependencies'].items():
            current = phase_tasks[f"{meta['track_id']}:{phase}"][0]
            for dependency in dependencies:
                by_id[current]['dependencies'].append(phase_tasks[f"{meta['track_id']}:{dependency}"][-1])
        for dependency in meta.get('milestone_dependencies', []):
            current = phase_tasks[f"{meta['track_id']}:{dependency['milestone']}"][0]
            by_id[current]['dependencies'].append(phase_tasks[dependency['requires']][-1])
    for task in tasks:
        task['dependencies'] = sorted(set(task['dependencies']))
    tasks.sort(key=key)
    catalog = {'schema_version': 1, 'purpose': 'Preparation DAG; not agent launch or lease authority', 'tasks': tasks}
    validate(catalog)
    return catalog


def validate(catalog):
    tasks = catalog['tasks']
    by_id = {task['id']: task for task in tasks}
    if len(by_id) != len(tasks):
        raise ValueError('Duplicate task IDs')
    visiting, seen = set(), set()
    def visit(task_id):
        if task_id not in by_id:
            raise ValueError(f'Unknown dependency {task_id}')
        if task_id in visiting:
            raise ValueError(f'Task cycle at {task_id}')
        if task_id in seen:
            return
        visiting.add(task_id)
        for dependency in by_id[task_id]['dependencies']:
            visit(dependency)
            if by_id[task_id]['accepted'] and not by_id[dependency]['accepted']:
                raise ValueError(f'Accepted task {task_id} has unaccepted prerequisite {dependency}')
        visiting.remove(task_id)
        seen.add(task_id)
    for task_id in by_id:
        visit(task_id)


def overlap(left, right):
    a, b = PurePosixPath(left), PurePosixPath(right)
    return a == b or a in b.parents or b in a.parents


def select(catalog, accepted, workers=1, active_paths=()):
    if workers < 1:
        raise ValueError('workers must be positive')
    chosen, reserved = [], list(active_paths)
    for task in sorted(catalog['tasks'], key=key):
        if task['id'] in accepted or not set(task['dependencies']) <= accepted:
            continue
        if any(overlap(a, b) for a in task['write_reservations'] for b in reserved):
            continue
        chosen.append(task)
        reserved.extend(task['write_reservations'])
        if len(chosen) == workers:
            break
    return chosen


def schedule(catalog, workers):
    accepted = {t['id'] for t in catalog['tasks'] if t['accepted']}
    waves = []
    while len(accepted) < len(catalog['tasks']):
        chosen = select(catalog, accepted, workers)
        if not chosen:
            raise ValueError('No progress: unsatisfied dependencies or invalid graph')
        wave = [t['id'] for t in chosen]
        waves.append(wave)
        accepted.update(wave)
    return waves


def safe_relative(path):
    return bool(path) and not PurePosixPath(path).is_absolute() and '..' not in PurePosixPath(path).parts


def packet_errors(root, packet, check_git=True):
    errors = []
    required = ('packet_id','task_id','target_repo','base_commit','objective','context_paths',
                'input_hashes','write_paths','protected_paths','leaf_dependencies','interface_contract',
                'steps','verification','acceptance','result_path','stop_conditions')
    for field in required:
        if field not in packet or packet[field] == 'REQUIRED':
            errors.append(f'Unbound field: {field}')
    if errors:
        return errors
    if packet.get('status') != 'prepared':
        errors.append('Packet is not prepared')
    if not re.fullmatch(r'[0-9a-f]{40}', packet['base_commit']):
        errors.append('Base must be an exact Git commit')
    if not safe_relative(packet['target_repo']):
        errors.append('Unsafe target repository path')
        return errors
    target = root/packet['target_repo']
    for field in ('context_paths','write_paths','protected_paths'):
        for path in packet[field]:
            if not safe_relative(path):
                errors.append(f'Unsafe {field} path: {path}')
    if not safe_relative(packet['result_path']):
        errors.append('Unsafe result path')
    for path in packet['context_paths']:
        if not (target/path).is_file():
            errors.append(f'Missing context: {path}')
    if not packet['input_hashes']:
        errors.append('No input hashes')
    for path, expected in packet['input_hashes'].items():
        if not safe_relative(path):
            errors.append(f'Unsafe input path: {path}')
            continue
        source = target/path
        if not source.is_file() or hashlib.sha256(source.read_bytes()).hexdigest() != expected:
            errors.append(f'Input hash drift: {path}')
    for path, expected in packet.get('source_hashes', {}).items():
        if not safe_relative(path) or not (root/path).is_file() or hashlib.sha256((root/path).read_bytes()).hexdigest() != expected:
            errors.append(f'Cross-repository source hash drift: {path}')
    if packet.get('context_budget_bytes'):
        total = sum((target/path).stat().st_size for path in packet['context_paths'] if (target/path).is_file())
        if total > packet['context_budget_bytes']:
            errors.append('Bound context budget exceeded')
    for path in packet['write_paths']:
        if any(overlap(path, protected) for protected in packet['protected_paths']):
            errors.append(f'Protected write path: {path}')
    if not packet['write_paths']:
        errors.append('No bounded output paths')
    if not any(overlap(packet['result_path'], path) for path in packet['write_paths']):
        errors.append('Result path is outside allowed writes')
    if (len(packet['write_paths']) > 5 or len(packet['verification']) > 3) and not packet.get('size_exception'):
        errors.append('Oversized packet: split or document reviewed exception')
    for field in ('context_paths','steps','verification','acceptance','stop_conditions'):
        if not packet[field]:
            errors.append(f'Empty {field}')
    for check in packet['verification']:
        if not isinstance(check.get('argv'), list) or not check['argv'] or not isinstance(check.get('expected_exit'), int):
            errors.append('Verification needs argv and expected_exit')
        if not safe_relative(check.get('cwd','')) or not (target/check.get('cwd','')).is_dir():
            errors.append('Verification cwd missing or unsafe')
        if not check.get('oracle'):
            errors.append('Verification needs behavioral oracle')
    if packet['verification'] and packet['verification'][-1].get('expected_exit') != 0:
        errors.append('Final verification must expect success')
    if check_git:
        run = subprocess.run(['git','-C',str(target),'rev-parse','HEAD'],capture_output=True,text=True)
        if run.returncode or run.stdout.strip() != packet['base_commit']:
            errors.append('Base commit drift: rebind/review before dispatch')
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    sub.add_parser('build')
    sub.add_parser('check')
    for name in ('ready','schedule'):
        part = sub.add_parser(name)
        part.add_argument('--mode', choices=('serial','parallel'), default='serial')
        part.add_argument('--workers', type=int, default=4)
        part.add_argument('--reserved-path', action='append', default=[])
    bind = sub.add_parser('bind')
    bind.add_argument('path', type=Path)
    packet = sub.add_parser('packet-check')
    packet.add_argument('path', type=Path)
    args = parser.parse_args()
    try:
        catalog = derive(ROOT)
        path = ROOT/CATALOG
        if args.command == 'build':
            path.write_text(json.dumps(catalog, indent=2)+'\n')
            print(f'Built {len(catalog["tasks"])} task records')
            return 0
        if not path.exists() or json.loads(path.read_text()) != catalog:
            raise ValueError('Task catalog drift; run build and review the diff')
        if args.command == 'bind':
            data = json.loads(args.path.read_text())
            if not re.fullmatch(r'[A-Za-z0-9_.-]+', data.get('packet_id','')):
                raise ValueError('Invalid packet ID')
            if not safe_relative(data.get('target_repo','')):
                raise ValueError('Invalid target repo')
            target = ROOT/data['target_repo']
            run = subprocess.run(['git','-C',str(target),'status','--porcelain'],capture_output=True,text=True)
            if run.returncode or run.stdout.strip():
                raise ValueError('Bind only against a clean target checkout; commit/review edits first')
            data['base_commit'] = subprocess.check_output(['git','-C',str(target),'rev-parse','HEAD'],text=True).strip()
            inputs = data.get('input_paths',list(data.get('input_hashes',{})))
            if any(not safe_relative(p) for p in inputs):
                raise ValueError('Unsafe input path')
            data['input_hashes'] = {p:hashlib.sha256((target/p).read_bytes()).hexdigest() for p in inputs}
            data['status'] = 'prepared'
            errors = packet_errors(ROOT,data)
            if errors:
                raise ValueError('; '.join(errors))
            output = ROOT/'.artifacts/packets'/f"{data['packet_id']}.json"
            output.parent.mkdir(parents=True,exist_ok=True)
            output.write_text(json.dumps(data,indent=2)+'\n')
            print(f'Bound {output}; coordinator review/claim still required before dispatch')
        elif args.command == 'check':
            print(f'{len(catalog["tasks"])} task records: full coverage, prerequisites and DAG valid')
        elif args.command == 'packet-check':
            data = json.loads(args.path.read_text())
            errors = packet_errors(ROOT, data)
            task = next((t for t in catalog['tasks'] if t['id'] == data.get('task_id')), None)
            if task is None:
                errors.append('Unknown parent task')
            elif any(not next(t for t in catalog['tasks'] if t['id'] == d)['accepted'] for d in task['dependencies']):
                errors.append('Parent-task prerequisite is not accepted')
            print(json.dumps({'errors': errors, 'authority':'Validation only; coordinator must claim/review leaf dependencies and dispatch'}, indent=2))
            return int(bool(errors))
        else:
            workers = 1 if args.mode == 'serial' else args.workers
            if workers < 1:
                raise ValueError('workers must be positive')
            if args.command == 'schedule':
                print(json.dumps({'mode':args.mode, 'hypothetical_waves':schedule(catalog,workers),
                                  'note':'Planning simulation only; does not accept or execute tasks'},indent=2))
            else:
                accepted = {t['id'] for t in catalog['tasks'] if t['accepted']}
                selected = select(catalog, accepted, workers, args.reserved_path)
                print(json.dumps({'preparation_candidates': selected,
                                  'note':'Not a lease or launch. Bind reviewed leaf packets; coordinator checks active reservations.'},indent=2))
        return 0
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f'ERROR: {error}')
        return 1

if __name__ == '__main__':
    raise SystemExit(main())
