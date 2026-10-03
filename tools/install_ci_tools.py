"""Install reviewed standalone Linux CI binaries from checksum-bound archives."""
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import tarfile
import urllib.request
import zipfile

root = Path(__file__).resolve().parents[1]
pins = json.loads((root / 'tools/ci-tools.json').read_text())
if platform.system() != 'Linux' or platform.machine() != 'x86_64':
    raise SystemExit('CI policy tools only support reviewed Linux x86_64 host')
bin_dir = root / '.artifacts/ci/bin'
bin_dir.mkdir(parents=True, exist_ok=True)
for name, key in [('actionlint', 'actionlint'), ('gitleaks', 'gitleaks'), ('cargo-deny', 'cargo-deny-archive'), ('zizmor', 'zizmor-archive')]:
    spec = pins[key]
    with urllib.request.urlopen(spec['url'], timeout=60) as response:
        data = response.read(80 * 1024 * 1024 + 1)
    if len(data) > 80 * 1024 * 1024 or hashlib.sha256(data).hexdigest() != spec['sha256']:
        raise SystemExit('archive size or checksum mismatch: ' + name)
    if spec['url'].endswith('.whl'):
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            candidates = [p for p in archive.namelist() if p.endswith('/scripts/' + name)]
            if len(candidates) != 1:
                raise SystemExit('ambiguous binary: ' + name)
            payload = archive.read(candidates[0])
    else:
        with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as archive:
            candidates = [p for p in archive.getmembers() if p.isfile() and Path(p.name).name == name]
            if len(candidates) != 1:
                raise SystemExit('ambiguous binary: ' + name)
            payload = archive.extractfile(candidates[0]).read()
    target = bin_dir / name
    target.write_bytes(payload)
    target.chmod(0o755)
    print(name + ': verified ' + spec['sha256'], flush=True)
with open(os.environ['GITHUB_PATH'], 'a') as output:
    output.write(str(bin_dir) + '\n')
