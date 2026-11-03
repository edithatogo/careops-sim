"""Independent PyArrow checks of explicit native R131 fixture publication.

Run r131_file_provenance::original_run_independent_reader_fixture with a fresh
CAREOPS_R131_READER_DESTINATION, then pass that directory here. This verifies
native parts; it is not a supported legacy JSON table reader.
"""
import hashlib
import json
from pathlib import Path
import sys
import unittest

import pyarrow as pa
import pyarrow.parquet as pq

ROOT = Path(sys.argv.pop(1)).resolve()


def read_parts(root):
    directory = root / 'capture'
    manifest = json.loads((directory / 'manifest.json').read_bytes())
    if manifest['status'] != 'complete' or manifest['schema'] != 'careops.ed.patient-result.v2':
        raise ValueError('unsupported or incomplete manifest')
    effective_bytes = (root / 'effective-input.json').read_bytes()
    effective = json.loads(effective_bytes)
    if effective['seed'] != 7 or effective['horizon_ticks'] != 8:
        raise ValueError('effective override mismatch')
    input_sha256 = hashlib.sha256(effective_bytes).hexdigest()
    if input_sha256 == hashlib.sha256((root / 'parsed-input.json').read_bytes()).hexdigest():
        raise ValueError('effective input incorrectly equals parsed baseline')
    frames = []
    for rep in manifest['replications']:
        sequence = 0
        for part in rep['parts']:
            name = part['name']
            if Path(name).name != name or '/' in name or '\\' in name:
                raise ValueError('uncontained part')
            path = directory / name
            if path.is_symlink() or not path.is_file():
                raise ValueError('nonregular part')
            data = path.read_bytes()
            if len(data) != part['bytes'] or hashlib.sha256(data).hexdigest() != part['sha256']:
                raise ValueError('part bytes/hash mismatch')
            file = pq.ParquetFile(pa.BufferReader(data))
            table = file.read()
            md = file.metadata.metadata or {}
            expected = {
                'schema': manifest['schema'], 'codec': manifest['codec'],
                'run_id': 'r131-independent', 'time_unit': 'tick',
                'effective_input_sha256': input_sha256,
                'input_hash_basis': 'normalized-effective-scenario',
                'producer': 'careops-ed/0.1.0',
                'replication_index': str(rep['summary']['replication_index']),
                'seed': str(rep['summary']['seed']), 'first_sequence': str(sequence),
                'rows': str(part['rows']), 'model_contract_sha256': rep['model_contract_sha256'],
                'polars_revision': '96439685eb2f95108f51087d2b76091917f2593c',
            }
            for key, value in expected.items():
                if md.get(('careops.' + key).encode()) != value.encode():
                    raise ValueError(f'file provenance mismatch: {key}')
            for key, column in [('effective_input_sha256', 'input_sha256'),
                                ('model_contract_sha256', 'model_contract_sha256'),
                                ('run_id', 'run_id'), ('time_unit', 'time_unit')]:
                if set(table[column].to_pylist()) != {md[('careops.' + key).encode()].decode()}:
                    raise ValueError(f'column provenance mismatch: {key}')
            if table.num_rows != part['rows'] or part['first_sequence'] != sequence:
                raise ValueError('row count/sequence mismatch')
            if table['sequence'].to_pylist() != list(range(sequence, sequence + table.num_rows)):
                raise ValueError('noncanonical sequence')
            for row_group in range(file.metadata.num_row_groups):
                group = file.metadata.row_group(row_group)
                if any(group.column(i).compression != 'ZSTD' for i in range(group.num_columns)):
                    raise ValueError('expected real Zstd')
            for tick in ['arrival_tick', 'start_tick', 'completion_tick', 'work_ticks', 'wait_ticks', 'elapsed_ticks']:
                hi, lo = table[tick + '_hi'], table[tick + '_lo']
                if hi.type != pa.uint64() or lo.type != pa.uint64():
                    raise ValueError('limb type')
                if hi.is_null().to_pylist() != lo.is_null().to_pylist():
                    raise ValueError('half-null limb')
                if any(v not in (None, 0) for v in hi.to_pylist()):
                    raise ValueError('unexpected ED source clock width')
            sequence += table.num_rows
            frames.append(table)
        if sequence != rep['summary']['arrivals']:
            raise ValueError('incomplete replication')
    if len(manifest['replications']) != 4:
        raise ValueError('incomplete fixture replications')
    return pa.concat_tables(frames)


class NativeFileReaderTests(unittest.TestCase):
    def test_actual_native_capture_and_manifest_refusal(self):
        table = read_parts(ROOT)
        self.assertEqual(table.num_rows, 20)
        self.assertGreater(table['completion_tick_lo'].null_count, 0)
        self.assertEqual(set(table['replication_index'].to_pylist()), {0, 1, 2, 3})
        manifest_path = ROOT / 'capture' / 'manifest.json'
        original = manifest_path.read_bytes()
        value = json.loads(original)
        try:
            value['status'] = 'incomplete'
            manifest_path.write_text(json.dumps(value))
            with self.assertRaisesRegex(ValueError, 'incomplete manifest'):
                read_parts(ROOT)
            manifest_path.write_bytes(original)
            value = json.loads(original)
            value['replications'][0]['parts'][0]['sha256'] = '0' * 64
            manifest_path.write_text(json.dumps(value))
            with self.assertRaisesRegex(ValueError, 'hash mismatch'):
                read_parts(ROOT)
        finally:
            manifest_path.write_bytes(original)


if __name__ == '__main__':
    unittest.main()
