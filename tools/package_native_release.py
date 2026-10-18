#!/usr/bin/env python3
"""Package a previously compiled native CLI; never invent missing artifacts."""
import argparse
import hashlib
import json
from pathlib import Path
import stat
import zipfile


def package(binary: Path, license_file: Path, output: Path, source: str, epoch: int, scenario: Path):
    import re
    if not re.fullmatch(r'[a-f0-9]{40}', source):
        raise ValueError('An exact source revision is required.')
    for path in (binary, license_file, scenario):
        if path.is_symlink() or not path.is_file():
            raise ValueError('Real regular binary and licence files are required.')
    data = binary.read_bytes()
    # ELF or 64-bit Mach-O. Shell scripts and rehearsal stubs are never eligible.
    if data[:4] not in (b'\x7fELF', b'\xcf\xfa\xed\xfe', b'\xfe\xed\xfa\xcf'):
        raise ValueError('A compiled ELF or Mach-O executable is required.')
    if not binary.stat().st_mode & stat.S_IXUSR:
        raise ValueError('The compiled CLI must be executable.')
    if not license_file.read_bytes().strip():
        raise ValueError('A nonempty source licence is required.')
    if output.exists() or output.is_symlink():
        raise ValueError('Preserve existing release revisions.')
    from datetime import datetime, timezone
    timestamp = datetime.fromtimestamp(epoch, timezone.utc)
    if not 1980 <= timestamp.year <= 2107:
        raise ValueError('Timestamp is outside the ZIP format range.')
    manifest = {'sourceCommit': source, 'binarySha256': hashlib.sha256(data).hexdigest(),
                'scope': 'Actual native CLI package; runtime qualification and signatures are separate.'}
    json.loads(scenario.read_text())
    files = {'bin/careops-ed': data, 'LICENSE': license_file.read_bytes(),
             'examples/nominal_mvp.json': scenario.read_bytes(),
             'build.json': (json.dumps(manifest, sort_keys=True, indent=2) + '\n').encode()}
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('xb') as stream, zipfile.ZipFile(stream, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, timestamp.timetuple()[:6])
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | (0o755 if name.startswith('bin/') else 0o644)) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, content)
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--license', type=Path, required=True)
    parser.add_argument('--scenario', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--created-epoch', type=int, required=True)
    args = parser.parse_args()
    package(args.binary, args.license, args.output, args.source_commit, args.created_epoch, args.scenario)


if __name__ == '__main__':
    main()
