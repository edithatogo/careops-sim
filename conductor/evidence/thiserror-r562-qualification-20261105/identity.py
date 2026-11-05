"""Apache-2.0. Actual ED build script and independent framed-hash transcript."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--graph',type=Path,required=True)
a=p.parse_args();repo=a.repo.resolve();out=a.out.resolve();out.mkdir(exist_ok=False)
packet=json.loads((Path(__file__).parent/'packet.json').read_text())
for b in packet['inputs']:assert hashlib.sha256((repo/b['path']).read_bytes()).hexdigest()==b['sha256']
compiler=subprocess.check_output(['rustup','which','rustc','--toolchain','1.99.1'],text=True).strip();version=subprocess.check_output([compiler,'-vV'])
cargo=subprocess.check_output(['rustup','which','cargo','--toolchain','1.99.1'],text=True).strip()
build=out/'buildscript';build.mkdir();(build/'Cargo.toml').write_text('[workspace]\n[package]\nname="thiserror-r562-identity"\nversion="0.0.0"\nedition="2021"\nlicense="Apache-2.0"\n[dependencies]\nsha2="=0.11.1"\n[[bin]]\nname="identity"\npath="identity.rs"\n')
shutil.copyfile(repo/'crates/careops-ed/build.rs',build/'identity.rs');shutil.copyfile(repo/'Cargo.lock',build/'Cargo.lock')
context=dict(os.environ,RUSTUP_TOOLCHAIN='1.99.1',RUSTC=compiler,CARGO_TARGET_DIR=str(build/'target'),CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_PROFILE_RELEASE_DEBUG='0')
for label,argv in [('metadata',[cargo,'metadata','--offline','--format-version','1','--manifest-path',str(build/'Cargo.toml')]),('compile',[cargo,'build','--offline','--locked','--release','--manifest-path',str(build/'Cargo.toml')])]:
 p=subprocess.run(argv,env=context,capture_output=True,text=True);(out/(label+'.stdout')).write_text(p.stdout);(out/(label+'.stderr')).write_text(p.stderr);assert p.returncode==0,p.stderr
 if label=='metadata':
  parent={(p['name'],p['version'],p.get('checksum')) for p in tomllib.loads((repo/'Cargo.lock').read_text())['package'] if p.get('source','').startswith('registry+')}
  private={(p['name'],p['version'],p.get('checksum')) for p in tomllib.loads((build/'Cargo.lock').read_text())['package'] if p.get('source','').startswith('registry+')};assert private<=parent
exe=build/'target/release/identity';sha2=list((build/'target/release/deps').glob('libsha2-*.rlib'));assert len(sha2)==1
roots=[f'crates/{x}' for x in ['careops-core','careops-ed','careops-ed-cli']]+[f'libs/kairos/crates/kairo-ecs-{x}' for x in ['types','rng','core','des','state','abm']]
files=['Cargo.toml','Cargo.lock','crates/careops-ed/build.rs']
for root in roots:files.append(root+'/Cargo.toml');files.extend(str(x.relative_to(repo)) for x in (repo/root/'src').rglob('*.rs'))
files=sorted(files,key=lambda name:Path(name).parts);fixture=out/'source-fixture'
for name in files:
 dest=fixture/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(repo/name,dest)
cfg=subprocess.check_output([compiler,'--print','cfg'],text=True);target_features=','.join(sorted(x.split('"')[1] for x in cfg.splitlines() if x.startswith('target_feature=')))
env={k:v for k,v in os.environ.items() if not k.startswith('CARGO_FEATURE_')};env.update(RUSTC=compiler,TARGET=next(x.split(': ')[1] for x in version.decode().splitlines() if x.startswith('host:')),PROFILE='release',OPT_LEVEL='3',DEBUG='false',CARGO_ENCODED_RUSTFLAGS='',CARGO_CFG_TARGET_FEATURE=target_features,CARGO_FEATURE_DEFAULT='1',CARGO_FEATURE_NATIVE_DATA='1')
fields=['TARGET','PROFILE','OPT_LEVEL','DEBUG','CARGO_ENCODED_RUSTFLAGS','CARGO_CFG_TARGET_FEATURE']
report={'source':packet['source'],'build_script_source_sha256':hashlib.sha256((repo/'crates/careops-ed/build.rs').read_bytes()).hexdigest(),'actual_executable_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'sha2_rlib_sha256':hashlib.sha256(sha2[0].read_bytes()).hexdigest(),'source_input_bindings':[{'path':name,'sha256':hashlib.sha256((repo/name).read_bytes()).hexdigest()} for name in files],'cases':[],'scope':'Actual source-bound build.rs and exact locked SHA2 closure freshly compiled by the probe compiler; independent Python transcript. No full ED app build or new candidate artifact read/resume proof.'}
report['compiler_version_stdout']=version.decode()
report['controlled_environment']={k:v for k,v in env.items() if k in fields or k.startswith('CARGO_FEATURE_')}
report['reproducer_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
def framed(d,data):d.update(len(data).to_bytes(8,'little'));d.update(data)
def expected(tree,context):
 d=hashlib.sha256(b'careops.ed.model-build.v1\0')
 for name in files:framed(d,name.encode());framed(d,(tree/name).read_bytes())
 framed(d,subprocess.check_output([context['RUSTC'],'-vV']))
 for name in fields:framed(d,name.encode());framed(d,context.get(name,'').encode())
 for key,value in sorted(context.items()):
  if key.startswith('CARGO_FEATURE_'):framed(d,key.encode());framed(d,value.encode())
 return d.hexdigest()
def check(label,tree,context=None):
 context=dict(env if context is None else context,CARGO_MANIFEST_DIR=str(tree/'crates/careops-ed'));p=subprocess.run([str(exe)],env=context,capture_output=True,text=True);(out/(label+'.stdout')).write_text(p.stdout);(out/(label+'.stderr')).write_text(p.stderr);assert p.returncode==0,p.stderr
 actual=next(line.split('=',2)[2] for line in p.stdout.splitlines() if line.startswith('cargo:rustc-env=CAREOPS_ED_MODEL_CONTRACT_SHA256='));oracle=expected(tree,context);assert actual==oracle
 report['cases'].append({'label':label,'actual':actual,'independent':oracle,'exit':p.returncode,'stdout_sha256':hashlib.sha256(p.stdout.encode()).hexdigest()});return actual
lock=fixture/'Cargo.lock';original_lock=lock.read_bytes();graph_report=json.loads((a.graph/'graph-report.json').read_text());assert graph_report['all_variants_pass'];report['graph_report_sha256']=hashlib.sha256((a.graph/'graph-report.json').read_bytes()).hexdigest()

base=check('baseline',fixture)
relocated=out/'relocated';shutil.copytree(fixture,relocated);assert check('relocation',relocated)==base
(fixture/'README.md').write_text('Documentation is excluded from runtime identity.');assert check('documentation-only',fixture)==base
originals={}
prepared=(a.graph.parent/'thiserror-r562-prepare-v1/candidate.rs').read_text()
import re
report['candidate_source_bindings']=[]
for item in packet['inputs']:
 name=item['type'];path=fixture/item['path'];original=path.read_text();originals[item['path']]=original
 def region(text,start):return text[start:text.index('\n}',start)+2]
 start=original.rfind('#[derive(',0,original.index('pub enum '+name));old_enum=region(original,start)
 cs=prepared.rfind('#[derive(',0,prepared.index('pub enum '+name));new_enum=region(prepared,cs)
 display=re.search(r'impl (?:std::fmt::)?Display for '+name+r'\b',original);old_display=region(original,display.start())
 new=original.replace(old_enum,new_enum,1).replace(old_display,'',1)
 new=re.sub(r'impl (?:std::error::)?Error for '+name+r' \{\}\n?','',new,count=1)
 for line in ['use std::error::Error;\n','use std::fmt::{Display, Formatter};\n']:
  if line in new:new=new.replace(line,'',1)
 path.write_text(new)
 p=subprocess.run(['rustup','run','1.99.1','rustfmt','--edition','2021',str(path)],capture_output=True,text=True);assert p.returncode==0,p.stderr
 candidate=path.read_bytes();report['candidate_source_bindings'].append({'path':item['path'],'sha256':hashlib.sha256(candidate).hexdigest()})
 assert check(name+'-declaration-source',fixture)!=base
 path.write_text(original)
 # Preserve the qualified candidate separately; never edit parent runtime.
 saved=out/'qualified-candidates'/item['path'];saved.parent.mkdir(parents=True,exist_ok=True);saved.write_bytes(candidate)
for label,crate in [('core-normal','careops-core'),('ed-normal','careops-ed')]:
 p=fixture/'crates'/crate/'Cargo.toml';original=p.read_bytes();p.write_bytes((a.graph/label/'crates'/crate/'Cargo.toml').read_bytes());assert check(label+'-manifest',fixture)!=base;p.write_bytes(original)
 lock.write_bytes((a.graph/label/'Cargo.lock').read_bytes());assert check(label+'-lock-row',fixture)!=base;lock.write_bytes(original_lock)
for item in packet['inputs']:(fixture/item['path']).write_bytes((out/'qualified-candidates'/item['path']).read_bytes())
source_only=check('all-eight-error-declarations',fixture);assert source_only!=base
for crate in ['careops-core','careops-ed']:
 (fixture/'crates'/crate/'Cargo.toml').write_bytes((a.graph/'combined/crates'/crate/'Cargo.toml').read_bytes())
lock.write_bytes((a.graph/'combined/Cargo.lock').read_bytes());both=check('all-eight-with-both-edges-and-actual-lock',fixture);assert both!=base
flags=dict(env,CARGO_ENCODED_RUSTFLAGS='-Copt-level=2');assert check('compiler-flags',fixture,flags)!=both
minimal={k:v for k,v in env.items() if not k.startswith('CARGO_FEATURE_')};assert check('minimal-features',fixture,minimal)!=both
older=dict(env,RUSTC=subprocess.check_output(['rustup','which','rustc','--toolchain','1.99.0'],text=True).strip());assert check('declared-floor-compiler',fixture,older)!=both
report['candidate_model_build_id']=both;report['baseline_model_build_id']=base;report['source_only_model_id']=source_only;report['all_cases_pass']=True
(out/'identity-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'cases':len(report['cases']),'baseline':base,'candidate':both,'all_pass':True},indent=2))
