#!/usr/bin/env python3
"""Package a previously compiled native CLI; never invent missing artifacts."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import stat
import zipfile


def package(binary: Path, license_file: Path, output: Path, source: str, epoch: int, scenario: Path, *, notices: dict[str, Path]):
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
    if not notices or 'project' not in notices:
        raise ValueError('The project notice and explicit upstream notices are required.')
    notice_bytes = {}
    for name, path in notices.items():
        identity = PurePosixPath(name)
        if (not name or not identity.parts or identity.is_absolute() or identity.as_posix() != name
                or any(part in ('.', '..') for part in identity.parts)
                or ":" in name or chr(92) in name):
            raise ValueError('Unsafe notice archive identity.')
        if path.is_symlink() or not path.is_file():
            raise ValueError('An original regular notice file is required.')
        content = path.read_bytes()
        if not content.strip():
            raise ValueError('Empty notices are not release evidence.')
        notice_bytes[name] = content
    manifest = {'noticeSha256': {name: hashlib.sha256(content).hexdigest()
                                for name, content in sorted(notice_bytes.items())},
                'noticeScope': 'Explicit project and pinned upstream notices; not a complete binary dependency closure.',
                'sourceCommit': source, 'binarySha256': hashlib.sha256(data).hexdigest(),
                'scope': 'Actual native CLI package; runtime qualification and signatures are separate.'}
    json.loads(scenario.read_text())
    files = {'bin/careops-ed': data, 'LICENSE': license_file.read_bytes(),
             'examples/nominal_mvp.json': scenario.read_bytes(),
             'build.json': (json.dumps(manifest, sort_keys=True, indent=2) + '\n').encode()}
    files.update({'notices/' + name: content for name, content in notice_bytes.items()})
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
    parser.add_argument('--notice', action='append', required=True, metavar='NAME=PATH')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--created-epoch', type=int, required=True)
    args = parser.parse_args()
    notices = {}
    for item in args.notice:
        name, separator, path = item.partition('=')
        if not separator or not path or name in notices:
            parser.error('Each notice requires a unique NAME=PATH.')
        notices[name] = Path(path)
    package(args.binary, args.license, args.output, args.source_commit, args.created_epoch, args.scenario, notices=notices)


if __name__ == '__main__':
    main()
