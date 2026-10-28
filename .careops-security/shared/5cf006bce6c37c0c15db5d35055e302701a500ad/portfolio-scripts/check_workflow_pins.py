"""Refuse floating remote actions/workflows without replacing module tests."""
import argparse
import re
from pathlib import Path

import yaml


def check(root: Path) -> list[str]:
    problems = []
    for path in sorted((root / '.github/workflows').glob('*')):
        if path.suffix not in ['.yml', '.yaml']:
            continue
        data = yaml.safe_load(path.read_text())
        def walk(node):
            if isinstance(node, dict):
                for key, value in node.items():
                    if key == 'uses' and isinstance(value, str):
                        local = value.startswith(('./', '$/'))
                        remote = re.fullmatch(r'[^\s@]+@[a-f0-9]{40}', value)
                        docker = re.fullmatch(r'docker://[^\s@]+@sha256:[a-f0-9]{64}', value)
                        if not (local or remote or docker):
                            problems.append(str(path.relative_to(root)) + ': floating or invalid uses reference')
                    else:
                        walk(value)
            elif isinstance(node, list):
                for value in node:
                    walk(value)
        walk(data)
    return problems


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path('.'))
    args = parser.parse_args()
    errors = check(args.root)
    if errors:
        raise SystemExit('\n'.join(errors))
    print('All remote action/workflow references use immutable commits or image digests.')
