"""Qualification reader for explicit disposable r131 native-file fixtures.

Consumes native Arrow/Polars tables directly. This is a test fixture verifier,
not a production legacy JSON table reader. Manifest mutations are confined to
its explicitly supplied owned fixture and restored after each negative control.
"""
import hashlib
import io
import json
import math
from pathlib import Path
import sys
import tempfile
import re
import unittest

import polars as pl
from polars.testing import assert_frame_equal
import pyarrow as pa
import pyarrow.csv as pcsv
import pyarrow.parquet as pq

ROOT = Path(sys.argv.pop(1)).resolve()
DTYPES = {'UInt64': (pa.uint64(), pl.UInt64), 'UInt32': (pa.uint32(), pl.UInt32),
          'Float64': (pa.float64(), pl.Float64), 'Boolean': (pa.bool_(), pl.Boolean),
          'String': (pa.large_string(), pl.String)}
TABLES = {'patients': ('careops.ed.patients.v2', 20),
          'calibration': ('careops.ed.synthetic-fixture-calibration.v2', 2),
          'physical': ('careops.tests.physical-edges.v2', 7),
          'empty': ('careops.tests.physical-edges.v2', 0),
          'multiple_batches': ('careops.tests.multiple-batches.v2', 524_288)}
LIMIT = 4 * 1024 * 1024


def load_fixture(root):
    manifest = json.loads((root / 'manifest.json').read_bytes())
    if manifest['schema'] != 'careops.tests.native-files.v2' or manifest['status'] != 'complete':
        raise ValueError('unsupported or incomplete manifest')
    if manifest['raw_input_sha256'] is not None:
        raise ValueError('fixture parsed API has no raw-input provenance')
    effective = (root / 'effective-input.json').read_bytes()
    parsed = (root / 'parsed-input.json').read_bytes()
    if hashlib.sha256(effective).hexdigest() != manifest['effective_input_sha256']:
        raise ValueError('effective input hash mismatch')
    if hashlib.sha256(parsed).hexdigest() != manifest['parsed_input_sha256'] or effective == parsed:
        raise ValueError('parsed input hash/basis mismatch')
    effective_value = json.loads(effective)
    if effective_value['seed'] != 7 or effective_value['horizon_ticks'] != 8:
        raise ValueError('effective override mismatch')
    if len(manifest['tables']) != len(TABLES) or {t['name'] for t in manifest['tables']} != set(TABLES):
        raise ValueError('missing/duplicate table declaration')
    model = manifest.get('model_contract_sha256')
    if not isinstance(model, str) or not re.fullmatch('[0-9a-f]{64}', model):
        raise ValueError('missing captured model identity')
    kinds = {'patients': 'original_run_capture', 'calibration': 'caller_supplied_synthetic_calibration',
             'physical': 'synthetic_physical_edge_vector', 'empty': 'typed_empty_projection',
             'multiple_batches': 'synthetic_multi_batch_row_vector'}
    result = {}
    for table in manifest['tables']:
        name = table['name']
        if (table['schema'], table['rows']) != TABLES[name] or type(table['rows']) is not int:
            raise ValueError('table schema/count mismatch')
        required = {'careops.table_schema': TABLES[name][0], 'careops.time_unit': 'tick',
                    'careops.fixture_kind': kinds[name],
                    'careops.polars_revision': '96439685eb2f95108f51087d2b76091917f2593c',
                    'careops.data_library_revision': 'f34750533c07e9e868481606e5fe5c89ec9b76be',
                    'careops.runtime_revision': '07410266bf54b6f73c59db2f8cbfbc5697c53734',
                    'careops.producer': 'careops-ed/0.1.0'}
        if name == 'patients':
            required.update({'careops.run_id': 'r131-domain-run',
                             'careops.effective_input_sha256': manifest['effective_input_sha256'],
                             'careops.parsed_input_sha256': manifest['parsed_input_sha256'],
                             'careops.input_hash_basis': 'normalized-effective-scenario',
                             'careops.model_contract_sha256': model})
        elif name == 'calibration':
            required.update({'careops.fixture_id': 'r131-fixture', 'careops.source_kind': 'synthetic_fixture'})
        if table['kind'] != kinds[name] or table['metadata'] != required:
            raise ValueError('missing or inconsistent required provenance declaration')
        columns = table['columns']
        if len({c[0] for c in columns}) != len(columns) or any(c[1] not in DTYPES for c in columns):
            raise ValueError('unsupported column contract')
        names = [c[0] for c in columns]
        arrow_schema = pa.schema([(n, DTYPES[t][0]) for n, t in columns])
        polars_schema = {n: DTYPES[t][1] for n, t in columns}
        if len(table['files']) != 3 or {f['format'] for f in table['files']} != {'parquet', 'ipc', 'csv'}:
            raise ValueError('missing/duplicate native format')
        normalized = []
        for file in table['files']:
            path = root / file['path']
            if Path(file['path']).name != file['path'] or path.is_symlink() or not path.is_file():
                raise ValueError('uncontained or nonregular native file')
            if type(file['bytes']) is not int or not 0 < file['bytes'] < LIMIT:
                raise ValueError('encoded fixture bound')
            with path.open('rb') as source:
                data = source.read(LIMIT + 1)
            if len(data) != file['bytes'] or hashlib.sha256(data).hexdigest() != file['sha256']:
                raise ValueError('file hash/byte mismatch')
            kind = file['format']
            if kind == 'parquet':
                native_file = pq.ParquetFile(pa.BufferReader(data))
                if native_file.metadata.num_rows != table['rows']:
                    raise ValueError('Parquet row declaration')
                metadata = native_file.metadata.metadata
                arrow = native_file.read()
                for group_index in range(native_file.metadata.num_row_groups):
                    group = native_file.metadata.row_group(group_index)
                    if any(group.column(i).compression != 'ZSTD' for i in range(group.num_columns)):
                        raise ValueError('expected real Zstd')
                frame = pl.read_parquet(io.BytesIO(data))
            elif kind == 'ipc':
                stream = pa.ipc.open_stream(pa.BufferReader(data))
                metadata = stream.schema.metadata
                batches = list(stream)
                if name == 'multiple_batches' and [b.num_rows for b in batches] != [262_144, 262_144]:
                    raise ValueError('actual IPC multiple-batch contract')
                arrow = pa.Table.from_batches(batches, schema=stream.schema)
                frame = pl.read_ipc_stream(io.BytesIO(data))
            else:
                metadata = None  # CSV uses its typed manifest as metadata carrier.
                arrow = pcsv.read_csv(pa.BufferReader(data), convert_options=pcsv.ConvertOptions(
                    column_types={f.name: f.type for f in arrow_schema}, null_values=[''],
                    strings_can_be_null=True, quoted_strings_can_be_null=False),
                    parse_options=pcsv.ParseOptions(newlines_in_values=True))
                frame = pl.read_csv(io.BytesIO(data), schema_overrides=polars_schema,
                                    empty_string_is_null=True)
            if kind != 'csv':
                for key, value in required.items():
                    if metadata.get(key.encode()) != value.encode():
                        raise ValueError('file provenance mismatch')
            if arrow.column_names != names or arrow.num_rows != table['rows']:
                raise ValueError('native shape mismatch')
            for field, (_name, dtype) in zip(arrow.schema, columns):
                # Pinned Parquet writes StringView; IPC uses LargeString.
                # Normalize only that documented string layout, not numerics.
                if dtype == 'String':
                    if not (pa.types.is_string_view(field.type) or pa.types.is_large_string(field.type)):
                        raise ValueError('unsupported native string layout')
                elif field.type != DTYPES[dtype][0]:
                    raise ValueError('numeric/Boolean type mismatch')
            arrow = arrow.cast(arrow_schema)
            assert_frame_equal(frame, pl.from_arrow(arrow), check_row_order=True, check_column_order=True, check_exact=True)
            normalized.append(frame)
        for other in normalized[1:]:
            assert_frame_equal(normalized[0], other, check_row_order=True, check_column_order=True, check_exact=True)
        result[name] = normalized[0]
    patients = result['patients']
    if patients['input_sha256'].unique().to_list() != [manifest['effective_input_sha256']]:
        raise ValueError('patient source column provenance')
    if patients['model_contract_sha256'].unique().to_list() != [model] or patients['run_id'].unique().to_list() != ['r131-domain-run']:
        raise ValueError('patient model/run column provenance')
    if patients['replication_index'].unique().sort().to_list() != [0, 1, 2, 3]:
        raise ValueError('patient replication grain')
    if patients['schema'].unique().to_list() != ['careops.ed.patients.v2']:
        raise ValueError('patient schema column')
    return result


class IndependentNativeFileTests(unittest.TestCase):
    def test_actual_native_formats_and_explicit_edge_vectors(self):
        tables = load_fixture(ROOT)
        edge = tables['physical']
        self.assertEqual(edge['entity_id_index'][0], 0)
        self.assertEqual(edge['entity_id_gen'][0], 2**32-1)
        self.assertIsNone(edge['entity_id_index'][2])
        self.assertIsNone(edge['entity_id_gen'][2])
        self.assertEqual((edge['tick_hi'][0], edge['tick_lo'][0]), (2**64-1, 2**64-1))
        self.assertEqual(edge['tick_lo'][1], 2**53+1)
        self.assertEqual(edge['text'].to_list(), ['comma,"quote\r\nline', '', '\\N', None, '雪', 'NA', 'null'])
        self.assertIsNone(edge['estimate'][1])
        self.assertLess(math.copysign(1.0, edge['estimate'][2]), 0)
        self.assertTrue(math.isnan(edge['estimate'][4]))
        self.assertEqual(edge['estimate'][5], math.inf)
        self.assertEqual(edge['estimate'][6], -math.inf)
        self.assertEqual(tables['calibration']['case_id'][0], 'case,"a\r\nline')
        self.assertEqual(tables['calibration']['start_tick_lo'].null_count(), 1)
        self.assertEqual(tables['patients']['completion_tick_lo'].null_count(), 4)
        multiple = tables['multiple_batches']['row_index']
        self.assertEqual((multiple[0], multiple[-1]), (0, 524_287))
        self.assertEqual(multiple.cast(pl.UInt64).sum(), 524_288 * 524_287 // 2)
        self.assertEqual(tables['empty'].height, 0)

    def test_incomplete_unknown_corrupt_and_effective_input_tampering_refuse(self):
        path = ROOT / 'manifest.json'
        original = path.read_bytes()
        try:
            value = json.loads(original)
            value['status'] = 'incomplete'
            path.write_text(json.dumps(value))
            with self.assertRaisesRegex(ValueError, 'incomplete manifest'):
                load_fixture(ROOT)
            value = json.loads(original)
            value['schema'] = 'careops.tests.native-files.v1'
            path.write_text(json.dumps(value))
            with self.assertRaisesRegex(ValueError, 'unsupported'):
                load_fixture(ROOT)
            value = json.loads(original)
            value['effective_input_sha256'] = '0' * 64
            path.write_text(json.dumps(value))
            with self.assertRaisesRegex(ValueError, 'effective input hash'):
                load_fixture(ROOT)
            value = json.loads(original)
            value['tables'][0]['metadata'].pop('careops.model_contract_sha256')
            path.write_text(json.dumps(value))
            with self.assertRaisesRegex(ValueError, 'required provenance declaration'):
                load_fixture(ROOT)
            value = json.loads(original)
            value['tables'][2]['metadata'] = {}
            path.write_text(json.dumps(value))
            with self.assertRaisesRegex(ValueError, 'required provenance declaration'):
                load_fixture(ROOT)
            value = json.loads(original)
            value['tables'][0]['files'][0]['sha256'] = '0' * 64
            path.write_text(json.dumps(value))
            with self.assertRaisesRegex(ValueError, 'file hash/byte mismatch'):
                load_fixture(ROOT)
            path.write_bytes(original)
            fd, filename = tempfile.mkstemp(prefix='.negative-source-', suffix='.parquet', dir=ROOT)
            import os
            os.close(fd)
            bad_file = Path(filename)
            try:
                value = json.loads(original)
                part = value['tables'][0]['files'][0]
                native = pq.read_table(ROOT / part['path']).replace_schema_metadata(None)
                pq.write_table(native, bad_file, compression='zstd')
                data = bad_file.read_bytes()
                part.update(path=bad_file.name, bytes=len(data), sha256=hashlib.sha256(data).hexdigest())
                path.write_text(json.dumps(value))
                with self.assertRaisesRegex(ValueError, 'file provenance mismatch'):
                    load_fixture(ROOT)
                value['tables'][0]['metadata'].pop('careops.model_contract_sha256')
                path.write_text(json.dumps(value))
                with self.assertRaisesRegex(ValueError, 'required provenance declaration'):
                    load_fixture(ROOT)
            finally:
                path.write_bytes(original)
                bad_file.unlink()
        finally:
            path.write_bytes(original)


if __name__ == '__main__':
    unittest.main()
