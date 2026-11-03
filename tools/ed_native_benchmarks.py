#!/usr/bin/env python3
"""Native ED CLI benchmark v1; JSON patient reconstruction is TEST-ONLY oracle data.

Production output is typed Parquet. The frozen alpha driver/reference are never
rewritten: this independent file reader reconstructs their exact canonical shape
only to compare the unchanged deterministic patient/summary facts.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
import time
import ed_benchmarks as frozen
from ed_native_benchmark_process import (
    BenchmarkError, MAX_OUTPUT, MAX_RSS_BYTES, MAX_WALL_SECONDS, run_bounded, unread_stdout_termination,
)
PROTOCOL = 'careops.ed.native-benchmark.v1'
MAX_CONTROL = 1024 * 1024
MAX_ENCODED = 64 * 1024 * 1024
MAX_PART = 4 * 1024 * 1024
PATIENT_COLUMNS = ('time_unit schema capture_schema run_id input_sha256 model_contract_sha256 '
    'replication_index seed sequence patient_id status arrival_tick_hi arrival_tick_lo '
    'start_tick_hi start_tick_lo completion_tick_hi completion_tick_lo work_ticks_hi work_ticks_lo '
    'wait_ticks_hi wait_ticks_lo elapsed_ticks_hi elapsed_ticks_lo scenario_id resource_id location_id capture_table_schema').split()
REPLICATION_COLUMNS = 'run_id replication_index seed horizon_ticks arrivals started completed unfinished mean_wait_ticks'.split()

METRIC_COLUMNS = ('name value unit denominator_name denominator_value numerator_count denominator_count '
    'estimand sample_unit weighting eligible_replications included_replications total_replications '
    'estimate variance standard_error ci_95_lower ci_95_upper uncertainty_replications ddof method status censored_count unfinished_count').split()

def closed_columns(table):
    if table=='patients':
        names=PATIENT_COLUMNS
        nullable={p+s for p in ['start_tick','completion_tick','wait_ticks','elapsed_ticks'] for s in ['_hi','_lo']}
        kind=lambda n:'UInt64' if n in ['replication_index','seed','sequence'] or n.endswith(('_tick_hi','_tick_lo','_ticks_hi','_ticks_lo')) else 'String'
    elif table=='replications':
        names=REPLICATION_COLUMNS; nullable={'mean_wait_ticks'}
        kind=lambda n:'String' if n=='run_id' else ('Float64' if n=='mean_wait_ticks' else 'UInt64')
    elif table=='metrics':
        names=METRIC_COLUMNS
        floats={'value','denominator_value','estimate','variance','standard_error','ci_95_lower','ci_95_upper'}
        nullable=floats|{'numerator_count','denominator_count','ddof'}
        integers={'numerator_count','denominator_count','eligible_replications','included_replications','total_replications','uncertainty_replications','censored_count','unfinished_count'}
        kind=lambda n:'Float64' if n in floats else ('UInt64' if n in integers else ('UInt32' if n=='ddof' else 'String'))
    else: raise BenchmarkError('unknown native table')
    return [{'name':n,'physical_type':kind(n),'nullable':n in nullable} for n in names]

def digest(raw):
    return hashlib.sha256(raw).hexdigest()

def _pairs(items):
    result = {}
    for key,value in items:
        if key in result:
            raise BenchmarkError('duplicate JSON control key')
        result[key] = value
    return result

def bounded_bytes(path, limit):
    path = Path(path)
    if path.is_symlink() or not path.is_file() or path.stat().st_size > limit:
        raise BenchmarkError('nonregular/oversized file')
    with path.open('rb') as stream:
        raw = stream.read(limit+1)
    if len(raw)>limit:
        raise BenchmarkError('growing file exceeds bound')
    return raw

def load(path, limit=MAX_CONTROL):
    raw = bounded_bytes(path, limit)
    try:
        return json.loads(raw, object_pairs_hook=_pairs,
                          parse_constant=lambda _: (_ for _ in ()).throw(BenchmarkError('nonfinite control')))
    except (ValueError,UnicodeError) as error:
        raise BenchmarkError('malformed control') from error

def _int(value):
    if type(value) is not int or value < 0:
        raise BenchmarkError('non-integer/nonnegative count or limb')
    return value

def integer_control_tree(value):
    # These native manifests/invocations contain no floating-point fields.
    # Comparison points live in typed native tables, not these small controls.
    if type(value) is float:
        raise BenchmarkError('floating alias in native integer control')
    if isinstance(value,dict):
        for item in value.values(): integer_control_tree(item)
    elif isinstance(value,list):
        for item in value: integer_control_tree(item)

def _tick(row, prefix):
    hi,lo = row[prefix+'_hi'],row[prefix+'_lo']
    if hi is None and lo is None:
        return None
    if hi is None or lo is None or _int(hi)>2**64-1 or _int(lo)>2**64-1:
        raise BenchmarkError('invalid tick limbs')
    return (hi << 64) | lo

def native_summary(output, raw, input_path, count):
    import pyarrow as pa
    import pyarrow.parquet as pq
    if pa.__version__ != "26.0.0": raise BenchmarkError("qualified PyArrow26.0.0 required")
    output = Path(output)
    completion = load(output/'manifest.json'); integer_control_tree(completion)
    if completion.get('schema') != 'careops.ed.cli-artifacts.v1' or completion.get('status') != 'complete':
        raise BenchmarkError('incomplete native CLI artifacts')
    if completion.get('invocation_file') != 'invocation.json' or completion.get('dataset_manifest') != 'data/manifest.json' or completion.get('batch_file') != 'batch_run.json':
        raise BenchmarkError('unsupported artifact control paths')
    if digest(bounded_bytes(output/'batch_run.json',MAX_CONTROL)) != completion.get('batch_sha256'):
        raise BenchmarkError('batch control checksum mismatch')
    invocation_raw = bounded_bytes(output/'invocation.json',MAX_CONTROL)
    if digest(invocation_raw) != completion.get('invocation_sha256'):
        raise BenchmarkError('invocation checksum mismatch')
    invocation = load(output/'invocation.json'); integer_control_tree(invocation)
    if invocation.get('raw_input_sha256') != digest(raw) or _int(invocation.get('replications')) != 1 or _int(invocation.get('base_seed')) != frozen.SEED:
        raise BenchmarkError('unbound native input/seed/replication')
    manifest_raw = bounded_bytes(output/'data/manifest.json',MAX_CONTROL)
    if digest(manifest_raw) != completion.get('dataset_manifest_sha256'):
        raise BenchmarkError('dataset control checksum mismatch')
    manifest = load(output/'data/manifest.json'); integer_control_tree(manifest)
    if (manifest.get('schema') != 'careops.ed.native-output.v1' or manifest.get('status') != 'complete'
            or manifest.get('format') != 'parquet' or manifest.get('compression') != 'zstd'
            or manifest.get('scenario_id') != f'd34-fifo-{count}' or manifest.get('time_unit') != 'tick'
            or _int(manifest.get('base_seed')) != frozen.SEED or not frozen._same_typed(manifest.get('replication_seeds'),[frozen.SEED])
            or _int(manifest.get('horizon_ticks')) != 2*count+2):
        raise BenchmarkError('native dataset identity/profile mismatch')
    if (invocation.get('model_contract_sha256') != manifest.get('model_contract_sha256')
            or invocation.get('parsed_input_sha256') != manifest.get('parsed_input_sha256')):
        raise BenchmarkError('invocation/dataset model or parsed-input binding mismatch')
    tables = manifest.get('tables')
    if not isinstance(tables,list) or [t.get('name') for t in tables] != ['patients','replications','metrics']:
        raise BenchmarkError('native table set/order mismatch')
    selected = {}; encoded = 0
    for table in tables:
        name = table['name']; rows = []; offset = 0
        if not frozen._same_typed(table.get('columns'),closed_columns(name)):
            raise BenchmarkError('noncanonical native column contract')
        schema = 'careops.ed.'+name+('.v2' if name=='patients' else '.v1')
        if table.get('schema') != schema:
            raise BenchmarkError('native table schema identity mismatch')
        parts = table.get('parts')
        if not isinstance(parts,list) or not parts or len(parts)>4096:
            raise BenchmarkError('unsupported native part count')
        for index,part in enumerate(parts):
            expected_name = f'{name}-{index:05}.parquet'
            if part.get('file') != expected_name or _int(part.get('first_row')) != offset or not 0 <= _int(part.get('rows')) <= 256:
                raise BenchmarkError('native part path/offset/count mismatch')
            path = output/'data'/expected_name
            if path.is_symlink() or not path.is_file() or path.stat().st_size > MAX_PART:
                raise BenchmarkError('nonregular/oversized native part')
            data = bounded_bytes(path,MAX_PART); encoded += len(data)
            if len(data)>MAX_PART or encoded>MAX_ENCODED or len(data)!=_int(part.get('bytes')) or digest(data)!=part.get('sha256'):
                raise BenchmarkError('native part hash/size mismatch')
            file = pq.ParquetFile(path)
            if file.metadata.num_rows != part['rows'] or file.metadata.num_rows > 256:
                raise BenchmarkError('Parquet footer exceeds row profile before allocation')
            for group in range(file.metadata.num_row_groups):
                for column in range(file.metadata.num_columns):
                    if file.metadata.row_group(group).column(column).compression != 'ZSTD':
                        raise BenchmarkError('native compression mismatch')
            frame = file.read()
            if frame.num_rows != part['rows'] or frame.column_names != [c['name'] for c in table['columns']]:
                raise BenchmarkError('native physical shape mismatch')
            metadata = frame.schema.metadata or {}
            for key,value in {'table':name,'run_id':manifest['batch_id'], 'rows':str(part['rows']), 'first_row':str(offset),
                              'effective_input_sha256':manifest['effective_input_sha256'],
                              'model_contract_sha256':manifest['model_contract_sha256']}.items():
                if metadata.get(('careops.'+key).encode()) != value.encode():
                    raise BenchmarkError('unbound native file metadata')
            physical = {'String':pa.string_view(), 'UInt64':pa.uint64(), 'UInt32':pa.uint32(), 'Float64':pa.float64()}
            for column in table['columns']:
                field = frame.schema.field(column['name'])
                if field.type != physical.get(column['physical_type']):
                    raise BenchmarkError('native physical type mismatch')
                if type(column['nullable']) is not bool or (not column['nullable'] and frame.column(column['name']).null_count):
                    raise BenchmarkError('native requiredness mismatch')
            rows.extend(frame.to_pylist()); offset += frame.num_rows
        if offset != _int(table['rows']):
            raise BenchmarkError('native table coverage mismatch')
        if name == 'patients' and [c['name'] for c in table['columns']] != PATIENT_COLUMNS:
            raise BenchmarkError('native patient schema mismatch')
        if name == 'replications' and [c['name'] for c in table['columns']] != REPLICATION_COLUMNS:
            raise BenchmarkError('native replication schema mismatch')
        selected[name] = rows
    if len(selected['patients']) != count or len(selected['replications']) != 1 or len(selected['metrics']) != 5:
        raise BenchmarkError('native known consumer cardinality mismatch')
    rep = selected['replications'][0]
    if _int(rep['replication_index']) != 0 or _int(rep['seed']) != frozen.SEED or _int(rep['horizon_ticks']) != 2*count+2 or rep['run_id'] != manifest['batch_id']:
        raise BenchmarkError('unbound native replication')
    patients = []
    for sequence,row in enumerate(selected['patients']):
        if (_int(row['sequence']) != sequence or _int(row['replication_index']) != 0 or _int(row['seed']) != frozen.SEED
                or row['run_id'] != manifest['batch_id'] or row['scenario_id'] != manifest['scenario_id']
                or row['input_sha256'] != manifest['effective_input_sha256']
                or row['model_contract_sha256'] != manifest['model_contract_sha256'] or row['time_unit'] != 'tick'):
            raise BenchmarkError('unbound native patient row')
        patients.append({'patient_id':row['patient_id'],'status':row['status'],
                         **{name:_tick(row,name) for name in ['arrival_tick','start_tick','completion_tick','work_ticks','wait_ticks','elapsed_ticks']}})
    summary = {'schema_version':1,'scenario_id':manifest['scenario_id'],'seed':frozen.SEED,'time_unit':'tick',
               'horizon_ticks':manifest['horizon_ticks'],
               **{key:rep[key] for key in ['arrivals','started','completed','unfinished']},'patients':patients}
    doc = {'manifest':{key:value for key,value in summary.items() if key not in ['arrivals','started','completed','unfinished','patients']}, 'summary':summary}
    doc['manifest'].update(input_sha256=digest(raw),input_file=str(input_path))
    return frozen.validate_document(doc, {'count':count,'scenario_id':f'd34-fifo-{count}'},digest(raw),str(input_path))

def _one_run(binary, run_dir, count, template):
    raw=frozen.make_input(template,count); frozen.preflight_input(raw)
    run_dir=Path(run_dir);run_dir.mkdir(parents=True,exist_ok=True)
    tag=f'n{count:05d}-{time.time_ns()}'; source=run_dir/(tag+'.input.json'); output=run_dir/(tag+'.native')
    source.write_bytes(raw)
    result=run_bounded([str(binary),'run',str(source),'--output-dir',str(output)])
    stdout=result.pop('stdout');stderr=result.pop('stderr')
    if result['timed_out'] or result['exit_code']!=0 or result['stdout_overflow']:
        raise BenchmarkError(f'native CLI failed bounded process: {result}; {stderr.decode(errors="replace")}')
    if result['elapsed_seconds'] > MAX_WALL_SECONDS:
        raise BenchmarkError('native child exceeded wall-time budget')
    if result['peak_rss_bytes'] > MAX_RSS_BYTES:
        raise BenchmarkError('native child exceeded peak-RSS budget')
    expected_stdout = bounded_bytes(output/'manifest.json',MAX_CONTROL)
    if not stdout or stdout != expected_stdout+b'\n':
        raise BenchmarkError('stdout does not match the closed completion control')
    summary=native_summary(output,raw,source,count)
    result.update(input_sha256=digest(raw),repeat_summary_sha256=digest(frozen.canonical(summary)),input_bytes=len(raw),output_bytes=len(stdout),
                  output_sha256=digest(stdout),input_path=str(source),output_path=str(output))
    return result

def adversarial_native_files(output, raw, input_path, count):
    """Actual produced-file mutations with recomputed hashes, restored exactly."""
    import copy
    import pyarrow as pa
    import pyarrow.parquet as pq
    output=Path(output); completion_path=output/'manifest.json'; dataset_path=output/'data/manifest.json'
    invocation_path=output/'invocation.json'
    original_manifest=load(dataset_path); original_completion=load(completion_path)
    part=original_manifest['tables'][0]['parts'][0]; part_path=output/'data'/part['file']
    paths=[completion_path,dataset_path,invocation_path,part_path]
    originals={p:bounded_bytes(p,MAX_PART) for p in paths}
    baseline=native_summary(output,raw,input_path,count); checked=[]
    def restore():
        for p,data in originals.items(): p.write_bytes(data)
    def manifest_change(change):
        value=copy.deepcopy(original_manifest); change(value)
        encoded=json.dumps(value,allow_nan=False).encode();dataset_path.write_bytes(encoded)
        completed=copy.deepcopy(original_completion);completed['dataset_manifest_sha256']=digest(encoded)
        completion_path.write_text(json.dumps(completed))
    def parquet_change(column, values, dtype, *, declare=False):
        table=pq.read_table(part_path); field=table.schema.field(column)
        table=table.set_column(table.schema.get_field_index(column),pa.field(column,dtype,nullable=field.nullable),pa.array(values(table),type=dtype))
        pq.write_table(table,part_path,compression='zstd')
        encoded=part_path.read_bytes()
        def change(manifest):
            p=manifest['tables'][0]['parts'][0];p['sha256']=digest(encoded);p['bytes']=len(encoded)
            if declare:
                next(c for c in manifest['tables'][0]['columns'] if c['name']==column)['physical_type']='Float64'
        manifest_change(change)
    def bad_model():
        invocation=load(invocation_path);invocation['model_contract_sha256']='0'*64
        encoded=json.dumps(invocation).encode();invocation_path.write_bytes(encoded)
        completed=copy.deepcopy(original_completion);completed['invocation_sha256']=digest(encoded)
        completion_path.write_text(json.dumps(completed))
    def bad_batch():
        completed=copy.deepcopy(original_completion);completed['batch_sha256']='0'*64
        completion_path.write_text(json.dumps(completed))
    def duplicate():
        completion_path.write_bytes(originals[completion_path].rstrip()[:-1]+b',"status":"complete"}')
    mutations=[
        ('batch-control-hash',bad_batch),('invocation-model-binding',bad_model),('duplicate-control-key',duplicate),
        ('float-seed',lambda:manifest_change(lambda m:m.__setitem__('base_seed',float(m['base_seed'])))),
        ('float-horizon',lambda:manifest_change(lambda m:m.__setitem__('horizon_ticks',float(m['horizon_ticks'])))),
        ('float-seed-list',lambda:manifest_change(lambda m:m.__setitem__('replication_seeds',[float(x) for x in m['replication_seeds']]))),
        ('float-table-rows',lambda:manifest_change(lambda m:m['tables'][0].__setitem__('rows',float(m['tables'][0]['rows'])))),
        ('part-hash',lambda:manifest_change(lambda m:m['tables'][0]['parts'][0].__setitem__('sha256','0'*64))),
        ('row-offset',lambda:manifest_change(lambda m:m['tables'][0]['parts'][0].__setitem__('first_row',1))),
        ('boolean-requiredness-alias',lambda:manifest_change(lambda m:m['tables'][0]['columns'][0].__setitem__('nullable',0))),
        ('float-provenance-self-declared',lambda:parquet_change('sequence',lambda t:[float(x) for x in t['sequence'].to_pylist()],pa.float64(),declare=True)),
        ('float-provenance-physical',lambda:parquet_change('sequence',lambda t:[float(x) for x in t['sequence'].to_pylist()],pa.float64())),
        ('wrong-sequence-rehashed',lambda:parquet_change('sequence',lambda t:[x+1 for x in t['sequence'].to_pylist()],pa.uint64())),
        ('wrong-patient-rehashed',lambda:parquet_change('patient_id',lambda t:['changed-patient']+t['patient_id'].to_pylist()[1:],pa.string_view())),
    ]
    try:
        for name,mutate in mutations:
            restore(); mutate()
            try: native_summary(output,raw,input_path,count)
            except BenchmarkError: checked.append(name)
            else: raise BenchmarkError('native file negative was accepted: '+name)
    finally: restore()
    if native_summary(output,raw,input_path,count)!=baseline:
        raise BenchmarkError('native fixture restoration changed canonical summary')
    return {'rejected':checked,'original_bytes_restored':True}

def stdout_probes(binary, run_dir, template):
    import csv
    import io
    run_dir=Path(run_dir);run_dir.mkdir(parents=True,exist_ok=True)
    raw=frozen.make_input(template,10000);frozen.preflight_input(raw)
    source=run_dir/'stdout.input.json';source.write_bytes(raw)
    root=run_dir/'slow.native'
    argv=[str(binary),'run',str(source),'--output-dir',str(root),'--format','csv','--stdout']
    slow=run_bounded(argv,timeout=MAX_WALL_SECONDS,stdout_limit=MAX_OUTPUT,slow_consumer_delay=0.001)
    if (slow['timed_out'] or slow['exit_code']!=0 or slow['stdout_overflow']
            or slow['elapsed_seconds']>MAX_WALL_SECONDS or not 0<slow['peak_rss_bytes']<=MAX_RSS_BYTES):
        raise BenchmarkError('slow native CSV failed exit/output/wall/RSS gate: '+repr({k:v for k,v in slow.items() if k not in ('stdout','stderr')})+' stderr='+repr(slow['stderr'][:256]))
    text=slow.pop('stdout').decode('utf-8'); errors=slow.pop('stderr')
    reader=csv.DictReader(io.StringIO(text,newline=''))
    if reader.fieldnames!=PATIENT_COLUMNS:
        raise BenchmarkError('slow CSV field contract differs')
    count=0
    for count,row in enumerate(reader,1):
        k=count-1
        expected={'patient_id':f'synthetic-patient-{count:05d}','status':'completed','replication_index':'0','seed':'7',
                  'sequence':str(k),'arrival_tick_hi':'0','arrival_tick_lo':'1','start_tick_hi':'0','start_tick_lo':str(1+2*k),
                  'completion_tick_hi':'0','completion_tick_lo':str(3+2*k),'work_ticks_hi':'0','work_ticks_lo':'2',
                  'wait_ticks_hi':'0','wait_ticks_lo':str(2*k),'elapsed_ticks_hi':'0','elapsed_ticks_lo':str(2+2*k)}
        if any(row.get(key)!=value for key,value in expected.items()):
            raise BenchmarkError('slow native CSV differs from exact FIFO oracle')
    if count!=10000 or not (root/'manifest.json').is_file():
        raise BenchmarkError('slow CSV or artifacts incomplete')
    slow.update(status='pass',bytes=len(text.encode()),output_sha256=digest(text.encode()),input_sha256=digest(raw),consumer_quantum_bytes=64*1024,delay_seconds_per_quantum=0.001)
    unread=unread_stdout_termination([str(binary),'run',str(source),'--output-dir',str(run_dir/'unread.native'),'--format','csv','--stdout'])
    return {'slow_stdout_consumer':slow,'blocked_child_termination':unread}

def preflight_checks(binary, run_dir, template):
    """Retain frozen zero-work, input-cap and platform storage-failure gates."""
    import platform
    from ed_benchmark_process import _run_to_full
    run_dir=Path(run_dir);run_dir.mkdir(parents=True,exist_ok=True)
    zero=json.loads(frozen.make_input(template,1));zero['patients'][0]['work_ticks']=0
    source=run_dir/'zero-work.input.json';source.write_text(json.dumps(zero))
    root=run_dir/'zero.native'
    result=run_bounded([str(binary),'run',str(source),'--output-dir',str(root)])
    error=result['stderr'].decode(errors='replace').lower()
    if result['timed_out'] or result['exit_code']==0 or 'work_ticks' not in error or 'json' in error:
        raise BenchmarkError('native zero-work scenario was accepted')
    try: frozen.preflight_input(b'x'*(frozen.MAX_INPUT+1))
    except BenchmarkError: pass
    else: raise BenchmarkError('oversized input preflight did not refuse')
    full='unverified'
    if Path('/dev/full').exists():
        valid=run_dir/'storage-probe.input.json';valid.write_bytes(frozen.make_input(template,1))
        probe=_run_to_full([str(binary),'run',str(valid),'--output-dir',str(run_dir/'full.native'),'--format','csv','--stdout'])
        error=probe['stderr'].lower()
        if (probe['timed_out'] or probe['exit_code']!=7 or probe['elapsed_seconds']>MAX_WALL_SECONDS
                or not 0<probe['peak_rss_bytes']<=MAX_RSS_BYTES
                or not any(term in error for term in (b'no space',b'enospc',b'os error 28'))):
            raise BenchmarkError('native /dev/full output-failure gate failed')
        full='pass'
    elif platform.system()=='Linux':
        raise BenchmarkError('Linux /dev/full is unavailable; exhausted-storage check cannot pass')
    return {'zero_work':'pass','oversized_input_preflight':'pass','dev_full':full}

def run_suite(binary, output):
    if sys.version_info[:3]!=(3,15,1): raise BenchmarkError('Python3.15.1 required')
    binary=Path(binary).resolve(); output=Path(output).resolve()
    template=json.loads(Path('crates/careops-ed/examples/one_patient.json').read_bytes())
    reference=load(Path('conductor/evidence/r103-remote-20261029/parity.json'),8*1024*1024)
    measurements=[]
    for case in reference['measurements']:
        actual=_one_run(binary,output.with_suffix('.runs'),case['case'],template)
        if actual['input_sha256']!=case['input_sha256'] or actual['repeat_summary_sha256']!=case['repeat_summary_sha256']:
            raise BenchmarkError('frozen native summary parity mismatch')
        actual.update({k:case[k] for k in ['case','repeat','kind']}); measurements.append(actual)
    first=measurements[0]
    checks=adversarial_native_files(Path(first['output_path']),Path(first['input_path']).read_bytes(),Path(first['input_path']),first['case'])
    checks.update(preflight_checks(binary,output.with_suffix('.preflight-runs'),template))
    checks.update(stdout_probes(binary,output.with_suffix('.stdout-runs'),template))
    result={'schema':PROTOCOL,'tested_dirty_source_base':frozen._git_head(), 'source_dirty':bool(__import__('subprocess').check_output(['git','status','--porcelain'])), 'source_sha256':{p:digest(Path(p).read_bytes()) for p in ['crates/careops-ed-cli/src/lib.rs','crates/careops-ed-cli/src/main.rs','tools/ed_native_benchmarks.py','tools/ed_native_benchmark_process.py']},'binary_sha256':digest(binary.read_bytes()),
            'driver_sha256':digest(Path(__file__).read_bytes()),'measurements':measurements,'adversarial_checks':checks,
            'budgets':{'max_wall_seconds':MAX_WALL_SECONDS,'max_rss_bytes':MAX_RSS_BYTES,'max_output_bytes':MAX_OUTPUT}}
    output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
    return result

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--binary',type=Path,required=True);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
    try: result=run_suite(args.binary,args.output)
    except (BenchmarkError,OSError,ValueError) as error:
        print(f'native benchmark failed: {error}',file=sys.stderr);return 1
    print(json.dumps({'schema':PROTOCOL,'measurements':len(result['measurements']),'status':'complete'}));return 0
if __name__=='__main__': raise SystemExit(main())
