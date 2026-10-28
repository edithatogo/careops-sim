"""Inventory an actual ZIP-family release artifact without executing or extracting it."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import stat
import zipfile
from datetime import UTC, datetime
from pathlib import Path, PurePosixPath

MAX_FILES = 10_000
MAX_BYTES = 512 * 1024 * 1024


def build(artifact: Path, name: str, source_commit: str, created_epoch: int) -> dict:
    if artifact.is_symlink() or not artifact.is_file():
        raise ValueError('A regular release artifact is required.')
    if not re.fullmatch(r'[a-f0-9]{40}', source_commit):
        raise ValueError('An exact source commit is required.')
    if not name.strip() or len(name) > 160:
        raise ValueError('A bounded artifact name is required.')
    if created_epoch < 0 or artifact.stat().st_size > MAX_BYTES:
        raise ValueError('Invalid timestamp or excessive artifact size.')
    with artifact.open('rb') as content:
        digest = hashlib.file_digest(content, 'sha256').hexdigest()
    files = []
    identities = set()
    total = 0
    with zipfile.ZipFile(artifact) as archive:
        if len(archive.infolist()) > MAX_FILES:
            raise ValueError('Too many archive entries.')
        for entry in archive.infolist():
            relative = entry.filename.removeprefix('./')
            path = PurePosixPath(relative)
            if not relative or path.is_absolute() or re.match(r'^[A-Za-z]:', relative) or any(ord(c) < 32 or ord(c) == 127 for c in relative) or '..' in path.parts or '\\' in relative or str(path) != relative.rstrip('/'):
                raise ValueError('Unsafe or ambiguous archive member.')
            if relative.rstrip('/') in identities:
                raise ValueError('Duplicate archive identity.')
            identities.add(relative.rstrip('/'))
            mode = entry.external_attr >> 16
            if stat.S_ISLNK(mode) or (stat.S_IFMT(mode) not in [0, stat.S_IFREG, stat.S_IFDIR]):
                raise ValueError('Archive links or special entries are unsupported.')
            if entry.flag_bits & 1:
                raise ValueError('Encrypted content cannot be qualified.')
            if entry.is_dir():
                continue
            total += entry.file_size
            if len(files) >= MAX_FILES or total > MAX_BYTES:
                raise ValueError('Archive exceeds the bounded inventory limits.')
            with archive.open(entry) as content:
                data = content.read(entry.file_size + 1)
            if len(data) != entry.file_size:
                raise ValueError('Archive content does not match declared size.')
            files.append((relative, hashlib.sha256(data).hexdigest(), hashlib.sha1(data, usedforsecurity=False).hexdigest()))
    if not files:
        raise ValueError('An empty archive is not a release package.')
    created = datetime.fromtimestamp(created_epoch, UTC).strftime('%Y-%m-%dT%H:%M:%SZ')
    document = {
        'spdxVersion': 'SPDX-2.3', 'dataLicense': 'CC0-1.0', 'SPDXID': 'SPDXRef-DOCUMENT',
        'name': name + ' artifact inventory',
        'documentNamespace': 'https://careops.example/sbom/' + source_commit + '/' + digest,
        'creationInfo': {'created': created, 'creators': ['Tool: careops-artifact-sbom/1']},
        'packages': [{
            'SPDXID': 'SPDXRef-Package', 'name': name,
            'downloadLocation': 'NOASSERTION', 'filesAnalyzed': True,
            'checksums': [{'algorithm': 'SHA256', 'checksumValue': digest}],
            'packageVerificationCode': {'packageVerificationCodeValue': hashlib.sha1(''.join(sorted(f[2] for f in files)).encode(), usedforsecurity=False).hexdigest()},
            'sourceInfo': 'Source commit: ' + source_commit,
            'licenseConcluded': 'NOASSERTION', 'licenseDeclared': 'NOASSERTION',
            'copyrightText': 'NOASSERTION',
            'comment': 'File inventory of this exact artifact. Resolved but unbundled dependencies require a separate lock-derived inventory; native operation and licence acceptance are not established.',
        }],
        'files': [],
        'relationships': [{'spdxElementId': 'SPDXRef-DOCUMENT', 'relationshipType': 'DESCRIBES', 'relatedSpdxElement': 'SPDXRef-Package'}],
    }
    for number, (relative, sha256, sha1) in enumerate(sorted(files), 1):
        identifier = 'SPDXRef-File-' + str(number)
        document['files'].append({'SPDXID': identifier, 'fileName': './' + relative,
                                 'checksums': [{'algorithm': 'SHA256', 'checksumValue': sha256}, {'algorithm': 'SHA1', 'checksumValue': sha1}],
                                 'licenseConcluded': 'NOASSERTION', 'copyrightText': 'NOASSERTION'})
        document['relationships'].append({'spdxElementId': 'SPDXRef-Package', 'relationshipType': 'CONTAINS', 'relatedSpdxElement': identifier})
    return document


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifact', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--name', required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--created-epoch', type=int, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.is_symlink():
        raise ValueError('Preserve the existing SBOM; choose a new output revision.')
    document = build(args.artifact, args.name, args.source_commit, args.created_epoch)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open('x') as output:
        output.write(json.dumps(document, indent=2) + '\n')
    print('Exact artifact file inventory generated; no publication or native operation.')


if __name__ == '__main__':
    main()
