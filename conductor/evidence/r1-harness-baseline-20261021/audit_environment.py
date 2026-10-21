"""Read-only audit of the existing qualified schema-validation environment."""
import hashlib
import importlib.metadata
import json
from pathlib import Path
import platform
import re
import sys
import jsonschema

lock = Path('requirements-schema-validation.lock')
expected = dict(re.findall(r'^([A-Za-z0-9_-]+)==([^\s]+)', lock.read_text(), re.M))
packages = {}
for name, version in expected.items():
    distribution = importlib.metadata.distribution(name)
    metadata = distribution.read_text('METADATA')
    record = distribution.read_text('RECORD')
    packages[name] = {'version': distribution.version, 'expected_version': version,
        'metadata_sha256': hashlib.sha256(metadata.encode()).hexdigest(),
        'record_sha256': hashlib.sha256(record.encode()).hexdigest()}
module = Path(jsonschema.__file__).resolve()
errors = [name + ' version mismatch' for name, value in packages.items()
          if value['version'] != value['expected_version']]
if not module.is_relative_to(Path(sys.prefix).resolve()):
    errors.append('jsonschema resolved outside selected environment')
report = {'executable': sys.executable, 'resolved_executable': str(Path(sys.executable).resolve()),
    'python_version': sys.version, 'prefix': sys.prefix, 'base_prefix': sys.base_prefix,
    'jsonschema_file': str(module), 'platform': platform.platform(), 'machine': platform.machine(),
    'requirements_lock_sha256': hashlib.sha256(lock.read_bytes()).hexdigest(),
    'packages': packages, 'errors': errors}
print(json.dumps(report, indent=2))
sys.exit(bool(errors))
