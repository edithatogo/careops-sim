import os,json,subprocess,time,hashlib,pathlib
root=pathlib.Path('${PRIVATE_VOLUME}/codex-worktrees/careops-r0-r1-20261021'); child=root/'libs/kairos'
out=pathlib.Path('/private/tmp/careops-r1-process-canonical-20261021');out.mkdir(exist_ok=True)
receipts=[]
def run(name,args,cwd=child,tool='1.99.1',extra=None,expected=0):
 bindir='/opt/homebrew/Cellar/rust/1.99.1/bin' if tool=='1.99.1' else '${PRIVATE_HOME}/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin'
 env=os.environ.copy();env.update(PATH=bindir+':'+env['PATH'],RUSTC=bindir+'/rustc',RUSTDOC=bindir+'/rustdoc',CARGO_TARGET_DIR=str(out/('target-'+tool)))
 if extra:env.update(extra)
 start=time.time();log=out/(name+'.log')
 with log.open('wb') as f:p=subprocess.run(args,cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
 r=dict(name=name,argv=args,cwd=str(cwd),toolchain=tool,environment={k:env[k] for k in ['RUSTC','RUSTDOC','CARGO_TARGET_DIR']},extra_environment=extra or {},exit_code=p.returncode,expected_exit=expected,elapsed_seconds=time.time()-start,log=str(log),log_sha256=hashlib.sha256(log.read_bytes()).hexdigest())
 receipts.append(r);(out/'receipts.json').write_text(json.dumps(receipts,indent=2)+'\n');print(name,p.returncode,flush=True);return p.returncode

binary='/private/tmp/careops-r1-native-20261021/target-1.99.1/debug/examples/c2_replay_demo'
checkpoint=str(out/'checkpoint.json')
run('c2-save',[binary,'save',checkpoint]);run('c2-restore-first',[binary,'restore',checkpoint]);run('c2-restore-second',[binary,'restore',checkpoint])
sentinel=out/'sentinel.json';sentinel.write_text('preexisting sentinel bytes\n');run('c2-sentinel',[binary,'save',str(sentinel)],expected=1)
(out/'c2-controls.json').write_text(json.dumps(dict(successful_save_restore=all(r['exit_code']==0 for r in receipts[:3]),repeat_equal=(out/'c2-restore-first.log').read_bytes()==(out/'c2-restore-second.log').read_bytes(),sentinel_unchanged=sentinel.read_text()=='preexisting sentinel bytes\n',sentinel_reason='checkpoint destination already exists' in (out/'c2-sentinel.log').read_text()),indent=2))
run('c5-original-process',['/opt/homebrew/bin/python3.14','scripts/qualify_c5.py','--binary','/private/tmp/careops-r1-native-20261021/target-1.99.0/debug/kairo-ecs-cli','--output',str(out/'c5-process')],tool='1.99.0')
