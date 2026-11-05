"""Apache-2.0. Isolated formatting qualification; never edits parent source."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import statistics
import subprocess
import time

p = argparse.ArgumentParser()
p.add_argument('--repo', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
p.add_argument('--prepare-only', action='store_true')
a = p.parse_args()
repo, out = a.repo.resolve(), a.out.resolve()
packet_path = Path(__file__).with_name('packet.json')
packet = json.loads(packet_path.read_text())
budgets = packet['budgets']
sha = lambda b: hashlib.sha256(b).hexdigest()
assert sha(Path(__file__).read_bytes()) == packet['reproducer_sha256']
assert subprocess.check_output(['git', '-C', str(repo), 'rev-parse', 'HEAD'], text=True).strip() == packet['source']
for item in packet['inputs']:
    assert sha((repo / item['path']).read_bytes()) == item['sha256'], item['path']
out.mkdir(exist_ok=False)
report = {'source': packet['source'], 'packet_sha256': sha(packet_path.read_bytes()), 'commands': [], 'build': {}, 'gates': {}}
def save():
    (out / 'report-private.json').write_text(json.dumps(report, indent=2) + '\n')
def run(argv, *, env=None, cwd=None, expected=(0,)):
    start = time.monotonic()
    result = subprocess.run(list(map(str, argv)), cwd=cwd or repo, env=env, capture_output=True, text=True)
    index = len(report['commands'])
    (out / f'command-{index}.stdout').write_text(result.stdout)
    (out / f'command-{index}.stderr').write_text(result.stderr)
    report['commands'].append({'argv': list(map(str, argv)), 'cwd': str(cwd or repo), 'exit': result.returncode,
                              'seconds': time.monotonic() - start, 'stdout_sha256': sha(result.stdout.encode()),
                              'stderr_sha256': sha(result.stderr.encode())})
    save()
    assert result.returncode in expected, (index, result.returncode, result.stderr[-2000:])
    return result
def block(text, start):
    end = text.index('\n}', start) + 2
    return text[start:end]

baseline, candidate, expressions = [], [], []
identity_variants = None
for item in packet['inputs']:
    text, name = (repo / item['path']).read_text(), item['type']
    start = text.rfind('#[derive(', 0, text.index('pub enum ' + name))
    enum = block(text, start)
    display = re.search(r'impl (?:std::fmt::)?Display for ' + name + r'\b', text)
    manual = block(text, display.start())
    error = re.search(r'impl (?:std::error::)?Error for ' + name + r' \{\}', text)[0]
    conversions = [block(text, m.start()) for m in re.finditer(r'impl From<IdentityError> for ' + name + r'\b', text)]
    baseline.append('\n'.join([enum, manual, error, *conversions]))
    arms = list(re.finditer(r'Self::(\w+)(?:\((\w+)\))?\s*=>', manual))
    formats = {}
    for index, arm in enumerate(arms):
        body = manual[arm.end():arms[index + 1].start() if index + 1 < len(arms) else len(manual)]
        literal = re.search(r'"(?:[^"\\]|\\.)*"', body)
        if literal:
            fmt = json.loads(literal[0])
            if arm[2]:
                fmt = fmt.replace('{' + arm[2] + '}', '{0}')
        else:
            assert 'error.fmt(' in body, (name, body)
            fmt = '{0}'
        formats[arm[1]] = fmt
    migrated = enum.replace('#[derive(', '#[derive(thiserror::Error, ', 1)
    variants = re.findall(r'^    (\w+)(?:\(([^\n]*)\))?,\s*$', enum, re.M)
    assert set(v[0] for v in variants) == set(formats), name
    if name == 'IdentityError':
        identity_variants = [v[0] for v in variants]
    for variant, field in variants:
        old = '    ' + variant + (f'({field})' if field else '') + ','
        migrated = migrated.replace(old, '    #[error(' + json.dumps(formats[variant], ensure_ascii=False) + ')]\n' + old)
        if not field:
            values = [(variant, name + '::' + variant)]
        elif field == 'IdentityError':
            assert identity_variants
            values = [(variant + '(' + v + ')', f'{name}::{variant}(IdentityError::{v})') for v in identity_variants]
        else:
            assert field in ("&'static str", 'String'), (name, field)
            payloads = ['fixture', 'unicode-λ:{opaque}', 'quoted-"fixture"\nnext']
            values = [(variant + str(i), f'{name}::{variant}(' + json.dumps(v, ensure_ascii=False) + ('.to_owned()' if field == 'String' else '') + ')') for i, v in enumerate(payloads)]
        for label, expr in values:
            check_copy = 'assert_copy(&e);' if re.search(r'\bCopy\b', enum.split('\n')[0]) else ''
            expressions.append('{ let e = ' + expr + '; assert_traits(&e); ' + check_copy + ' rows.push((' + json.dumps(name + '::' + label) + '.to_owned(), Box::new(e))); }')
    candidate.append('\n'.join([migrated, *conversions]))

anchors = '''
assert_eq!(CapacityError::InvalidCapacity("fixture").to_string(), "invalid capacity snapshot: fixture");
assert_eq!(CapacityError::ConflictingPool("pool-λ".into()).to_string(), "conflicting capacity declarations for pool pool-λ");
assert_eq!(IdentityError::UnknownPerson.to_string(), "identity command references an unknown person");
assert_eq!(BirthMessageError::EmptyInfantSet.to_string(), "birth event must contain at least one infant");
assert_eq!(TransferError::InvalidTransition.to_string(), "transfer command is invalid in the current state");
assert_eq!(ArrivalError::InvalidVersion.to_string(), "version must be 1");
assert_eq!(EdIdentityError::InvalidSourceKey.to_string(), "synthetic ED identity key is empty or too long");
assert_eq!(OperationalError::Ineligible.to_string(), "staff role is not eligible for this task and zone");
assert_eq!(BoundaryError::ImmediateExpiryDefaultProhibited.to_string(), "ST002: immediate expiry is not an accepted universal default; explicit policy required");
assert_eq!(BirthMessageError::from(IdentityError::UnknownPerson), BirthMessageError::Identity(IdentityError::UnknownPerson));
assert_eq!(EdIdentityError::from(IdentityError::UnknownPerson), EdIdentityError::Ledger(IdentityError::UnknownPerson));
'''
main = '''
fn assert_traits<E: Error + Clone + Eq + Send + Sync + 'static>(_: &E) {}
fn assert_copy<E: Copy>(_: &E) {}
fn main() {
ANCHORS
let mut rows: Vec<(String, Box<dyn Error>)> = Vec::new();
EXPRESSIONS
for (_, e) in &rows { assert!(e.source().is_none(), "source contract changed: {e}"); }
let checksum: usize = rows.iter().map(|(_, e)| e.to_string().len()).sum();
let args: Vec<_> = std::env::args().collect();
if args[1] == "cases" {
  for (label, e) in &rows { println!("{}|{:?}|{:?}", label, e.to_string(), e); }
  println!("@@{}:{}", rows.len(), checksum); return;
}
let loops: u64 = args[2].parse().unwrap();
let mut sum = 0usize;
for _ in 0..128 { for (_, e) in &rows { sum = sum.wrapping_add(std::hint::black_box(e).to_string().len()); } }
std::hint::black_box(sum); sum = 0;
let start = std::time::Instant::now();
for _ in 0..loops { for (_, e) in &rows { sum = sum.wrapping_add(std::hint::black_box(e).to_string().len()); } }
let elapsed = start.elapsed().as_nanos(); std::hint::black_box(sum);
assert_eq!(sum, loops as usize * checksum);
println!("{{\\"elapsed_ns\\":{},\\"formats\\":{},\\"checksum\\":{}}}", elapsed, loops * rows.len() as u64, sum);
}
'''.replace('ANCHORS', anchors).replace('EXPRESSIONS', '\n'.join(expressions))
source = 'use std::error::Error;\n#[cfg(not(feature="candidate"))] use std::fmt::{Display, Formatter};\n' + '\n'.join(baseline) + main
candidate_source = 'use std::error::Error;\n' + '\n'.join(candidate) + main
report.update(case_count=len(expressions), baseline_source_sha256=sha(source.encode()), candidate_source_sha256=sha(candidate_source.encode()), original_manual_implementation_lines=sum(len(s.splitlines()) for s in baseline), candidate_declaration_lines=sum(len(s.splitlines()) for s in candidate))
if a.prepare_only:
    (out / 'baseline.rs').write_text(source)
    (out / 'candidate.rs').write_text(candidate_source)
    save()
    print(json.dumps({'prepared_cases': len(expressions), 'timed_or_built': False}))
    raise SystemExit(0)
manifest = '[workspace]\n[package]\nname="thiserror-r562-probe"\nversion="0.0.0"\nedition="2021"\nlicense="Apache-2.0"\n[features]\ncandidate=["dep:thiserror"]\n[dependencies]\nthiserror={version="=2.0.21",optional=true}\n[[bin]]\nname="probe"\npath="main.rs"\n'
bins, envs = {}, {}
for mode, src in [('baseline', source), ('candidate', candidate_source)]:
    assert shutil.disk_usage(out).free >= budgets['disk_stop_free_bytes']
    root = out / mode; root.mkdir()
    (root / 'Cargo.toml').write_text(manifest); (root / 'main.rs').write_text(src)
    shutil.copyfile(repo / 'Cargo.lock', root / 'Cargo.lock')
    compiler = subprocess.check_output(['rustup', 'which', 'rustc', '--toolchain', '1.99.1'], text=True).strip()
    cargo = subprocess.check_output(['rustup', 'which', 'cargo', '--toolchain', '1.99.1'], text=True).strip()
    assert run([compiler, '-vV']).stdout == packet['toolchain']['rustc_vV']
    assert run([cargo, '--version']).stdout == packet['toolchain']['cargo_version']
    env = dict(os.environ, RUSTUP_TOOLCHAIN='1.99.1', RUSTC=compiler, CARGO_TARGET_DIR=str(root / 'target'), CARGO_BUILD_JOBS='1', CARGO_INCREMENTAL='0')
    envs[mode] = env
    run([cargo, 'metadata', '--offline', '--format-version', '1', '--manifest-path', root / 'Cargo.toml', '--features', 'candidate'], env=env)
    argv = [cargo, 'build', '--offline', '--locked', '--release', '--manifest-path', root / 'Cargo.toml'] + (['--features', 'candidate'] if mode == 'candidate' else [])
    run(argv, env=env); cold = report['commands'][-1]['seconds']
    run(argv, env=env); warm = report['commands'][-1]['seconds']
    exe = root / 'qualified-probe'; shutil.copy2(root / 'target/release/probe', exe); bins[mode] = exe
    report['build'][mode] = {'cold_seconds': cold, 'warm_seconds': warm, 'binary_bytes': exe.stat().st_size, 'binary_sha256': sha(exe.read_bytes()), 'binary_format': 'Mach-O' if exe.read_bytes()[:4] == b'\xcf\xfa\xed\xfe' else 'unverified'}
    report['gates'][mode + '_cold'] = cold <= budgets['cold_fresh_target_seconds_max']
    report['gates'][mode + '_warm'] = warm <= budgets['warm_build_seconds_max']
base_cases = run([bins['baseline'], 'cases']).stdout
candidate_cases = run([bins['candidate'], 'cases']).stdout
assert base_cases == candidate_cases
report['case_output_sha256'] = sha(base_cases.encode())
report['gates']['all_variant_message_trait_source_parity'] = True
root = out / 'candidate'
for label, bad in [('bad-message', candidate_source.replace('invalid capacity snapshot: {0}', 'changed capacity snapshot: {0}', 1)), ('bad-source', candidate_source.replace('Identity(IdentityError),', 'Identity(#[source] IdentityError),', 1))]:
    assert bad != candidate_source
    (root / 'main.rs').write_text(bad)
    run([cargo, 'build', '--offline', '--locked', '--release', '--manifest-path', root / 'Cargo.toml', '--features', 'candidate'], env=envs['candidate'])
    run([root / 'target/release/probe', 'cases'], expected=(101,))
    report['gates'][label + '_refuses'] = True
(root / 'main.rs').write_text(candidate_source)
values, rss = {'baseline': [], 'candidate': []}, {'baseline': [], 'candidate': []}
pilot = {m: json.loads(run([e, 'bench', 64]).stdout)['elapsed_ns'] / 64 for m, e in bins.items()}
loops = max(64, math.ceil(400000000 / min(pilot.values())))
for pair in range(budgets['samples_per_workload']):
    for mode in (['baseline', 'candidate'] if pair % 2 == 0 else ['candidate', 'baseline']):
        result = run(['/usr/bin/time', '-l', bins[mode], 'bench', loops])
        data = json.loads(result.stdout); assert data['elapsed_ns'] >= budgets['minimum_timed_ns']
        values[mode].append(data['elapsed_ns'] / data['formats'])
        rss[mode].append(int(re.search(r'(\d+)\s+maximum resident set size', result.stderr)[1]))
b, c = (statistics.median(values[m]) for m in ('baseline', 'candidate'))
report['performance'] = {'unit': 'ns per formatted error over complete variant corpus', 'loops': loops, 'values': values, 'rss': rss, 'ratio': c / b, 'excess_ns': c - b}
report['gates']['latency'] = c / b <= budgets['candidate_median_ratio_max'] or c - b <= budgets['tiny_max_excess_ns']
report['gates']['rss'] = max(rss['candidate']) - max(rss['baseline']) <= budgets['peak_rss_growth_bytes_max']
report['gates']['binary_growth'] = report['build']['candidate']['binary_bytes'] - report['build']['baseline']['binary_bytes'] <= budgets['binary_growth_bytes_max']
report['all_gates_pass'] = all(report['gates'].values()); save()
print(json.dumps({'cases': report['case_count'], 'gates': report['gates'], 'build': report['build'], 'performance': report['performance']}, indent=2))
