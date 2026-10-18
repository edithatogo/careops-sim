"""Freeze official Rust candidates and qualify them without promoting source pins."""
from __future__ import annotations

import argparse
from datetime import date
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib
import urllib.request

CHANNELS = ('stable', 'beta', 'nightly')
HOST = 'x86_64-unknown-linux-gnu'
ORIGIN = 'https://static.rust-lang.org/dist/'
STAGES = ('metadata', 'install', 'binding', 'tests', 'doctests', 'clippy')


def digest(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def fetch(url: str) -> bytes:
    with urllib.request.urlopen(url, timeout=30) as response:
        raw = response.read(8_000_001)
    if len(raw) > 8_000_000:
        raise ValueError('Manifest exceeds bounded input size')
    return raw


def select(channel: str, raw: bytes, baseline: str) -> dict | None:
    if channel not in CHANNELS or not re.fullmatch(r'\d+\.\d+\.\d+', baseline):
        raise ValueError('Invalid channel or canonical baseline')
    metadata = tomllib.loads(raw.decode('utf-8'))
    day = metadata['date']
    if date.fromisoformat(day).isoformat() != day:
        raise ValueError('Manifest date is not exact ISO format')
    compiler = metadata['pkg']['rustc']
    release = compiler['version'].split(' ', 1)[0]
    pattern = {'stable': r'\d+\.\d+\.\d+', 'beta': r'\d+\.\d+\.\d+-beta\.\d+',
               'nightly': r'\d+\.\d+\.\d+-nightly'}[channel]
    commit = compiler['git_commit_hash']
    if not re.fullmatch(pattern, release) or not re.fullmatch(r'[a-f0-9]{40}', commit):
        raise ValueError('Manifest compiler identity is invalid')
    for package in ('rustc', 'cargo', 'rust-std'):
        if metadata['pkg'][package]['target'][HOST]['available'] is not True:
            raise ValueError('Required native package unavailable')
    if channel == 'stable' and tuple(map(int, release.split('.'))) <= tuple(map(int, baseline.split('.'))):
        return None
    return {'channel': channel, 'date': day, 'toolchain': release if channel == 'stable' else f'{channel}-{day}',
            'release': release, 'compilerCommit': commit, 'host': HOST, 'baseline': baseline,
            'manifestSha256': digest(raw), 'manifestUrl': f'{ORIGIN}{day}/channel-rust-{channel}.toml'}


def compiler_matches(text: str, candidate: dict) -> None:
    fields = dict(line.split(': ', 1) for line in text.splitlines() if ': ' in line)
    expected = {'release': candidate['release'], 'commit-hash': candidate['compilerCommit'], 'host': candidate['host']}
    if any(fields.get(key) != value for key, value in expected.items()):
        raise ValueError('Installed compiler differs from frozen manifest')


def write(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def discover(output: Path, github_output: Path, baseline: str) -> int:
    receipt = {'baseline': baseline, 'channels': {}, 'candidates': [], 'promoted': False}
    output.mkdir(parents=True, exist_ok=True)
    for channel in CHANNELS:
        try:
            raw = fetch(f'{ORIGIN}channel-rust-{channel}.toml')
            (output / f'channel-rust-{channel}.toml').write_bytes(raw)
            candidate = select(channel, raw, baseline)
            if candidate:
                if fetch(candidate['manifestUrl']) != raw:
                    raise ValueError('Dated manifest does not match discovery bytes')
                receipt['candidates'].append(candidate)
            receipt['channels'][channel] = {'state': 'candidate-resolved' if candidate else 'no-newer-stable',
                                             'manifestSha256': digest(raw)}
        except (OSError, ValueError, KeyError, TypeError) as error:
            receipt['channels'][channel] = {'state': 'discovery-failed', 'errorType': type(error).__name__}
    failed = any(item['state'] == 'discovery-failed' for item in receipt['channels'].values())
    receipt['state'] = 'discovery-failed' if failed else 'discovered'
    write(output / 'discovery.json', receipt)
    with github_output.open('a') as handle:
        handle.write('matrix=' + json.dumps({'candidate': receipt['candidates']}, separators=(',', ':')) + '\n')
        handle.write('available=' + ('true' if receipt['candidates'] else 'false') + '\n')
    return int(failed)


def source_snapshot(root: Path | None = None) -> dict:
    root = (Path.cwd() if root is None else root).resolve()
    def git(*arguments, input=None):
        return subprocess.check_output(['git', '-C', str(root), *arguments], text=True, input=input)
    if Path(git('rev-parse', '--show-toplevel').strip()).resolve() != root:
        raise ValueError('Source checkout is missing or uninitialized')
    if git('status', '--porcelain', '--untracked-files=normal', '--ignore-submodules=none', '--', '.', ':(exclude).artifacts/rust-canary').strip():
        raise ValueError('Source checkout is not clean')
    tracked = {}
    expected_blobs = {}
    links = {}
    for entry in git('ls-files', '--stage', '-z').split('\0'):
        if not entry:
            continue
        metadata, name = entry.split('\t', 1)
        mode, expected, stage = metadata.split()
        if stage != '0':
            raise ValueError('Unmerged source index')
        if name == '.artifacts/rust-canary' or name.startswith('.artifacts/rust-canary/'):
            continue
        path = root / name
        if mode == '160000':
            if not path.resolve().is_relative_to(root) or path.resolve() == root:
                raise ValueError('Submodule escapes source checkout')
            child = source_snapshot(path)
            if child['commit'] != expected:
                raise ValueError('Actual submodule commit differs from pinned index')
            links[name] = child
        elif mode == '120000':
            if not path.is_symlink():
                raise ValueError('Tracked symlink was replaced')
            raw = os.fsencode(os.readlink(path))
            algorithm = 'sha1' if len(expected) == 40 else 'sha256'
            if hashlib.new(algorithm, b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() != expected:
                raise ValueError('Tracked symlink differs from pinned blob')
            tracked[name] = {'mode': mode, 'sha256': digest(raw)}
        else:
            if path.is_symlink() or not path.is_file():
                raise ValueError('Tracked source file was replaced')
            tracked[name] = {'mode': mode, 'executable': bool(path.stat().st_mode & 0o111), 'sha256': digest(path.read_bytes())}
            if any(ord(character) < 32 for character in name):
                raise ValueError('Unsupported control character in source path')
            expected_blobs[name] = expected
    actual_blobs = git('hash-object', '--stdin-paths', input=''.join(json.dumps(name, ensure_ascii=False) + '\n' for name in expected_blobs)).splitlines()
    if actual_blobs != list(expected_blobs.values()):
        raise ValueError('Actual source bytes differ from the pinned Git blobs')
    return {'commit': git('rev-parse', 'HEAD').strip(), 'trackedFileCount': len(tracked),
            'trackedSha256': digest(json.dumps(tracked, sort_keys=True, separators=(',', ':')).encode()), 'gitlinks': links}


def commands_for(toolchain: str) -> dict:
    return dict([('install', ['rustup', 'toolchain', 'install', toolchain, '--profile', 'minimal', '--component', 'clippy']),
                 ('binding', ['rustup', 'run', toolchain, 'rustc', '-Vv']),
                 ('tests', ['rustup', 'run', toolchain, 'cargo', 'test', '--locked', '--workspace', '--lib', '--bins', '--tests']),
                 ('doctests', ['rustup', 'run', toolchain, 'cargo', 'test', '--locked', '--workspace', '--doc']),
                 ('clippy', ['rustup', 'run', toolchain, 'cargo', 'clippy', '--locked', '--workspace', '--all-targets', '--', '-D', 'warnings'])])


def qualify(candidate: dict, output: Path, baseline: str) -> int:
    output.mkdir(parents=True, exist_ok=True)
    stages = dict.fromkeys(STAGES, 'skipped')
    receipt = {'candidate': candidate, 'stages': stages, 'state': 'setup-failed', 'runtimeTestsExecuted': False,
               'promoted': False, 'productionWrites': False, 'commands': {}, 'exitCodes': {}}
    active = 'metadata'
    protected = {name: Path(name).read_bytes() for name in ('rust-toolchain.toml', 'Cargo.lock')}
    try:
        receipt['sourceBefore'] = source_snapshot()
        receipt['sourceCommit'] = receipt['sourceBefore']['commit']
        channel = candidate['channel']
        if channel not in CHANNELS or candidate['baseline'] != baseline:
            raise ValueError('Unqualified channel or baseline')
        day = date.fromisoformat(candidate['date']).isoformat()
        url = f'{ORIGIN}{day}/channel-rust-{channel}.toml'
        raw = fetch(url)
        if select(channel, raw, baseline) != candidate:
            raise ValueError('Candidate differs from frozen official manifest')
        (output / 'manifest.toml').write_bytes(raw)
        stages['metadata'] = 'success'
        toolchain = candidate['toolchain']
        for active, command in commands_for(toolchain).items():
            if source_snapshot() != receipt['sourceBefore']:
                raise ValueError('Source drift before native command')
            if active in ('tests', 'doctests', 'clippy'):
                receipt['runtimeTestsExecuted'] = True
            receipt['commands'][active] = command
            env = dict(os.environ, CARGO_TARGET_DIR=str((output / 'target').resolve()), RUSTUP_TOOLCHAIN=toolchain)
            if active != 'install':
                tools = {name: subprocess.check_output(['rustup', 'which', '--toolchain', toolchain, name], text=True).strip() for name in ('rustc', 'rustdoc', 'cargo')}
                if any(not Path(path).is_absolute() or Path(path).name != name for name, path in tools.items()) or len({str(Path(path).parent) for path in tools.values()}) != 1:
                    raise ValueError('Rust tools do not share the selected toolchain directory')
                receipt['tools'] = tools
                env.update(RUSTC=tools['rustc'], RUSTDOC=tools['rustdoc'], PATH=str(Path(tools['cargo']).parent) + os.pathsep + env.get('PATH', ''))
            with (output / f'{active}.log').open('wb') as log:
                result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, env=env, timeout=1200)
            receipt['exitCodes'][active] = result.returncode
            if source_snapshot() != receipt['sourceBefore']:
                raise ValueError('Source drift after native command')
            if result.returncode:
                stages[active] = 'failure'
                if active in ('install', 'binding'):
                    break
            else:
                if active == 'binding':
                    compiler_matches((output / 'binding.log').read_text(), candidate)
                stages[active] = 'success'
        receipt['state'] = 'qualified' if all(value == 'success' for value in stages.values()) else ('executed-failure' if receipt['runtimeTestsExecuted'] else 'setup-failed')
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        stages[active] = 'failure'
        receipt['errorType'] = type(error).__name__
        receipt['state'] = 'executed-failure' if receipt['runtimeTestsExecuted'] else 'setup-failed'
    finally:
        try:
            receipt['sourceAfter'] = source_snapshot()
            unchanged = receipt.get('sourceBefore') == receipt['sourceAfter'] and all(Path(name).read_bytes() == raw for name, raw in protected.items())
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            unchanged = False
            receipt['sourceErrorType'] = type(error).__name__
        receipt['sourcePinSha256'] = {name: digest(raw) for name, raw in protected.items()}
        receipt['sourcePinsUnchanged'] = unchanged
        if not unchanged:
            receipt['state'] = 'source-drift'
        receipt['logSha256'] = {p.name: digest(p.read_bytes()) for p in output.glob('*.log')}
        write(output / 'qualification.json', receipt)
    return int(receipt['state'] != 'qualified')


def validate_qualified_evidence(receipt: dict, output: Path, source: str) -> None:
    if receipt['sourceCommit'] != source or receipt['sourceBefore'] != receipt['sourceAfter'] or receipt['sourceAfter'] != source_snapshot():
        raise ValueError('Qualification source evidence differs')
    candidate = receipt['candidate']
    baseline = tomllib.loads(Path('rust-toolchain.toml').read_text())['toolchain']['channel']
    if select(candidate['channel'], (output / 'manifest.toml').read_bytes(), baseline) != candidate:
        raise ValueError('Frozen candidate evidence differs')
    commands = commands_for(candidate['toolchain'])
    if receipt['commands'] != commands or set(receipt['exitCodes']) != set(commands):
        raise ValueError('Native command evidence is incomplete')
    if any(type(value) is not int or value != 0 for value in receipt['exitCodes'].values()):
        raise ValueError('Native command exit evidence contradicts qualification')
    tools = receipt['tools']
    if set(tools) != {'rustc', 'rustdoc', 'cargo'} or any(not Path(path).is_absolute() or Path(path).name != name for name, path in tools.items()) or len({str(Path(path).parent) for path in tools.values()}) != 1:
        raise ValueError('Canonical tool binding evidence is incomplete')
    expected_logs = {stage + '.log' for stage in commands}
    if set(receipt['logSha256']) != expected_logs:
        raise ValueError('Native log evidence is incomplete')
    for name, expected in receipt['logSha256'].items():
        if digest((output / name).read_bytes()) != expected:
            raise ValueError('Native log digest differs')
    compiler_matches((output / 'binding.log').read_text(), candidate)
    if receipt['sourcePinSha256'] != {name: digest(Path(name).read_bytes()) for name in ('rust-toolchain.toml', 'Cargo.lock')}:
        raise ValueError('Canonical pin evidence differs')


def finalize(output: Path, outcome: str, source: str) -> int:
    path = output / 'qualification.json'
    invalid = False
    raw = None
    try:
        raw = path.read_bytes()
        receipt = json.loads(raw)
        if not isinstance(receipt, dict):
            raise ValueError('Receipt must be an object')
    except (OSError, ValueError, TypeError):
        if path.exists():
            path.with_suffix('.json.invalid').write_bytes(path.read_bytes())
        receipt = {'state': 'evidence-invalid', 'runtimeTestsExecuted': None}
        invalid = True
    if receipt.get('state') == 'qualified':
        if outcome != 'success' or receipt.get('sourceCommit') != source or receipt.get('sourcePinsUnchanged') is not True or receipt.get('runtimeTestsExecuted') is not True or not isinstance(receipt.get('stages'), dict) or set(receipt['stages']) != set(STAGES) or any(v != 'success' for v in receipt['stages'].values()):
            receipt['state'] = 'evidence-invalid'
            invalid = True
    if receipt.get('state') == 'qualified':
        try:
            validate_qualified_evidence(receipt, output, source)
        except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError):
            receipt['state'] = 'evidence-invalid'
            invalid = True
    if outcome == 'success' and receipt.get('state') != 'qualified':
        receipt['state'] = 'evidence-invalid'
        invalid = True
    if invalid:
        receipt['runtimeTestsExecuted'] = None
        if raw is not None:
            path.with_suffix('.json.invalid').write_bytes(raw)
    receipt.update(qualificationStepOutcome=outcome, jobSourceCommit=source, promoted=False, productionWrites=False)
    write(path, receipt)
    return int(invalid)


def finalize_discovery(output: Path, outcome: str, source: str) -> int:
    path = output / 'discovery.json'
    raw = None
    invalid = False
    try:
        raw = path.read_bytes()
        receipt = json.loads(raw)
        if not isinstance(receipt, dict):
            raise ValueError('Discovery receipt must be an object')
        if outcome == 'success' and (receipt.get('state') != 'discovered' or set(receipt.get('channels', {})) != set(CHANNELS)):
            raise ValueError('Successful discovery evidence is incomplete')
    except (OSError, ValueError, TypeError):
        invalid = raw is not None or outcome == 'success'
        receipt = {'state': 'evidence-invalid' if invalid else 'discovery-failed', 'candidates': [], 'channels': {}}
    if invalid and raw is not None:
        path.with_suffix('.json.invalid').write_bytes(raw)
    if outcome != 'success' and receipt.get('state') == 'discovered':
        receipt['state'] = 'discovery-failed'
    receipt.update(discoveryStepOutcome=outcome, jobSourceCommit=source, promoted=False)
    write(path, receipt)
    return int(invalid)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    discovery = sub.add_parser('discover')
    discovery.add_argument('--output', type=Path, required=True)
    discovery.add_argument('--github-output', type=Path, required=True)
    qualification = sub.add_parser('qualify')
    qualification.add_argument('--output', type=Path, required=True)
    for command in ('finalize', 'finalize-discovery'):
        terminal = sub.add_parser(command)
        terminal.add_argument('--output', type=Path, required=True)
        terminal.add_argument('--outcome', choices=['success', 'failure', 'cancelled', 'skipped'], required=True)
        terminal.add_argument('--source', required=True)
    args = parser.parse_args()
    if args.command == 'finalize':
        return finalize(args.output, args.outcome, args.source)
    if args.command == 'finalize-discovery':
        return finalize_discovery(args.output, args.outcome, args.source)
    baseline = tomllib.loads(Path('rust-toolchain.toml').read_text())['toolchain']['channel']
    if args.command == 'discover':
        return discover(args.output, args.github_output, baseline)
    return qualify(json.loads(os.environ['RUST_CANARY_CANDIDATE']), args.output, baseline)


if __name__ == '__main__':
    raise SystemExit(main())
