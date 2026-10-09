from pathlib import Path
import hashlib,json,re
root=Path(__file__).resolve().parent
repo=Path.cwd()
expected_tree='fd58d564b5709f75f102a1c09b02b1e1e9e8e444'
h=lambda b:hashlib.sha256(b).hexdigest()
results=[]
run=json.loads((root/'run.json').read_text())
assert run['headSha']=='63a72aa563258777686ee3d2e25794628daae4ec'
assert run['status']=='completed' and run['conclusion']=='success'
checks=json.loads((root/'pr-checks.json').read_text())
assert checks['headRefOid']==run['headSha']
assert all(c.get('conclusion') in ['SUCCESS','SKIPPED'] for c in checks['statusCheckRollup'])
for host in ['ubuntu-24.04','macos-15']:
    art=root/('c2-native-'+host);meta=art/'c2-native-ci'/host
    tree=(meta/'source-tree.log').read_text().strip();assert tree==expected_tree
    toolchain=(meta/'source.log').read_text();assert 'release: 1.99.0' in toolchain
    expected_host='x86_64-unknown-linux-gnu' if host=='ubuntu-24.04' else 'aarch64-apple-darwin'
    assert 'host: '+expected_host in toolchain
    hashes={}
    for line in (meta/'portable-source.sha256').read_text().splitlines():
        digest,path=line.split(None,1);path=path.strip();assert h((repo/path).read_bytes())==digest;hashes[path]=digest
    assert len(hashes)==4
    cases=[]
    for block in (art/'c2-process-matrix/process-invocations.txt').read_text().strip().split('\n\n'):
        r=dict(line.split('=',1) for line in block.splitlines());p=art/'c2-process-matrix'/Path(r['input_file']).name
        assert h(p.read_bytes())==r['input_sha256']
        trace=next(line.removeprefix('C2_PORTABLE_PROCESS_TRACE:') for line in p.with_suffix('.child.stdout.log').read_text().splitlines() if line.startswith('C2_PORTABLE_PROCESS_TRACE:'))
        assert h(trace.encode())==r['actual_trace_hex_sha256']
        assert h(bytes.fromhex(trace))==r['expected_trace_sha256']
        assert r['child_exit_code']=='Some(0)';cases.append(r['case'])
    assert len(cases)==len(set(cases))==19
    proofs=[]
    for folder,marker in [('c2-generic-context-tests','C2_GENERIC_CONTEXT_TRACE:'),('c2-identity-rng','C2_IDENTITY_RNG_TRACE:')]:
        invocations=list((art/folder).glob('*.invocation.txt'));assert len(invocations)==1
        p=invocations[0];base=p.name.removesuffix('.invocation.txt');r=dict(line.split('=',1) for line in p.read_text().splitlines());assert r['child_exit']=='Some(0)'
        assert h(p.with_name(base+'.c2').read_bytes())==r['image_sha256']
        stdout=p.with_name(base+'.child.stdout').read_text()
        if folder=='c2-generic-context-tests':assert r['expected_trace'] in stdout
        else:
            trace=next(line.removeprefix(marker) for line in stdout.splitlines() if line.startswith(marker));assert trace==r['expected_continuation']+'|'+r['expected_after_sha256']
        proofs.append({'proof':folder,'input_sha256':r['image_sha256']})
    rows=[tuple(map(int,m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', (meta/'flow-tests.log').read_text())]
    counts={'passed':sum(x[0] for x in rows),'failed':sum(x[1] for x in rows),'ignored':sum(x[2] for x in rows),'groups':len(rows)}
    assert counts=={'passed':592,'failed':0,'ignored':5,'groups':50}
    results.append({'host':host,'compiled_commit':toolchain.splitlines()[0],'tree':tree,'source_hashes':hashes,'rust_release':'1.99.0','rust_host':expected_host,'flow_tests':counts,'matrix_cases':cases,'additional_proofs':proofs,'portable_children_verified':21})
output={'run':37964052376,'reviewed_head':run['headSha'],'reviewed_tree':expected_tree,'host_readbacks':results,'checks':{'success':sum(c.get('conclusion')=='SUCCESS' for c in checks['statusCheckRollup']),'skipped':[c['name'] for c in checks['statusCheckRollup'] if c.get('conclusion')=='SKIPPED']},'scope':'Actual hosted source/tree/toolchain and complete process input/exit/control-trace reconciliation; phase/release/clinical/C5 acceptance separate.'}
(root/'readback.json').write_text(json.dumps(output,indent=2)+'\n')
print('Both hosted sources match reviewed tree; 42 portable child runs reconciled; checks=',output['checks'])
