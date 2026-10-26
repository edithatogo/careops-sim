"""Negative cases for downstream review records; no acceptance is fabricated."""
import copy
import io
from contextlib import redirect_stdout
from unittest.mock import patch
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('context', Path(__file__).parents[1]/'tools/context.py')
context = importlib.util.module_from_spec(spec)
spec.loader.exec_module(context)


class DependencyHandoffTests(unittest.TestCase):
    def setUp(self):
        self.data = json.loads((context.ROOT/'conductor/dependency-handoffs.json').read_text())
        self.original_rows = copy.deepcopy(self.data['receipts'])

    def check_data(self, data):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'conductor').mkdir()
            for row in self.original_rows:
                follow = row.get('impact_review', {}).get('follow_up')
                if follow and '/' not in follow:
                    path = root/'conductor/tracks'/follow/'spec.md'
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text('Fixture track')
            (root/'conductor/dependency-handoffs.json').write_text(json.dumps(data))
            return context.validate_dependency_handoffs(root)

    def completed(self):
        data = copy.deepcopy(self.data)
        review = data['receipts'][0]['impact_review']
        review.update(status='complete', upstream_commit='a'*40, upstream_decision='adopt',
                      reviewer='consumer owner', reviewed_at='2026-10-24',
                      consumer_context='Fixture contract v1', rationale='No affected consumer',
                      evidence=['synthetic review evidence'], outcome='no_action', follow_up=None)
        return data

    def test_live_register(self):
        self.assertEqual(context.validate_dependency_handoffs(context.ROOT), [])

    def test_no_action_can_complete_without_implementation(self):
        self.assertEqual(self.check_data(self.completed()), [])

    def test_duplicate_rejected(self):
        self.data['receipts'].append(copy.deepcopy(self.data['receipts'][0]))
        self.assertTrue(any('Duplicate' in e for e in self.check_data(self.data)))

    def test_unknown_outcome_and_disposition_rejected(self):
        self.data['receipts'][0]['impact_review']['outcome'] = 'auto_accept'
        self.data['receipts'][0]['direct_dependency_disposition'] = 'install'
        errors = self.check_data(self.data)
        self.assertTrue(any('outcome' in e for e in errors))
        self.assertTrue(any('disposition' in e for e in errors))

    def test_completed_review_requires_each_receipt_field(self):
        for field in ('upstream_commit', 'upstream_decision', 'reviewer', 'reviewed_at',
                      'consumer_context', 'rationale', 'evidence', 'outcome'):
            with self.subTest(field=field):
                data = self.completed()
                data['receipts'][0]['impact_review'][field] = None
                self.assertTrue(self.check_data(data))

    def test_action_requires_follow_up(self):
        data = self.completed()
        for outcome in ('bounded_adaptation', 'implementation_track', 'pin_update_and_verification'):
            data['receipts'][0]['impact_review']['outcome'] = outcome
            self.assertTrue(any('follow-up' in e for e in self.check_data(data)))

    def test_path_escape_and_missing_follow_up_rejected(self):
        for follow in ('../missing', '/etc/passwd', 'nonexistent_track'):
            with self.subTest(follow=follow):
                self.data['receipts'][0]['impact_review']['follow_up'] = follow
                self.assertTrue(any('follow-up' in e for e in self.check_data(self.data)))

    def test_malformed_types_report_errors(self):
        for field, value in (('outcome', []), ('status', 'accepted'), ('implications', {})):
            with self.subTest(field=field):
                self.data['receipts'][0]['impact_review'][field] = value
                self.assertTrue(self.check_data(self.data))

    def test_context_command_propagates_register_failure(self):
        with patch.object(context, 'validate', return_value={'errors': []}), \
             patch.object(context, 'validate_dependency_handoffs', return_value=['invalid register']), \
             patch('sys.argv', ['context.py', 'check']), redirect_stdout(io.StringIO()):
            self.assertEqual(context.main(), 1)
