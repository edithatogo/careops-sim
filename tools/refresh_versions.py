#!/usr/bin/env python3
"""Read public registries; report candidates, never install or upgrade packages."""
import concurrent.futures
import datetime
import json
from pathlib import Path
import tomllib
import urllib.request

CRATES = ('arrow-array','arrow-schema','arrow-ipc','parquet','serde','serde_json','toml',
          'rayon','rand','wgpu','burn','proptest','criterion','clap','clap_builder',
          'insta','cargo-nextest',
          'cargo-deny','cargo-audit','cargo-semver-checks','cargo-llvm-cov',
          'cargo-mutants','cargo-fuzz','zizmor')
REPOS = ('actions/checkout','actions/upload-artifact','actions/download-artifact',
         'github/codeql-action','renovatebot/renovate','rhysd/actionlint',
         'gitleaks/gitleaks','openai/codex')

def read(url):
    req = urllib.request.Request(url, headers={'User-Agent':'careops-sim-version-audit/1.0'})
    with urllib.request.urlopen(req, timeout=30) as response:
        return response.read()

def query(kind, name):
    url = (f'https://crates.io/api/v1/crates/{name}' if kind == 'crate'
           else f'https://api.github.com/repos/{name}/releases/latest')
    if name == 'github/codeql-action':
        url = f'https://api.github.com/repos/{name}/releases?per_page=50'
    try:
        data = json.loads(read(url))
        if name == 'github/codeql-action':
            data = next(r for r in data if r['tag_name'].startswith('v')
                        and not r['prerelease'] and not r['draft'])
        if kind == 'crate':
            version = data['crate']['max_stable_version']
            detail = next(v for v in data['versions'] if v['num'] == version)
            return {'kind':kind, 'name':name, 'version':version, 'rust_version':detail.get('rust_version'),
                    'released_at':detail['created_at'], 'source':url, 'status':'candidate_unvalidated'}
        return {'kind':kind, 'name':name, 'version':data['tag_name'], 'released_at':data['published_at'],
                'source':url, 'release_url':data['html_url'], 'status':'candidate_unvalidated'}
    except Exception as error:
        return {'kind':kind,'name':name,'source':url,'status':'unavailable','error':str(error)}

def main():
    entries = [('crate',n) for n in CRATES] + [('github-release',n) for n in REPOS]
    with concurrent.futures.ThreadPoolExecutor(max_workers=5) as pool:
        records = list(pool.map(lambda entry: query(*entry), entries))
    url = 'https://static.rust-lang.org/dist/channel-rust-stable.toml'
    try:
        rust = tomllib.loads(read(url).decode())
        records.insert(0, {'kind':'toolchain','name':'rust','version':rust['pkg']['rust']['version'],
                          'release_date':rust['date'],'source':url,'status':'candidate_unvalidated'})
    except Exception as error:
        records.insert(0, {'kind':'toolchain','name':'rust','source':url,'status':'unavailable','error':str(error)})
    report = {'schema_version':1,'checked_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'policy':'Live stable candidates; not an installed lockfile or compatibility approval. Refresh before adoption.',
              'entries':records}
    path = Path(__file__).resolve().parents[1] / 'conductor/evidence/dependency-candidates.json'
    path.write_text(json.dumps(report,indent=2)+'\n')
    for record in records:
        print(record['name'], record.get('version','UNAVAILABLE'), 'MSRV='+str(record.get('rust_version','n/a')))
    print(f'Recorded {len(records)} registry observations in {path}')
    return int(any(r['status']=='unavailable' for r in records))

if __name__ == '__main__':
    raise SystemExit(main())
