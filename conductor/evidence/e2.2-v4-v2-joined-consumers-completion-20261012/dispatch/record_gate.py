"""UNBOUND preparation template. Bind and independently review before execution."""
import datetime, hashlib, json, os, re, subprocess, sys, time
from pathlib import Path
PACKET_PATH = Path(__file__).with_name('packet.json')
TOOLCHAIN = Path('/Users/doughnut/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin')
LANE_STATUSES = {
    'routing': 'accepted_flow_v4_routing_source_only',
    'abm': 'accepted_transit_v2_direct_wire_source_only',
    'preview': 'accepted_v4_preview_source_only',
    'coherent': 'accepted_v4_coherent_source_only',
    'portable': 'accepted_v4_portable_source_only',
}
def sha(b): return hashlib.sha256(b).hexdigest()
def require(ok, reason):
    if not ok: raise SystemExit(reason)
def new_json(p, value):
    with p.open('x') as f: json.dump(value, f, indent=2); f.write('\n')
def git(root, *argv): return subprocess.check_output(['git', *argv], cwd=root).decode().strip()
def source_hashes(root, packet): return {s:sha((root/s).read_bytes()) for s in packet['source_paths']}
def versions():
    v={n:subprocess.check_output([str(TOOLCHAIN/n),'--version']).decode().strip() for n in ('rustc','cargo','rustfmt')}
    require(v['rustc'].startswith('rustc 1.99.0 ') and v['cargo'].startswith('cargo 1.99.0 '),'Rust/Cargo1.99 required')
    return v
def lane_source_map(obj):
    return obj.get('source_hashes') or obj.get('owned_source_hashes') or {}
def verify_lane(root, lane, row):
    require(lane in LANE_STATUSES and row.get('expected_acceptance_status'),f'{lane}: acceptance status remains unbound')
    if LANE_STATUSES[lane] is not None:
        require(row['expected_acceptance_status']==LANE_STATUSES[lane],f'{lane}: wrong acceptance status binding')
    else:
        require(row['expected_acceptance_status'].startswith('accepted_'),f'{lane}: must bind an actual accepted V4 status')
    require(row.get('acceptance_path') and row.get('result_path') and row.get('acceptance_sha256') and row.get('final_result_sha256'),f'{lane}: still unbound')
    acceptance=Path(row['acceptance_path']); result=Path(row['result_path'])
    require(sha(acceptance.read_bytes())==row['acceptance_sha256'] and sha(result.read_bytes())==row['final_result_sha256'],f'{lane}: accepted evidence bytes changed')
    accepted=json.loads(acceptance.read_text()); report=json.loads(result.read_text())
    require(accepted.get('status')==row['expected_acceptance_status'] and accepted.get('accepted_commit')==row.get('accepted_commit'),f'{lane}: acceptance semantics mismatch')
    require(accepted.get('final_result_sha256')==row['final_result_sha256'],f'{lane}: final result digest not bound by acceptance')
    require(report.get('status')=='ready_for_review' and report.get('ready_for_review', True) is True,f'{lane}: result is not ready_for_review')
    require(report.get('head') and report.get('base_commit'),f'{lane}: result head/base missing')
    gate_base=accepted.get('gate_base_commit')
    expected_report_head=gate_base or accepted['accepted_commit']
    require(report['head']==expected_report_head and report['base_commit']==expected_report_head,f'{lane}: report head/base does not match accepted gate base')
    accepted_sources=accepted.get('source_hashes') or {}
    report_sources=lane_source_map(report)
    require(accepted_sources and report_sources==accepted_sources,f'{lane}: acceptance/result source maps differ')
    required=accepted.get('required_success_names') or []
    require(required and required==report.get('required_success_names'),f'{lane}: required success names differ')
    gate_rows=report.get('gates') or []
    test_rows=[g for g in gate_rows if g.get('gate')=='tests']
    require(len(test_rows)==1 and test_rows[0].get('exit_code')==0 and test_rows[0].get('command_exit_code')==0,f'{lane}: successful test receipt missing')
    test_row=test_rows[0]
    observed=test_row.get('required_success_names_observed') or test_row.get('required_success_names') or []
    if test_row.get('successful_named_tests') is not None:
        observed_names=set(test_row['successful_named_tests'])
        observed_names.update(test_row.get('required_success_names') or [])
        observed_names.update(test_row.get('required_success_names_observed') or [])
        observed_names.update(n.split('::')[-1] for n in list(observed_names))
    else:
        observed_names=set(observed)
    require(set(required)<=observed_names,f'{lane}: result test receipt does not prove every accepted name')
    require(not test_row.get('failed_named_tests') and not test_row.get('missing_required_success_names'),f'{lane}: result test receipt reports failed/missing names')
    tests=accepted.get('tests') or {}
    require(tests.get('failed')==0 and tests.get('passed',0)>0 and tests.get('ignored',0)>=0,f'{lane}: acceptance test summary is not successful')
    toolchain=accepted.get('toolchain') or {}
    if isinstance(toolchain,str):
        require(lane=='abm' and toolchain=='Rust/Cargo 1.99.0',f'{lane}: unsupported legacy toolchain description')
        toolchain=test_row.get('toolchain') or {}
    require(isinstance(toolchain,dict) and toolchain.get('rustc','').startswith('rustc 1.99.0 ') and toolchain.get('cargo','').startswith('cargo 1.99.0 '),f'{lane}: accepted toolchain is not Rust/Cargo 1.99')
    if report.get('source_hashes_after') is not None:
        require(report['source_hashes_after']==report_sources,f'{lane}: source changed during qualification')
    require(accepted_sources and all(sha((root/p).read_bytes())==h for p,h in accepted_sources.items()),f'{lane}: accepted source differs from joined tree')
    for p,h in accepted_sources.items():
        raw=subprocess.check_output(['git','show',f"{accepted['accepted_commit']}:{p}"],cwd=root)
        require(sha(raw)==h,f'{lane}: accepted commit tree source mismatch for {p}')
    if lane=='portable':
        require('v4' in accepted['status'].lower() and 'v3' not in accepted['status'].lower(), 'Portable predecessor must be V4, not V3')
    return {'status':accepted['status'],'accepted_commit':accepted['accepted_commit'],'gate_base_commit':gate_base,'result_sha256':row['final_result_sha256'],'source_hashes':accepted_sources,'required_success_names':accepted['required_success_names'],'tests':tests,'toolchain':toolchain}
def binding():
    require(PACKET_PATH.is_file(),'Unbound template: no packet.json')
    packet=json.loads(PACKET_PATH.read_text());require(packet.get('status')=='prepared' and packet.get('base_commit') and packet.get('target_worktree'),'Unbound template; root binding required')
    root=Path(packet['target_worktree']); authority=json.loads(Path(packet['dispatch_authority_path']).read_text())
    require(authority.get('status')=='GO' and authority['packet_sha256']==sha(PACKET_PATH.read_bytes()) and authority['driver_sha256']==sha(Path(__file__).read_bytes()) and authority['packet_path']==str(PACKET_PATH) and authority['target_worktree']==str(root) and authority['base_commit']==packet['base_commit'],'Root external authority mismatch')
    require(git(root,'rev-parse','HEAD')==packet['base_commit'] and not git(root,'status','--porcelain'),'Clean exact joined head required')
    require(packet.get('binding_complete') is True,'Unbound preparation template')
    require(len(packet['source_paths'])==16 and len(set(packet['source_paths']))==16,'Expected five consumer + seven routing + four ABM source paths')
    tracked=git(root,'ls-files').splitlines()
    tracked_inputs={p for p in tracked if p.endswith('.rs') or Path(p).name in ('Cargo.toml','Cargo.lock','rust-toolchain.toml')}
    require(tracked_inputs<=set(packet['input_hashes']),'All tracked Rust/manifest inputs must be frozen')
    require(packet['input_hashes'] and all(sha((root/p).read_bytes())==h for p,h in packet['input_hashes'].items()),'Bound input drift')
    lanes=packet.get('accepted_lanes') or {}
    require(set(lanes)==set(LANE_STATUSES),'Five accepted lanes required')
    lane_evidence={name:verify_lane(root,name,row) for name,row in lanes.items()}
    require(lane_evidence['portable']['status'].startswith('accepted_'),'Portable V4 acceptance required; prior V3 evidence is insufficient')
    require(packet['required_success_names'] and len(set(packet['required_success_names']))==len(packet['required_success_names']),'Unbound or duplicate proof inventory')
    hashes=source_hashes(root,packet);require(hashes==packet['expected_source_hashes'],'Source drift')
    for name,evidence in lane_evidence.items():
        require(all(hashes.get(p)==h for p,h in evidence['source_hashes'].items()),f'{name}: accepted source not present at joined head')
    return packet,root,hashes,lane_evidence

def argv_for(packet):
    packages=['-p','kairo-ecs-core','-p','kairo-ecs-des','-p','kairo-ecs-abm','-p','kairo-ecs-calibration','--features','kairo-ecs-calibration/flow']
    return {'tests':[str(TOOLCHAIN/'cargo'),'test','--locked',*packages,'--','--test-threads=1'],
        'clippy':[str(TOOLCHAIN/'cargo'),'clippy','--locked',*packages,'--all-targets','--','-D','warnings'],
        'format':[str(TOOLCHAIN/'rustfmt'),'--edition','2021','--check','--config','skip_children=true',*[p for p in packet['source_paths'] if p.endswith('.rs')] ]}

def summaries(raw, packet):
    text=raw.decode(errors='replace');rows=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
    observed=[n for n in packet['required_success_names'] if re.search(rb'test [^\n]*'+re.escape(n.encode())+rb' \.\.\. ok',raw)]
    return {'passed':sum(int(x[0]) for x in rows),'failed':sum(int(x[1]) for x in rows),'ignored':sum(int(x[2]) for x in rows),'suite_summaries':len(rows),'required_success_names_observed':observed,'missing_suite_markers':[m for m in packet['required_suite_markers'] if m.encode() not in raw]}

def history(out, root, packet, gates, tool):
    starts={p.name.removesuffix('-started.json'):p for p in out.glob('*-started.json')};ends={p.name.removesuffix('-result.json'):p for p in out.glob('*-result.json')}
    require(starts.keys()==ends.keys(),'Unmatched started/terminal receipt; reconcile')
    rows=[]
    for stem,path in ends.items():
        a=json.loads(starts[stem].read_text());b=json.loads(path.read_text());g=b['gate'];require(g in gates,'Unexpected gate')
        for k in ('gate','argv','cwd','head','packet_sha256','start','toolchain','source_hashes','log'):require(a[k]==b[k],'Receipt pair mismatch')
        require(a['status']=='started' and b['status']=='finished' and b['argv']==gates[g] and b['cwd']==str(root) and b['head']==packet['base_commit'] and b['packet_sha256']==sha(PACKET_PATH.read_bytes()) and b['toolchain']==tool,'Receipt binding mismatch')
        require(b['exit_code']!=0 or b['command_exit_code']==0,'Passing receipt requires actual command exit0')
        require(b['source_hashes']==b['source_hashes_after'] and set(b['source_hashes'])==set(packet['source_paths']),'Source changed during gate')
        require(b['started_sha256']==sha(starts[stem].read_bytes()),'Started receipt hash mismatch')
        require(str(b['log']).startswith(packet['logs_prefix']+'/'),'Log path outside scope')
        raw=(root/b['log']).read_bytes();require(sha(raw)==b['log_sha256'],'Raw log hash mismatch')
        if g=='tests':require(b['test_summary']==summaries(raw,packet),'Observed tests disagree with raw log')
        require(b['end']>=b['start'],'Invalid receipt chronology')
        rows.append({**b,'receipt_path':str(path.relative_to(root)),'receipt_sha256':sha(path.read_bytes())})
    rows.sort(key=lambda r:r['start'])
    for a,b in zip(rows,rows[1:]):require(a['end']<=b['start'],'Overlapping final gates')
    return rows

def raw_evidence(root, test, packet):
    parser=packet['portable_parser']
    path=Path(parser['path']); raw=path.read_bytes()
    require(sha(raw)==parser['sha256'],'Accepted portable parser drift')
    ns={'__name__':'_final_parser_only','__file__':str(path)}
    exec(compile(raw,str(path),'exec'),ns)
    ns['ROOT']=root
    inherited=packet['portable_evidence_contract']
    process=(root/test['log']).parent/(Path(test['log']).stem+'-process')
    result=ns['verify_process_and_legacy'](inherited,process)
    require(result[0]['successful_child_cases']==33 and result[0]['decoded_v3_trace_proofs']==4 and result[1]['literal_goldens']==19,'Exact33/4/19 raw proof required')
    return result

def main():
    require(len(sys.argv)==2 and sys.argv[1] in ('tests','clippy','format','finalize'),'Choose tests, clippy, format or finalize')
    action=sys.argv[1];packet,root,current,lane_evidence=binding();gates=argv_for(packet);tool=versions();out=root/packet['logs_prefix'];out.mkdir(parents=True,exist_ok=True);rows=history(out,root,packet,gates,tool)
    if action=='finalize':
        failures=[r for r in rows if r['exit_code']!=0]
        if failures:
            new_json(root/packet['result_path'],{'status':'stopped_final_failure','head':packet['base_commit'],'gates':rows,'scope':'No acceptance; failed raw receipts preserved'});return 0
        require([r['gate'] for r in rows]==['tests','clippy','format'] and all(r['source_hashes']==current for r in rows),'Exactly three ordered passing gates at current sources required')
        require(set(packet['required_success_names'])<=set(rows[0]['test_summary']['required_success_names_observed']),'Missing required proofs')
        process,golden=raw_evidence(root,rows[0],packet)
        new_json(root/packet['result_path'],{'status':'ready_for_root_acceptance_review','packet_id':packet['packet_id'],'packet_sha256':sha(PACKET_PATH.read_bytes()),'head':packet['base_commit'],'commit':packet['base_commit'],'worktree':str(root),'toolchain':tool,'input_hashes':packet['input_hashes'],'input_hashes_verified_count':len(packet['input_hashes']),'protected_sources':current,'accepted_lane_evidence':lane_evidence,'gates':rows,'test_summary':rows[0]['test_summary'],'process_evidence':process,'legacy_golden_parity':golden,'scope':'Joined FlowV4/TransitV2 consumer qualification only; parent pin/ED/runtime release remain separate'});return 0
    require(not any(r['exit_code']!=0 for r in rows),'A final gate failed; no rerun or correction in this packet')
    order=['tests','clippy','format'];require(len(rows)<3 and action==order[len(rows)] and all(r['source_hashes']==current for r in rows),'Wrong gate order or source drift')
    stem=f'{action}-{time.time_ns()}-{os.getpid()}';log=out/(stem+'.log');started=out/(stem+'-started.json')
    row={'gate':action,'argv':gates[action],'cwd':str(root),'head':packet['base_commit'],'packet_sha256':sha(PACKET_PATH.read_bytes()),'start':datetime.datetime.now(datetime.timezone.utc).isoformat(),'toolchain':tool,'source_hashes':current,'log':str(log.relative_to(root)),'status':'started'};new_json(started,row)
    env=dict(os.environ);env.update(PATH=str(TOOLCHAIN)+os.pathsep+env.get('PATH',''),PYTHONDONTWRITEBYTECODE='1',CARGO_HOME='/tmp/c2-route-receipt-cargo-home',CARGO_TARGET_DIR=str(out/'target'),CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0');env['KAIRO_C2_PROCESS_ARTIFACT_DIR']=str(out/(stem+'-process'))
    with log.open('x') as f:actual=subprocess.run(gates[action],cwd=root,env=env,stdout=f,stderr=subprocess.STDOUT).returncode
    raw=log.read_bytes();after=source_hashes(root,packet);code=actual;error=None;summary=summaries(raw,packet) if action=='tests' else None
    if after!=current:code=2;error='Source changed during final gate'
    if action=='tests' and actual==0 and (set(summary['required_success_names_observed'])!=set(packet['required_success_names']) or summary['failed'] or summary['missing_suite_markers']):code=2;error='Missing required proofs or failed test summary'
    try:binding()
    except SystemExit:code=2;error='Post-gate binding integrity failure'
    row.update(status='finished',end=datetime.datetime.now(datetime.timezone.utc).isoformat(),exit_code=code,command_exit_code=actual,source_hashes_after=after,log_sha256=sha(raw),started_sha256=sha(started.read_bytes()),test_summary=summary,gate_error=error);terminal=out/(stem+'-result.json');new_json(terminal,row);print(json.dumps({'exit_code':code,'receipt_path':str(terminal.relative_to(root)),'log_sha256':row['log_sha256']}));return code
if __name__=='__main__':raise SystemExit(main())
