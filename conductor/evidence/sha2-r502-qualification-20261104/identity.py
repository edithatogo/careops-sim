"""Apache-2.0. Run the actual ED build script against bounded synthetic inputs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

p = argparse.ArgumentParser()
p.add_argument('--repo', type=Path, required=True)
p.add_argument('--probe', type=Path, required=True)
p.add_argument('--fixture-name', default='identity-fixture-v2')
a = p.parse_args(); repo = a.repo.resolve(); probe = a.probe.resolve()
packet = json.loads((Path(__file__).parent/'packet.json').read_text())
for binding in packet['inputs']:
    name, expected = binding['path'], binding['sha256']
    assert hashlib.sha256((repo/name).read_bytes()).hexdigest() == expected, name
subprocess.run(['git','-C',str(repo),'diff','--quiet','--ignore-submodules=none',packet['source'],'--','Cargo.toml','Cargo.lock','crates','libs/kairos'],check=True)
root = probe / a.fixture_name; root.mkdir(exist_ok=False)
source_roots = [f'crates/{n}' for n in ('careops-core','careops-ed','careops-ed-cli')]
source_roots += [f'libs/kairos/crates/kairo-ecs-{n}' for n in ('types','rng','core','des','state','abm')]
paths = ['Cargo.toml','Cargo.lock','crates/careops-ed/build.rs']
for name in source_roots:
    paths.append(name + '/Cargo.toml')
    paths.extend(str(f.relative_to(repo)) for f in (repo/name/'src').rglob('*.rs'))
for name in paths:
    dest = root/name; dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(repo/name, dest)
compiler = shutil.which('rustc')
version = subprocess.check_output([compiler,'-vV'])
features = sorted(line.split('"')[1] for line in subprocess.check_output([compiler,'--print','cfg'],text=True).splitlines() if line.startswith('target_feature='))
env = {k:v for k,v in os.environ.items() if not k.startswith('CARGO_FEATURE_')}
env.update(RUSTC=compiler, TARGET=next(x.split(': ')[1] for x in version.decode().splitlines() if x.startswith('host:')), PROFILE='release', OPT_LEVEL='3', DEBUG='false', CARGO_ENCODED_RUSTFLAGS='', CARGO_CFG_TARGET_FEATURE=','.join(features))
fields = ['TARGET','PROFILE','OPT_LEVEL','DEBUG','CARGO_ENCODED_RUSTFLAGS','CARGO_CFG_TARGET_FEATURE']
subprocess.run(['cargo','build','--locked','--offline','--release','--manifest-path',probe/'candidate/Cargo.toml','--features','candidate','--bin','identity'],env=dict(env,CARGO_TARGET_DIR=str(probe/'candidate/target')),check=True)
exe = probe/'candidate/target/release/identity'
report = {'source_build_script_sha256':hashlib.sha256((repo/'crates/careops-ed/build.rs').read_bytes()).hexdigest(),'compiler_version_sha256':hashlib.sha256(version).hexdigest(),'controlled_env':{k:env[k] for k in fields},'cases':[]}
def check(label, tree, delta=None):
    context = dict(env, CARGO_MANIFEST_DIR=str(tree/'crates/careops-ed'))
    context.update(delta or {})
    h = hashlib.sha256(b'careops.ed.model-build.v1\0')
    def framed(value): h.update(len(value).to_bytes(8,'little')); h.update(value)
    for name in sorted(paths, key=lambda n: Path(n).parts):
        framed(name.encode()); framed((tree/name).read_bytes())
    framed(version)
    for name in fields:
        framed(name.encode()); framed(context[name].encode())
    for key in sorted(k for k in context if k.startswith('CARGO_FEATURE_')):
        framed(key.encode()); framed(context[key].encode())
    result = subprocess.run([exe],env=context,capture_output=True,check=True,text=True)
    actual = next(s.split('=',2)[2] for s in result.stdout.splitlines() if s.startswith('cargo:rustc-env=CAREOPS_ED_MODEL_CONTRACT_SHA256='))
    assert actual == h.hexdigest(), label
    report['cases'].append({'case':label,'sha256':actual,'independent_transcript_matches':True,'exit':result.returncode})
    return actual
baseline = check('actual_current_source',repo)
assert check('same_bytes_relocated',root) == baseline
(root/'crates/careops-ed/src/qualification-note.md').write_text('documentation only\n')
assert check('added_documentation',root) == baseline
helper = root/'crates/careops-ed/src/api_runner.rs'; old = helper.read_bytes()
# Use the exact proposed helper, retaining every unrelated source byte.
text = old.decode(); start = text.index('pub fn compute_sha256_hex('); end = text.index('\npub const MAX_SCENARIO_PAYLOAD_BYTES',start)
probe_text = (Path(__file__).parent/'probe.rs').read_text(); begin = probe_text.index('fn compute_sha256_hex('); finish = probe_text.index('\nfn main()',begin)
helper.write_text(text[:start] + 'pub ' + probe_text[begin:finish] + '\n' + text[end:])
assert check('proposed_helper_only',root) != baseline
helper.write_bytes(old)
manifest = root/'crates/careops-ed/Cargo.toml'; old = manifest.read_bytes()
manifest.write_text(old.decode().replace('[dependencies]\n','[dependencies]\nsha2.workspace = true\n',1))
assert check('proposed_normal_edge_only',root) != baseline
manifest.write_bytes(old)
assert check('changed_compiler_flags',root,{'CARGO_ENCODED_RUSTFLAGS':'-C\x1ftarget-cpu=native'}) != baseline
assert check('changed_feature_binding',root,{'CARGO_FEATURE_SYNTHETIC':'1'}) != baseline
assert check('restored_inputs',root) == baseline
report['status']='pass_actual_build_script_and_independent_transcript'
report['compatibility_decision']='Standard file digests unchanged; changed model-build identity intentionally rejects old execution resume. Read historical artifacts with original recorded model tag/qualified original software. Do not rewrite old tags, bypass checks or invent a format version for equal SHA256 semantics.'
(probe/'identity-report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
