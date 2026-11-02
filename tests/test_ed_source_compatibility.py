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
