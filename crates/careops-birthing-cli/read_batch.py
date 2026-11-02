#!/usr/bin/env python3
"""Independent PyArrow reader for the bounded synthetic B7 publication.

Run with the repository-local Python3.15.1 and pinned reader requirements.
The required domain manifest carries tick=second and effective-input provenance;
the generic event-log time_scale column remains its schema's literal `ticks`.
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


def read_batch(root, expected_input):
    batch = root / expected_input['batch_id']
    completed = load(batch / 'batch_manifest.json')
    if len(completed) != expected_input['replications']:
        raise ValueError("incomplete batch")
    reports = []
    for index, result in enumerate(completed):
        if result['index'] != index:
            raise ValueError("unordered or duplicated replication")
        name = result['directory']
        if Path(name).name != name or '/' in name or '\\' in name:
            raise ValueError("invalid replication path")
        directory = batch / name
        if directory.is_symlink():
            raise ValueError("symlink replication")
        manifest = load(directory / 'birthing_manifest.json')
        native = load(directory / 'run_manifest.json')
        if (manifest['schema_version'] != 1 or not manifest['completed']
                or manifest['input'] != expected_input or manifest['tick_unit'] != 'second'
                or manifest['replication'] != result
                or manifest['runtime_revision'] != '07410266bf54b6f73c59db2f8cbfbc5697c53734'
                or manifest['data_revision'] != '4134a3733ba7e833a8c520621ec2af5a4bd4b31a'
                or native['status'] != 'Completed' or native['time_scale'] != 'second'
                or native['replication_id'] != index or native['seed'] != result['seed']):
            raise ValueError("unbound source/input/completion metadata")
        encoded_input = json.dumps(manifest['input'], separators=(',', ':'), ensure_ascii=False).encode()
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
            table = ipc.open_stream(pa.BufferReader(raw)).read_all()
            parquet_part = native['parts'][part]
            if parquet_part['file_name'] != f'part-{part+1:05}.parquet':
                raise ValueError("invalid Parquet part")
            path = directory / parquet_part['file_name']
            if path.is_symlink() or path.stat().st_size > 8 * 1024 * 1024:
                raise ValueError("unsupported Parquet file")
            parquet = pq.ParquetFile(path)
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
            capture.extend(left)
        capture.sort(key=lambda row: row['sequence'])
        rows = []
        for sequence, record in enumerate(capture, 1):
            row = json.loads(record['payload_ref'])
            if (record['schema_version'] != 2 or record['run_id'] != native['run_id']
                    or record['sequence'] != sequence or record['event_id_index'] != sequence
                    or record['event_id_gen'] != 0 or record['time_scale'] != 'ticks'
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
                        'canonical_sha256': canonical, 'births': len(births)})
    return {'reader': 'PyArrow', 'version': pa.__version__, 'replications': reports}


if __name__ == '__main__':
    if sys.version_info[:3] != (3, 15, 1):
        raise SystemExit('Python3.15.1 required')
    if len(sys.argv) != 3:
        raise SystemExit('usage: read_batch.py PRIVATE_OUTPUT_ROOT INPUT.json')
    print(json.dumps(read_batch(Path(sys.argv[1]), load(Path(sys.argv[2]), 65536)), sort_keys=True))
