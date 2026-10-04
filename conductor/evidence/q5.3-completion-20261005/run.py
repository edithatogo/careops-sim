import hashlib,json,subprocess,sys,time
from pathlib import Path
r=Path(__file__).resolve().parents[2]
out=r/'.artifacts/q53-native'
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip()
commands=[
 ['rustup','run','1.99.0','cargo','test','--locked','-p','kairo-ecs-des','--target-dir','.artifacts/q53-native/target'],
 ['rustup','run','1.99.0','cargo','test','--locked','-p','kairo-ecs-des','--no-default-features','--target-dir','.artifacts/q53-native/target'],
 ['rustup','run','1.76.0','cargo','test','--locked','-p','kairo-ecs-des','--no-default-features','--test','des_resource_queue_v1','--test','q53_legacy_public_surface_v1','--test','flow_builder_migration_v1','--target-dir','.artifacts/q53-native/msrv-target'],
 ['rustup','run','1.99.0','cargo','fmt','--all','--check'],
 ['rustup','run','1.99.0','cargo','clippy','--locked','-p','kairo-ecs-des','--all-targets','--target-dir','.artifacts/q53-native/target','--','-D','warnings'],
 ['rustup','run','1.99.0','cargo','test','--locked','-p','kairo-ecs-des','--doc','--target-dir','.artifacts/q53-native/target'],
 ['rustup','run','1.99.0','cargo','run','--locked','-q','-p','kairo-ecs-des','--example','flow_fifo_migration','--target-dir','.artifacts/q53-native/target'],
 ['rustup','run','1.99.0','cargo','run','--locked','-q','-p','kairo-ecs-des','--example','flow_staff_bed_cleaning','--target-dir','.artifacts/q53-native/target'],
 ['pwsh','-NoProfile','-File','docs/design/validate-compatibility-pack.ps1'],
 ['node','scripts/validation/validate-track21-27-evidence-boundaries.mjs'],
]
receipts=[]
for i,argv in enumerate(commands):
 if subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip()!=head: raise SystemExit('HEAD drift')
 start=time.time(); print('START',i,argv,flush=True)
 with (out/f'{i:02d}.stdout.log').open('wb') as stdout,(out/f'{i:02d}.stderr.log').open('wb') as stderr:
  result=subprocess.run(argv,cwd=r,stdout=stdout,stderr=stderr)
 row={'argv':argv,'cwd':str(r),'head':head,'python':sys.version,'started_unix':start,'ended_unix':time.time(),'exit_status':result.returncode,'logs':{}}
 for suffix in ('stdout','stderr'):
  path=out/f'{i:02d}.{suffix}.log';row['logs'][suffix]={'path':str(path.relative_to(r)),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
 receipts.append(row);(out/'receipts.json').write_text(json.dumps(receipts,indent=2)+'\n')
 print('END',i,'exit',result.returncode,flush=True)
 if result.returncode: raise SystemExit(result.returncode)
