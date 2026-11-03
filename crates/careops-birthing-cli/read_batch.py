#!/usr/bin/env python3
"""Independent PyArrow reader for the bounded synthetic B7 publication.

Run with the repository-local Python3.15.1 and pinned reader requirements.
The required domain manifest carries tick=second and effective-input provenance;
the generic event-log time_scale column explicitly records `second` as well.
"""
import hashlib
import json
from pathlib import Path
import sys

import pyarrow as pa
import pyarrow.ipc as ipc
import pyarrow.parquet as pq


def load(path, limit=262144):
    if path.is_symlink() or not path.is_file() or path.stat().st_size > limit:
        raise ValueError("unsupported or oversized declaration")
    return json.loads(path.read_bytes())


def digest(value):
    return hashlib.sha256(value).hexdigest()


def fnv64(value):
    result = 0xcbf29ce484222325
    for byte in value:
        result = ((result ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{result:016x}'


def validate_theatre(rows, expected_input):
    cases = [row for row in rows if row['kind'] == 'theatre_case']
    births = [row for row in rows if row['kind'] == 'birth' and row['data']['route'] == 'operative']
    if len(cases) != expected_input['scenario']['days'] or len(births) != len(cases):
        raise ValueError('missing operative timing/assessment records')
    callback_fields = [('Accepted', 'accepted'), ('TransferDeparted', 'departed'),
                       ('TransferArrived', 'arrived'), ('ServiceStarted', 'started'),
                       ('ProcedureStarted', 'incision'), ('Birth', 'birth'),
                       ('ServiceCompleted', 'completed'), ('RoomReady', 'cleaned')]
    checked = 0
    for record in cases:
        case = record['data']
        timings = case['timings']
        if (case['episode'] != record['episode'] or timings['cancelled'] is not None
                or not any(b['person'] == record['person'] and b['episode'] == record['episode']
                           and b['tick'] == timings['birth'] for b in births)):
            raise ValueError('operative birth is not paired with actual case')
        ordered = [case['decision_tick']] + [timings[field] for _, field in callback_fields]
        if any(t is None for t in ordered) or ordered != sorted(ordered):
            raise ValueError('invalid operative timing order')
        if len(case['callbacks']) != len(callback_fields):
            raise ValueError('missing or duplicated actual callbacks')
        for callback, field in callback_fields:
            matched = [c for c in case['callbacks'] if c['callback'] == callback]
            if len(matched) != 1 or matched[0]['tick'] != timings[field] or matched[0]['episode'] != record['episode']:
                raise ValueError('callback does not match actual timing')
        assessments = case['assessments']
        if not assessments:
            raise ValueError('missing observed assessment')
        for name, field in [('transfer', 'departed'), ('incision', 'incision'), ('birth', 'birth')]:
            metric = case['metrics'][name]
            eligible = [a for a in assessments if a['observed_at_tick'] <= timings[field]]
            if not eligible:
                raise ValueError('metric has no available assessment')
            assessment = max(eligible, key=lambda a: a['observed_at_tick'])
            policy = assessment['policy']
            urgency = assessment['urgency']
            if (metric['assessment_tick'] != assessment['observed_at_tick']
                    or metric['elapsed'] != timings[field] - case['decision_tick']
                    or metric['policy_revision'] != policy['revision']
                    or metric['scheme_version'] != policy['scheme_version']
                    or assessment['elapsed_wait_ticks'] != assessment['observed_at_tick'] - case['decision_tick']
                    or urgency['assessed_at_tick'] != assessment['observed_at_tick']
                    or not urgency['effective_date'] or assessment['interruption'] != 'NotEligible'):
                raise ValueError('metric/observed policy linkage mismatch')
            for key in ['jurisdiction_id', 'scheme_id', 'scheme_version', 'source_authority', 'citation']:
                if not policy[key] or urgency[key] != policy[key]:
                    raise ValueError('missing or mismatched policy provenance')
            rules = [r for r in policy['rules'] if r['category'] == urgency['category']]
            if (len(rules) != 1 or rules[0]['targets'] != assessment['rule']['targets']
                    or rules[0]['priority_rank'] != assessment['rule']['priority_rank']):
                raise ValueError('assessment rule does not bind policy category')
            target = assessment['rule']['targets'][name]
            expected_breach = None if target is None else metric['elapsed'] > target
            if metric['target'] != target or metric['breached'] != expected_breach:
                raise ValueError('wrong target/zero-target/breach accounting')
            checked += 1
    return checked


def validate_uncertainty(manifest):
    expected = {
        'schema': 'careops.birthing.observation-uncertainty.v1',
        'method': 'not_estimated',
        'reason': 'case_observations_no_declared_sampling_estimand',
        'observation_unit': 'synthetic_case',
        'experiment_unit': 'independent_replication',
        'metrics': ['delay_seconds', 'origin_occupied_seconds', 'target_breach'],
    }
    actual = manifest.get('uncertainty')
    if (type(manifest.get('schema_version')) is not int
            or manifest['schema_version'] != 2 or actual != expected):
        raise ValueError('missing or unsupported observation uncertainty contract')
    return actual


def validate_native_metadata(actual, manifest, native, index, first_sequence, rows):
    expected = {
        'careops.schema_version': '2',
        'careops.run_id': native['run_id'],
        'careops.effective_input_sha256': manifest['effective_input_sha256'],
        'careops.model_contract_sha256': manifest['replication']['model_contract_sha256'],
        'careops.runtime_revision': manifest['runtime_revision'],
        'careops.data_revision': manifest['data_revision'],
        'careops.time_unit': manifest['tick_unit'],
        'careops.replication_index': str(index),
        'careops.seed': str(manifest['replication']['seed']),
        'careops.first_sequence': str(first_sequence),
        'careops.rows': str(rows),
    }
    decoded = {k.decode('utf-8'): v.decode('utf-8') for k, v in (actual or {}).items()
               if k != b'ARROW:schema'}
    if decoded != expected:
        raise ValueError('native capture metadata mismatch')


def read_batch(root, expected_input):
    batch = root / expected_input['batch_id']
    completed = load(batch / 'batch_manifest.json')
    if len(completed) != expected_input['replications']:
        raise ValueError("incomplete batch")
    reports = []
    model_contract = None
    uncertainty = None
    for index, result in enumerate(completed):
        if result['index'] != index:
            raise ValueError("unordered or duplicated replication")
        name = result['directory']
        if (Path(name).name != name or '/' in name or '\\' in name
                or not name.startswith(expected_input['batch_id'] + '-attempt')
                or not name.endswith(f'-rep{index}')):
            raise ValueError("invalid replication path")
        directory = batch / name
        if directory.is_symlink():
            raise ValueError("symlink replication")
        manifest = load(directory / 'birthing_manifest.json')
        uncertainty = validate_uncertainty(manifest)
        model = result['model_contract_sha256']
        if (len(model) != 64 or any(c not in '0123456789abcdef' for c in model)
                or (model_contract is not None and model_contract != model)):
            raise ValueError('mixed or malformed model/build contract identity')
        model_contract = model
        native = load(directory / 'run_manifest.json')
        if (manifest['schema_version'] != 2 or not manifest['completed']
                or manifest['input'] != expected_input or manifest['tick_unit'] != 'second'
                or manifest['replication'] != result
                or manifest['runtime_revision'] != '07410266bf54b6f73c59db2f8cbfbc5697c53734'
                or manifest['data_revision'] != 'f34750533c07e9e868481606e5fe5c89ec9b76be'
                or native['status'] != 'Completed' or native['time_scale'] != 'second'
                or native['replication_id'] != index or native['seed'] != result['seed']
                or native['run_id'] != name.removesuffix(f'-rep{index}')):
            raise ValueError("unbound source/input/completion metadata")
        encoded_input = manifest['effective_input_json'].encode('utf-8')
        bound = json.loads(manifest['effective_input_json'])
        if bound != {'input': expected_input, 'model_contract_sha256': model}:
            raise ValueError("canonical effective input values differ")
        if manifest['effective_input_sha256'] != digest(encoded_input) or native['effective_input_hash'] != digest(encoded_input):
            raise ValueError("effective input digest mismatch")
        if not 1 <= len(manifest['ipc']) <= 64 or len(native['parts']) != len(manifest['ipc']):
            raise ValueError("unbounded or unmatched parts")
        capture = []
        for part, receipt in enumerate(manifest['ipc']):
            if receipt['file'] != f'ipc-{part:05}.arrow' or not 1 <= receipt['rows'] <= 64:
                raise ValueError("invalid IPC part")
            path = directory / receipt['file']
            if path.is_symlink() or path.stat().st_size > 8 * 1024 * 1024:
                raise ValueError("unsupported IPC file")
            raw = path.read_bytes()
            if digest(raw) != receipt['sha256']:
                raise ValueError("IPC hash mismatch")
            stream = ipc.open_stream(pa.BufferReader(raw))
            validate_native_metadata(stream.schema.metadata, manifest, native, index, len(capture) + 1, receipt['rows'])
            table = stream.read_all()
            parquet_part = native['parts'][part]
            if parquet_part['file_name'] != f'part-{part+1:05}.parquet':
                raise ValueError("invalid Parquet part")
            path = directory / parquet_part['file_name']
            if path.is_symlink() or path.stat().st_size > 8 * 1024 * 1024:
                raise ValueError("unsupported Parquet file")
            if fnv64(path.read_bytes()) != parquet_part['file_hash']:
                raise ValueError('Parquet content hash mismatch')
            parquet = pq.ParquetFile(path)
            validate_native_metadata(parquet.metadata.metadata, manifest, native, index, len(capture) + 1, receipt['rows'])
            for row_group in range(parquet.metadata.num_row_groups):
                for column in range(parquet.metadata.num_columns):
                    if parquet.metadata.row_group(row_group).column(column).compression != 'ZSTD':
                        raise ValueError("Parquet compression is not ZSTD")
            other = parquet.read()
            for field, expected in [('schema_version', pa.uint16()), ('event_id_index', pa.uint64()),
                                    ('event_id_gen', pa.uint32()), ('entity_id_index', pa.uint64()),
                                    ('time_ticks_hi', pa.uint64()), ('time_ticks_lo', pa.uint64()),
                                    ('sequence', pa.uint64())]:
                if table.schema.field(field).type != expected or other.schema.field(field).type != expected:
                    raise ValueError("typed Arrow identity/tick schema mismatch")
            left = sorted(table.to_pylist(), key=lambda row: row['sequence'])
            right = sorted(other.to_pylist(), key=lambda row: row['sequence'])
            if left != right or len(left) != receipt['rows'] or len(right) != parquet_part['row_count']:
                raise ValueError("independent format/value/count mismatch")
            if [row['sequence'] for row in left] != list(range(len(capture) + 1, len(capture) + 1 + receipt['rows'])):
                raise ValueError('native part sequence metadata mismatch')
            capture.extend(left)
        capture.sort(key=lambda row: row['sequence'])
        rows = []
        for sequence, record in enumerate(capture, 1):
            row = json.loads(record['payload_ref'])
            if (record['schema_version'] != 2 or record['run_id'] != native['run_id']
                    or record['sequence'] != sequence or record['event_id_index'] != sequence
                    or record['event_id_gen'] != 0 or record['time_scale'] != 'second'
                    or record['priority'] != 0
                    or record['entity_id_gen'] != (0 if row['person'] else None)
                    or (record['time_ticks_hi'] << 64) + record['time_ticks_lo'] != row['tick']
                    or record['entity_id_index'] != (row['person'] or None)
                    or record['event_kind'] != row['kind'] or record['status'] != 'dispatched'):
                raise ValueError("canonical capture identity/value mismatch")
            rows.append(row)
        canonical = digest(json.dumps(rows, separators=(',', ':'), ensure_ascii=False).encode())
        if canonical != result['canonical_sha256'] or len(rows) != result['rows']:
            raise ValueError("canonical result digest mismatch")
        births = [row for row in rows if row['kind'] == 'birth']
        conservation = [row for row in rows if row['kind'] == 'conservation']
        if len(births) != expected_input['scenario']['days'] * 2 or len(conservation) != len(births):
            raise ValueError("missing actual birth/conservation capture")
        if any(row['data']['people'] != row['data']['open_episodes'] or row['data']['held_origin_claims'] != 0 for row in conservation):
            raise ValueError("conservation failure")
        reports.append({'index': index, 'seed': result['seed'], 'rows': len(rows),
                        'canonical_sha256': canonical, 'births': len(births),
                        'policy_metrics_checked': validate_theatre(rows, expected_input)})
    return {'reader': 'PyArrow', 'version': pa.__version__, 'model_contract_sha256': model_contract,
            'uncertainty': uncertainty, 'replications': reports}


if __name__ == '__main__':
    if sys.version_info[:3] != (3, 15, 1):
        raise SystemExit('Python3.15.1 required')
    if len(sys.argv) != 3:
        raise SystemExit('usage: read_batch.py PRIVATE_OUTPUT_ROOT INPUT.json')
    print(json.dumps(read_batch(Path(sys.argv[1]), load(Path(sys.argv[2]), 65536)), sort_keys=True))
