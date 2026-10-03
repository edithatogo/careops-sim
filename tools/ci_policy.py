"""Fail-closed full parent dependency, secret and workflow policy checks."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
PINS = json.loads((ROOT / 'tools/ci-tools.json').read_text())


def main():
    logs = ROOT / '.artifacts/ci/policy'
    logs.mkdir(parents=True, exist_ok=True)
    zizmor = ['zizmor'] if shutil.which('zizmor') else ['uv', 'tool', 'run', '--from', 'zizmor==' + PINS['zizmor'], 'zizmor']
    tools = [(['actionlint', '--version'], PINS['actionlint']['version']),
             (['gitleaks', 'version'], PINS['gitleaks']['version']),
             (['cargo-deny', '--version'], PINS['cargo-deny']),
             (zizmor + ['--version'], PINS['zizmor'])]
    for argv, version in tools:
        result = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True)
        print(result.stdout, end='')
        if result.returncode or version not in result.stdout.split():
            raise RuntimeError('tool version mismatch: ' + argv[0])
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1"], cwd=ROOT))
    for package in metadata["packages"]:
        for dep in package["dependencies"]:
            if dep["source"] is not None and dep["req"] == "*":
                raise RuntimeError("wildcard external dependency: " + dep["name"])
    commands = [(['actionlint', '.github/workflows/ci.yml'], 'actionlint'),
                (zizmor + ['--no-progress', '--offline', '--persona', 'auditor', '.github/workflows/ci.yml'], 'zizmor'),
                (['gitleaks', 'git', '.', '--redact', '--no-banner'], 'gitleaks'),
                (['cargo', 'deny', '--locked', '--workspace', '--all-features', '--config', 'deny.toml', 'check', 'advisories', 'bans', 'licenses', 'sources'], 'cargo-deny')]
    failures = []
    for argv, name in commands:
        with (logs / (name + '.log')).open('w') as log:
            result = subprocess.run(argv, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        print(name + ': exit ' + str(result.returncode), flush=True)
        if result.returncode:
            failures.append(name)
            print((logs / (name + '.log')).read_text()[-12000:])
    (logs / 'result.json').write_text(json.dumps({'failed': failures, 'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()}, indent=2) + '\n')
    return int(bool(failures))


if __name__ == '__main__':
    sys.exit(main())
