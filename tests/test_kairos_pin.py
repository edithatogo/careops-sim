import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('pin', Path(__file__).parents[1] / 'tools/check_kairos_pin.py')
pin = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pin)


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args], text=True, stderr=subprocess.DEVNULL).strip()


class KairosPinTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.child = self.root / 'libs/kairos'
        self.child.mkdir(parents=True)
        for root in (self.root, self.child):
            git(root, 'init')
            git(root, 'config', 'user.email', 'fixture@example.invalid')
            git(root, 'config', 'user.name', 'Fixture')
        (self.child / 'source').write_text('reviewed engine source\n')
        git(self.child, 'add', 'source')
        git(self.child, 'commit', '-m', 'reviewed')
        self.sha = git(self.child, 'rev-parse', 'HEAD')
        (self.root / 'conductor/evidence').mkdir(parents=True)
        self.contract = {'kairos_commit': self.sha, 'owner_ci': {'head_sha': self.sha, 'conclusion': 'success', 'passed_hosts': ['x86_64-unknown-linux-gnu', 'aarch64-apple-darwin']}, 'source_sha256': {'source': hashlib.sha256((self.child / 'source').read_bytes()).hexdigest()}}
        self.save_contract()
        (self.root / 'conductor/current-state.json').write_text(json.dumps({'submodule_pins': {'libs/kairos': self.sha}}))
        git(self.root, 'add', 'conductor')
        git(self.root, 'update-index', '--add', '--cacheinfo', '160000,' + self.sha + ',libs/kairos')
        git(self.root, 'commit', '-m', 'reviewed parent')

    def save_contract(self):
        (self.root / 'conductor/evidence/d2.4-kairos-contract.json').write_text(json.dumps(self.contract))

    def test_reviewed_exact_pin_passes(self):
        self.assertEqual([], pin.check(self.root))

    def test_incompatible_gitlink_fails_despite_green_upstream_evidence(self):
        # Same engine bytes, successful owner record; only parent integration differs.
        git(self.child, 'commit', '--allow-empty', '-m', 'unreviewed integration commit')
        other = git(self.child, 'rev-parse', 'HEAD')
        self.contract['owner_ci']['conclusion'] = 'success'
        self.save_contract()
        git(self.root, 'update-index', '--cacheinfo', '160000,' + other + ',libs/kairos')
        git(self.root, 'commit', '-m', 'incompatible pin')
        errors = pin.check(self.root)
        self.assertIn('committed gitlink differs from reviewed compatibility commit', errors)
        self.assertIn('checked-out Kairos differs from reviewed compatibility commit', errors)

    def test_metadata_and_checkout_cannot_override_committed_gitlink(self):
        (self.root / 'conductor/current-state.json').write_text(json.dumps({'submodule_pins': {'libs/kairos': '0' * 40}}))
        self.assertIn('expected metadata differs from reviewed compatibility commit', pin.check(self.root))

    def test_missing_owner_platform_evidence_fails(self):
        self.contract['owner_ci']['passed_hosts'] = ['x86_64-unknown-linux-gnu']
        self.save_contract()
        self.assertTrue(pin.check(self.root))

    def test_changed_engine_bytes_fail(self):
        (self.child / 'source').write_text('changed source\n')
        self.assertIn('reviewed source drift: source', pin.check(self.root))
