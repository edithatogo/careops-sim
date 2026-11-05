"""Apache-2.0. Offline manifest/lock resolution only; stubs never compiled."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();repo=a.repo.resolve();out=a.out.resolve();out.mkdir(exist_ok=False)
env=dict(os.environ,RUSTUP_TOOLCHAIN='1.99.1');argv=['cargo','metadata','--offline','--locked','--no-deps','--format-version','1'];p=subprocess.run(argv,cwd=repo,env=env,capture_output=True,text=True);(out/'current.stdout').write_text(p.stdout);(out/'current.stderr').write_text(p.stderr);assert p.returncode==0,p.stderr
m=json.loads(p.stdout);members=[p for p in m['packages'] if p['id'] in m['workspace_members']]
assert {x['name'] for x in members if not Path(x['manifest_path']).relative_to(repo).parts[0]=='libs'}=={'careops-sim','careops-core','careops-ed','careops-ed-cli','careops-birthing','careops-birthing-cli'}
manifest_paths=[Path('Cargo.toml')]+[p.relative_to(repo) for p in (repo/'crates').glob('*/Cargo.toml')]
manifest_paths += [Path('libs/kairos')/p for p in subprocess.check_output(['git','-C',str(repo/'libs/kairos'),'ls-files','*Cargo.toml'],text=True).splitlines()]
manifest_paths=sorted(set(manifest_paths))
original_lock=(repo/'Cargo.lock').read_bytes();old=tomllib.loads(original_lock.decode());key=lambda p:(p['name'],p['version'],p.get('source',''))
old_packages={key(p):p for p in old['package']};report={'source':subprocess.check_output(['git','-C',str(repo),'rev-parse','HEAD'],text=True).strip(),'input_lock_sha256':hashlib.sha256(original_lock).hexdigest(),'workspace_manifest_bindings':[],'variants':[],'scope':'Exact manifests and parent lock; empty source target stubs and immutable upstream manifest snapshots. Cargo resolution only, never parent compilation or a new runtime acceptance.'}
for path in manifest_paths:report['workspace_manifest_bindings'].append({'path':str(path),'sha256':hashlib.sha256((repo/path).read_bytes()).hexdigest()})
report['actual_workspace_members']=[p['name'] for p in members]
def add_edge(text,kind):
 parsed=tomllib.loads(text);assert 'tempfile' not in parsed.get(kind,{})
 header='['+kind+']';new=text.replace(header,header+'\ntempfile = "=3.27.0"') if header in text else text+'\n'+header+'\ntempfile = "=3.27.0"\n'
 after=tomllib.loads(new);after[kind].pop('tempfile')
 if not after[kind] and kind not in parsed:after.pop(kind)
 assert after==parsed;return new
variants={'core-normal':[('careops-core','dependencies')],'cli-dev':[('careops-ed-cli','dev-dependencies')],'ed-dev':[('careops-ed','dev-dependencies')],'combined':[('careops-core','dependencies'),('careops-ed-cli','dev-dependencies'),('careops-ed','dev-dependencies')]}
for label,edges in variants.items():
 root=out/label;root.mkdir();shutil.copyfile(repo/'Cargo.lock',root/'Cargo.lock');(root/'before.lock').write_bytes(original_lock)
 for manifest in manifest_paths:
  dest=root/manifest;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(repo/manifest,dest)
  parsed=tomllib.loads((repo/manifest).read_text())
  if 'package' in parsed:
   targets=['src/lib.rs','src/main.rs','build.rs']
   for section in ['lib']:
    if parsed.get(section,{}).get('path'):targets.append(parsed[section]['path'])
   for section in ['bin','test','bench','example']:
    targets.extend(x['path'] for x in parsed.get(section,[]) if 'path' in x)
   for target in targets:
    source=manifest.parent/target
    if (repo/source).is_file():
     f=root/source;f.parent.mkdir(parents=True,exist_ok=True);f.write_text('// Metadata-only target stub; never compiled.\n')
 for pkg in members:
  for target in pkg['targets']:
   source=Path(target['src_path']).relative_to(repo);file=root/source;file.parent.mkdir(parents=True,exist_ok=True);file.write_text('// Metadata-only target stub; never compiled.\n')
 for crate,kind in edges:
  path=root/'crates'/crate/'Cargo.toml';path.write_text(add_edge(path.read_text(),kind))
 argv=['cargo','metadata','--offline','--format-version','1','--manifest-path',str(root/'Cargo.toml')];p=subprocess.run(argv,cwd=root,env=env,capture_output=True,text=True);(root/'metadata.stdout').write_text(p.stdout);(root/'metadata.stderr').write_text(p.stderr);assert p.returncode==0,p.stderr
 new_bytes=(root/'Cargo.lock').read_bytes();new=tomllib.loads(new_bytes.decode());packages={key(p):p for p in new['package']};assert set(packages)==set(old_packages);changed=[]
 expected={crate for crate,_ in edges if crate!='careops-ed'}
 for k,before in old_packages.items():
  after=packages[k]
  if before!=after:
   assert k[0] in expected and k[2]=='',k;restored=copy.deepcopy(after);assert set(after['dependencies'])-set(before['dependencies'])=={'tempfile'};assert not(set(before['dependencies'])-set(after['dependencies']));restored['dependencies']=before['dependencies'];assert restored==before;changed.append(k[0])
 assert set(changed)==expected,(label,changed,expected)
 report['variants'].append({'name':label,'edges':edges,'argv':argv,'exit':p.returncode,'before_lock_sha256':hashlib.sha256(original_lock).hexdigest(),'after_lock_sha256':hashlib.sha256(new_bytes).hexdigest(),'changed_workspace_package_rows':sorted(changed),'registry_packages_versions_sources_checksums_unchanged':True,'all_other_package_rows_identical':True,'metadata_stdout_sha256':hashlib.sha256(p.stdout.encode()).hexdigest()})
assert (repo/'Cargo.lock').read_bytes()==original_lock;report['all_variants_pass']=True;(out/'graph-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report['variants'],indent=2))
