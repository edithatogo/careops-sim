#!/usr/bin/env python3
"""P5.2 smoke: actual native CLI inputs twice, stable typed-table equality."""
from __future__ import annotations
import hashlib
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tests/ed'))
from native_cli_support import invoke, read_run

EXAMPLES = [
    'crates/careops-ed/examples/one_patient.json',
    'crates/careops-ed/examples/nominal_mvp.json',
    'crates/careops-ed/examples/constrained_mvp.json',
    'model-inputs/ed/profiles/p4-minimal-deterministic.json',
    'model-inputs/ed/profiles/p4-generic-nominal.json',
    'model-inputs/ed/profiles/p4-constrained-surge.json',
]


def logical_provenance(manifest):
    # File hashes bind each run's bytes independently in read_run. Neither file
    # bytes nor physical part boundaries are a cross-run reproducibility claim.
    identity = {key: manifest[key] for key in [
        'schema', 'status', 'format', 'compression', 'batch_id', 'scenario_id',
        'time_unit', 'horizon_ticks', 'base_seed', 'replication_seeds',
        'effective_input_sha256', 'parsed_input_sha256', 'raw_input_sha256',
        'model_contract_sha256', 'runtime_revision', 'data_revision',
        'polars_revision', 'producer',
    ]}
    identity['tables'] = [{key: table[key] for key in ['name', 'schema', 'rows', 'columns']}
                          for table in manifest['tables']]
    return identity


def main():
    errors = []
    with tempfile.TemporaryDirectory(prefix='careops-p52-') as temporary:
        for index, relative in enumerate(EXAMPLES):
            try:
                runs = []
                for repeat in range(2):
                    root = Path(temporary) / f'{index}-{repeat}'
                    proc = invoke('run', ROOT / relative, '--output-dir', root)
                    if proc.returncode:
                        raise RuntimeError(proc.stderr.strip())
                    runs.append(read_run(root))
                if logical_provenance(runs[0][0]) != logical_provenance(runs[1][0]):
                    raise RuntimeError('non-repeatable native provenance')
                for name, table in runs[0][1].items():
                    if not table.equals(runs[1][1][name], check_metadata=False):
                        raise RuntimeError(f'non-repeatable typed table {name}')
                print(f'PASS {relative}: input={hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()}')
            except (RuntimeError, AssertionError) as error:
                errors.append(f'{relative}: {error}')
    for error in errors:
        print('ERROR: ' + error, file=sys.stderr)
    if errors:
        return 1
    print('P5.2 smoke PASS: six actual scenarios repeat with equal native data and provenance')
    return 0


if __name__ == '__main__':
    sys.exit(main())
