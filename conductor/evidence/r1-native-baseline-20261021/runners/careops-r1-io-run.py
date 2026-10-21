import os,json,subprocess,time,hashlib,pathlib
root=pathlib.Path('/Volumes/PortableSSD/codex-worktrees/careops-r0-r1-20261021'); child=root/'libs/kairos'
out=pathlib.Path('/tmp/careops-r1-io-20261021');out.mkdir(exist_ok=True)
receipts=[]
def run(name,args,cwd=child,tool='1.99.1',extra=None,expected=0):
 bindir='/opt/homebrew/Cellar/rust/1.99.1/bin' if tool=='1.99.1' else '/Users/doughnut/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin'
 env=os.environ.copy();env.update(PATH=bindir+':'+env['PATH'],RUSTC=bindir+'/rustc',RUSTDOC=bindir+'/rustdoc',CARGO_TARGET_DIR=str(out/('target-'+tool)))
 if extra:env.update(extra)
 start=time.time();log=out/(name+'.log')
 with log.open('wb') as f:p=subprocess.run(args,cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
 r=dict(name=name,argv=args,cwd=str(cwd),toolchain=tool,environment={k:env[k] for k in ['RUSTC','RUSTDOC','CARGO_TARGET_DIR']},extra_environment=extra or {},exit_code=p.returncode,expected_exit=expected,elapsed_seconds=time.time()-start,log=str(log),log_sha256=hashlib.sha256(log.read_bytes()).hexdigest())
 receipts.append(r);(out/'receipts.json').write_text(json.dumps(receipts,indent=2)+'\n');print(name,p.returncode,flush=True);return p.returncode

reader='/tmp/careops-r1-arrow-reader-20261021/bin/python'
run('reader-versions',[reader,'-c','import platform,pyarrow,numpy; print(platform.python_version(),pyarrow.__version__,numpy.__version__); assert platform.python_version()=="3.14.8"; assert pyarrow.__version__=="25.0.1"; assert numpy.__version__=="2.5.3"'])
run('fixture-integrity',[reader,'crates/kairo-ecs-arrow-io/tests/fixtures/interop_v1/generate.py','--check'])
env={'KAIROS_INTEROP_OUTDIR':str(out/'interop'),'KAIROS_C11_PHYSICAL_OUTDIR':str(out/'c11'),'C43_OUTPUT_DIR':str(out/'c43'),'C44_DIAGNOSTICS_DIR':str(out/'c44')}
(out/'interop').mkdir(exist_ok=True)
run('arrow-io',['cargo','test','--locked','-p','kairo-ecs-arrow-io','--all-features'],extra=env)
run('resource-lifecycle',['cargo','test','--locked','-p','kairo-ecs-arrow','--features','resource-lifecycle-io'],extra=env)
run('c43-native',['cargo','test','--locked','-p','kairo-ecs-calibration','--no-default-features','--features','ipc,parquet','--test','arrow_output_c43','--test','sidecar_adapter_c43','--test','sidecar_pipeline_c43','c43_'],extra=env)
run('reader-controls',[reader,'conformance/c43/readback.py','--self-test'])
run('c43-readback',[reader,'conformance/c43/readback.py','--require-manifests','--require-statistical-diagnostics','--mutation-controls',str(out/'c43')])
for case in ('overlap-and-ties','empty-and-insufficient'):
 run('diagnostics-'+case,[reader,'conformance/c43/statistical_diagnostics.py',str(out/'c44'/case)])
run('interop-readback',[reader,'crates/kairo-ecs-arrow-io/tests/fixtures/interop_v1/generate.py','--check-rust',str(out/'interop')])
run('c11-readback',[reader,'crates/kairo-ecs-arrow-io/tests/fixtures/calibration_physical_v2/generate.py','--check-rust',str(out/'c11')])
