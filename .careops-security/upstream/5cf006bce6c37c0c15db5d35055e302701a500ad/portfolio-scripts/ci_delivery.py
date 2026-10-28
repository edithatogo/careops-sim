"""Shared source-only CI primitives: exact resolution, locks and honest outcomes."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
from pathlib import Path

PIN = re.compile(r'^([A-Za-z0-9][A-Za-z0-9_.-]*)==([A-Za-z0-9][A-Za-z0-9_.+!-]*)$')
SHA = re.compile(r'^[a-f0-9]{40}$')
BILLING = re.compile(r'recent account payments have failed|spending limit needs to be increased|billing.*(?:limit|payment)', re.I)


def exact_requirements(path: Path) -> dict[str, str]:
    result = {}
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        match = PIN.fullmatch(line)
        if not match:
            raise ValueError('CI requirements must contain exact registry pins only.')
        name, version = match.groups()
        key = re.sub(r'[-_.]+', '-', name).lower()
        if key in result:
            raise ValueError('Duplicate CI dependency.')
        result[key] = version
    if not result:
        raise ValueError('Empty CI requirements.')
    return result


def verify_declared_lock(root: Path) -> None:
    """Required source CI must use the declared exact versions, not a stale lock."""
    expected = exact_requirements(root / 'requirements-ci.txt')
    selected = {
        re.sub(r'[-_.]+', '-', name).lower(): version
        for name, version in re.findall(r'^([A-Za-z0-9_.-]+)==([^\s\\]+)', (root / 'requirements-ci.lock').read_text(), re.M)
    }
    if any(selected.get(name) != version for name, version in expected.items()):
        raise ValueError('Committed CI lock differs from declared exact tools; apply the generated repair.')


def peel_commit(obj: dict, get_tag) -> str:
    """Resolve annotated tags to commits; comparing tag-object IDs is incorrect."""
    seen = set()
    for _ in range(8):
        kind, sha = obj.get('type'), obj.get('sha', '')
        if not isinstance(sha, str) or not SHA.fullmatch(sha) or sha in seen:
            raise ValueError('Invalid or cyclic upstream reference.')
        seen.add(sha)
        if kind == 'commit':
            return sha
        if kind != 'tag':
            raise ValueError('Upstream reference does not identify a commit.')
        obj = get_tag(sha)['object']
    raise ValueError('Too many annotated tag layers.')


def classify_run(run: dict, jobs: list[dict], annotations: list[dict]) -> dict:
    """A job refused before execution is blocked, never a tested failure/pass."""
    billing = any(BILLING.search(str(a.get('message', ''))) for a in annotations)
    failures = [j.get('name', 'unnamed') for j in jobs if j.get('conclusion') == 'failure' and any(s.get('status') == 'completed' and s.get('conclusion') not in ['skipped', None] for s in j.get('steps', []))]
    if run.get('status') != 'completed':
        state = 'pending'
    elif failures:
        state = 'executed-failure'
    elif billing and run.get('conclusion') != 'success':
        state = 'billing-blocked'
    elif run.get('conclusion') == 'success' and jobs and any(j.get('conclusion') == 'success' for j in jobs) and all(j.get('conclusion') in ['success', 'skipped'] for j in jobs):
        state = 'passed'
    elif run.get('conclusion') in ['cancelled', 'skipped', 'timed_out']:
        state = run['conclusion']
    else:
        state = 'unverified-failure' if run.get('conclusion') == 'failure' else 'unverified'
    # Emit an allowlisted boolean category, never annotation-derived account text.
    return {'state': state, 'billingAnnotationPresent': True if billing else False, 'executedFailureJobs': failures, 'runId': run.get('id'), 'headSha': run.get('head_sha'), 'operationalAcceptance': False}


def version_key(value: str):
    match = re.fullmatch(r'(\d+)\.(\d+)\.(\d+)(?:-(alpha|beta|rc)\.(\d+))?', value)
    if not match:
        raise ValueError('Unsupported runtime version format.')
    major, minor, patch, stage, number = match.groups()
    return (int(major), int(minor), int(patch), {None: 3, 'rc': 2, 'beta': 1, 'alpha': 0}[stage], int(number or 0))


def select_python_preview(manifest: list[dict], baseline: str = '3.15.1', platform_version: str = '24.04') -> dict:
    base = version_key(baseline)
    candidates = []
    for release in manifest:
        if release.get('stable') is not False:
            continue
        version = release.get('version', '')
        try:
            key = version_key(version)
        except (ValueError, TypeError):
            continue
        if key <= base or key[:2] != base[:2]:
            continue
        assets = [f for f in release.get('files', []) if f.get('platform') == 'linux' and f.get('platform_version') == platform_version and f.get('arch') == 'x64' and str(f.get('download_url', '')).startswith('https://github.com/actions/python-versions/releases/download/')]
        if assets:
            candidates.append((key, version, assets[0]['download_url']))
    if not candidates:
        return {'state': 'unavailable', 'baseline': baseline, 'candidate': None, 'runtimeTestsExecuted': False, 'promoted': False}
    _, version, asset = max(candidates)
    return {'state': 'candidate-resolved', 'baseline': baseline, 'candidate': version, 'asset': asset, 'runtimeTestsExecuted': False, 'promoted': False}


def finalize_runtime(discovery: dict, outcome: str, discovery_outcome: str = 'success') -> dict:
    if outcome not in ['success', 'failure', 'cancelled', 'skipped']:
        raise ValueError('Unknown runtime test outcome.')
    if discovery_outcome not in ['success', 'failure', 'cancelled', 'skipped']:
        raise ValueError('Unknown runtime discovery outcome.')
    result = dict(discovery)
    result['discoveryOutcome'] = discovery_outcome
    if discovery_outcome != 'success':
        if outcome in ['success', 'failure']:
            raise ValueError('Runtime execution cannot qualify unsuccessful discovery.')
        result.update(state={'failure': 'discovery-failed', 'cancelled': 'cancelled', 'skipped': 'not-executed'}[discovery_outcome],
                      runtimeTestsExecuted=False, runtimeTestOutcome=outcome, promoted=False)
        return result
    if not SHA.fullmatch(str(result.get('manifestRevision', ''))) or not re.fullmatch(r'[a-f0-9]{64}', str(result.get('manifestSha256', ''))):
        raise ValueError('Missing immutable manifest identity.')
    candidate = result.get('candidate')
    if candidate is None:
        if result.get('state') != 'unavailable':
            raise ValueError('Absent candidate contradicts discovery state.')
    else:
        if result.get('state') != 'candidate-resolved' or not isinstance(candidate, str):
            raise ValueError('Candidate contradicts discovery state.')
        if (version_key(candidate) <= version_key(result.get('baseline', ''))
                or version_key(candidate)[:2] != version_key(result.get('baseline', ''))[:2]
                or not re.search(r'alpha|beta|rc', candidate)):
            raise ValueError('Candidate must be an exact newer prerelease in the required Python line.')
        if not str(result.get('asset', '')).startswith('https://github.com/actions/python-versions/releases/download/'):
            raise ValueError('Candidate asset must come from the frozen upstream manifest.')
    if not result.get('candidate'):
        if outcome in ['success', 'failure']:
            raise ValueError('Runtime tests cannot execute without a resolved candidate.')
        result['state'] = 'unavailable'
        result['runtimeTestsExecuted'] = False
    else:
        result['runtimeTestsExecuted'] = outcome in ['success', 'failure']
        result['state'] = {'success': 'qualified', 'failure': 'executed-failure', 'cancelled': 'cancelled', 'skipped': 'setup-blocked'}[outcome]
    result['runtimeTestOutcome'] = outcome
    result['promoted'] = False
    return result


def regenerate(root: Path, output: Path, *, preview: bool = False) -> dict:
    if not output.resolve().is_relative_to(root.resolve()):
        raise ValueError('Generated lock must stay inside the selected source workspace.')
    requirements = root / 'requirements-ci.txt'
    pins = exact_requirements(requirements)
    source = requirements
    if preview:
        source = output.with_suffix('.in')
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text('\n'.join(pins) + '\n')
    output.parent.mkdir(parents=True, exist_ok=True)
    cmd = ['uv', 'pip', 'compile', str(source.relative_to(root)), '--no-config', '--generate-hashes', '--python-version', '3.15.1', '--output-file', str(output.relative_to(root)), '--quiet']
    if preview:
        cmd += ['--upgrade', '--prerelease', 'allow']
    subprocess.run(cmd, cwd=root, check=True)
    # The candidate receipt contains exact package versions and a lock digest,
    # never credential stores, native data or arbitrary registry URLs.
    resolved = {}
    for line in output.read_text().splitlines():
        match = re.match(r'^([A-Za-z0-9_.-]+)==([^ \\]+)', line)
        if match:
            resolved[match[1]] = match[2]
    if not resolved or '--hash=sha256:' not in output.read_text():
        raise ValueError('Generated lock is incomplete.')
    return {'channel': 'preview-candidate' if preview else 'declared-exact-pins', 'dependencies': resolved, 'lockSha256': hashlib.sha256(output.read_bytes()).hexdigest(), 'promoted': False, 'nativeWrites': False}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    lock = sub.add_parser('lock')
    lock.add_argument('--root', type=Path, default=Path('.'))
    lock.add_argument('--output', type=Path, required=True)
    lock.add_argument('--receipt', type=Path, required=True)
    lock.add_argument('--preview', action='store_true')
    verify = sub.add_parser('verify-lock')
    verify.add_argument('--root', type=Path, default=Path('.'))
    outcome = sub.add_parser('classify')
    outcome.add_argument('--input', type=Path, required=True)
    outcome.add_argument('--output', type=Path, required=True)
    runtime = sub.add_parser('runtime')
    runtime.add_argument('--manifest', type=Path, required=True)
    runtime.add_argument('--manifest-revision', required=True)
    runtime.add_argument('--platform-version', required=True)
    runtime.add_argument('--output', type=Path, required=True)
    runtime.add_argument('--github-output', type=Path, required=True)
    finalize = sub.add_parser('runtime-finalize')
    finalize.add_argument('--receipt', type=Path, required=True)
    finalize.add_argument('--outcome', choices=['success', 'failure', 'cancelled', 'skipped'], required=True)
    finalize.add_argument('--discovery-outcome', choices=['success', 'failure', 'cancelled', 'skipped'], default='success')
    args = parser.parse_args()
    if args.command == 'verify-lock':
        verify_declared_lock(args.root)
        print('Committed CI lock matches declared exact versions.')
    elif args.command == 'lock':
        result = regenerate(args.root.resolve(), args.output.resolve(), preview=args.preview)
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(json.dumps(result, indent=2) + '\n')
        print('Exact hash lock and candidate receipt generated; no promotion performed.')
    elif args.command == 'runtime-finalize':
        raw = None
        invalid = False
        try:
            raw = args.receipt.read_bytes() if args.receipt.exists() else None
            discovery = json.loads(raw) if raw is not None else {}
            if not isinstance(discovery, dict) or (args.discovery_outcome == 'success' and 'candidate' not in discovery):
                raise ValueError('Missing or invalid discovery evidence.')
            result = finalize_runtime(discovery, args.outcome, args.discovery_outcome)
        except (OSError, ValueError, TypeError):
            invalid = True
            result = {'state': 'evidence-invalid', 'discoveryOutcome': args.discovery_outcome,
                      'runtimeTestOutcome': args.outcome, 'runtimeTestsExecuted': None, 'promoted': False}
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        if invalid and raw is not None:
            args.receipt.with_name(args.receipt.name + '.invalid').write_bytes(raw)
        args.receipt.write_text(json.dumps(result, indent=2) + '\n')
        print('Runtime qualification outcome recorded without promotion.')
        if invalid:
            raise SystemExit(1)
    elif args.command == 'runtime':
        if not SHA.fullmatch(args.manifest_revision):
            parser.error('Runtime manifest revision must be an immutable commit.')
        result = select_python_preview(json.loads(args.manifest.read_text()), platform_version=args.platform_version)
        result['manifestRevision'] = args.manifest_revision
        result['manifestSha256'] = hashlib.sha256(args.manifest.read_bytes()).hexdigest()
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2) + '\n')
        with args.github_output.open('a') as handle:
            handle.write('available=' + ('true' if result['candidate'] else 'false') + '\n')
            handle.write('version=' + (result['candidate'] or '') + '\n')
        print('Runtime preview discovery recorded; absence does not qualify a runtime.')
    else:
        data = json.loads(args.input.read_text())
        result = classify_run(data['run'], data.get('jobs', []), data.get('annotations', []))
        if not args.output.resolve().is_relative_to((Path.cwd() / '.local').resolve()):
            parser.error('Hosted outcome receipt must remain in ignored .local storage.')
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with open(args.output, 'w', opener=lambda path, flags: os.open(path, flags, 0o600)) as handle:
            os.fchmod(handle.fileno(), 0o600)
            handle.write(json.dumps(result, indent=2) + '\n')
        print('Hosted outcome classified; inspect the private receipt.')


if __name__ == '__main__':
    main()
