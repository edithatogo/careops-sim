"""Four-path direct wire ABM gate receipt driver."""
import datetime, hashlib, json, os, re, subprocess, sys, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
ART=ROOT/'.artifacts/e22-transit-v2-direct-wire-20261012'
PACKET_PATH=ART/'dispatch/packet.json'
TOOLCHAIN=Path('/Users/doughnut/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin')
OWNED=['crates/kairo-ecs-abm/src/flow_adapter/transit_checkpoint_wire.rs', 'crates/kairo-ecs-abm/src/flow_adapter.rs', 'crates/kairo-ecs-abm/src/lib.rs', 'crates/kairo-ecs-abm/tests/transit_context_checkpoint_wire_v2.rs']
REQUIRED=['transit_v2_wire_actual_ready_moving_roundtrip', 'transit_v2_wire_paused_resume_suffix', 'transit_v2_wire_arrived_binding_and_suppression', 'transit_v2_wire_resume_witness_live_stale_noop', 'transit_v2_wire_literal_bytes_and_v1_compatibility', 'transit_v2_wire_exact_total_budget_real_rebind', 'transit_v2_wire_count_text_and_checked_size_limits', 'transit_v2_wire_malformed_tags_truncation_and_trailing', 'transit_v2_wire_owner_reference_ticket_rejection', 'transit_v2_wire_codec_schema_and_remaining_budget']
GATE_ORDER=('clippy','tests','format')
GATES={
 'clippy':[str(TOOLCHAIN/'cargo'),'clippy','--locked','-p','kairo-ecs-abm','--all-targets','--','-D','warnings'],
 'tests':[str(TOOLCHAIN/'cargo'),'test','--locked','-p','kairo-ecs-abm','--','--test-threads=1'],
 'format':[str(TOOLCHAIN/'rustfmt'),'--edition','2021','--check','--config','skip_children=true',*OWNED],
}
def h(b): return hashlib.sha256(b).hexdigest()
def git(*args):
 p=subprocess.run(['git',*args],cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 if p.returncode: raise SystemExit('git verification failed')
 return p.stdout.strip()
def source_hashes(): return {p:h((ROOT/p).read_bytes()) if (ROOT/p).is_file() else None for p in OWNED}
def write_new(path,value):
 with path.open('x',encoding='utf-8') as f: json.dump(value,f,indent=2); f.write('\n')
def raw_receipts(logs, packet, packet_hash):
 started={p.name[:-len('-started.json')]:p for p in logs.glob('*-started.json')}
 results=[]
 for path in sorted(logs.glob('*-result.json')):
  stem=path.name[:-len('-result.json')]
  if stem not in started: raise SystemExit('terminal receipt has no matching started receipt')
  row=json.loads(path.read_text());start=json.loads(started[stem].read_text())
  if row.get('status')!='finished' or row.get('packet_sha256')!=packet_hash or row.get('head')!=git('rev-parse','HEAD'):raise SystemExit('raw terminal receipt malformed or bound to another packet/head')
  if row.get('started_receipt_path')!=str(started[stem].relative_to(ROOT)) or start.get('gate')!=row.get('gate'):raise SystemExit('started/result receipt pair mismatch')
  start_sha=h(started[stem].read_bytes())
  if row.get('started_receipt_sha256')!=start_sha:raise SystemExit('terminal receipt does not bind its started-receipt bytes')
  common=('gate','argv','cwd','head','packet_sha256','toolchain','source_hashes','log','started_receipt_path')
  if any(row.get(k)!=start.get(k) for k in common):raise SystemExit('started/result immutable command bindings differ')
  if row.get('gate') not in GATES or row.get('argv')!=GATES[row['gate']] or row.get('cwd')!=str(ROOT):raise SystemExit('receipt command differs from immutable gate contract')
  if row.get('head')!=packet['base_commit'] or set(row.get('source_hashes',{}))!=set(OWNED):raise SystemExit('receipt HEAD/source set differs from packet')
  if row.get('source_hashes_after')!=row.get('source_hashes'):raise SystemExit('source changed during captured gate')
  versions=row.get('toolchain',{})
  if not versions.get('rustc','').startswith('rustc 1.99.0 ') or not versions.get('cargo','').startswith('cargo 1.99.0 '):raise SystemExit('receipt toolchain differs from pinned Rust/Cargo 1.99')
  log=ROOT/row['log']
  if not log.is_file() or h(log.read_bytes())!=row.get('log_sha256'):raise SystemExit('raw gate log missing or hash mismatch')
  row['receipt_path']=str(path.relative_to(ROOT));row['receipt_sha256']=h(path.read_bytes());results.append(row)
 if set(started)-{p.name[:-len('-result.json')] for p in logs.glob('*-result.json')}:raise SystemExit('unmatched started receipt; reconcile before another gate')
 results.sort(key=lambda row:row.get('start',''))
 return results
def dispatch_authority(packet, packet_hash, failures):
 authority_path=Path(packet['dispatch_authority_path'])
 if not authority_path.is_file():raise SystemExit('root dispatch authority is absent; source gates are not authorized')
 try: authority=json.loads(authority_path.read_text())
 except Exception: raise SystemExit('root dispatch authority JSON is invalid')
 driver_hash=h((ART/'dispatch/record_gate.py').read_bytes())
 expected={'status':'GO','packet_sha256':packet_hash,'driver_sha256':driver_hash,
           'worktree':str(ROOT),'base_commit':packet['base_commit']}
 if any(authority.get(k)!=v for k,v in expected.items()):raise SystemExit('root dispatch authority does not bind this exact packet, driver, worktree and base')
 approved=authority.get('repair_authorized_after_failures',[])
 if failures:
  if authority.get('repair_authorized') is not True or sorted(approved)!=sorted(failures):raise SystemExit('failed gate requires fresh root GO authorizing the exact failed receipt paths')
 elif authority.get('repair_authorized') is True or approved:raise SystemExit('repair authorization supplied without a recorded failed gate')
 return authority
def next_gate(rows, failures):
 if failures:
  failed_paths=set(failures); last=max(i for i,r in enumerate(rows) if r.get('receipt_path') in failed_paths)
  recovery=rows[last+1:]
  if any(r.get('exit_code')!=0 for r in recovery):raise SystemExit('new failure requires root review before another gate')
  if [r.get('gate') for r in recovery]!=list(GATE_ORDER[:len(recovery)]):raise SystemExit('authorized repair sequence must restart at Clippy and follow gate order')
  if len(recovery)>=len(GATE_ORDER):raise SystemExit('all gates already run after authorized repair')
  return GATE_ORDER[len(recovery)]
 if [r.get('gate') for r in rows]!=list(GATE_ORDER[:len(rows)]) or any(r.get('exit_code')!=0 for r in rows):
  raise SystemExit('gate order is invalid or a failed gate awaits root review')
 if len(rows)>=len(GATE_ORDER):raise SystemExit('all three gates already have terminal receipts')
 return GATE_ORDER[len(rows)]
def readiness(rows,current):
 last={}
 for i,row in enumerate(rows):
  if row.get('status')=='finished':last[row.get('gate')]=(i,row)
 if any(g not in last for g in ('clippy','tests','format')):return False
 chosen=[last[g] for g in ('clippy','tests','format')]
 if any(row.get('exit_code')!=0 for _,row in chosen):return False
 hashes=[entry[1].get('source_hashes') for entry in chosen]
 if not(chosen[0][0]<chosen[1][0]<chosen[2][0] and chosen[2][0]==len(rows)-1):return False
 return all(x==current and row.get('source_hashes_after')==current for x,(_,row) in zip(hashes,chosen)) and set(REQUIRED)<=set(chosen[1][1].get('required_success_names_observed',[]))
def main():
 if len(sys.argv)!=2 or sys.argv[1] not in GATES:raise SystemExit('choose clippy, tests, or format')
 gate=sys.argv[1];packet=json.loads(PACKET_PATH.read_text());packet_hash=h(PACKET_PATH.read_bytes());result_path=ROOT/packet['result_path']
 if git('rev-parse','HEAD')!=packet['base_commit']:raise SystemExit('frozen HEAD drift; stop')
 for path,expected in packet['protected_production_hashes'].items():
  if h((ROOT/path).read_bytes())!=expected:raise SystemExit('protected production drift; stop')
 for path,expected in packet['historical_evidence_hashes'].items():
  bound=Path(path) if Path(path).is_absolute() else ROOT/path
  if not bound.is_file() or h(bound.read_bytes())!=expected:raise SystemExit('original historical evidence drift; stop')
 for path,expected in packet['input_hashes'].items():
  if path in OWNED:continue
  p=ROOT/path
  if not p.is_file() or h(p.read_bytes())!=expected:raise SystemExit('bound input drift; stop')
 external=packet['external_evidence'];acceptance=Path(external['parent_acceptance_path'])
 if not acceptance.is_file() or h(acceptance.read_bytes())!=external['parent_acceptance_sha256']:raise SystemExit('durable native acceptance evidence drift; stop')
 accepted=json.loads(acceptance.read_text())
 if accepted.get('status')!=external['native_acceptance_status'] or accepted.get('final_result_sha256')!=external['joined_native_result_sha256'] or accepted.get('accepted_commit')!=external['native_accepted_commit']:raise SystemExit('durable native acceptance does not bind the joined result/head')
 joined=Path(external['joined_native_result_path'])
 if not joined.is_file() or h(joined.read_bytes())!=external['joined_native_result_sha256']:raise SystemExit('joined native qualification result drift; stop')
 joined_result=json.loads(joined.read_text())
 if joined_result.get('status')!='ready_for_review' or joined_result.get('head')!=external['joined_native_head']:raise SystemExit('joined native qualification result status/head mismatch')
 parent=Path('/Users/doughnut/Documents/careops-sim-session-worktrees/c2-implementation-20261006')
 pin=git('-C',str(parent),'ls-tree','HEAD','libs/kairos').split()
 if len(pin)<3 or pin[2]!=external['parent_pin']:raise SystemExit('parent Kairos pin differs from packet-bound native prerequisite')
 if not result_path.is_file():raise SystemExit('worker must record initial path/hash verification in result.json before source edits')
 result=json.loads(result_path.read_text());initial=result.get('initial_owned_path_verification',{})
 if initial.get('existing_source_hashes')!=packet['base_source_hashes'] or initial.get('absent_paths')!=packet['initially_absent_owned_paths'] or initial.get('verified') is not True:raise SystemExit('worker initial ownership verification missing or mismatched')
 for path,expected in packet['base_source_hashes'].items():
  base=subprocess.run(['git','show',packet['base_commit']+':'+path],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  if expected is None:
   if base.returncode==0:raise SystemExit('new wire test unexpectedly exists in base')
  elif base.returncode or h(base.stdout)!=expected:raise SystemExit('packet base source hash mismatch; stop')
 if any(not(ROOT/p).is_file() for p in OWNED):raise SystemExit('all four owned Rust paths must exist before a gate')
 allowed=set(OWNED)|{packet['result_path']};logs=ART/'logs';logs.mkdir(parents=True,exist_ok=True)
 if git('status','--porcelain'):
  changed=set(filter(None,git('diff','--name-only','-z','HEAD').split('\x00')))|set(filter(None,git('ls-files','--others','--exclude-standard','-z').split('\x00')))
  if any(p not in allowed and not p.startswith(str(logs.relative_to(ROOT))+'/') for p in changed):raise SystemExit('out-of-scope source changes; stop')
 rows=raw_receipts(logs,packet,packet_hash);failures=[r['receipt_path'] for r in rows if r.get('exit_code')!=0]
 dispatch_authority(packet,packet_hash,failures)
 if gate!=next_gate(rows,failures):raise SystemExit('gate call violates frozen Clippy → tests → format order; stop for root review')
 if len(failures)>=packet['development_failure_limit']:raise SystemExit('failed development execution ceiling reached')
 versions={n:subprocess.run([str(TOOLCHAIN/n),'--version'],text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE) for n in ('rustc','cargo')}
 if any(p.returncode or not p.stdout.startswith(n+' 1.99.0 ') for n,p in versions.items()):raise SystemExit('Rust/Cargo 1.99.0 unavailable')
 stem=f'{gate}-{time.time_ns()}-{os.getpid()}';log=logs/(stem+'.log');before=source_hashes()
 row={'gate':gate,'argv':GATES[gate],'cwd':str(ROOT),'head':git('rev-parse','HEAD'),'packet_sha256':packet_hash,'start':datetime.datetime.now(datetime.timezone.utc).isoformat(),'toolchain':{k:p.stdout.strip() for k,p in versions.items()},'source_hashes':before,'log':str(log.relative_to(ROOT)),'status':'started','started_receipt_path':str((logs/(stem+'-started.json')).relative_to(ROOT))}
 started_path=logs/(stem+'-started.json');write_new(started_path,row);started_sha=h(started_path.read_bytes())
 env=dict(os.environ);env.update(PATH=str(TOOLCHAIN)+os.pathsep+env.get('PATH',''),PYTHONDONTWRITEBYTECODE='1',CARGO_HOME='/tmp/c2-route-receipt-cargo-home',CARGO_TARGET_DIR=str(logs/'target'),CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0')
 with log.open('x',encoding='utf-8') as stream:command_exit=subprocess.run(GATES[gate],cwd=ROOT,env=env,stdout=stream,stderr=subprocess.STDOUT).returncode
 content=log.read_bytes();exit_code=command_exit;error=None;observed=[]
 if gate=='tests':
  observed=[n for n in REQUIRED if re.search(rb'test [^\n]*'+n.encode()+rb' \.\.\. ok',content)]
  if exit_code==0 and len(observed)!=len(REQUIRED):exit_code=2;error='Missing required successful tests: '+','.join(sorted(set(REQUIRED)-set(observed)))
 after=source_hashes()
 if after!=before:exit_code=2;error='Owned source changed during captured command'
 row.update(status='finished',end=datetime.datetime.now(datetime.timezone.utc).isoformat(),exit_code=exit_code,command_exit_code=command_exit,gate_error=error,log_sha256=h(content),source_hashes_after=after,started_receipt_sha256=started_sha,required_success_names_observed=observed if gate=='tests' else None,required_success_count_observed=len(observed) if gate=='tests' else None)
 receipt=logs/(stem+'-result.json');write_new(receipt,row);row['receipt_path']=str(receipt.relative_to(ROOT));row['receipt_sha256']=h(receipt.read_bytes());rows.append(row)
 result['base_commit']=packet['base_commit'];result['head']=git('rev-parse','HEAD');result['base_source_hashes']=packet['base_source_hashes'];result['owned_source_hashes']=after;result['required_test_names']=REQUIRED;result['gates']=rows;result['failed_development_attempts']=failures+([row['receipt_path']] if exit_code!=0 else [])
 test_rows=[r for r in rows if r.get('gate')=='tests'];latest=test_rows[-1] if test_rows else {};result['required_success_names']=latest.get('required_success_names_observed') or [];result['required_success_count']=len(result['required_success_names'])
 result['status']='ready_for_review' if readiness(rows,after) else ('stopped_failure_ceiling' if len(result['failed_development_attempts'])>=packet['development_failure_limit'] else ('stopped_for_root_review' if exit_code!=0 else 'in_progress'));result_path.write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({'gate':gate,'exit_code':exit_code,'receipt_path':row['receipt_path'],'log':row['log']}));return exit_code
if __name__=='__main__':raise SystemExit(main())
