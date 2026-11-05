import copy
import json
from pathlib import Path
import unittest
from unittest import mock

from tools import ed_source_compatibility as gate

ROOT = Path(__file__).resolve().parents[1]


class EDSourceCompatibilityTests(unittest.TestCase):
    def setUp(self):
        self.receipt = json.loads((ROOT / gate.RECEIPT).read_text())
        self.reference = json.loads((ROOT / gate.REFERENCE).read_text())
        self.bindings = copy.deepcopy(self.receipt['bindings'])
        self.join = json.loads((ROOT / gate.JOIN_PATH).read_text())

    def validate(self, receipt):
        gate.validate_receipt(receipt, self.bindings, self.reference)

    def test_actual_frozen_and_current_source_binding(self):
        gate.verify_binding(ROOT, self.join)
        self.validate(self.receipt)

    def test_every_source_runtime_and_driver_binding_is_exact(self):
        for key in self.bindings:
            with self.subTest(key=key):
                value = copy.deepcopy(self.receipt)
                value['bindings'][key] = 'changed'
                with self.assertRaises(ValueError):
                    self.validate(value)

    def test_missing_duplicate_unknown_and_unequal_cases_fail(self):
        for field in ['input_sha256', 'baseline_summary_sha256', 'current_summary_sha256',
                      'baseline_exit', 'current_exit', 'case', 'repeat', 'kind']:
            value = copy.deepcopy(self.receipt)
            value['cases'][0][field] = 99 if 'exit' in field else 'changed'
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.validate(value)
        for change in [lambda v: v['cases'].pop(),
                       lambda v: v['cases'].__setitem__(1, v['cases'][0]),
                       lambda v: v.__setitem__('cases', {}),
                       lambda v: v['cases'].__setitem__(0, None),
                       lambda v: v['cases'][0].__setitem__('current_exit', False)]:
            value = copy.deepcopy(self.receipt)
            change(value)
            with self.assertRaises(ValueError):
                self.validate(value)

    def test_unknown_schema_seed_and_scope_do_not_inherit_a7_acceptance(self):
        for key, changed in [('schema_version', 2), ('schema_version', True), ('seed', 8),
                             ('scope', 'all_A7'), ('renewed_a7_acceptance', True)]:
            value = copy.deepcopy(self.receipt)
            value[key] = changed
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate(value)

    def test_historical_join_cannot_be_rewritten(self):
        for key in ['ed_source_sha256', 'qualified_runtime_tree', 'kairos_runtime']:
            value = copy.deepcopy(self.join)
            value[key] = '0' * 64
            with self.subTest(key=key), self.assertRaises(ValueError):
                gate.verify_binding(ROOT, value)

    def test_missing_frozen_git_object_fails_without_fallback(self):
        with mock.patch.object(gate, 'git_blob', side_effect=ValueError('required frozen source object unavailable')):
            with self.assertRaisesRegex(ValueError, 'frozen source object'):
                gate.verify_binding(ROOT, self.join)

    def test_changed_source_cannot_pass_without_receipt(self):
        with mock.patch.object(gate, 'RECEIPT', 'absent-compatibility-receipt.json'):
            with self.assertRaisesRegex(ValueError, 'needs exact compatibility'):
                gate.verify_binding(ROOT, self.join)

    def test_unrecorded_current_source_fails(self):
        original = gate.digest
        current = (ROOT / gate.SOURCE).read_bytes()
        with mock.patch.object(gate, 'digest', side_effect=lambda data: '0' * 64 if data == current else original(data)):
            with self.assertRaises(ValueError):
                gate.verify_binding(ROOT, self.join)

    def test_data_promotion_requires_exact_scope_and_all_bindings(self):
        receipt = json.loads((ROOT / gate.DATA_RECEIPT).read_text())
        expected = receipt['promotion_bindings']
        bindings = receipt['summary_compatibility']['bindings']
        gate.validate_data_receipt(receipt, expected, bindings, self.reference)
        for key in expected:
            value = copy.deepcopy(receipt)
            value['promotion_bindings'][key] = 'changed'
            with self.subTest(key=key), self.assertRaises(ValueError):
                gate.validate_data_receipt(value, expected, bindings, self.reference)
        for key, changed in [('scope', 'all_A7'), ('schema_version', True), ('renewed_a7_acceptance', True)]:
            value = copy.deepcopy(receipt)
            value[key] = changed
            with self.subTest(key=key), self.assertRaises(ValueError):
                gate.validate_data_receipt(value, expected, bindings, self.reference)
        value = copy.deepcopy(receipt)
        value['summary_compatibility']['cases'].pop()
        with self.assertRaises(ValueError):
            gate.validate_data_receipt(value, expected, bindings, self.reference)

    def test_missing_data_receipt_cannot_inherit_historical_parity(self):
        with mock.patch.object(gate, 'DATA_RECEIPT', 'absent-data-receipt.json'):
            with self.assertRaisesRegex(ValueError, 'data promotion needs exact'):
                gate.verify_binding(ROOT, self.join)

    def test_old_receipt_bytes_are_anchored_to_accepted_git_object(self):
        original = gate.git_blob
        def changed(root, commit, path):
            blob = original(root, commit, path)
            return blob + b' ' if commit == gate.HISTORICAL_RECEIPT_COMMIT and path == gate.RECEIPT else blob
        with mock.patch.object(gate, 'git_blob', side_effect=changed):
            with self.assertRaisesRegex(ValueError, 'historical compatibility receipt changed'):
                gate.verify_binding(ROOT, self.join)

    def test_equal_baseline_source_cannot_bypass_new_data_boundary(self):
        original = gate.digest
        current = (ROOT / gate.SOURCE).read_bytes()
        with mock.patch.object(gate, 'digest', side_effect=lambda data: self.join['ed_source_sha256'] if data == current else original(data)):
            with self.assertRaises(ValueError):
                gate.verify_binding(ROOT, self.join)

    def test_unknown_revision_and_changed_lock_or_manifest_fail(self):
        manifest = gate.tomllib.loads((ROOT / 'crates/careops-ed/Cargo.toml').read_text())
        manifest['dependencies']['kairo-ecs-arrow']['rev'] = '0' * 40
        with mock.patch.object(gate.tomllib, 'loads', return_value=manifest):
            with self.assertRaisesRegex(ValueError, 'unqualified data source'):
                gate.verify_binding(ROOT, self.join)
        original = gate.digest
        for path in ['Cargo.lock', 'crates/careops-ed/Cargo.toml']:
            content = (ROOT / path).read_bytes()
            with self.subTest(path=path), mock.patch.object(gate, 'digest', side_effect=lambda data: '0' * 64 if data == content else original(data)):
                with self.assertRaisesRegex(ValueError, 'data promotion scope or exact binding'):
                    gate.verify_binding(ROOT, self.join)

    def test_exact_metadata_edges_only(self):
        lock = gate.git_blob(ROOT, gate.NATIVE_RECEIPT_COMMIT, 'Cargo.lock')
        manifest = gate.git_blob(ROOT, gate.NATIVE_RECEIPT_COMMIT, 'crates/careops-ed/Cargo.toml')
        gate.validate_metadata_edges(ROOT, lock, manifest)
        for old, new in [(b'=0.2.1', b'=0.2.2'), (b'default-features = false, features = ["ipc"]', b'default-features = 0, features = ["ipc"]'), (b'features = ["ipc"]', b'features = ["ipc", "flight-data"]'),
                         (b'default-features = false, features = ["ipc"]', b'default-features = true, features = ["ipc"]'),
                         (b'optional = true }\npolars-parquet-format', b'optional = false }\npolars-parquet-format')]:
            self.assertIn(old, manifest)
            with self.subTest(change=new), self.assertRaises(ValueError):
                gate.validate_metadata_edges(ROOT, lock, manifest.replace(old, new))
        with self.assertRaises(ValueError):
            gate.validate_metadata_edges(ROOT, lock + b'\nunknown_profile = true\n', manifest)
        with self.assertRaises(ValueError):
            gate.validate_metadata_edges(ROOT, lock.replace(b'version = "0.2.1"', b'version = "0.2.2"'), manifest)

    def test_native_receipt_all_bindings_scope_and_cases_are_exact(self):
        data = (ROOT / gate.DATA_RECEIPT).read_bytes()
        expected = gate.native_bindings(ROOT, data)
        summary = json.loads(data)['summary_compatibility']
        value = {'schema_version': 1, 'scope': 'native_wire_dependencies_summary_parity_only',
                 'renewed_a7_acceptance': False, 'native_bindings': expected,
                 'summary_compatibility': summary}
        bindings = summary['bindings']
        gate.validate_native_receipt(value, expected, bindings, self.reference)
        for key in expected:
            changed = copy.deepcopy(value)
            changed['native_bindings'][key] = 'changed'
            with self.subTest(key=key), self.assertRaises(ValueError):
                gate.validate_native_receipt(changed, expected, bindings, self.reference)
        for key, replacement in [('scope', 'all_A7'), ('schema_version', True), ('renewed_a7_acceptance', True)]:
            changed = copy.deepcopy(value); changed[key] = replacement
            with self.assertRaises(ValueError):
                gate.validate_native_receipt(changed, expected, bindings, self.reference)
        changed = copy.deepcopy(value)
        changed['native_bindings']['metadata_edges']['polars-arrow-format']['default-features'] = 0
        with self.assertRaises(ValueError):
            gate.validate_native_receipt(changed, expected, bindings, self.reference)
        changed = copy.deepcopy(value); changed['summary_compatibility']['cases'].pop()
        with self.assertRaises(ValueError):
            gate.validate_native_receipt(changed, expected, bindings, self.reference)

    def test_data_receipt_is_immutable_and_native_receipt_is_required(self):
        original = gate.git_blob
        def changed(root, commit, path):
            raw = original(root, commit, path)
            return raw + b' ' if commit == gate.DATA_RECEIPT_COMMIT and path == gate.DATA_RECEIPT else raw
        with mock.patch.object(gate, 'git_blob', side_effect=changed):
            with self.assertRaisesRegex(ValueError, 'immutable data compatibility'):
                gate.verify_binding(ROOT, self.join)
        with mock.patch.object(gate, 'NATIVE_RECEIPT', 'absent-native-receipt.json'):
            with self.assertRaisesRegex(ValueError, 'native wire dependencies need exact'):
                gate.verify_binding(ROOT, self.join)


    def test_native_wire_receipt_remains_immutable_during_cli_cutover(self):
        original=gate.git_blob
        def changed(root,commit,path):
            raw=original(root,commit,path)
            return raw+b' ' if commit==gate.NATIVE_RECEIPT_COMMIT and path==gate.NATIVE_RECEIPT else raw
        with mock.patch.object(gate,'git_blob',side_effect=changed):
            with self.assertRaisesRegex(ValueError,'immutable native wire'):
                gate.verify_binding(ROOT,self.join)

    def test_cli_cutover_receipt_is_closed_and_preserves_all_cases(self):
        native=(ROOT/gate.NATIVE_RECEIPT).read_bytes()
        expected=gate.cli_bindings(ROOT,native)
        summary=json.loads(native)['summary_compatibility']; bindings=summary['bindings']
        receipt={'schema_version':1,'scope':'native_cli_cutover_summary_parity_only',
                 'renewed_a7_acceptance':False,'cli_bindings':expected,'summary_compatibility':summary}
        gate.validate_cli_receipt(receipt,expected,bindings,self.reference)
        for key in expected:
            value=copy.deepcopy(receipt);value['cli_bindings'][key]='changed'
            with self.subTest(key=key),self.assertRaises(ValueError):gate.validate_cli_receipt(value,expected,bindings,self.reference)
        for key,value in [('schema_version',True),('scope','all_A7'),('renewed_a7_acceptance',True)]:
            changed=copy.deepcopy(receipt);changed[key]=value
            with self.assertRaises(ValueError):gate.validate_cli_receipt(changed,expected,bindings,self.reference)
        changed=copy.deepcopy(receipt);changed['summary_compatibility']['cases'].pop()
        with self.assertRaises(ValueError):gate.validate_cli_receipt(changed,expected,bindings,self.reference)

    def test_cli_case_identity_cannot_coerce_boolean_or_float(self):
        native=(ROOT/gate.NATIVE_RECEIPT).read_bytes(); expected=gate.cli_bindings(ROOT,native)
        summary=json.loads(native)['summary_compatibility']; bindings=summary['bindings']
        for key,value in [('case',100.0),('repeat',False)]:
            changed=copy.deepcopy(summary);changed['cases'][0][key]=value
            receipt={'schema_version':1,'scope':'native_cli_cutover_summary_parity_only','renewed_a7_acceptance':False,'cli_bindings':expected,'summary_compatibility':changed}
            with self.assertRaises(ValueError):gate.validate_cli_receipt(receipt,expected,bindings,self.reference)

    def test_missing_cli_cutover_receipt_cannot_pass(self):
        with mock.patch.object(gate,'CLI_RECEIPT','absent-cli-receipt.json'):
            with self.assertRaisesRegex(ValueError,'CLI cutover needs exact'):
                gate.verify_binding(ROOT,self.join)


class FormattedReaderProfileTests(unittest.TestCase):
    def test_formatted_reader_requires_its_new_receipt(self):
        join = json.loads((ROOT / gate.JOIN_PATH).read_text())
        with mock.patch.object(gate, 'FORMAT_RECEIPT', 'absent-format-receipt.json'):
            with self.assertRaisesRegex(ValueError, 'formatted native reader needs exact'):
                gate.verify_binding(ROOT, join)

    def test_unknown_reader_source_cannot_inherit_format_acceptance(self):
        join = json.loads((ROOT / gate.JOIN_PATH).read_text())
        with mock.patch.object(gate, 'FORMATTED_READER_SHA', '0' * 64):
            with self.assertRaisesRegex(ValueError, 'unqualified native reader'):
                gate.verify_binding(ROOT, join)

    def test_format_case_identity_rejects_numeric_aliases(self):
        native = json.loads((ROOT / gate.NATIVE_RECEIPT).read_text())
        summary = native['summary_compatibility']
        reference = json.loads((ROOT / gate.REFERENCE).read_text())
        for key, replacement in [('case', float(summary['cases'][0]['case'])),
                                 ('repeat', bool(summary['cases'][0]['repeat']))]:
            changed = copy.deepcopy(native)
            changed['summary_compatibility']['cases'][0][key] = replacement
            with self.subTest(key=key), self.assertRaises(ValueError):
                gate.validate_native_receipt(changed, native['native_bindings'],
                                             summary['bindings'], reference)


class SharedImportPromotionTests(unittest.TestCase):
    """Run only after the real first adoption commit and receipt exist.

    There is no skip/fallback or generated passing receipt for a pending pin.
    """
    def setUp(self):
        self.receipt = json.loads((ROOT / gate.SHARED_RECEIPT).read_bytes())
        self.reference = json.loads((ROOT / gate.REFERENCE).read_bytes())
        self.join = json.loads((ROOT / gate.JOIN_PATH).read_bytes())
        self.expected = gate.shared_bindings(ROOT, gate.SHARED_QUALIFICATION_COMMIT)
        self.bindings = self.receipt['summary_compatibility']['bindings']

    def test_actual_current_shared_profile_and_immutable_first_receipt(self):
        gate.verify_binding(ROOT, self.join)
        self.assertTrue(gate.strict_equal(self.receipt['promotion_bindings'], self.expected))
        self.assertEqual((ROOT / gate.SHARED_RECEIPT).read_bytes(),
                         gate.git_blob(ROOT, gate.SHARED_QUALIFICATION_COMMIT, gate.SHARED_RECEIPT))
        current = gate.shared_bindings(ROOT)
        successor = gate.shared_bindings(ROOT, gate.SHA2_QUALIFICATION_COMMIT)
        sha2_changed = [p for p, value in self.expected['source_sha256'].items()
                        if successor['source_sha256'][p] != value]
        self.assertEqual(sha2_changed, ['crates/careops-ed/Cargo.toml'])
        self.assertTrue(gate.strict_equal(current, gate.shared_bindings(ROOT, gate.THISERROR_QUALIFICATION_COMMIT)))
        changed = [p for p, value in self.expected['source_sha256'].items()
                   if current['source_sha256'][p] != value]
        self.assertEqual(changed, ['Cargo.lock', 'crates/careops-ed/Cargo.toml'])

    def test_shared_promotion_requires_its_new_receipt(self):
        with mock.patch.object(gate, 'SHARED_RECEIPT', 'absent-shared-receipt.json'):
            with self.assertRaisesRegex(ValueError, 'shared trusted import promotion needs exact'):
                gate.verify_binding(ROOT, self.join)

    def test_every_promotion_critical_source_is_bound_to_first_snapshot(self):
        original = Path.read_bytes
        for path in gate.SHARED_SOURCES:
            selected = ROOT / path
            def changed(current, selected=selected):
                data = original(current)
                return data + b'\n' if current == selected else data
            with self.subTest(path=path), mock.patch.object(Path, 'read_bytes', changed):
                with self.assertRaisesRegex(ValueError, 'data promotion scope or exact binding'):
                    gate.verify_binding(ROOT, self.join)

    def test_source_and_receipt_codrift_cannot_rewrite_accepted_profile(self):
        source = ROOT / 'crates/careops-ed/src/native_reader.rs'
        receipt_path = ROOT / gate.SHARED_RECEIPT
        original = Path.read_bytes
        source_bytes = original(source) + b'\n'
        value = copy.deepcopy(self.receipt)
        value['promotion_bindings']['source_sha256'][str(source.relative_to(ROOT))] = gate.digest(source_bytes)
        def changed(current):
            if current == source:
                return source_bytes
            if current == receipt_path:
                return json.dumps(value).encode()
            return original(current)
        with mock.patch.object(Path, 'read_bytes', changed):
            with self.assertRaisesRegex(ValueError, 'immutable shared trusted import'):
                gate.verify_binding(ROOT, self.join)

    def test_new_receipt_anchor_cannot_change(self):
        original = gate.git_blob
        def changed(root, commit, path):
            data = original(root, commit, path)
            return data + b' ' if commit == gate.SHARED_QUALIFICATION_COMMIT and path == gate.SHARED_RECEIPT else data
        with mock.patch.object(gate, 'git_blob', changed):
            with self.assertRaisesRegex(ValueError, 'immutable shared trusted import'):
                gate.verify_binding(ROOT, self.join)

    def test_exact_root_default_feature_target_and_importer_pin(self):
        manifest = gate.tomllib.loads((ROOT / 'Cargo.toml').read_text())
        gate.validate_shared_root_manifest(manifest)
        for change in [
            lambda v: v['features'].__setitem__('default', ['native-ffi-acceptance']),
            lambda v: v['features'].__setitem__('unexpected', []),
            lambda v: v['test'][0].__setitem__('required-features', []),
            lambda v: v['dependencies']['kairo-data-ffi'].__setitem__('rev', '0' * 40),
            lambda v: v['dependencies']['kairo-data-ffi'].__setitem__('package', 'kairo-ecs-ffi'),
            lambda v: v['dependencies']['kairo-data-ffi'].__setitem__('optional', 1),
        ]:
            value = copy.deepcopy(manifest)
            change(value)
            with self.assertRaisesRegex(ValueError, 'shared root feature/dependency/test profile'):
                gate.validate_shared_root_manifest(value)

    def test_shared_receipt_scope_bindings_cases_and_exact_types(self):
        gate.validate_shared_receipt(self.receipt, self.expected, self.bindings, self.reference)
        for key in self.expected:
            value = copy.deepcopy(self.receipt)
            value['promotion_bindings'][key] = 'changed'
            with self.subTest(binding=key), self.assertRaises(ValueError):
                gate.validate_shared_receipt(value, self.expected, self.bindings, self.reference)
        value = copy.deepcopy(self.receipt)
        value['summary_compatibility']['seed'] = 7.0
        with self.assertRaises(ValueError):
            gate.validate_shared_receipt(value, self.expected, self.bindings, self.reference)
        for key, replacement in [('case', 100.0), ('repeat', True), ('current_exit', False)]:
            value = copy.deepcopy(self.receipt)
            value['summary_compatibility']['cases'][0][key] = replacement
            with self.subTest(case_field=key), self.assertRaises(ValueError):
                gate.validate_shared_receipt(value, self.expected, self.bindings, self.reference)
        for key, replacement in [('schema_version', True), ('scope', 'all_R14'),
                                 ('renewed_a7_acceptance', True)]:
            value = copy.deepcopy(self.receipt)
            value[key] = replacement
            with self.subTest(field=key), self.assertRaises(ValueError):
                gate.validate_shared_receipt(value, self.expected, self.bindings, self.reference)
        value = copy.deepcopy(self.receipt)
        value['summary_compatibility']['cases'].pop()
        with self.assertRaises(ValueError):
            gate.validate_shared_receipt(value, self.expected, self.bindings, self.reference)
        for key, replacement in [('data_commit', '0' * 40), ('runtime_commit', '0' * 40),
                                 ('polars_commit', '0' * 40)]:
            changed = dict(self.bindings, **{key: replacement})
            with self.subTest(pin=key), self.assertRaises(ValueError):
                gate.validate_shared_receipt(self.receipt, self.expected, changed, self.reference)


class Sha2ManifestSuccessorTests(unittest.TestCase):
    def setUp(self):
        self.old = gate.git_blob(ROOT, gate.SHARED_QUALIFICATION_COMMIT, 'crates/careops-ed/Cargo.toml')
        self.new = self.old.replace(b'[dependencies]\n', b'[dependencies]\nsha2.workspace = true\n', 1)
        self.lock = gate.git_blob(ROOT, gate.SHARED_QUALIFICATION_COMMIT, 'Cargo.lock')

    def validate(self, manifest, lock=None):
        gate.validate_sha2_successor_manifest(self.old, manifest, self.lock, self.lock if lock is None else lock)

    def test_exact_normal_edge_only(self):
        self.validate(self.new)

    def test_unknown_dependency_feature_version_and_type_refuse(self):
        for candidate in [self.old, self.new.replace(b'sha2.workspace = true', b'sha2.workspace = false'), self.new.replace(b'sha2.workspace = true', b'sha2 = "=0.11.2"'), self.new.replace(b'tempfile = {', b'unknown = "=1.0.0"\ntempfile = {'), self.new.replace(b'default = ["native-data"]', b'default = []'), self.new.replace(b'=3.27.0', b'=3.28.0')]:
            with self.subTest(candidate=candidate):
                with self.assertRaises(ValueError): self.validate(candidate)

    def test_lock_drift_refuses_even_when_only_whitespace(self):
        with self.assertRaises(ValueError): self.validate(self.new, self.lock + b'\n')


class Sha2AnchoredSuccessorTests(unittest.TestCase):
    def setUp(self):
        self.join = json.loads((ROOT / gate.JOIN_PATH).read_bytes())
        self.receipt = json.loads((ROOT / gate.SHA2_RECEIPT).read_bytes())

    def test_actual_qualified_successor_and_original_receipt_both_validate(self):
        gate.verify_binding(ROOT, self.join)

    def test_original_exact_profile_remains_valid(self):
        expected = gate.shared_bindings(ROOT, gate.SHARED_QUALIFICATION_COMMIT)
        gate.validate_shared_receipt(
            json.loads((ROOT / gate.SHARED_RECEIPT).read_bytes()), expected,
            json.loads((ROOT / gate.SHARED_RECEIPT).read_bytes())['summary_compatibility']['bindings'],
            json.loads((ROOT / gate.REFERENCE).read_bytes()))
        gate.verify_sha2_successor(ROOT, expected,
                                   gate.shared_bindings(ROOT, gate.SHA2_QUALIFICATION_COMMIT))
        with self.assertRaises(ValueError):
            gate.verify_thiserror_successor(ROOT, expected, expected)

    def test_every_new_digest_build_and_contract_source_drift_refuses(self):
        original = Path.read_bytes
        for path in gate.SHA2_SOURCES:
            selected = ROOT / path
            def changed(current):
                data = original(current)
                return data + b'\n' if current == selected else data
            with self.subTest(path=path), mock.patch.object(Path, 'read_bytes', changed):
                with self.assertRaisesRegex(ValueError, 'exact binding mismatch'):
                    gate.verify_binding(ROOT, self.join)

    def test_mutable_receipt_and_source_codrift_cannot_rebind_successor(self):
        selected = ROOT / gate.SHA2_SOURCES[0]
        receipt_path = ROOT / gate.SHA2_RECEIPT
        original = Path.read_bytes
        altered = original(selected) + b'\n'
        receipt = copy.deepcopy(self.receipt)
        receipt['source_bindings'][0]['sha256'] = gate.digest(altered)
        def changed(current):
            if current == selected: return altered
            if current == receipt_path: return json.dumps(receipt).encode()
            return original(current)
        with mock.patch.object(Path, 'read_bytes', changed):
            with self.assertRaisesRegex(ValueError, 'immutable SHA2 successor'):
                gate.verify_binding(ROOT, self.join)

    def test_missing_changed_or_nonobject_receipt_refuses(self):
        receipt_path = ROOT / gate.SHA2_RECEIPT
        original = Path.read_bytes
        for value in [b'[]', original(receipt_path)+b'\n']:
            def changed(current): return value if current == receipt_path else original(current)
            with self.subTest(value=value[:20]), mock.patch.object(Path, 'read_bytes', changed):
                with self.assertRaisesRegex(ValueError, 'immutable SHA2 successor'):
                    gate.verify_binding(ROOT, self.join)
        with mock.patch.object(gate, 'SHA2_RECEIPT', 'missing-sha2-receipt.json'):
            with self.assertRaisesRegex(ValueError, 'SHA2 successor needs exact compatibility'):
                gate.verify_binding(ROOT, self.join)

    def test_immutable_anchor_receipt_and_manifest_cannot_drift(self):
        original = gate.git_blob
        for path in [gate.SHA2_RECEIPT, 'crates/careops-ed/Cargo.toml']:
            def changed(root, commit, name):
                value = original(root, commit, name)
                return value+b'\n' if commit == gate.SHA2_QUALIFICATION_COMMIT and name == path else value
            with self.subTest(path=path), mock.patch.object(gate, 'git_blob', changed):
                with self.assertRaises(ValueError): gate.verify_binding(ROOT, self.join)


class TempfileManifestSuccessorTests(unittest.TestCase):
    def setUp(self):
        self.old = {p: gate.git_blob(ROOT, gate.TEMPFILE_BASE_COMMIT, p) for p in gate.TEMPFILE_MANIFESTS}
        self.new = {p: gate.git_blob(ROOT, gate.TEMPFILE_QUALIFICATION_COMMIT, p) for p in gate.TEMPFILE_MANIFESTS}
        self.old_lock = gate.git_blob(ROOT, gate.TEMPFILE_BASE_COMMIT, 'Cargo.lock')
        self.lock = gate.git_blob(ROOT, gate.TEMPFILE_QUALIFICATION_COMMIT, 'Cargo.lock')

    def validate(self, manifests=None, lock=None):
        gate.validate_tempfile_successor_profile(self.old, self.new if manifests is None else manifests,
                                                self.old_lock, self.lock if lock is None else lock)

    def test_exact_three_edges_and_two_workspace_rows(self):
        self.validate()

    def test_manifest_versions_features_unknown_edges_and_types_refuse(self):
        for path in gate.TEMPFILE_MANIFESTS:
            for transform in [lambda b: b.replace(b'=3.27.0', b'=3.28.0'),
                              lambda b: b + b'unknown = "=1.0.0"\n',
                              lambda b: b.replace(b'tempfile = "=3.27.0"', b'tempfile = false'),
                              lambda b: b.replace(b'tempfile = "=3.27.0"', b'tempfile = { version = "=3.27.0", default-features = false }')]:
                manifests = dict(self.new); manifests[path] = transform(manifests[path])
                with self.subTest(path=path), self.assertRaises(ValueError): self.validate(manifests)

    def test_registry_checksum_version_extra_row_root_and_other_workspace_changes_refuse(self):
        for lock in [self.old_lock, self.lock.replace(b'version = 4', b'version = 3', 1),
                     self.lock.replace(b'checksum = "', b'checksum = "0', 1),
                     self.lock + b'\n[[package]]\nname="unknown"\nversion="1.0.0"\n',
                     self.lock.replace(b'"serde_json",', b'"serde_json",\n "unknown",', 1)]:
            with self.subTest(lock=lock[:20]), self.assertRaises(ValueError): self.validate(lock=lock)


class TempfileAnchoredSuccessorTests(unittest.TestCase):
    def setUp(self):
        self.join = json.loads((ROOT / gate.JOIN_PATH).read_bytes())

    def test_actual_qualified_successor_preserves_old_receipts(self):
        gate.verify_binding(ROOT, self.join)

    def test_every_new_bound_file_refuses_unknown_drift(self):
        original = Path.read_bytes
        for path in gate.TEMPFILE_SOURCES:
            selected = ROOT / path
            def changed(current):
                data = original(current)
                return data + b'\n' if current == selected else data
            with self.subTest(path=path), mock.patch.object(Path, 'read_bytes', changed):
                with self.assertRaisesRegex(ValueError, 'exact binding mismatch'):
                    gate.verify_binding(ROOT, self.join)

    def test_source_and_receipt_codrift_cannot_rebind(self):
        original = Path.read_bytes
        selected = ROOT / 'crates/careops-core/src/experiment/io.rs'
        altered = original(selected) + b'\n'
        receipt = json.loads((ROOT / gate.TEMPFILE_RECEIPT).read_bytes())
        for row in receipt['source_bindings']:
            if row['path'] == 'crates/careops-core/src/experiment/io.rs': row['sha256'] = gate.digest(altered)
        def changed(current):
            if current == selected: return altered
            if current == ROOT / gate.TEMPFILE_RECEIPT: return json.dumps(receipt).encode()
            return original(current)
        with mock.patch.object(Path, 'read_bytes', changed), self.assertRaisesRegex(ValueError, 'immutable Tempfile successor'):
            gate.verify_binding(ROOT, self.join)

    def test_missing_nonobject_modified_receipt_and_missing_anchor_refuse(self):
        original = Path.read_bytes
        for value in [b'[]', original(ROOT / gate.TEMPFILE_RECEIPT) + b'\n']:
            def changed(current): return value if current == ROOT / gate.TEMPFILE_RECEIPT else original(current)
            with mock.patch.object(Path, 'read_bytes', changed), self.assertRaisesRegex(ValueError, 'immutable Tempfile successor'):
                gate.verify_binding(ROOT, self.join)
        with mock.patch.object(gate, 'TEMPFILE_QUALIFICATION_COMMIT', '0' * 40), self.assertRaisesRegex(ValueError, 'frozen source object'):
            gate.verify_binding(ROOT, self.join)
        with mock.patch.object(gate, 'TEMPFILE_RECEIPT', 'missing-tempfile-receipt.json'), self.assertRaisesRegex(ValueError, 'Tempfile successor needs exact'):
            gate.verify_binding(ROOT, self.join)

    def test_changed_base_anchor_cannot_inherit_acceptance(self):
        original = gate.git_blob
        def changed(root, commit, path):
            value = original(root, commit, path)
            return value + b'\n' if commit == gate.TEMPFILE_BASE_COMMIT and path == 'Cargo.lock' else value
        with mock.patch.object(gate, 'git_blob', changed), self.assertRaisesRegex(ValueError, 'base differs'):
            gate.verify_binding(ROOT, self.join)

    def test_initial_tempfile_receipt_is_immutable(self):
        original = Path.read_bytes
        selected = ROOT / gate.TEMPFILE_INITIAL_RECEIPT
        def changed(current):
            value = original(current)
            return value + b'\n' if current == selected else value
        with mock.patch.object(Path, 'read_bytes', changed), self.assertRaisesRegex(ValueError, 'immutable Tempfile initial'):
            gate.verify_binding(ROOT, self.join)


class ThiserrorSuccessorTests(unittest.TestCase):
    def setUp(self):
        self.original = gate.shared_bindings(ROOT, gate.SHARED_QUALIFICATION_COMMIT)
        self.current = gate.shared_bindings(ROOT)

    def verify(self):
        gate.verify_thiserror_successor(ROOT, self.original, self.current)

    def test_actual_closed_contract_checkpoint(self):
        self.assertEqual(len(gate.THISERROR_SOURCES), 29)
        self.verify()

    def test_historical_tempfile_adapter_is_explicit_and_fixed(self):
        previous = gate.shared_bindings(ROOT, gate.TEMPFILE_QUALIFICATION_COMMIT)
        gate.verify_tempfile_successor(ROOT, self.original, previous,
                                      source_checkpoint=gate.TEMPFILE_QUALIFICATION_COMMIT)
        with self.assertRaises(ValueError):
            gate.verify_tempfile_successor(ROOT, self.original, previous)
        for selector in (True, 'HEAD', gate.THISERROR_QUALIFICATION_COMMIT):
            with self.subTest(selector=selector), self.assertRaises(ValueError):
                gate.verify_tempfile_successor(ROOT, self.original, previous, source_checkpoint=selector)

    def test_all_29_current_files_refuse_drift(self):
        original = Path.read_bytes
        for path in gate.THISERROR_SOURCES:
            def changed(current, selected=ROOT / path):
                data = original(current)
                return data + b'\n' if current == selected else data
            with self.subTest(path=path), mock.patch.object(Path, 'read_bytes', changed):
                with self.assertRaises(ValueError): self.verify()

    def test_omitted_binding_and_source_receipt_codrift_refuse(self):
        original = Path.read_bytes
        path = ROOT / gate.THISERROR_ERROR_SOURCES[0]
        receipt_path = ROOT / gate.THISERROR_RECEIPT
        altered = original(path) + b'\n'
        for omit in (False, True):
            receipt = json.loads(original(receipt_path))
            if omit: receipt['source_bindings'].pop()
            else:
                for row in receipt['source_bindings']:
                    if row['path'] == gate.THISERROR_ERROR_SOURCES[0]: row['sha256'] = gate.digest(altered)
            def changed(current):
                if current == path: return altered
                if current == receipt_path: return json.dumps(receipt).encode()
                return original(current)
            with self.subTest(omit=omit), mock.patch.object(Path, 'read_bytes', changed):
                with self.assertRaisesRegex(ValueError, 'immutable thiserror source'): self.verify()

    def test_qualification_and_source_receipt_anchors_refuse_drift(self):
        original = gate.git_blob
        for anchor, path in [(gate.THISERROR_BASE_COMMIT, gate.THISERROR_QUALIFICATION),
                             (gate.THISERROR_QUALIFICATION_COMMIT, gate.THISERROR_RECEIPT)]:
            def changed(root, commit, name):
                data = original(root, commit, name)
                return data + b' ' if (commit, name) == (anchor, path) else data
            with self.subTest(path=path), mock.patch.object(gate, 'git_blob', changed):
                with self.assertRaisesRegex(ValueError, 'immutable thiserror'): self.verify()

    def test_closed_manifest_root_and_fuzz_projection(self):
        paths = ('crates/careops-core/Cargo.toml', 'crates/careops-ed/Cargo.toml')
        old = {p: gate.git_blob(ROOT, gate.THISERROR_BASE_COMMIT, p) for p in paths}
        current = {p: (ROOT / p).read_bytes() for p in paths}
        old_lock = gate.git_blob(ROOT, gate.THISERROR_BASE_COMMIT, 'Cargo.lock')
        old_fuzz = gate.git_blob(ROOT, gate.THISERROR_BASE_COMMIT, 'fuzz/Cargo.lock')
        lock = (ROOT / 'Cargo.lock').read_bytes(); fuzz = (ROOT / 'fuzz/Cargo.lock').read_bytes()
        q = json.loads((ROOT / gate.THISERROR_QUALIFICATION).read_bytes())
        def validate(manifests=current, root_lock=lock, fuzz_lock=fuzz):
            gate.validate_thiserror_successor_profile(old, manifests, old_lock, root_lock, old_fuzz, fuzz_lock, q)
        validate()
        for p in paths:
            for before, after in [(b'=2.0.21', b'=2.0.22'),
                                  (b'thiserror = "=2.0.21"', b'thiserror = { version = "=2.0.21", features = ["unknown"] }')]:
                altered = dict(current); altered[p] = current[p].replace(before, after)
                with self.subTest(path=p), self.assertRaises(ValueError): validate(manifests=altered)
        for raw, key in [(lock, 'root_lock'), (fuzz, 'fuzz_lock')]:
            for altered in [raw.replace(b'checksum = "', b'checksum = "0', 1),
                            raw + b'\n[[package]]\nname="unknown"\nversion="1.0.0"\n',
                            raw + b'\nunknown_metadata = true\n']:
                with self.subTest(key=key), self.assertRaises(ValueError): validate(**{key: altered})
