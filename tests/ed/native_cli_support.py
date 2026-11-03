"""Independent native CLI consumer utilities; no build or legacy JSON fallback."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]
NOMINAL = ROOT / 'crates/careops-ed/examples/nominal_mvp.json'
CONSTRAINED = ROOT / 'crates/careops-ed/examples/constrained_mvp.json'
MALFORMED = ROOT / 'crates/careops-ed/examples/malformed_mvp.json'


def binary_path():
    selected = os.environ.get('CAREOPS_ED_BINARY')
    if selected:
        binary = Path(selected).expanduser().resolve()
    else:
        target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
        if not target.is_absolute():
            target = ROOT / target
        binary = target / os.environ.get('CAREOPS_ED_PROFILE', 'debug') / ('careops-ed.exe' if os.name == 'nt' else 'careops-ed')
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise RuntimeError(f'Build the actual careops-ed-cli first or select CAREOPS_ED_BINARY: {binary}')
    return binary


def invoke(*args):
    return subprocess.run([str(binary_path()), *map(str, args)], cwd=ROOT,
                          env={**os.environ, 'POLARS_MAX_THREADS': '1'},
                          capture_output=True, text=True, timeout=60)


def control(path):
    return json.loads(Path(path).read_bytes())


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read_dataset(root):
    """Read real native files with independent PyArrow, retaining explicit types."""
    import pyarrow as pa
    import pyarrow.csv as pcsv
    import pyarrow.parquet as pq
    root = Path(root)
    manifest = control(root / 'manifest.json')
    if manifest['schema'] != 'careops.ed.native-output.v1' or manifest['status'] != 'complete':
        raise AssertionError('native dataset is incomplete')
    types = {'String': pa.large_string(), 'UInt64': pa.uint64(),
             'UInt32': pa.uint32(), 'Float64': pa.float64()}
    tables = {}
    for table in manifest['tables']:
        chunks = []
        cursor = 0
        for part in table['parts']:
            path = root / part['file']
            if part['first_row'] != cursor or sha(path) != part['sha256'] or path.stat().st_size != part['bytes']:
                raise AssertionError('native part binding differs')
            if manifest['format'] == 'parquet':
                chunk = pq.read_table(path)
            elif manifest['format'] == 'ipc':
                with path.open('rb') as stream:
                    chunk = pa.ipc.open_stream(stream).read_all()
            elif manifest['format'] == 'csv':
                chunk = pcsv.read_csv(path, parse_options=pcsv.ParseOptions(newlines_in_values=True), convert_options=pcsv.ConvertOptions(
                    column_types={c['name']: types[c['physical_type']] for c in table['columns']},
                    strings_can_be_null=True, quoted_strings_can_be_null=False, null_values=['']))
            else:
                raise AssertionError('unknown native format')
            if chunk.column_names != [c['name'] for c in table['columns']] or chunk.num_rows != part['rows']:
                raise AssertionError('native physical rows/columns differ')
            for field, expected in zip(chunk.schema, table['columns']):
                if expected['physical_type'] == 'String':
                    valid = pa.types.is_string(field.type) or pa.types.is_large_string(field.type) or pa.types.is_string_view(field.type)
                else:
                    valid = field.type == types[expected['physical_type']]
                if not valid or (not expected['nullable'] and chunk[field.name].null_count):
                    raise AssertionError(f'native physical type/null contract differs: {field.name}')
            cursor += chunk.num_rows
            chunks.append(chunk)
        if cursor != table['rows']:
            raise AssertionError('native table coverage differs')
        tables[table['name']] = pa.concat_tables(chunks)
    if set(tables) != {'patients', 'replications', 'metrics'}:
        raise AssertionError('unknown native table set')
    return manifest, tables


def read_run(root):
    root = Path(root)
    complete = control(root / 'manifest.json')
    if complete['schema'] != 'careops.ed.cli-artifacts.v1' or complete['status'] != 'complete':
        raise AssertionError('CLI artifacts incomplete')
    for field, hash_field in [('invocation_file', 'invocation_sha256'),
                              ('batch_file', 'batch_sha256'),
                              ('dataset_manifest', 'dataset_manifest_sha256')]:
        if sha(root / complete[field]) != complete[hash_field]:
            raise AssertionError('CLI completion checksum differs')
    return read_dataset(root / 'data')


def metric_rows(tables):
    # Test-only assertions over already typed native rows; never production output.
    return {row['name']: row for row in tables['metrics'].to_pylist()}


def stable_rows(tables):
    return {name: table.to_pylist() for name, table in tables.items()}


def prepare_capture_only(root, archive):
    """Model capture-complete/export-not-started using actual captured state.

    This is not a process-kill test. Preserve published outputs in a sibling
    directory; the retained R113 Rust lane proves process-crash recovery.
    """
    root, archive = Path(root), Path(archive)
    archive.mkdir()
    captured = {p.name: sha(p) for p in (root / 'capture').glob('rep-*-seq-*.part')}
    if not captured:
        raise AssertionError('actual capture parts required')
    invocation_hash = sha(root / 'invocation.json')
    for name in ['manifest.json', 'data', 'batch_run.json']:
        shutil.move(root / name, archive / name)
    return captured, invocation_hash


class NativeCliCase(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        binary_path()
        import pyarrow as pa
        if pa.__version__ != '26.0.0':
            raise RuntimeError('Qualified PyArrow26.0.0 required')

    def run_ok(self, root, scenario=NOMINAL, command='run', *options):
        proc = invoke(command, scenario, '--output-dir', root, *options)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertEqual(json.loads(proc.stdout), control(Path(root) / 'manifest.json'))
        return read_run(root)
