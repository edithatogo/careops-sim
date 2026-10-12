"""Artifact-only owner qualification; original source packet closed at three failures."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import struct

ROOT = Path(__file__).resolve().parents[3]
ART = Path(__file__).resolve().parent
PACKET_PATH = ART / 'packet.json'
TOOLCHAIN = Path('/Users/doughnut/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin')
OWNED = 'crates/kairo-ecs-calibration/src/c2_portable_checkpoint/process_tests.rs'
ORDER = ('clippy', 'tests', 'format')
GATES = {
    'clippy': [str(TOOLCHAIN/'cargo'), 'clippy', '--locked', '-p', 'kairo-ecs-calibration', '--features', 'flow', '--all-targets', '--', '-D', 'warnings'],
    'tests': [str(TOOLCHAIN/'cargo'), 'test', '--locked', '-p', 'kairo-ecs-calibration', '--features', 'flow', '--', '--test-threads=1'],
    'format': [str(TOOLCHAIN/'rustfmt'), '--edition', '2021', '--check', '--config', 'skip_children=true', OWNED],
}
REQUIRED = ['portable_v4_handoff_seven_fresh_process_cases', 'portable_v4_handoff_schema2_rebind_negative_controls', 'portable_v4_handoff_schema2_caps_and_no_fallback']

def digest(data): return hashlib.sha256(data).hexdigest()
def filehash(path): return digest((ROOT/path).read_bytes())
def git(*args): return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
def write_new(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2); stream.write('\n')
def source_hashes(): return {OWNED: filehash(OWNED)}
def fail(message): raise SystemExit(message)
def changed_paths():
    raw=subprocess.check_output(['git','status','--porcelain=v1','-z','--untracked-files=all'],cwd=ROOT)
    return {item[3:] for item in raw.decode().split('\0') if item}
def source_scope_check(packet):
    if changed_paths(): fail('artifact-only qualification requires clean tracked/untracked source')


def authority_check(packet, packet_hash):
    path=ROOT/packet['dispatch_authority_path']
    if not path.is_file(): fail('root dispatch GO record missing')
    authority=json.loads(path.read_text())
    expected={'status':'GO','packet_sha256':packet_hash,
              'driver_sha256':digest(Path(__file__).read_bytes()),
              'worktree':str(ROOT),'base_commit':packet['base_commit']}
    if any(authority.get(k)!=v for k,v in expected.items()): fail('root dispatch authority mismatch')
    return authority

def authorize_recovery(packet, rows, action, authority):
    if authority.get('repair_authorized') or authority.get('repair_authorized_after_failures'):
        fail('owner qualification has no repair/recovery authority')
    if any(row['exit_code']!=0 for row in rows): fail('new owner qualification failure closes this packet; no rerun')
    if [row['gate'] for row in rows]!=list(ORDER[:len(rows)]): fail('owner gates must be Clippy/tests/format exactly once in order')
    if action!='finalize' and (len(rows)>=len(ORDER) or action!=ORDER[len(rows)]): fail('wrong owner gate order')
    return [],rows

def immutable_binding(packet):
    if git('rev-parse','HEAD')!=packet['base_commit']: fail('owner qualification HEAD drift')
    source_scope_check(packet)
    for path,expected in packet['input_hashes'].items():
        if not (ROOT/path).is_file() or filehash(path)!=expected: fail('qualification protected input drift: '+path)
    if source_hashes()!=packet['base_source_hashes']: fail('frozen corrected source drift')
    for path,expected in packet['historical_evidence_hashes'].items():
        if not (ROOT/path).is_file() or filehash(path)!=expected: fail('closed history drift: '+path)
    failures=[]
    for item in packet['historical_gate_receipts']:
        end=ROOT/item['receipt_path'];r=json.loads(end.read_text());start=ROOT/item['started_path'];a=json.loads(start.read_text())
        if r['started_receipt_sha256']!=digest(start.read_bytes()) or a['status']!='started' or r['status']!='finished': fail('historical receipt lifecycle mismatch')
        if any(a[k]!=r[k] for k in ('gate','argv','cwd','head','packet_sha256','driver_sha256','toolchain','source_hashes','log')): fail('historical pair binding mismatch')
        if r['source_hashes']!=r['source_hashes_after'] or filehash(r['log'])!=r['log_sha256']: fail('historical raw source/log mismatch')
        if r['exit_code']==0 and r['command_exit_code']!=0: fail('historical passing receipt has nonzero command exit')
        if r['exit_code']!=0: failures.append(item['receipt_path'])
    if failures!=packet['historical_failed_receipts'] or len(failures)!=3: fail('closed historical failure count must be three')


def parse_process_invocations(path):
    records=[]
    for block in re.split(r'\n\s*\n',path.read_text(errors='replace').strip()):
        if not block: continue
        row={}
        for line in block.splitlines():
            key,sep,value=line.partition('=')
            if not sep: fail('malformed process invocation record')
            row[key]=value
        records.append(row)
    return records

def literal_golden_map(source):
    text=source.decode('utf-8') if isinstance(source,bytes) else source
    match=re.search(r'const LEGACY_ENVELOPE_SHA256:.*?= \[([\s\S]*?)\n\];',text)
    if not match: fail('literal V1 golden declaration missing')
    pairs=re.findall(r'\(\s*"([^"]+)"\s*,\s*"([0-9a-f]{64})"\s*,?\s*\)',match.group(1))
    values=dict(pairs)
    if len(values)!=19 or len(pairs)!=19: fail('expected exactly 19 named source literals')
    return values

def verify_process_and_legacy(packet, process_root):
    inherited={row['case']:row for row in packet['inherited_process_case_records']}
    required_new=packet['new_process_case_ids']
    required=set(inherited)|set(required_new)
    if len(inherited)!=26 or len(required_new)!=7 or len(required)!=33: fail('packet inherited/new process inventory is not 26+7')
    invocation_files=sorted(process_root.rglob('process-invocations.txt'))
    invocations=[]
    for path in invocation_files: invocations.extend(parse_process_invocations(path))
    by_case={row.get('case'):row for row in invocations}
    if len(invocations)!=33 or len(by_case)!=33 or set(by_case)!=required: fail('actual child process invocation set differs from inherited26+new7')
    case_records=[]
    for case in sorted(required):
        invocation=by_case[case]
        raw=Path(invocation.get('input_file',''))
        if not raw.is_absolute(): raw=ROOT/raw
        try: raw.resolve().relative_to(process_root.resolve())
        except ValueError: fail('child checkpoint escapes captured process artifact directory')
        if not raw.is_file(): fail('raw child checkpoint missing: '+case)
        raw_bytes=raw.read_bytes(); input_hash=digest(raw_bytes)
        if input_hash!=invocation.get('input_sha256'): fail('raw child checkpoint hash mismatch: '+case)
        if invocation.get('child_exit_code')!='Some(0)': fail('child did not exit successfully: '+case)
        stdout=raw.with_suffix('.child.stdout.log')
        stderr=raw.with_suffix('.child.stderr.log')
        if not stdout.is_file() or not stderr.is_file(): fail('child stdout/stderr missing: '+case)
        traces=re.findall(rb'C2_PORTABLE_PROCESS_TRACE:([0-9a-f]+)',stdout.read_bytes())
        if len(traces)!=1: fail('expected one child trace marker: '+case)
        trace_hex=traces[0].decode('ascii'); decoded=bytes.fromhex(trace_hex)
        if digest(trace_hex.encode('ascii'))!=invocation.get('actual_trace_hex_sha256'): fail('child trace hex hash mismatch: '+case)
        expected=invocation.get('expected_trace_sha256')
        if not expected or digest(decoded)!=expected: fail('decoded child trace differs from expected source suffix: '+case)
        old=inherited.get(case)
        if old and (input_hash!=old.get('input_sha256') or expected!=old.get('expected_trace_sha256')):
            fail('inherited child checkpoint/trace changed from accepted result: '+case)
        case_records.append({'case':case,'child_exit_code':'Some(0)','input_file':str(raw.relative_to(ROOT)),'input_sha256':input_hash,'expected_trace_sha256':expected,'actual_trace_hex_sha256':digest(trace_hex.encode('ascii')),'stdout':str(stdout.relative_to(ROOT)),'stdout_sha256':digest(stdout.read_bytes()),'stderr':str(stderr.relative_to(ROOT)),'stderr_sha256':digest(stderr.read_bytes())})
    old_v3={row['case']:row['expected_raw_sha256'] for row in packet['inherited_decoded_v3_trace_proofs']}
    proof_files=sorted(process_root.rglob('decoded-trace-byte-hashes.txt'))
    proofs={}; proof_sources={}; child_trace_hashes={row['case']:row['expected_trace_sha256'] for row in case_records}
    for path in proof_files:
        fields={};rows=[]
        for line in path.read_text().splitlines():
            if line.startswith('case=') and fields: rows.append(fields); fields={}
            key,sep,value=line.partition('=')
            if sep: fields[key]=value
        if fields: rows.append(fields)
        for row in rows:
            case=row.get('case')
            if case in proofs or case not in old_v3: fail('unexpected or duplicate V3 decoded-trace proof')
            if (row.get('expected_raw_sha256')!=old_v3[case]
                    or row.get('actual_decoded_raw_sha256')!=old_v3[case]
                    or child_trace_hashes.get(case)!=old_v3[case]):
                fail('V3 decoded trace proof differs from accepted raw child trace')
            proofs[case]=old_v3[case]
            proof_sources[case]={'path':str(path.relative_to(ROOT)),'sha256':digest(path.read_bytes())}
    if len(proof_files)!=packet['inherited_decoded_v3_trace_proof_files'] or proofs!=old_v3: fail('inherited four V3 decoded-trace proofs/files differ from accepted result')
    baseline=subprocess.run(['git','show',packet['base_commit']+':'+OWNED],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    if baseline.returncode: fail('cannot read inherited baseline source literals')
    source_goldens=literal_golden_map(baseline.stdout)
    if literal_golden_map((ROOT/OWNED).read_bytes()) != source_goldens:
        fail('current source literal golden map differs from bound baseline')
    expected_goldens={row['case']:row['expected_sha256'] for row in packet['inherited_literal_golden_cases']}
    if source_goldens!=expected_goldens: fail('baseline source literal map differs from accepted 19 goldens')
    golden_rows=[]
    for case,expected in sorted(expected_goldens.items()):
        matches=[p for p in (process_root/'legacy-matrix').glob('*.c2') if re.fullmatch(re.escape(case)+r'-\d+-\d+\.c2',p.name)] if (process_root/'legacy-matrix').is_dir() else []
        if len(matches)!=1: fail('expected one captured raw legacy envelope for '+case)
        actual=digest(matches[0].read_bytes())
        if actual!=expected: fail('legacy raw envelope differs from literal '+case)
        golden_rows.append({'case':case,'expected_sha256':expected,'actual_sha256':actual,'raw_envelope':str(matches[0].relative_to(ROOT))})
    return ({'case_records':case_records,'successful_child_cases':33,'unique_case_names':33,'decoded_trace_hash_proofs':[{'case':c,'expected_raw_sha256':h,'actual_decoded_raw_sha256':h,'proof_source_path':proof_sources[c]['path'],'proof_source_sha256':proof_sources[c]['sha256']} for c,h in sorted(proofs.items())],'decoded_v3_trace_proofs':4,'raw_artifacts_verified':len(case_records)+len(proof_files)+len(golden_rows)},
            {'cases':golden_rows,'literal_goldens':19,'matched_raw_envelope_artifacts':19})

def main():
    if len(sys.argv) != 2 or sys.argv[1] not in {*GATES,'finalize'}: fail('choose clippy, tests, format, or finalize')
    gate = sys.argv[1]
    packet = json.loads(PACKET_PATH.read_text())
    packet_hash = digest(PACKET_PATH.read_bytes())
    immutable_binding(packet)
    if packet.get('gate_driver_sha256') != digest(Path(__file__).read_bytes()): fail('gate driver hash differs from immutable packet')
    if packet.get('gate_argv') != GATES: fail('actual Cargo/rustfmt commands differ from packet binding')
    if git('rev-parse','HEAD') != packet['base_commit']: fail('joined HEAD drift')
    for path, expected in packet['input_hashes'].items():
        if not (ROOT/path).is_file() or filehash(path) != expected: fail('bound input drift: '+path)
    prior_acceptance=Path(packet['prior_consumer_acceptance_path'])
    prior_result=Path(packet['prior_consumer_result_path'])
    if (not prior_acceptance.is_file() or digest(prior_acceptance.read_bytes())!=packet['prior_consumer_acceptance_sha256']
            or not prior_result.is_file() or digest(prior_result.read_bytes())!=packet['prior_consumer_result_sha256']): fail('accepted inherited consumer evidence drift')
    lane_statuses={'flow_v4_routing':'accepted_flow_v4_routing_source_only',
                   'transit_v2_direct_wire':'accepted_transit_v2_direct_wire_source_only'}
    for evidence in packet['external_acceptances']:
        ap=Path(evidence['acceptance_path']);rp=Path(evidence['result_path'])
        if (not ap.is_file() or digest(ap.read_bytes())!=evidence['acceptance_sha256']
                or not rp.is_file() or digest(rp.read_bytes())!=evidence['result_sha256']): fail('accepted routing/TransitV2 evidence drift')
        av=json.loads(ap.read_text());rv=json.loads(rp.read_text())
        if (evidence.get('lane') not in lane_statuses
                or evidence.get('expected_status')!=lane_statuses[evidence['lane']]
                or av.get('status')!=lane_statuses[evidence['lane']]
                or av.get('status')!=evidence['status']
                or av.get('accepted_commit')!=evidence['accepted_commit']
                or av.get('final_result_sha256')!=evidence['result_sha256']): fail('source acceptance semantic binding mismatch')
        gate_base=av.get('gate_base_commit')
        if gate_base and rv.get('head')!=gate_base and rv.get('base_commit')!=gate_base: fail('source result does not match acceptance gate base')
        if rv.get('status') not in ('ready_for_review','ready_for_root_review'): fail('source result status is not review-ready')
        if subprocess.run(['git','merge-base','--is-ancestor',evidence['accepted_commit'],packet['base_commit']],cwd=ROOT).returncode: fail('accepted source commit absent from target base')
        for path,expected in evidence['source_hashes'].items():
            tree=subprocess.run(['git','show',evidence['accepted_commit']+':'+path],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
            if tree.returncode or digest(tree.stdout)!=expected or filehash(path)!=expected: fail('accepted source tree mismatch')
        result_sources=(rv.get('source_hashes_after') or rv.get('owned_source_hashes') or rv.get('final_source_hashes') or rv.get('source_hashes'))
        if not result_sources or any(result_sources.get(path)!=value for path,value in evidence['source_hashes'].items()): fail('accepted source result hashes differ from acceptance')
    if not (ROOT/OWNED).is_file(): fail('owned test source missing')
    base = subprocess.run(['git','show',packet['base_commit']+':'+OWNED],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    if base.returncode or digest(base.stdout) != packet['base_source_hashes'][OWNED]: fail('owned base hash mismatch')
    result_path = ROOT/packet['result_path']
    if not result_path.is_file(): fail('worker must record initial path verification before a gate')
    result = json.loads(result_path.read_text())
    initial = result.get('initial_owned_path_verification',{})
    if initial.get('verified') is not True or initial.get('existing_source_hashes') != packet['base_source_hashes'] or initial.get('absent_paths') != []:
        fail('initial owned-path verification missing or mismatched')
    source_scope_check(packet)
    if not (ROOT/OWNED).is_file() or not (ROOT/ART.relative_to(ROOT)).is_dir(): fail('owned paths missing')
    logs = ROOT/packet['logs_path']
    logs.mkdir(parents=True, exist_ok=True)
    started = {p.name[:-len('-started.json')]:p for p in logs.glob('*-started.json')}
    terminal = {p.name[:-len('-result.json')]:p for p in logs.glob('*-result.json')}
    if set(started) != set(terminal): fail('unmatched gate start/result receipt')
    rows=[]
    for stem, path in terminal.items():
        row=json.loads(path.read_text()); start=json.loads(started[stem].read_text())
        if row.get('started_receipt_sha256') != digest(started[stem].read_bytes()): fail('started receipt hash mismatch')
        if start.get('status')!='started' or row.get('status')!='finished': fail('receipt lifecycle status mismatch')
        if any(row.get(k)!=start.get(k) for k in ('gate','argv','cwd','head','packet_sha256','driver_sha256','toolchain','source_hashes','log')): fail('receipt pair immutable fields differ')
        if row.get('packet_sha256')!=packet_hash or row.get('driver_sha256')!=digest(Path(__file__).read_bytes()): fail('receipt is bound to a stale packet or driver')
        if row.get('gate') not in GATES or row.get('argv')!=GATES[row['gate']] or row.get('cwd')!=str(ROOT): fail('gate argv/cwd mismatch')
        if row.get('head')!=packet['base_commit'] or row.get('source_hashes_after')!=row.get('source_hashes'): fail('gate HEAD/source stability mismatch')
        if row.get('source_hashes') != start.get('source_hashes'): fail('gate source preimage mismatch')
        if row.get('toolchain') != {n:subprocess.check_output([str(TOOLCHAIN/n),'--version'],text=True).strip() for n in ('rustc','cargo')}: fail('pinned toolchain mismatch')
        log=ROOT/row['log']
        if not log.is_file() or digest(log.read_bytes())!=row.get('log_sha256'): fail('raw gate log hash mismatch')
        row['receipt_path']=str(path.relative_to(ROOT)); row['receipt_sha256']=digest(path.read_bytes()); rows.append(row)
    rows.sort(key=lambda r:r['start'])
    failed_rows=[r for r in rows if r['exit_code']!=0]
    if len(failed_rows)>=packet['development_failure_limit']: fail('development failure ceiling reached')
    if any(r['command_exit_code']!=0 for r in rows if r['exit_code']==0): fail('passing gate has nonzero command exit')
    expected_failures=[r['receipt_path'] for r in failed_rows]
    if result.get('failed_development_attempts',[]) != expected_failures: fail('result failure list differs from chronological immutable raw receipts')
    authority=authority_check(packet,packet_hash)
    expected_failures,recovery=authorize_recovery(packet,rows,gate,authority)
    if recovery and any(r.get('source_hashes')!=source_hashes() for r in recovery):
        fail('source changed during an authorized gate sequence; restart requires root review')
    if gate=='finalize':
        if len(recovery)!=3 or any(r['exit_code']!=0 for r in recovery): fail('latest authorized ordered attempt must contain three passing gates before finalize')
        final_source=source_hashes()
        if any(r.get('source_hashes')!=final_source or r.get('source_hashes_after')!=final_source for r in recovery): fail('latest gates do not bind identical final source')
        tests=next(r for r in recovery if r['gate']=='tests')
        process_root=Path(tests['process_artifact_dir'])
        process_evidence,goldens=verify_process_and_legacy(packet,process_root)
        final_gates=[]
        for r in recovery:
            final_gates.append(r)
        result.update(status='ready_for_review',ready_for_review=True,qualification=False,base_commit=packet['base_commit'],head=git('rev-parse','HEAD'),packet_sha256=packet_hash,source_hashes=final_source,source_hashes_after=final_source,gates=final_gates,required_success_names=REQUIRED,process_evidence=process_evidence,legacy_golden_parity=goldens,all_receipts=[{'receipt_path':r['receipt_path'],'receipt_sha256':r['receipt_sha256'],'log':r['log'],'log_sha256':r['log_sha256']} for r in rows],failed_development_attempts=expected_failures,historical_failed_receipts=packet['historical_failed_receipts'],historical_failure_count=3,fresh_failure_count=0,cumulative_failure_count=3)
        result_path.write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps({'status':result['status'],'result_path':packet['result_path'],'process_cases':33,'v3_decoded':4,'goldens':19}))
        return 0
    versions={n:subprocess.run([str(TOOLCHAIN/n),'--version'],capture_output=True,text=True) for n in ('rustc','cargo')}
    if any(p.returncode or not p.stdout.startswith(n+' 1.99.0 ') for n,p in versions.items()): fail('Rust/Cargo 1.99.0 unavailable')
    stem=f'{gate}-{time.time_ns()}-{os.getpid()}'
    log=logs/(stem+'.log'); start_path=logs/(stem+'-started.json')
    before=source_hashes(); argv=GATES[gate]
    source_scope_check(packet)
    row={'gate':gate,'argv':argv,'cwd':str(ROOT),'head':git('rev-parse','HEAD'),'packet_sha256':packet_hash,'driver_sha256':digest(Path(__file__).read_bytes()),'start':datetime.datetime.now(datetime.timezone.utc).isoformat(),'toolchain':{n:p.stdout.strip() for n,p in versions.items()},'source_hashes':before,'log':str(log.relative_to(ROOT)),'status':'started'}
    write_new(start_path,row); started_sha=digest(start_path.read_bytes())
    env=dict(os.environ);env.update(PATH=str(TOOLCHAIN)+os.pathsep+env.get('PATH',''),PYTHONDONTWRITEBYTECODE='1',CARGO_HOME='/tmp/c2-route-receipt-cargo-home',CARGO_TARGET_DIR=str(logs/'target'),CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',KAIRO_C2_PROCESS_ARTIFACT_DIR=str(logs/(stem+'-process')))
    Path(env['KAIRO_C2_PROCESS_ARTIFACT_DIR']).mkdir(parents=True,exist_ok=False)
    with log.open('x',encoding='utf-8') as stream: command_exit=subprocess.run(argv,cwd=ROOT,env=env,stdout=stream,stderr=subprocess.STDOUT).returncode
    content=log.read_bytes(); exit_code=command_exit; error=None; observed=[]
    process_verification=None
    if gate=='tests' and exit_code==0:
        observed=[name for name in REQUIRED if re.search(rb'test [^\n]*'+name.encode()+rb' \.\.\. ok',content)]
        if observed != REQUIRED:
            exit_code=2; error='Missing named process/control tests: '+repr([n for n in REQUIRED if n not in observed])
        else:
            try:
                process_verification,golden_verification=verify_process_and_legacy(
                    packet,Path(env['KAIRO_C2_PROCESS_ARTIFACT_DIR']))
            except SystemExit as exc:
                exit_code=2; error='Raw child/V3/golden verification failed: '+str(exc)
            else:
                process_verification={'process':process_verification,'legacy':golden_verification}
    after=source_hashes()
    try: immutable_binding(packet)
    except (SystemExit, OSError, ValueError, subprocess.CalledProcessError) as exc: exit_code=2; error=str(exc)
    if after!=before: exit_code=2; error='Owned source changed during captured gate'
    row.update(status='finished',end=datetime.datetime.now(datetime.timezone.utc).isoformat(),exit_code=exit_code,command_exit_code=command_exit,gate_error=error,log_sha256=digest(content),source_hashes_after=after,started_receipt_sha256=started_sha,required_success_names_observed=observed if gate=='tests' else None,process_verification=process_verification,process_artifact_dir=env['KAIRO_C2_PROCESS_ARTIFACT_DIR'] if gate=='tests' else None)
    receipt=logs/(stem+'-result.json');write_new(receipt,row)
    row['receipt_path']=str(receipt.relative_to(ROOT));row['receipt_sha256']=digest(receipt.read_bytes());rows.append(row)
    test_rows=[r for r in rows if r.get('gate')=='tests']
    observed_tests=test_rows[-1].get('required_success_names_observed',[]) if test_rows else []
    result.update(base_commit=packet['base_commit'],head=git('rev-parse','HEAD'),base_source_hashes=packet['base_source_hashes'],owned_source_hashes=after,gates=rows,required_test_names=REQUIRED,required_success_names=observed_tests,required_success_count=len(observed_tests),status=('stopped_for_root_review' if exit_code else 'in_progress'),qualification=False)
    result['failed_development_attempts']=[r['receipt_path'] for r in rows if r['exit_code']!=0]
    result.update(historical_failed_receipts=packet['historical_failed_receipts'],historical_failure_count=3,fresh_failure_count=len(result['failed_development_attempts']),cumulative_failure_count=3+len(result['failed_development_attempts']))
    # Retain an explicit hash inventory for all raw child stdout/stderr/checkpoint/process records.
    process_root=logs/(stem+'-process'); inventory=[]
    for path in sorted(process_root.rglob('*')):
        if path.is_file(): inventory.append({'path':str(path.relative_to(ROOT)),'bytes':path.stat().st_size,'sha256':digest(path.read_bytes())})
    row['process_artifact_inventory']=inventory
    result_path.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'gate':gate,'exit_code':exit_code,'receipt_path':row['receipt_path'],'log':row['log']}))
    return exit_code
if __name__=='__main__': raise SystemExit(main())
