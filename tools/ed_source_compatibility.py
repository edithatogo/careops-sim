"""Keep historical ED qualification immutable; require exact migration evidence."""
import hashlib
import json
import subprocess
import tomllib
from pathlib import Path

SOURCE = 'crates/careops-ed/src/lib.rs'
RECEIPT = 'conductor/evidence/polars-integration-20261103/source-compatibility.json'
REFERENCE = 'conductor/evidence/r103-remote-20261029/parity.json'
DRIVERS = ('tools/ed_benchmarks.py', 'tools/ed_benchmark_process.py',
           'crates/careops-ed/examples/one_patient.json')
JOIN_COMMIT = '99cb1c74ca93d9721e86e353ffb41d97aa4e7046'
JOIN_PATH = 'conductor/evidence/a7-two-consumer-20261103/join.json'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git_blob(root, commit, path):
    try:
        return subprocess.check_output(['git', 'show', f'{commit}:{path}'], cwd=root,
                                       stderr=subprocess.PIPE)
    except subprocess.CalledProcessError as error:
        raise ValueError('required frozen source object unavailable') from error


def validate_receipt(receipt, bindings, reference):
    if not isinstance(receipt, dict) or type(receipt.get('schema_version')) is not int or receipt['schema_version'] != 1 or receipt.get('scope') != 'observer_capture_summary_parity_only':
        raise ValueError('unsupported compatibility receipt or scope')
    if receipt.get('renewed_a7_acceptance') is not False or receipt.get('seed') != 7:
        raise ValueError('receipt cannot renew historical A7 acceptance')
    if receipt.get('bindings') != bindings:
        raise ValueError('compatibility source/runtime/driver binding mismatch')
    rows = receipt.get('cases', [])
    expected = {(r['case'], r['repeat'], r['kind']): r for r in reference['measurements']}
    if not isinstance(rows, list) or len(rows) != 26 or len(expected) != 26:
        raise ValueError('all 26 frozen cases are required')
    seen = set()
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError('invalid compatibility case')
        key = (row.get('case'), row.get('repeat'), row.get('kind'))
        original = expected.get(key)
        if original is None or key in seen:
            raise ValueError('duplicate or unknown compatibility case')
        seen.add(key)
        wanted = original['repeat_summary_sha256']
        if (row.get('input_sha256') != original['input_sha256']
                or row.get('baseline_summary_sha256') != wanted
                or row.get('current_summary_sha256') != wanted
                or type(row.get('baseline_exit')) is not int or row['baseline_exit'] != 0
                or type(row.get('current_exit')) is not int or row['current_exit'] != 0):
            raise ValueError('compatibility case/input/result mismatch')


def verify_binding(root, join):
    root = Path(root)
    if join != json.loads(git_blob(root, JOIN_COMMIT, JOIN_PATH)):
        raise ValueError('immutable historical join changed')
    baseline = join['qualified_runtime_tree']
    baseline_hash = digest(git_blob(root, baseline, SOURCE))
    if baseline_hash != join['ed_source_sha256']:
        raise ValueError('historical join no longer binds its exact qualified source')
    current_hash = digest((root / SOURCE).read_bytes())
    if current_hash == baseline_hash:
        return
    try:
        receipt = json.loads((root / RECEIPT).read_text())
    except (OSError, ValueError) as error:
        raise ValueError('changed ED source needs exact compatibility evidence') from error
    reference_bytes = (root / REFERENCE).read_bytes()
    if reference_bytes != git_blob(root, baseline, REFERENCE):
        raise ValueError('frozen analytical reference drift')
    drivers = {p: digest((root / p).read_bytes()) for p in DRIVERS}
    if any(drivers[p] != digest(git_blob(root, baseline, p)) for p in DRIVERS):
        raise ValueError('frozen compatibility driver/input drift')
    runtime = subprocess.check_output(['git', '-C', str(root / 'libs/kairos'),
                                       'rev-parse', 'HEAD'], text=True).strip()
    manifest = tomllib.loads((root / 'crates/careops-ed/Cargo.toml').read_text())
    deps = manifest['dependencies']
    bindings = {
        'baseline_commit': baseline, 'baseline_source_sha256': baseline_hash,
        'current_source_sha256': current_hash, 'source_path': SOURCE,
        'runtime_commit': runtime, 'data_commit': deps['kairo-ecs-arrow']['rev'],
        'polars_commit': deps['polars']['rev'], 'driver_sha256': drivers,
        'reference_sha256': digest(reference_bytes),
    }
    if runtime != join['kairos_runtime'] or bindings['data_commit'] != join['native_data_component']:
        raise ValueError('compatibility changes the qualified runtime/data boundary')
    validate_receipt(receipt, bindings, json.loads(reference_bytes))
