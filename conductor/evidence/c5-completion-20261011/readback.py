"""Bounded artifact readback; no runtime execution or acceptance authority."""
import hashlib, json, pathlib, re, subprocess, sys
repo=pathlib.Path(sys.argv[1]).resolve(); hosted=pathlib.Path(sys.argv[2]); output=pathlib.Path(sys.argv[3])
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def git(ref): return subprocess.check_output(['git','-C',str(repo),'rev-parse',ref],text=True).strip()
def counts(p):
 rows=[tuple(map(int,r)) for r in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',p.read_text())]
 assert rows, p
 return dict(zip(['passed','failed','ignored'],map(sum,zip(*rows))))
def inventory(p): return {str(f.relative_to(p)):sha(f) for f in sorted(p.rglob('*')) if f.is_file()}
head=git('HEAD'); tree=git('HEAD^{tree}'); report={'head':head,'tree':tree,'hosts':{}}; reference=None; summaries=[]
for host in ['ubuntu-24.04','macos-15']:
 d=hosted/('c5-native-'+host); p=d/'process'; r=json.loads((p/'result.json').read_text()); provenance=r['provenance']
 assert r['status']=='pass' and r['source_tree']==tree and not r['source_dirty']
 assert (d/'source-tree.log').read_text().strip()==tree
 assert (d/'source.log').read_text().strip()==r['source_commit']==provenance['source_commit']
 assert provenance['source_dirty']=='false'
 assert 'rustc 1.99.0 ' in provenance['toolchain'] and 'release: 1.99.0' in (d/'toolchain.log').read_text()
 for line in (d/'source.sha256').read_text().splitlines():
  digest, name=line.split(maxsplit=1); assert sha(repo/name)==digest,name
 assert provenance['cargo_lock_sha256']==sha(repo/'Cargo.lock')
 assert provenance['study_sha256']==r['study_sha256']==sha(repo/'examples/calibration/c5/study.json')
 assert provenance['executable_sha256']==r['binary_sha256']
 assert r['workers']==[1,2,4]
 assert r['cooperative_resume']==r['hard_kill_resume']=='byte_identical'
 for command in r['commands']:
  assert command['exit']==(-9 if command['label']=='hard-kill-after-durable-shard' else 0)
 baseline=inventory(p/'w1'); assert len(baseline)==60
 for name in ['w2','w4','resume','killed']: assert inventory(p/name)==baseline,name
 summary=json.loads((p/'w1/summary.json').read_text()); assert summary==r['summary']
 assert sha(p/'w1/summary.json')==r['summary_sha256']
 assert summary['winner']=='c000004' and summary['complete'] and summary['winner_frozen'] and summary['mc_indeterminate']
 winner=next(c for c in summary['candidates'] if c['candidate_id']==summary['winner'])
 assert winner['params']=={'route_ticks':2,'work_ticks':3} and winner['validation_objective_mean']==0
 final=summary['final_test']; assert final['completed_count']==final['attempt_count']==2
 assert final['failed_count']==final['censored_count']==final['mean_completion_error']==0
 results={str(f.relative_to(p/'w1')):json.loads(f.read_text())['result'] for f in (p/'w1').rglob('task-*.json')}
 assert len(results)==56
 if reference is None: reference=results
 else: assert reference==results
 summaries.append(r['summary_sha256'])
 report['hosts'][host]={'source_commit':r['source_commit'],'source_tree':tree,'source_hashes_match':True,'study_native':counts(d/'study-native-tests.log'),'cli':counts(d/'cli-tests.log'),'summary_sha256':r['summary_sha256'],'bundle_files':60,'shards':56,'whole_bundle_worker_resume_equality':True,'hard_kill_exit':-9,'resume_exit':0,'winner':summary['winner'],'winner_params':winner['params'],'final':final,'toolchain':(d/'toolchain.log').read_text(),'binary_hash_receipt':r['binary_sha256']}
assert len(set(summaries))==1
report.update(status='pass',cross_host_summary_equal=True,cross_host_task_results_equal=True,limits='Executable hashes are receipt values; hosted binaries were not downloaded. Whole bundles are compared within each host; architecture-specific provenance differs across hosts. Synthetic fixture only; no speedup or clinical claim.')
output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'status':'pass','head':head,'hosts':list(report['hosts'])}))
