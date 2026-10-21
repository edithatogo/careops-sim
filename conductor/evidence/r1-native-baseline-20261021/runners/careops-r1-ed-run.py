import os,json,subprocess,time,hashlib,pathlib
root=pathlib.Path('/Volumes/PortableSSD/codex-worktrees/careops-r0-r1-20261021'); child=root/'libs/kairos'
out=pathlib.Path('/private/tmp/careops-r1-ed-output-20261021');out.mkdir(exist_ok=True)
receipts=[]
def run(name,args,cwd=child,tool='1.99.1',extra=None,expected=0):
 bindir='/opt/homebrew/Cellar/rust/1.99.1/bin' if tool=='1.99.1' else '/Users/doughnut/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin'
 env=os.environ.copy();env.update(PATH=bindir+':'+env['PATH'],RUSTC=bindir+'/rustc',RUSTDOC=bindir+'/rustdoc',CARGO_TARGET_DIR=str(out/('target-'+tool)))
 if extra:env.update(extra)
 start=time.time();log=out/(name+'.log')
 with log.open('wb') as f:p=subprocess.run(args,cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
 r=dict(name=name,argv=args,cwd=str(cwd),toolchain=tool,environment={k:env[k] for k in ['RUSTC','RUSTDOC','CARGO_TARGET_DIR']},extra_environment=extra or {},exit_code=p.returncode,expected_exit=expected,elapsed_seconds=time.time()-start,log=str(log),log_sha256=hashlib.sha256(log.read_bytes()).hexdigest())
 receipts.append(r);(out/'receipts.json').write_text(json.dumps(receipts,indent=2)+'\n');print(name,p.returncode,flush=True);return p.returncode

run('ed-cli-build',['cargo','build','--locked','-p','careops-ed-cli'],cwd=root,extra={'CARGO_TARGET_DIR':'/private/tmp/careops-r1-native-20261021/target-1.99.1'})
binary='/private/tmp/careops-r1-native-20261021/target-1.99.1/debug/careops-ed'
for name in ('first','second'):
 run('ed-'+name,[binary,'run',str(root/'crates/careops-ed/examples/nominal_mvp.json'),'--seed','42','--output',str(out/(name+'.json'))],cwd=root)
if (out/'first.json').exists() and (out/'second.json').exists():
 (out/'repeat.json').write_text(json.dumps({'byte_equal':(out/'first.json').read_bytes()==(out/'second.json').read_bytes()}))
