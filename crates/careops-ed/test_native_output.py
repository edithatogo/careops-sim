"""Independent fixture qualification of actual production ED native output.

Run only against an explicitly generated disposable r132 fixture. Standard
PyArrow and Polars receive typed native columns; no JSON row reconstruction.
"""
import copy
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

import polars as pl
from polars.testing import assert_frame_equal
import pyarrow as pa
import pyarrow.csv as pcsv
import pyarrow.parquet as pq

# Closed profile: production fixtures default to the new accepted data component;
# retained historical fixture verification explicitly selects its original revision.
DATA_REVISIONS = frozenset({'473ff2c0e640a7e457f500593f6ea1f06602ab35', 'f34750533c07e9e868481606e5fe5c89ec9b76be'})
EXPECTED_DATA_REVISION = os.environ.get('CAREOPS_NATIVE_EXPECTED_DATA_REVISION', '473ff2c0e640a7e457f500593f6ea1f06602ab35')
if EXPECTED_DATA_REVISION not in DATA_REVISIONS:
    raise ValueError('unsupported native fixture data revision profile')

ROOT = Path(sys.argv.pop(1)).resolve()
TYPES = {'String': (pa.large_string(), pl.String), 'UInt64': (pa.uint64(), pl.UInt64),
         'UInt32': (pa.uint32(), pl.UInt32), 'Float64': (pa.float64(), pl.Float64)}
PATIENT_NAMES = ('time_unit schema capture_schema run_id input_sha256 model_contract_sha256 '
                 'replication_index seed sequence patient_id status arrival_tick_hi arrival_tick_lo '
                 'start_tick_hi start_tick_lo completion_tick_hi completion_tick_lo work_ticks_hi '
                 'work_ticks_lo wait_ticks_hi wait_ticks_lo elapsed_ticks_hi elapsed_ticks_lo '
                 'scenario_id resource_id location_id capture_table_schema').split()
PATIENT_TEXT = set('time_unit schema capture_schema run_id input_sha256 model_contract_sha256 '
                   'patient_id status scenario_id resource_id location_id capture_table_schema'.split())
PATIENT_NULL = set('start_tick_hi start_tick_lo completion_tick_hi completion_tick_lo '
                   'wait_ticks_hi wait_ticks_lo elapsed_ticks_hi elapsed_ticks_lo'.split())
REP_NAMES = 'run_id replication_index seed horizon_ticks arrivals started completed unfinished mean_wait_ticks'.split()
METRIC_NAMES = ('name value unit denominator_name denominator_value numerator_count denominator_count '
                'estimand sample_unit weighting eligible_replications included_replications total_replications '
                'estimate variance standard_error ci_95_lower ci_95_upper uncertainty_replications ddof '
                'method status censored_count unfinished_count').split()
METRIC_TEXT = set('name unit denominator_name estimand sample_unit weighting method status'.split())
METRIC_FLOAT = set('value denominator_value estimate variance standard_error ci_95_lower ci_95_upper'.split())
METRIC_NULL = METRIC_FLOAT | {'numerator_count', 'denominator_count', 'ddof'}
CONTRACTS = {
    'patients': ('careops.ed.patients.v2', [(n, 'String' if n in PATIENT_TEXT else 'UInt64', n in PATIENT_NULL) for n in PATIENT_NAMES]),
    'replications': ('careops.ed.replications.v1', [(n, 'String' if n == 'run_id' else 'Float64' if n == 'mean_wait_ticks' else 'UInt64', n == 'mean_wait_ticks') for n in REP_NAMES]),
    'metrics': ('careops.ed.metrics.v1', [(n, 'String' if n in METRIC_TEXT else 'Float64' if n in METRIC_FLOAT else 'UInt32' if n == 'ddof' else 'UInt64', n in METRIC_NULL) for n in METRIC_NAMES]),
}
CEILINGS = dict(max_total_rows=1_000_000, max_parts=4096, max_rows_per_part=256,
                max_bytes_per_part=4*1024*1024, max_manifest_bytes=1024*1024)
MANIFEST_KEYS = set(('schema status format compression batch_id scenario_id time_unit horizon_ticks '
                     'base_seed replication_seeds effective_input_sha256 parsed_input_sha256 raw_input_sha256 '
                     'model_contract_sha256 runtime_revision data_revision polars_revision producer limits tables').split())


def object_without_duplicates(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError('duplicate control key')
        value[key] = item
    return value


def read_controls(path):
    if path.is_symlink() or not path.is_file() or path.stat().st_size > 1024*1024:
        raise ValueError('missing or oversized native completion control')
    return json.loads(path.read_bytes(), object_pairs_hook=object_without_duplicates,
                      parse_constant=lambda v: (_ for _ in ()).throw(ValueError(v)))


def required_metadata(manifest, table, part):
    return {f'careops.{k}': str(v) for k, v in {
        'schema': table['schema'], 'output_schema': 'careops.ed.native-output.v1',
        'codec': {'parquet': 'parquet-zstd', 'ipc': 'arrow-ipc-stream', 'csv': 'csv-necessary-quotes-empty-null'}[manifest['format']],
        'table': table['name'], 'run_id': manifest['batch_id'], 'scenario_id': manifest['scenario_id'],
        'time_unit': manifest['time_unit'], 'first_row': part['first_row'], 'rows': part['rows'],
        'effective_input_sha256': manifest['effective_input_sha256'],
        'parsed_input_sha256': manifest['parsed_input_sha256'], 'model_contract_sha256': manifest['model_contract_sha256'],
        'runtime_revision': manifest['runtime_revision'], 'data_revision': manifest['data_revision'],
        'polars_revision': manifest['polars_revision'], 'producer': manifest['producer'],
    }.items()}


def read_dataset(root):
    manifest = read_controls(root / 'manifest.json')
    if (set(manifest) != MANIFEST_KEYS or manifest['schema'] != 'careops.ed.native-output.v1'
            or manifest['status'] != 'complete' or manifest['format'] not in {'parquet', 'ipc', 'csv'}
            or manifest['compression'] != ('zstd' if manifest['format'] == 'parquet' else None)
            or manifest['raw_input_sha256'] is not None
            or manifest['data_revision'] != EXPECTED_DATA_REVISION
            or manifest['runtime_revision'] != '07410266bf54b6f73c59db2f8cbfbc5697c53734'
            or manifest['polars_revision'] != '96439685eb2f95108f51087d2b76091917f2593c'
            or manifest['producer'] != 'careops-ed/0.1.0'):
        raise ValueError('unknown or incomplete native output contract')
    for name in ['effective_input_sha256', 'parsed_input_sha256', 'model_contract_sha256']:
        if not isinstance(manifest[name], str) or len(manifest[name]) != 64 or any(c not in '0123456789abcdef' for c in manifest[name]):
            raise ValueError('invalid provenance digest')
    limits = manifest['limits']
    if set(limits) != set(CEILINGS) or any(type(limits[k]) is not int or not 0 < limits[k] <= v for k, v in CEILINGS.items()):
        raise ValueError('unqualified export budgets')
    if (root / 'manifest.json').stat().st_size > limits['max_manifest_bytes']:
        raise ValueError('declared manifest byte budget')
    if [t['name'] for t in manifest['tables']] != list(CONTRACTS):
        raise ValueError('missing or unknown native table')
    result = {}; total_rows = total_parts = 0
    for table in manifest['tables']:
        schema, contract = CONTRACTS[table['name']]
        declared = [dict(name=n, physical_type=t, nullable=z) for n, t, z in contract]
        if (set(table) != {'name', 'schema', 'rows', 'columns', 'parts'} or table['schema'] != schema
                or table['columns'] != declared or type(table['rows']) is not int or table['rows'] < 0 or not table['parts']):
            raise ValueError('native table column contract')
        arrow_schema = pa.schema([(n, TYPES[t][0]) for n, t, _ in contract])
        polars_schema = {n: TYPES[t][1] for n, t, _ in contract}
        frames = []; next_row = 0
        for index, part in enumerate(table['parts']):
            extension = {'parquet': 'parquet', 'ipc': 'arrow', 'csv': 'csv'}[manifest['format']]
            if (set(part) != {'file', 'first_row', 'rows', 'bytes', 'sha256'}
                    or part['file'] != f"{table['name']}-{index:05d}.{extension}"
                    or any(type(part[k]) is not int for k in ['first_row', 'rows', 'bytes'])
                    or part['first_row'] != next_row or not 0 <= part['rows'] <= limits['max_rows_per_part']
                    or (part['rows'] == 0 and table['rows'] != 0)
                    or not 0 < part['bytes'] <= limits['max_bytes_per_part']):
                raise ValueError('invalid part declaration or row sequence')
            path = root / part['file']
            if path.is_symlink() or not path.is_file():
                raise ValueError('nonregular native part')
            with path.open('rb') as source:
                data = source.read(limits['max_bytes_per_part'] + 1)
            if len(data) != part['bytes'] or hashlib.sha256(data).hexdigest() != part['sha256']:
                raise ValueError('native part hash or bytes')
            metadata = None
            if manifest['format'] == 'parquet':
                native = pq.ParquetFile(pa.BufferReader(data))
                if native.metadata.num_rows != part['rows']:
                    raise ValueError('Parquet row declaration')
                for i in range(native.metadata.num_row_groups):
                    group = native.metadata.row_group(i)
                    if any(group.column(j).compression != 'ZSTD' for j in range(group.num_columns)):
                        raise ValueError('expected real Zstd')
                metadata = native.metadata.metadata
                arrow = native.read()
                frame = pl.read_parquet(io.BytesIO(data))
            elif manifest['format'] == 'ipc':
                stream = pa.ipc.open_stream(pa.BufferReader(data)); metadata = stream.schema.metadata
                arrow = stream.read_all(); frame = pl.read_ipc_stream(io.BytesIO(data))
            else:
                arrow = pcsv.read_csv(pa.BufferReader(data), parse_options=pcsv.ParseOptions(newlines_in_values=True),
                                      convert_options=pcsv.ConvertOptions(column_types=arrow_schema,
                                          null_values=[''], strings_can_be_null=True, quoted_strings_can_be_null=False))
                frame = pl.read_csv(io.BytesIO(data), schema_overrides=polars_schema, empty_string_is_null=True)
            if manifest['format'] != 'csv':
                observed = {k.decode(): v.decode() for k, v in (metadata or {}).items() if k.startswith(b'careops.')}
                if observed != required_metadata(manifest, table, part):
                    raise ValueError('required embedded provenance differs or is absent')
            # Normalize only the qualified Arrow UTF8 layout difference. Numeric
            # and Boolean physical columns must already have the exact types.
            for field, wanted in zip(arrow.schema, arrow_schema):
                if wanted.type != pa.large_string() and field.type != wanted.type:
                    raise ValueError('native numeric physical type changed')
            arrow = arrow.cast(arrow_schema)
            assert_frame_equal(frame, pl.from_arrow(arrow), check_exact=True)
            if frame.columns != list(polars_schema) or frame.schema != pl.Schema(polars_schema) or frame.height != part['rows']:
                raise ValueError('native column order, physical types or row count')
            for name, _, nullable in contract:
                if not nullable and frame[name].null_count():
                    raise ValueError('null required domain column')
            frames.append(frame); next_row += part['rows']; total_parts += 1
        if next_row != table['rows']:
            raise ValueError('incomplete native table rows')
        total_rows += next_row; result[table['name']] = pl.concat(frames)
    if total_rows > limits['max_total_rows'] or total_parts > limits['max_parts']:
        raise ValueError('aggregate native export budget')
    replications = result['replications']
    seeds = manifest['replication_seeds']
    if (not isinstance(seeds, list) or not seeds
            or any(type(seed) is not int or not 0 <= seed <= 2**64-1 for seed in seeds)
            or replications['seed'].to_list() != seeds
            or replications['replication_index'].to_list() != list(range(len(seeds)))
            or replications['horizon_ticks'].to_list() != [manifest['horizon_ticks']] * len(seeds)
            or replications['run_id'].to_list() != [manifest['batch_id']] * len(seeds)):
        raise ValueError('replication seed, horizon or run provenance')
    patients = result['patients']
    joined = patients.join(replications.select('replication_index', 'seed').rename({'seed': '_declared_seed'}),
                           on='replication_index', how='left', maintain_order='left')
    if joined['_declared_seed'].null_count() or joined.filter(pl.col('seed') != pl.col('_declared_seed')).height:
        raise ValueError('patient seed does not bind replication')
    for column, value in [('schema', 'careops.ed.patients.v2'), ('run_id', manifest['batch_id']),
                          ('scenario_id', manifest['scenario_id']), ('time_unit', manifest['time_unit']),
                          ('input_sha256', manifest['effective_input_sha256']),
                          ('model_contract_sha256', manifest['model_contract_sha256'])]:
        if patients[column].to_list() != [value] * patients.height:
            raise ValueError('patient provenance does not bind actual columns')
    return manifest, result


class ProductionNativeOutputTests(unittest.TestCase):
    def test_actual_outputs_have_exact_types_values_nulls_and_provenance(self):
        datasets = [read_dataset(ROOT / name) for name in ['parquet', 'ipc', 'csv']]
        manifest, frames = datasets[0]
        self.assertEqual([f.height for f in frames.values()], [20, 4, 5])
        self.assertEqual(manifest['horizon_ticks'], 8)
        self.assertEqual(manifest['base_seed'], 7)
        self.assertNotEqual(manifest['effective_input_sha256'], manifest['parsed_input_sha256'])
        self.assertIn('case,"a\r\nline ☃', frames['patients']['patient_id'].to_list())
        self.assertGreater(frames['patients']['completion_tick_lo'].null_count(), 0)
        self.assertEqual(frames['replications']['seed'].to_list(), manifest['replication_seeds'])
        for _, other in datasets[1:]:
            for name in frames:
                assert_frame_equal(frames[name], other[name], check_exact=True)

    def test_maximum_native_producer_tick_is_exact_in_all_formats(self):
        datasets = [read_dataset(ROOT / 'max-tick' / name) for name in ['parquet', 'ipc', 'csv']]
        for manifest, frames in datasets:
            self.assertEqual(manifest['horizon_ticks'], 2**64 - 1)
            self.assertEqual(frames['patients']['completion_tick_lo'].to_list(), [2**64 - 1])
            self.assertEqual(frames['patients']['completion_tick_hi'].to_list(), [0])
        for _, frames in datasets[1:]:
            for name in frames:
                assert_frame_equal(datasets[0][1][name], frames[name], check_exact=True)

    def test_incomplete_hash_schema_and_missing_embedded_metadata_refuse(self):
        path = ROOT / 'parquet' / 'manifest.json'; original = path.read_bytes(); manifest = json.loads(original)
        try:
            changes = [lambda d: d.update(status='incomplete'), lambda d: d.update(schema='unknown'),
                       lambda d: d['tables'][0]['parts'][0].update(sha256='0'*64),
                       lambda d: d['tables'][0]['parts'].pop(),
                       lambda d: d['tables'][0]['columns'][0].update(nullable=True),
                       lambda d: d.update(effective_input_sha256='0'*64),
                       lambda d: d['replication_seeds'].__setitem__(0, 999),
                       lambda d: d.update(horizon_ticks=99)]
            for change in changes:
                value = copy.deepcopy(manifest); change(value); path.write_text(json.dumps(value))
                with self.assertRaises(ValueError): read_dataset(path.parent)
            path.write_bytes(b'{"schema":"duplicate",' + original.lstrip()[1:])
            with self.assertRaisesRegex(ValueError, 'duplicate control'): read_dataset(path.parent)
        finally:
            path.write_bytes(original)
        with tempfile.TemporaryDirectory(prefix='r132-negative-', dir=ROOT) as temporary:
            negative = Path(temporary)
            for source in path.parent.iterdir(): shutil.copyfile(source, negative / source.name)
            declaration = copy.deepcopy(manifest); part = declaration['tables'][0]['parts'][0]
            file = negative / part['file']; arrow = pq.read_table(file).replace_schema_metadata(None)
            pq.write_table(arrow, file, compression='zstd', store_schema=False)
            part['bytes'] = file.stat().st_size; part['sha256'] = hashlib.sha256(file.read_bytes()).hexdigest()
            (negative / 'manifest.json').write_text(json.dumps(declaration))
            with self.assertRaisesRegex(ValueError, 'embedded provenance'): read_dataset(negative)
            (negative / 'manifest.json').unlink()
            with self.assertRaisesRegex(ValueError, 'completion control'): read_dataset(negative)


if __name__ == '__main__':
    unittest.main()
