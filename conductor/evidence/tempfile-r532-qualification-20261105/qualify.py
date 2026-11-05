"""Apache-2.0. Isolated decision probe; never changes parent manifests/runtime."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import resource
import shutil
import signal
import statistics
import subprocess
import time
import tomllib

p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--out',type=Path,required=True)
a=p.parse_args();repo=a.repo.resolve();out=a.out.resolve();evidence=Path(__file__).resolve().parent
packet=json.loads((evidence/'packet.json').read_text());budgets=packet['budgets']
for b in packet['inputs']:assert hashlib.sha256((repo/b['path']).read_bytes()).hexdigest()==b['sha256'],b['path']
subprocess.run(['git','-C',str(repo),'diff','--quiet',packet['source'],'--','Cargo.toml','Cargo.lock','crates','libs/kairos','deny.toml'],check=True)
out.mkdir(exist_ok=False)
report={'source':packet['source'],'execution_checkout':subprocess.check_output(['git','-C',str(repo),'rev-parse','HEAD'],text=True).strip(),'commands':[],'build':{},'compatibility':[],'performance':[],'gates':{}}
def sha(data):return hashlib.sha256(data).hexdigest()
def save(): (out/'report-private.json').write_text(json.dumps(report,indent=2)+'\n')
def run(argv,env=None,cwd=None,preexec=None,expected=(0,)):
    start=time.monotonic();result=subprocess.run([str(x) for x in argv],env=env,cwd=cwd,capture_output=True,text=True,preexec_fn=preexec)
    i=len(report['commands']);(out/f'command-{i}.stdout').write_text(result.stdout);(out/f'command-{i}.stderr').write_text(result.stderr)
    report['commands'].append({'argv':[str(x) for x in argv],'cwd':str(cwd or repo),'exit':result.returncode,'seconds':time.monotonic()-start,'stdout_sha256':sha(result.stdout.encode()),'stderr_sha256':sha(result.stderr.encode())});save()
    assert result.returncode in expected,(i,result.returncode,result.stderr[-1600:]);return result
compiler=subprocess.check_output(['rustup','which','rustc','--toolchain','1.99.1'],text=True).strip();cargo=subprocess.check_output(['rustup','which','cargo','--toolchain','1.99.1'],text=True).strip()
report['rustc']=run([compiler,'-vV']).stdout;report['cargo']=run([cargo,'--version']).stdout
host=next(s.split(': ')[1] for s in report['rustc'].splitlines() if s.startswith('host:'))
parent=tomllib.loads((repo/'Cargo.lock').read_text())['package'];registry={(x['name'],x['version'],x.get('checksum')) for x in parent if x.get('source','').startswith('registry+')}
serde_decl=tomllib.loads((repo/'Cargo.toml').read_text())['workspace']['dependencies']['serde']
serde_version=serde_decl['version'] if isinstance(serde_decl,dict) else serde_decl
manifest='''[workspace]\n[package]\nname="tempfile-r532-probe"\nversion="0.0.0"\nedition="2021"\nrust-version="1.99"\nlicense="Apache-2.0"\n[features]\ncandidate=["dep:tempfile"]\n[dependencies]\nserde={version="=1.0.228",features=["derive"]}\ntempfile={version="=3.27.0",optional=true}\n[[bin]]\nname="cost"\npath="cost.rs"\n[[bin]]\nname="contracts"\npath="contracts.rs"\nrequired-features=["candidate"]\n'''
source=(repo/'crates/careops-ed/src/capture.rs').read_text();start=source.index('pub fn durable_checkpoint_with_sync(');fragment=source[start:source.index('\n}\n',start)+3]
adapter='''use std::{fs::{self,File,OpenOptions},io::{self,Write},path::Path,sync::atomic::{AtomicU64,Ordering}};\ntype Result<T>=std::result::Result<T,io::Error>;\nfn invalid(message:impl std::fmt::Display)->io::Error {io::Error::other(message.to_string())}\nfn output(error:io::Error)->io::Error {error}\n'''
report['checkpoint_fragment']={'source_sha256':sha(fragment.encode()),'adapter_sha256':sha(adapter.encode()),'scope':'Actual production function with io-error adapters, not full ED module/physical durability qualification.'}
bins={};envs={}
for mode in ['baseline','candidate']:
    assert shutil.disk_usage(out).free>=budgets['disk_stop_free_bytes'],'insufficient disk before build'
    root=out/mode;root.mkdir();(root/'Cargo.toml').write_text(manifest.replace('=1.0.228',serde_version));shutil.copyfile(repo/'Cargo.lock',root/'Cargo.lock');(root/'experiment').mkdir()
    shutil.copyfile(repo/'crates/careops-core/src/experiment/io.rs',root/'experiment/baseline.rs');shutil.copyfile(repo/'crates/careops-core/src/experiment/error.rs',root/'experiment/error.rs');shutil.copyfile(evidence/'candidate.rs',root/'experiment/candidate.rs');(root/'experiment/mod.rs').write_text('pub mod error;\npub mod baseline;\n#[cfg(feature="candidate")] pub mod candidate;\n')
    for f in ['cost.rs','contracts.rs']:shutil.copyfile(evidence/f,root/f)
    (root/'checkpoint_seam.rs').write_text(adapter+fragment)
    env=dict(os.environ,RUSTUP_TOOLCHAIN='1.99.1',RUSTC=compiler,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_PROFILE_RELEASE_DEBUG='0',CARGO_TARGET_DIR=str(root/'target'));envs[mode]=env
    metadata=json.loads(run([cargo,'metadata','--offline','--format-version','1','--manifest-path',root/'Cargo.toml','--features','candidate','--filter-platform',host],env=env).stdout)
    lock=tomllib.loads((root/'Cargo.lock').read_text())['package'];closure={(x['name'],x['version'],x.get('checksum')) for x in lock if x.get('source','').startswith('registry+')};assert closure<=registry,closure-registry
    report['registry_closure']=sorted(closure);report['resolved_features']=[{'name':x['name'],'version':x['version'],'features':next((n['features'] for n in metadata['resolve']['nodes'] if n['id']==x['id']),[]),'rust_version':x.get('rust_version'),'license':x.get('license')} for x in metadata['packages']]
    argv=[cargo,'build','--offline','--locked','--release','--manifest-path',root/'Cargo.toml','--bin','cost']+(['--features','candidate'] if mode=='candidate' else [])
    run(argv,env=env);cold=report['commands'][-1]['seconds'];run(argv,env=env);warm=report['commands'][-1]['seconds'];exe=root/'target/release/cost';bins[mode]=exe
    report['build'][mode]={'cold_seconds':cold,'warm_seconds':warm,'binary_bytes':exe.stat().st_size,'binary_sha256':sha(exe.read_bytes())}
    report['gates'][mode+'_cold']=cold<=budgets['cold_fresh_target_seconds_max'];report['gates'][mode+'_warm']=warm<=budgets['warm_build_seconds_max'];save()
root=out/'candidate';run([cargo,'build','--offline','--locked','--release','--manifest-path',root/'Cargo.toml','--features','candidate','--bin','contracts'],env=envs['candidate'])
for mask in [0o022,0o077]:
    cwd=out/f'contracts-{mask:o}';cwd.mkdir();result=run([root/'target/release/contracts',cwd],cwd=cwd,preexec=lambda m=mask:os.umask(m));report['compatibility'].append({'child_umask':oct(mask),'rust_cases':json.loads(result.stdout)})
    for mode,exe in bins.items():
        for existing in [False,True]:
            d=out/f'permissions-{mask:o}-{mode}-{existing}';d.mkdir();file=d/'output';
            if existing:file.write_bytes(b'old');file.chmod(0o600)
            assert run([exe,'once',file,1024],preexec=lambda m=mask:os.umask(m)).stdout.strip()=='ok'
            assert file.read_bytes()==bytes(i%251 for i in range(1024));actual=file.stat().st_mode&0o777;assert actual==0o666&~mask
            report['compatibility'].append({'case':'create-or-overwrite-bytes-permissions','mode':mode,'child_umask':oct(mask),'preexisting':existing,'output_mode':oct(actual),'sha256':sha(file.read_bytes())})
def fault_setup():
    os.umask(0o022);signal.signal(signal.SIGXFSZ,signal.SIG_IGN);resource.setrlimit(resource.RLIMIT_FSIZE,(128,128))
for mode,exe in bins.items():
    d=out/f'write-fault-{mode}';d.mkdir();file=d/'output';file.write_bytes(b'original');result=run([exe,'once',file,1024],preexec=fault_setup);assert result.stdout.strip()=='err:OUTPUT_WRITE_FAILED';assert file.read_bytes()==b'original';scratch=[p for p in d.iterdir() if p!=file]
    assert len(scratch)==(1 if mode=='baseline' else 0)
    report['compatibility'].append({'case':'actual-partial-write-error','mode':mode,'limit_bytes':128,'destination_unchanged':True,'scratch_files':len(scratch),'scratch_bytes':[p.stat().st_size for p in scratch]})
bench=out/'bench';bench.mkdir();file=bench/'output'
for size in budgets['sizes']:
    expected=bytes(i%251 for i in range(size));pilot={}
    for mode in bins:pilot[mode]=json.loads(run([bins[mode],'bench',file,size,16]).stdout)['elapsed_ns']/16
    iterations=max(16,math.ceil(400000000/min(pilot.values())));values={'baseline':[],'candidate':[]};rss={'baseline':[],'candidate':[]};durations={'baseline':[],'candidate':[]}
    for pair in range(budgets['samples_per_size']):
        for mode in (['baseline','candidate'] if pair%2==0 else ['candidate','baseline']):
            result=run(['/usr/bin/time','-l',bins[mode],'bench',file,size,iterations]);m=json.loads(result.stdout);assert m['elapsed_ns']>=budgets['minimum_timed_ns'];assert file.read_bytes()==expected
            values[mode].append(m['elapsed_ns']/m['ops']);durations[mode].append(m['elapsed_ns']);rss[mode].append(int(re.search(r'(\d+)\s+maximum resident set size',result.stderr)[1]))
    b=statistics.median(values['baseline']);c=statistics.median(values['candidate']);ratio=c/b;excess=c-b;passed=ratio<=budgets['large_candidate_median_ratio_max'] or (size==1024 and excess<=budgets['tiny_max_excess_ns'])
    report['performance'].append({'size':size,'iterations':iterations,'pilot_ns_per_write':pilot,'ns_per_write':values,'timed_ns':durations,'peak_rss_bytes':rss,'candidate_median_ratio':ratio,'candidate_excess_ns':excess,'pass':passed,'scope':'Page-cache writes without fsync; no persistence/full-app/portable-speedup claim.'});report['gates']['latency_'+str(size)]=passed;report['gates']['rss_'+str(size)]=max(rss['candidate'])-max(rss['baseline'])<=budgets['peak_rss_growth_bytes_max'];save()
report['gates']['binary_growth']=report['build']['candidate']['binary_bytes']-report['build']['baseline']['binary_bytes']<=budgets['binary_growth_bytes_max'];report['gates']['all_behavior_controls']=True;report['all_frozen_cost_gates_pass']=all(report['gates'].values());save();print(json.dumps({'gates':report['gates'],'cost_pass':report['all_frozen_cost_gates_pass'],'compatibility_records':len(report['compatibility']),'build':report['build'],'ratios':[x['candidate_median_ratio'] for x in report['performance']]},indent=2))
