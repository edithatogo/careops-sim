"""Install reviewed standalone Linux CI binaries from checksum-bound archives."""
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import tarfile
import urllib.request
import urllib.error
import time
import zipfile


MAX_ARCHIVE_BYTES = 80 * 1024 * 1024
TRANSIENT_HTTP = {408, 429, 500, 502, 503, 504}

def download(spec, name, opener=None, sleep=None):
    opener = urllib.request.urlopen if opener is None else opener
    sleep = time.sleep if sleep is None else sleep
    for attempt in range(5):
        try:
            with opener(spec['url'], timeout=60) as response:
                data = response.read(MAX_ARCHIVE_BYTES + 1)
        except urllib.error.HTTPError as error:
            if error.code not in TRANSIENT_HTTP or attempt == 4:
                raise
            print(f'{name}: transient HTTP {error.code}; retry {attempt + 1}/4', flush=True)
            sleep(0.5 * (2 ** attempt))
            continue
        except (TimeoutError, ConnectionError):
            if attempt == 4:
                raise
            print(f'{name}: transient transport failure; retry {attempt + 1}/4', flush=True)
            sleep(0.5 * (2 ** attempt))
            continue
        if len(data) > MAX_ARCHIVE_BYTES or hashlib.sha256(data).hexdigest() != spec['sha256']:
            raise SystemExit('archive size or checksum mismatch: ' + name)
        return data
    raise AssertionError('unreachable retry state')

def main():
    root = Path(__file__).resolve().parents[1]
    pins = json.loads((root / 'tools/ci-tools.json').read_text())
    if platform.system() != 'Linux' or platform.machine() != 'x86_64':
        raise SystemExit('CI policy tools only support reviewed Linux x86_64 host')
    bin_dir = root / '.artifacts/ci/bin'
    bin_dir.mkdir(parents=True, exist_ok=True)
    for name, key in [('actionlint', 'actionlint'), ('gitleaks', 'gitleaks'), ('cargo-deny', 'cargo-deny-archive'), ('zizmor', 'zizmor-archive')]:
        spec = pins[key]
        data = download(spec, name)
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

if __name__ == "__main__":
    main()
