"""Keep historical ED qualification immutable; require exact migration evidence."""
import copy
import hashlib
import json
import subprocess
import tomllib
from pathlib import Path

SOURCE = 'crates/careops-ed/src/lib.rs'
RECEIPT = 'conductor/evidence/polars-integration-20261103/source-compatibility.json'
DATA_RECEIPT = 'conductor/evidence/polars-integration-20261103/data-source-compatibility.json'
DATA_PROMOTION = 'f34750533c07e9e868481606e5fe5c89ec9b76be'
DATA_RECEIPT_COMMIT = '43511e665706cf31c434758675ce291d8c12547b'
NATIVE_RECEIPT = 'conductor/evidence/polars-integration-20261103/native-reader-source-compatibility.json'
NATIVE_SOURCES = ('crates/careops-ed/src/native_output.rs', 'crates/careops-ed/src/native_reader.rs',
                  'crates/careops-ed-cli/src/lib.rs', 'crates/careops-ed-cli/src/main.rs')
METADATA_EDGES = {
    'polars-arrow-format': {'version': '=0.2.1', 'default-features': False, 'features': ['ipc'], 'optional': True},
    'polars-parquet-format': {'version': '=0.1.0', 'default-features': False, 'optional': True},
}
NATIVE_RECEIPT_COMMIT = '1b6ca2deaead85f1bc0da50c1d6aa00df54e2cba'
CLI_RECEIPT = 'conductor/evidence/polars-integration-20261103/native-cli-source-compatibility.json'
CLI_DRIVERS = ('tools/ed_native_benchmarks.py', 'tools/ed_native_benchmark_process.py')
CLI_SOURCES = ('crates/careops-ed-cli/src/lib.rs', 'crates/careops-ed-cli/src/main.rs')
HISTORICAL_RECEIPT_COMMIT = '079c061d2bcc5a5fda373a3f9236d09235a2bca8'
REFERENCE = 'conductor/evidence/r103-remote-20261029/parity.json'
DRIVERS = ('tools/ed_benchmarks.py', 'tools/ed_benchmark_process.py',
           'crates/careops-ed/examples/one_patient.json')
JOIN_COMMIT = '99cb1c74ca93d9721e86e353ffb41d97aa4e7046'
JOIN_PATH = 'conductor/evidence/a7-two-consumer-20261103/join.json'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def strict_equal(left, right):
    """Keep Boolean/integer types distinct in closed parsed control contracts."""
    try:
        return json.dumps(left, sort_keys=True, allow_nan=False) == json.dumps(right, sort_keys=True, allow_nan=False)
    except (TypeError, ValueError):
        return False


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
    try:
        receipt_bytes = (root / RECEIPT).read_bytes()
        receipt = json.loads(receipt_bytes)
    except (OSError, ValueError) as error:
        raise ValueError('changed ED source needs exact compatibility evidence') from error
    if receipt_bytes != git_blob(root, HISTORICAL_RECEIPT_COMMIT, RECEIPT):
        raise ValueError('immutable historical compatibility receipt changed')
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
    if runtime != join['kairos_runtime']:
        raise ValueError('compatibility changes the qualified runtime boundary')
    if current_hash == baseline_hash and bindings['data_commit'] == join['native_data_component']:
        return
    reference = json.loads(reference_bytes)
    historical_bindings = dict(bindings, data_commit=join['native_data_component'])
    # The accepted original receipt is checked independently, never rewritten to
    # inherit a new data revision. Existing frozen source/driver constraints stay.
    validate_receipt(receipt, historical_bindings, reference)
    if bindings['data_commit'] == join['native_data_component']:
        return
    if bindings['data_commit'] != DATA_PROMOTION:
        raise ValueError('unqualified data source revision')
    expected = {
        'from_data_commit': join['native_data_component'],
        'to_data_commit': DATA_PROMOTION,
        'runtime_commit': runtime, 'polars_commit': bindings['polars_commit'],
        'current_source_sha256': current_hash,
        'lock_sha256': digest(git_blob(root, DATA_RECEIPT_COMMIT, 'Cargo.lock')),
        'ed_manifest_sha256': digest(git_blob(root, DATA_RECEIPT_COMMIT, 'crates/careops-ed/Cargo.toml')),
        'historical_receipt_sha256': digest(receipt_bytes),
        'historical_join_sha256': digest(git_blob(root, JOIN_COMMIT, JOIN_PATH)),
        'reference_sha256': digest(reference_bytes), 'driver_sha256': drivers,
    }
    try:
        data_bytes = (root / DATA_RECEIPT).read_bytes()
        data_receipt = json.loads(data_bytes)
    except (OSError, ValueError) as error:
        raise ValueError('data promotion needs exact compatibility evidence') from error
    if data_bytes != git_blob(root, DATA_RECEIPT_COMMIT, DATA_RECEIPT):
        raise ValueError('immutable data compatibility receipt changed')
    validate_data_receipt(data_receipt, expected, bindings, reference)
    lock_bytes = (root / 'Cargo.lock').read_bytes()
    manifest_bytes = (root / 'crates/careops-ed/Cargo.toml').read_bytes()
    if (digest(lock_bytes) == expected['lock_sha256']
            and digest(manifest_bytes) == expected['ed_manifest_sha256']):
        return
    validate_metadata_edges(root, lock_bytes, manifest_bytes)
    native_expected = native_bindings(root, data_bytes)
    try:
        native_bytes = (root / NATIVE_RECEIPT).read_bytes()
        native_receipt = json.loads(native_bytes)
    except (OSError, ValueError) as error:
        raise ValueError('data promotion scope or exact binding mismatch: native wire dependencies need exact compatibility evidence') from error
    if native_bytes != git_blob(root, NATIVE_RECEIPT_COMMIT, NATIVE_RECEIPT):
        raise ValueError('immutable native wire compatibility receipt changed')
    historical_native = copy.deepcopy(native_expected)
    historical_native['source_sha256'] = {p:digest(git_blob(root,NATIVE_RECEIPT_COMMIT,p)) for p in NATIVE_SOURCES}
    validate_native_receipt(native_receipt, historical_native, bindings, reference)
    if strict_equal(native_expected, historical_native):
        return
    for path in NATIVE_SOURCES:
        if path not in CLI_SOURCES and native_expected['source_sha256'][path] != historical_native['source_sha256'][path]:
            raise ValueError('unqualified native reader/output source change outside CLI cutover')
    try:
        cli_receipt = json.loads((root / CLI_RECEIPT).read_text())
    except (OSError, ValueError) as error:
        raise ValueError('native CLI cutover needs exact compatibility evidence') from error
    validate_cli_receipt(cli_receipt,cli_bindings(root,native_bytes),bindings,reference)


def validate_data_receipt(receipt, expected, bindings, reference):
    if (not isinstance(receipt, dict)
            or type(receipt.get('schema_version')) is not int or receipt['schema_version'] != 1
            or receipt.get('scope') != 'data_source_summary_parity_only'
            or receipt.get('renewed_a7_acceptance') is not False
            or receipt.get('promotion_bindings') != expected):
        raise ValueError('data promotion scope or exact binding mismatch')
    validate_receipt(receipt.get('summary_compatibility'), bindings, reference)


def validate_metadata_edges(root, lock_bytes, manifest_bytes):
    """Permit exactly two metadata interfaces already in the qualified closure."""
    old_manifest = tomllib.loads(git_blob(root, DATA_RECEIPT_COMMIT,
                                        'crates/careops-ed/Cargo.toml').decode())
    expected_manifest = copy.deepcopy(old_manifest)
    expected_manifest['dependencies'].update(copy.deepcopy(METADATA_EDGES))
    expected_manifest['features']['native-data'].extend('dep:' + name for name in METADATA_EDGES)
    expected_lock = tomllib.loads(git_blob(root, DATA_RECEIPT_COMMIT, 'Cargo.lock').decode())
    ed = [p for p in expected_lock['package'] if p['name'] == 'careops-ed']
    if len(ed) != 1:
        raise ValueError('qualified lock has no unique ED package')
    ed[0]['dependencies'] = sorted(ed[0]['dependencies'] + list(METADATA_EDGES))
    try:
        actual_manifest = tomllib.loads(manifest_bytes.decode())
        actual_lock = tomllib.loads(lock_bytes.decode())
        # Canonical JSON keeps TOML integer and Boolean types distinct.
        matched = strict_equal(actual_manifest, expected_manifest) and strict_equal(actual_lock, expected_lock)
    except (TypeError, ValueError, UnicodeError) as error:
        raise ValueError('data promotion scope or exact binding mismatch: malformed metadata profile') from error
    if not matched:
        raise ValueError('data promotion scope or exact binding mismatch: unknown metadata dependency profile')


def native_bindings(root, data_bytes):
    return {
        'data_receipt_commit': DATA_RECEIPT_COMMIT,
        'data_receipt_sha256': digest(data_bytes),
        'lock_sha256': digest((root / 'Cargo.lock').read_bytes()),
        'ed_manifest_sha256': digest((root / 'crates/careops-ed/Cargo.toml').read_bytes()),
        'metadata_edges': copy.deepcopy(METADATA_EDGES),
        'source_sha256': {p: digest((root / p).read_bytes()) for p in NATIVE_SOURCES},
    }


def validate_native_receipt(receipt, expected, bindings, reference):
    if (not isinstance(receipt, dict)
            or type(receipt.get('schema_version')) is not int or receipt['schema_version'] != 1
            or receipt.get('scope') != 'native_wire_dependencies_summary_parity_only'
            or receipt.get('renewed_a7_acceptance') is not False
            or not strict_equal(receipt.get('native_bindings'), expected)):
        raise ValueError('data promotion scope or exact binding mismatch: native wire receipt')
    validate_receipt(receipt.get('summary_compatibility'), bindings, reference)


def cli_bindings(root, native_bytes):
    return {
        'native_receipt_commit':NATIVE_RECEIPT_COMMIT,
        'native_receipt_sha256':digest(native_bytes),
        'lock_sha256':digest((root/'Cargo.lock').read_bytes()),
        'ed_manifest_sha256':digest((root/'crates/careops-ed/Cargo.toml').read_bytes()),
        'cli_manifest_sha256':digest((root/'crates/careops-ed-cli/Cargo.toml').read_bytes()),
        'source_sha256':{p:digest((root/p).read_bytes()) for p in CLI_SOURCES},
        'native_driver_sha256':{p:digest((root/p).read_bytes()) for p in CLI_DRIVERS},
        'driver_protocol':'careops.ed.native-benchmark.v1',
        'process_protocol':'careops.ed.native-benchmark-process.v1',
    }


def validate_cli_receipt(receipt, expected, bindings, reference):
    if (not isinstance(receipt,dict)
            or type(receipt.get('schema_version')) is not int or receipt['schema_version'] != 1
            or receipt.get('scope') != 'native_cli_cutover_summary_parity_only'
            or receipt.get('renewed_a7_acceptance') is not False
            or not strict_equal(receipt.get('cli_bindings'),expected)):
        raise ValueError('native CLI cutover scope or exact binding mismatch')
    summary=receipt.get('summary_compatibility')
    if not isinstance(summary,dict) or not isinstance(summary.get('cases'),list):
        raise ValueError('native CLI cutover needs typed case evidence')
    for case in summary['cases']:
        if (not isinstance(case,dict) or type(case.get('case')) is not int
                or type(case.get('repeat')) is not int or type(case.get('kind')) is not str):
            raise ValueError('native CLI cutover case identity must preserve exact types')
    validate_receipt(summary,bindings,reference)
