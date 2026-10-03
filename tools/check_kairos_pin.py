"""Reject unreviewed parent gitlinks even when upstream native tests are green."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args], text=True).strip()


def check(root):
    errors = []
    try:
        contract = json.loads((root / 'conductor/evidence/d2.4-kairos-contract.json').read_text())
        expected = contract['kairos_commit']
        if len(expected) != 40 or any(c not in '0123456789abcdef' for c in expected):
            raise ValueError('invalid reviewed commit')
        owner = contract['owner_ci']
        if owner['head_sha'] != expected or owner['conclusion'] != 'success' or set(owner['passed_hosts']) != {'x86_64-unknown-linux-gnu', 'aarch64-apple-darwin'}:
            raise ValueError('missing exact-commit native owner acceptance')
        state = json.loads((root / 'conductor/current-state.json').read_text())
        if state['submodule_pins']['libs/kairos'] != expected:
            errors.append('expected metadata differs from reviewed compatibility commit')
        tree = git(root, 'ls-tree', 'HEAD', '--', 'libs/kairos').split()
        index = git(root, 'ls-files', '--stage', '--', 'libs/kairos').split()
        for label, entry in [('committed gitlink', tree), ('index gitlink', index)]:
            if not entry or entry[0] != '160000' or (entry[2] if label == 'committed gitlink' else entry[1]) != expected:
                errors.append(label + ' differs from reviewed compatibility commit')
        kairos = root / 'libs/kairos'
        if Path(git(kairos, 'rev-parse', '--show-toplevel')).resolve() != kairos.resolve():
            errors.append('Kairos is not initialized as its own repository')
        if git(kairos, 'rev-parse', 'HEAD') != expected:
            errors.append('checked-out Kairos differs from reviewed compatibility commit')
        if git(kairos, 'status', '--porcelain', '--untracked-files=no'):
            errors.append('Kairos has uncommitted tracked changes')
        for path, digest in contract['source_sha256'].items():
            if hashlib.sha256((kairos / path).read_bytes()).hexdigest() != digest:
                errors.append('reviewed source drift: ' + path)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        errors.append('missing/invalid integration evidence: ' + str(error))
    return errors


if __name__ == '__main__':
    errors = check(ROOT)
    print(json.dumps({'errors': errors}, indent=2))
    sys.exit(bool(errors))
