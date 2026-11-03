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
        lock = (ROOT / 'Cargo.lock').read_bytes()
        manifest = (ROOT / 'crates/careops-ed/Cargo.toml').read_bytes()
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
