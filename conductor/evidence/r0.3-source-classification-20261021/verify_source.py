"""Verify the retained R0.3 source hashes against immutable Git objects."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

manifest = json.loads(Path(sys.argv[1]).read_text())
failed = []
for path, expected in manifest['source_sha256'].items():
    data = subprocess.check_output(['git', 'show', manifest['parent_source'] + ':' + path])
    if hashlib.sha256(data).hexdigest() != expected:
        failed.append(path)
print(json.dumps({'checked': len(manifest['source_sha256']), 'mismatches': failed}))
sys.exit(bool(failed))
