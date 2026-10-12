"""Fail-closed full parent dependency, secret and workflow policy checks."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
PINS = json.loads((ROOT / 'tools/ci-tools.json').read_text())


def validate_rust_toolchain(environ=None):
    """Return canonical Cargo only when every active Rust binding is 1.99.0."""
    environ = os.environ if environ is None else environ
    expected_toolchain = '1.99.0'
    if environ.get('RUSTUP_TOOLCHAIN') != expected_toolchain:
        raise RuntimeError('RUSTUP_TOOLCHAIN must be exactly 1.99.0')
    rustc = environ.get('RUSTC', '')
    rustdoc = environ.get('RUSTDOC', '')
    if not rustc or not rustdoc or not Path(rustc).is_absolute() or not Path(rustdoc).is_absolute():
        raise RuntimeError('RUSTC and RUSTDOC must be absolute canonical tool paths')
    bin_dir = Path(rustc).parent
    if Path(rustdoc).parent != bin_dir:
        raise RuntimeError('RUSTC and RUSTDOC must come from one toolchain bin directory')
    expected_paths = {'rustc': str(Path(rustc)), 'rustdoc': str(Path(rustdoc)), 'cargo': str(bin_dir / 'cargo')}
    for name, expected_path in expected_paths.items():
        resolved = shutil.which(name, path=environ.get('PATH'))
        if resolved is None or os.path.realpath(resolved) != os.path.realpath(expected_path):
            raise RuntimeError(f'PATH-resolved {name} does not match canonical Rust 1.99.0 toolchain')
        result = subprocess.run([expected_path, '--version'], capture_output=True, text=True)
        if result.returncode or not result.stdout.startswith(f'{name} {expected_toolchain} '):
            raise RuntimeError(f'{name} executable is not Rust 1.99.0')
    return expected_paths['cargo']


def validate_workflow_toolchain(workflow=None):
    """Reject Rust selectors outside the sole 1.99.0 lane and missing bindings."""
    workflow = (ROOT / '.github/workflows/ci.yml').read_text() if workflow is None else workflow
    violations = []
    for pattern, description in (
        (r'\bcargo\s+\+\S+', 'cargo +toolchain overrides are forbidden'),
        (r'rustup toolchain install (?!1\.99\.0\b)[^\s]+', 'only Rust 1.99.0 may be installed'),
        (r'--toolchain\s+(?!1\.99\.0\b)[^\s]+', 'only Rust 1.99.0 toolchain selectors are allowed'),
        (r'(?m)^.*\bRUSTUP_TOOLCHAIN(?:=|:\s*)(?!1\.99\.0\b)[^\s]+', 'only RUSTUP_TOOLCHAIN=1.99.0 is allowed'),
        (r'\b(?:cargo|rustc|rustdoc)\s+(?:\+)?(?:stable|beta|nightly|default)(?:\b|[-.]\d)', 'toolchain aliases are forbidden'),
        (r'\b(?:cargo|rustc|rustdoc)\s+\+(?:[0-9]+\.[0-9]+(?:\.[0-9]+)?)(?![^\s]*)', 'explicit cargo overrides are forbidden'),
        (r'\bcargo\s+fuzz\b|\bcargo\s+miri\b|--sanitizer\s+address', 'nightly-only runtime gates are unavailable'),
    ):
        if re.search(pattern, workflow):
            violations.append(description)
    jobs = re.split(r'(?m)^  ([A-Za-z0-9_-]+):\n', workflow)
    for index in range(1, len(jobs), 2):
        name, body = jobs[index], jobs[index + 1]
        if re.search(r'\bcargo\b|\brustc\b|\brustdoc\b', body):
            for required in ('RUSTUP_TOOLCHAIN=1.99.0', 'RUSTC', 'RUSTDOC', 'command -v cargo'):
                if required not in body:
                    violations.append(f'{name} job is missing canonical tool binding: {required}')
    if 'cargo test --workspace --lib --bins --tests --locked' not in workflow:
        violations.append('required default-feature workspace test lane is missing')
    if 'cargo deny --manifest-path fuzz/Cargo.toml --config fuzz/deny.toml --locked check' not in workflow:
        violations.append('locked static fuzz dependency/license audit is missing')
    if violations:
        raise RuntimeError('; '.join(violations))


def main():
    cargo = validate_rust_toolchain()
    validate_workflow_toolchain()
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
    metadata = json.loads(subprocess.check_output([cargo, "metadata", "--locked", "--format-version", "1"], cwd=ROOT))
    for package in metadata["packages"]:
        for dep in package["dependencies"]:
            if dep["source"] is not None and dep["req"] == "*":
                raise RuntimeError("wildcard external dependency: " + dep["name"])
    commands = [(['actionlint', '.github/workflows/ci.yml'], 'actionlint'),
                (zizmor + ['--no-progress', '--offline', '--persona', 'auditor', '.github/workflows/ci.yml'], 'zizmor'),
                (['gitleaks', 'git', '.', '--redact', '--no-banner'], 'gitleaks'),
                ([cargo, 'deny', '--locked', '--workspace', '--all-features', '--config', 'deny.toml', 'check', 'advisories', 'bans', 'licenses', 'sources'], 'cargo-deny')]
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
