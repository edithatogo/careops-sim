import os,json,subprocess,time,hashlib,pathlib
root=pathlib.Path('${PRIVATE_VOLUME}/codex-worktrees/careops-r0-r1-20261021'); child=root/'libs/kairos'
out=pathlib.Path('/tmp/careops-r1-native-20261021');out.mkdir(exist_ok=True)
receipts=[]
def run(name,args,cwd=child,tool='1.99.1',extra=None,expected=0):
 bindir='/opt/homebrew/Cellar/rust/1.99.1/bin' if tool=='1.99.1' else '${PRIVATE_HOME}/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin'
 env=os.environ.copy();env.update(PATH=bindir+':'+env['PATH'],RUSTC=bindir+'/rustc',RUSTDOC=bindir+'/rustdoc',CARGO_TARGET_DIR=str(out/('target-'+tool)))
 if extra:env.update(extra)
 start=time.time();log=out/(name+'.log')
 with log.open('wb') as f:p=subprocess.run(args,cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
 r=dict(name=name,argv=args,cwd=str(cwd),toolchain=tool,environment={k:env[k] for k in ['RUSTC','RUSTDOC','CARGO_TARGET_DIR']},extra_environment=extra or {},exit_code=p.returncode,expected_exit=expected,elapsed_seconds=time.time()-start,log=str(log),log_sha256=hashlib.sha256(log.read_bytes()).hexdigest())
 receipts.append(r);(out/'receipts.json').write_text(json.dumps(receipts,indent=2)+'\n');print(name,p.returncode,flush=True);return p.returncode
run('rust-1991',['rustc','-vV']);run('rust-1990',['rustc','-vV'],tool='1.99.0')
run('parent-ed',['cargo','test','--locked','--workspace','--all-features'],cwd=root)
run('flow-q-c',['cargo','test','--locked','-p','kairo-ecs-des','-p','kairo-ecs-abm','-p','kairo-ecs-calibration','--features','kairo-ecs-calibration/flow,kairo-ecs-abm/test-support','--','--nocapture'])
run('c2-example-build',['cargo','build','--locked','-p','kairo-ecs-calibration','--features','flow,kairo-ecs-abm/test-support','--example','c2_replay_demo'])
binary=str(out/'target-1.99.1/debug/examples/c2_replay_demo'); checkpoint=str(out/'checkpoint.json')
if pathlib.Path(binary).exists():
 run('c2-save',[binary,'save',checkpoint]);run('c2-restore-first',[binary,'restore',checkpoint]);run('c2-restore-second',[binary,'restore',checkpoint])
 sentinel=out/'sentinel.json';sentinel.write_text('preexisting sentinel bytes\n');run('c2-sentinel',[binary,'save',str(sentinel)],expected=1)
 (out/'c2-controls.json').write_text(json.dumps(dict(repeat_equal=(out/'c2-restore-first.log').read_bytes()==(out/'c2-restore-second.log').read_bytes(),sentinel_unchanged=sentinel.read_text()=='preexisting sentinel bytes\n'),indent=2))
run('c5-build-original',['cargo','build','--locked','-p','kairo-ecs-cli'],tool='1.99.0')
run('c5-original-process',['/opt/homebrew/bin/python3.14','scripts/qualify_c5.py','--binary',str(out/'target-1.99.0/debug/kairo-ecs-cli'),'--output',str(out/'c5-process')],tool='1.99.0')
