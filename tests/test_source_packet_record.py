import copy
import json
import unittest
from pathlib import Path

from tools.validate_source_packet_record import record_errors


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = json.loads((ROOT / 'model-inputs/ed/schema/source-verification-record.schema.json').read_text())
EXAMPLES = json.loads((ROOT / 'model-inputs/ed/schema/source-verification-examples.json').read_text())
BASE = next(x['record'] for x in EXAMPLES['cases'] if x['record']['evidence_type'] == 'unresolved_gap')


class SourcePacketRecordTests(unittest.TestCase):
    def setUp(self):
        self.record = copy.deepcopy(BASE)
        self.record['parameter_ids'] = ['ed.durations.cleaning']
        self.packet = {'parameter_ids': ['ed.durations.cleaning'], 'output_path': 'model-inputs/ed/des/evidence/cleaning.json'}
        self.path = ROOT / self.packet['output_path']

    def test_matching_gap_passes_without_empirical_promotion(self):
        self.assertEqual(record_errors(self.record, self.packet, self.path, ROOT, SCHEMA), [])

    def test_valid_catalogue_id_for_wrong_family_is_rejected(self):
        self.record['parameter_ids'] = ['ed.durations.intrinsicwork']
        self.assertIn('record parameter IDs differ from packet', record_errors(self.record, self.packet, self.path, ROOT, SCHEMA))

    def test_extra_parameter_is_not_silently_adopted(self):
        self.record['parameter_ids'].append('ed.durations.turnaround')
        self.assertIn('record parameter IDs differ from packet', record_errors(self.record, self.packet, self.path, ROOT, SCHEMA))

    def test_record_in_another_reserved_output_is_rejected(self):
        errors = record_errors(self.record, self.packet, self.path.with_name('handover.json'), ROOT, SCHEMA)
        self.assertIn('record path differs from packet output', errors)

    def test_schema_failure_prevents_mapping_pass(self):
        del self.record['gap']['owner']
        self.assertIn('source record fails schema validation', record_errors(self.record, self.packet, self.path, ROOT, SCHEMA))


if __name__ == '__main__':
    unittest.main()
