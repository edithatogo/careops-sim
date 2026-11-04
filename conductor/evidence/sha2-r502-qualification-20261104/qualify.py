"""Apache-2.0. Reproduce a private, isolated SHA2 decision probe, not adoption."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import statistics
import subprocess
import time
import tomllib

ap = argparse.ArgumentParser()
ap.add_argument('--repo', type=Path, required=True)
ap.add_argument('--out', type=Path, required=True)
a = ap.parse_args()
repo, out = a.repo.resolve(), a.out.resolve()
evidence = Path(__file__).resolve().parent
packet = json.loads((evidence / 'packet.json').read_text())
checkout = subprocess.check_output(['git', '-C', str(repo), 'rev-parse', 'HEAD'], text=True).strip()
# Later documentation receipts can reproduce frozen runtime bytes without
# pretending that their commit is the original tested source.
subprocess.run(['git', '-C', str(repo), 'merge-base', '--is-ancestor', packet['source'], checkout], check=True)
subprocess.run(['git', '-C', str(repo), 'diff', '--quiet', '--ignore-submodules=none', packet['source'], '--', 'Cargo.toml', 'Cargo.lock', 'crates', 'libs/kairos'], check=True)
for binding in packet['inputs']:
    name, digest = binding['path'], binding['sha256']
    assert hashlib.sha256((repo / name).read_bytes()).hexdigest() == digest, name
out.mkdir(exist_ok=False)
report = {'source': packet['source'], 'execution_checkout': checkout, 'commands': [], 'build': {}, 'vectors': [], 'performance': []}
def save():
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
def run(argv, *, env=None):
    start = time.monotonic()
    p = subprocess.run([str(x) for x in argv], env=env, capture_output=True, text=True)
    index = len(report['commands'])
    (out / f'command-{index}.stdout').write_text(p.stdout)
    (out / f'command-{index}.stderr').write_text(p.stderr)
    report['commands'].append({'argv': [str(x).replace(str(out), '${PROBE}').replace(str(repo), '${REPO}') for x in argv], 'exit': p.returncode, 'seconds': time.monotonic()-start, 'stdout_sha256': hashlib.sha256(p.stdout.encode()).hexdigest(), 'stderr_sha256': hashlib.sha256(p.stderr.encode()).hexdigest()})
    save()
    assert p.returncode == 0, (index, p.stderr)
    return p
text = (repo / 'crates/careops-ed/src/api_runner.rs').read_text()
start = text.index('pub fn compute_sha256_hex(')
end = text.index('\npub const MAX_SCENARIO_PAYLOAD_BYTES', start)
baseline = text[start:end].strip() + '\n'
(out / 'baseline.rs').write_text(baseline)
report['baseline_lines'] = len(baseline.splitlines())
report['baseline_sha256'] = hashlib.sha256(baseline.encode()).hexdigest()
manifest = '''[workspace]\n[package]\nname="sha2-r502-probe"\nversion="0.0.0"\nedition="2021"\nlicense="Apache-2.0"\n[features]\ncandidate=["dep:sha2"]\n[dependencies]\nsha2={version="=0.11.1",optional=true}\n[[bin]]\nname="probe"\npath="probe.rs"\n[[bin]]\nname="identity"\npath="identity.rs"\nrequired-features=["candidate"]\n'''
# Seed both isolated manifests with the parent lock, then permit only root/pruning
# reconciliation. Exact retained registry versions/checksums are compared below.
parent = tomllib.loads((repo/'Cargo.lock').read_text())['package']
parent_registry = {(p['name'], p['version'], p.get('checksum')) for p in parent if p.get('source', '').startswith('registry+')}
envs, bins = {}, {}
for mode in ('baseline', 'candidate'):
    root = out/mode
    root.mkdir()
    (root/'Cargo.toml').write_text(manifest)
    shutil.copyfile(evidence/'probe.rs', root/'probe.rs')
    shutil.copyfile(repo/'crates/careops-ed/build.rs', root/'identity.rs')
    shutil.copyfile(repo/'Cargo.lock', root/'Cargo.lock')
    env = dict(os.environ, CARGO_TARGET_DIR=str(root/'target'), R502_BASELINE=str(out/'baseline.rs'))
    envs[mode] = env
    run(['cargo','metadata','--offline','--format-version','1','--manifest-path',root/'Cargo.toml','--features','candidate'], env=env)
    locked = tomllib.loads((root/'Cargo.lock').read_text())['package']
    closure = {(p['name'],p['version'],p.get('checksum')) for p in locked if p.get('source','').startswith('registry+')}
    assert closure <= parent_registry, closure-parent_registry
    report['registry_closure'] = sorted(closure)
    argv = ['cargo','build','--offline','--locked','--release','--manifest-path',root/'Cargo.toml','--bin','probe']
    if mode == 'candidate': argv += ['--features','candidate']
    cold = run(argv, env=env)
    cold_s = report['commands'][-1]['seconds']
    warm = run(argv, env=env)
    warm_s = report['commands'][-1]['seconds']
    bins[mode] = root/'target/release/probe'
    report['build'][mode] = {'cold_seconds':cold_s,'warm_seconds':warm_s,'binary_bytes':bins[mode].stat().st_size,'binary_sha256':hashlib.sha256(bins[mode].read_bytes()).hexdigest()}
    assert cold_s <= packet['budgets']['cold_fresh_target_build_seconds']
    assert warm_s <= packet['budgets']['warm_build_seconds']
assert report['build']['candidate']['binary_bytes']-report['build']['baseline']['binary_bytes'] <= packet['budgets']['max_binary_growth_bytes']
cases = [b'', b'abc', b'abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq']
cases += [bytes((i*131+17)%256 for i in range(n)) for n in (1,55,56,63,64,65,127,128,129,1024,65536,1048576)]
cases += [bytes(range(256)), b'a'*1000000]
cases += [n.to_bytes(8,'little') for n in (0,1,2**32,2**63,2**64-1)]
for i, data in enumerate(cases):
    file = out/'input.bin'; file.write_bytes(data)
    expected = hashlib.sha256(data).hexdigest()
    for mode in bins:
        actual = run([bins[mode],'digest',file]).stdout.strip()
        assert actual == expected, (i, mode)
    report['vectors'].append({'bytes':len(data),'expected':expected})
# Genuine input-corruption control, plus malformed expected values cannot pass.
data = bytes(range(256)); expected = hashlib.sha256(data).hexdigest()
file = out/'input.bin'; file.write_bytes(bytes([data[0]^1])+data[1:])
for mode in bins:
    actual = run([bins[mode],'digest',file]).stdout.strip()
    assert actual != expected
    def matches_expected(value, wanted):
        return re.fullmatch(r'[0-9a-f]{64}', wanted) is not None and value == wanted
    assert matches_expected(actual, hashlib.sha256(file.read_bytes()).hexdigest())
    assert not matches_expected(actual, expected)
    assert not matches_expected(actual, actual[:-1])
    assert not matches_expected(actual, 'g'*64)
report['negative_scope'] = 'Both probe digests detect a single input-bit flip; malformed/truncated expectations differ. This does not test parent digest parsing/recovery.'
for size, iterations in packet['budgets']['sizes_iterations']:
    values = {m:[] for m in bins}; rss = {m:[] for m in bins}
    for pair in range(packet['budgets']['samples']):
        order = ('baseline','candidate') if pair%2 == 0 else ('candidate','baseline')
        for mode in order:
            p = run(['/usr/bin/time','-l',bins[mode],'bench',size,iterations])
            elapsed = int(p.stdout.strip()); assert elapsed >= packet['budgets']['min_sample_ns']
            values[mode].append(elapsed/iterations)
            rss[mode].append(int(re.search(r'(\d+)\s+maximum resident set size',p.stderr)[1]))
    ratio = statistics.median(values['candidate'])/statistics.median(values['baseline'])
    result = {'bytes':size,'iterations':iterations,'ns_per_hash':values,'peak_rss_bytes':rss,'candidate_median_ratio':ratio}
    report['performance'].append(result); save()
    assert ratio <= packet['budgets']['max_candidate_median_ratio'], result
    if size == packet['budgets']['tiny_bytes']:
        assert statistics.median(values['candidate'])-statistics.median(values['baseline']) <= packet['budgets']['tiny_max_excess_ns_per_hash']
    assert max(rss['candidate'])-max(rss['baseline']) <= packet['budgets']['max_peak_rss_growth_bytes']
report['status'] = 'pass_local_isolated_probe_only'; save()
print(json.dumps({k:report[k] for k in ('status','build','registry_closure','performance')},indent=2))
