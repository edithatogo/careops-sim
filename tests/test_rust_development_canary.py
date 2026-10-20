"""Refuse invented compiler candidates and false qualification receipts."""
from copy import deepcopy
import json
import os
from pathlib import Path
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools import rust_development_canary as canary


def manifest(channel='beta', available=True):
    version = {'stable': '1.99.1', 'beta': '1.100.0-beta.7', 'nightly': '1.101.0-nightly'}[channel]
    text = f'date = "2026-10-18"\n[pkg.rustc]\nversion = "{version} (ccccccccc 2026-10-17)"\ngit_commit_hash = "' + 'c' * 40 + '"\n'
    for package in ('rustc', 'cargo', 'rust-std'):
        text += f'[pkg.{package}.target.{canary.HOST}]\navailable = {str(available).lower()}\n'
    return text.encode()


class RustDevelopmentCanary(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.previous = Path.cwd()
        self.addCleanup(os.chdir, self.previous)
        os.chdir(self.temporary.name)
        Path('rust-toolchain.toml').write_text('[toolchain]\nchannel = "1.99.0"\n')
        Path('Cargo.lock').write_text('exact source lock\n')
        self.output = Path('evidence')
        self.raw = manifest()
        self.candidate = canary.select('beta', self.raw, '1.99.0')

    def test_exact_dated_candidates_and_absent_newer_stable(self):
        self.assertEqual(self.candidate['toolchain'], 'beta-2026-10-18')
        self.assertEqual(self.candidate['manifestSha256'], canary.digest(self.raw))
        self.assertEqual(canary.select('nightly', manifest('nightly'), '1.99.0')['toolchain'], 'nightly-2026-10-18')
        self.assertIsNone(canary.select('stable', manifest('stable'), '1.99.1'))
        for raw in (manifest(available=False), self.raw.replace(b'cccccccccc', b'xxxxxxxxxx'), self.raw.replace(b'2026-10-18', b'../escape'), self.raw.replace(b'1.100.0-beta.7', b'not-a-version')):
            with self.subTest(raw=raw[:40]), self.assertRaises((ValueError, KeyError)):
                canary.select('beta', raw, '1.99.0')

    def test_frozen_manifest_mismatch_refuses_installation(self):
        wrong = dict(self.candidate, manifestSha256='0' * 64)
        with patch.object(canary, 'source_snapshot', side_effect=self.fixture_snapshot), patch.object(canary, 'fetch', return_value=self.raw), patch.object(canary.subprocess, 'check_output', return_value='a' * 40), patch.object(canary.subprocess, 'run') as run:
            self.assertEqual(canary.qualify(wrong, self.output, '1.99.0'), 1)
            run.assert_not_called()
        self.assertEqual(json.loads((self.output / 'qualification.json').read_text())['stages']['metadata'], 'failure')

    def test_compiler_identity_requires_release_commit_and_host(self):
        good = f"release: {self.candidate['release']}\ncommit-hash: {'c' * 40}\nhost: {canary.HOST}\n"
        canary.compiler_matches(good, self.candidate)
        for wrong in (good.replace('c' * 40, 'd' * 40), good.replace(canary.HOST, 'another-host'), good.replace(self.candidate['release'], '1.99.0')):
            with self.assertRaises(ValueError):
                canary.compiler_matches(wrong, self.candidate)

    def fixture_snapshot(self):
        source = Path('source.rs')
        return {'commit': 'a' * 40, 'trackedFileCount': 3, 'trackedSha256': canary.digest(source.read_bytes()) if source.exists() else 'd' * 64, 'gitlinks': {}}

    def run_qualification(self, failed_stage=None, wrong_binding=False, drift=False, source_drift=False):
        commands = []
        Path('source.rs').write_text('reviewed source')
        def run(command, **kwargs):
            stage = 'install' if command[1] == 'toolchain' else ('binding' if command[3] == 'rustc' else ('clippy' if 'clippy' in command else ('doctests' if '--doc' in command else 'tests')))
            commands.append(stage)
            if stage == 'binding':
                commit = 'd' * 40 if wrong_binding else 'c' * 40
                kwargs['stdout'].write(f"release: {self.candidate['release']}\ncommit-hash: {commit}\nhost: {canary.HOST}\n".encode())
            if source_drift and stage == 'tests':
                Path('source.rs').write_text('different source')
            if drift and stage == 'tests':
                Path('Cargo.lock').write_text('unexpected mutation')
            return SimpleNamespace(returncode=1 if stage == failed_stage else 0)
        def check_output(command, **kwargs):
            return 'a' * 40 if command[0] == 'git' else '/fixture/toolchain/bin/' + command[-1]
        with patch.object(canary, 'source_snapshot', side_effect=self.fixture_snapshot), patch.object(canary, 'fetch', return_value=self.raw), patch.object(canary.subprocess, 'check_output', side_effect=check_output), patch.object(canary.subprocess, 'run', side_effect=run):
            result = canary.qualify(self.candidate, self.output, '1.99.0')
        return result, json.loads((self.output / 'qualification.json').read_text()), commands

    def test_missing_setup_and_wrong_binding_never_execute_tests(self):
        for stage, wrong in [('install', False), (None, True)]:
            result, receipt, commands = self.run_qualification(stage, wrong)
            self.assertEqual(result, 1)
            self.assertEqual(receipt['state'], 'setup-failed')
            self.assertFalse(receipt['runtimeTestsExecuted'])
            self.assertNotIn('tests', commands)

    def test_test_failure_and_source_drift_remain_unqualified(self):
        result, receipt, commands = self.run_qualification('tests')
        self.assertEqual(result, 1)
        self.assertEqual(receipt['state'], 'executed-failure')
        self.assertEqual(receipt['exitCodes']['tests'], 1)
        self.assertIn('clippy', commands)
        result, receipt, _ = self.run_qualification(drift=True)
        self.assertEqual(result, 1)
        self.assertEqual(receipt['state'], 'source-drift')
        self.assertFalse(receipt['promoted'])

    def test_terminal_recorder_refuses_missing_corrupt_and_contradictory_evidence(self):
        path = self.output / 'qualification.json'
        self.assertEqual(canary.finalize(self.output, 'failure', 'a' * 40), 1)
        path.write_bytes(b'{partial')
        self.assertEqual(canary.finalize(self.output, 'failure', 'a' * 40), 1)
        self.assertEqual(path.with_suffix('.json.invalid').read_bytes(), b'{partial')
        for change in ({'sourceCommit': 'b' * 40}, {'sourcePinsUnchanged': False}, {'stages': {'tests': 'skipped'}}, {'stages': None}):
            receipt = {'state': 'qualified', 'sourceCommit': 'a' * 40, 'sourcePinsUnchanged': True, 'stages': {'tests': 'success'}}
            receipt.update(change)
            path.write_text(json.dumps(receipt))
            self.assertEqual(canary.finalize(self.output, 'success', 'a' * 40), 1)
            self.assertEqual(json.loads(path.read_text())['state'], 'evidence-invalid')

    def test_complete_qualification_preserves_real_stage_and_log_evidence(self):
        result, receipt, _ = self.run_qualification()
        self.assertEqual(result, 0)
        self.assertEqual(receipt['state'], 'qualified')
        self.assertEqual(set(receipt['stages']), set(canary.STAGES))
        self.assertTrue(receipt['runtimeTestsExecuted'])
        self.assertEqual(receipt['logSha256']['binding.log'], canary.digest((self.output / 'binding.log').read_bytes()))
        with patch.object(canary, 'source_snapshot', side_effect=self.fixture_snapshot):
            self.assertEqual(canary.finalize(self.output, 'success', 'a' * 40), 0)
            self.assertEqual(canary.finalize(self.output, 'failure', 'a' * 40), 1)

    def test_changed_source_stops_later_stages_without_changing_root_pins(self):
        _, receipt, commands = self.run_qualification(source_drift=True)
        self.assertEqual(receipt['state'], 'source-drift')
        self.assertFalse(receipt['sourcePinsUnchanged'])
        self.assertNotIn('doctests', commands)
        self.assertEqual(Path('Cargo.lock').read_text(), 'exact source lock\n')

    def test_qualified_terminal_requires_commands_exits_bindings_and_logs(self):
        _, good, _ = self.run_qualification()
        path = self.output / 'qualification.json'
        for field in ('commands', 'exitCodes', 'candidate', 'tools', 'logSha256', 'sourceBefore', 'sourceAfter', 'sourcePinSha256'):
            invalid = deepcopy(good)
            del invalid[field]
            path.write_text(json.dumps(invalid))
            with self.subTest(missing=field), patch.object(canary, 'source_snapshot', side_effect=self.fixture_snapshot):
                self.assertEqual(canary.finalize(self.output, 'success', 'a' * 40), 1)
                self.assertEqual(json.loads(path.read_text())['state'], 'evidence-invalid')
        for field, key, value in [('exitCodes', 'tests', 1), ('logSha256', 'tests.log', '0' * 64), ('commands', 'tests', ['true']), ('tools', 'rustc', '/different/bin/rustc')]:
            invalid = deepcopy(good)
            invalid[field][key] = value
            path.write_text(json.dumps(invalid))
            with self.subTest(field=field), patch.object(canary, 'source_snapshot', side_effect=self.fixture_snapshot):
                self.assertEqual(canary.finalize(self.output, 'success', 'a' * 40), 1)
        path.write_text(json.dumps(good))
        (self.output / 'tests.log').write_text('tampered actual log')
        with patch.object(canary, 'source_snapshot', side_effect=self.fixture_snapshot):
            self.assertEqual(canary.finalize(self.output, 'success', 'a' * 40), 1)
            self.assertIsNone(json.loads(path.read_text())['runtimeTestsExecuted'])

    def git(self, root, *arguments):
        return subprocess.check_output(['git', '-C', str(root), *arguments], text=True, stderr=subprocess.DEVNULL).strip()

    def repository(self, name):
        root = Path(name).resolve()
        root.mkdir()
        self.git(root, 'init', '-q')
        self.git(root, 'config', 'user.name', 'Fictional Fixture')
        self.git(root, 'config', 'user.email', 'fixture@example.invalid')
        (root / 'source.rs').write_text('reviewed source')
        self.git(root, 'add', 'source.rs')
        self.git(root, 'commit', '-qm', 'fictional source')
        return root

    def test_real_git_source_snapshot_detects_tracked_and_assume_unchanged_edits(self):
        root = self.repository('checkout')
        baseline = canary.source_snapshot(root)
        artifact = root / '.artifacts/rust-canary/test.log'
        artifact.parent.mkdir(parents=True)
        artifact.write_text('allowed output')
        self.assertEqual(canary.source_snapshot(root), baseline)
        (root / 'source.rs').write_text('unreviewed source')
        with self.assertRaises(ValueError):
            canary.source_snapshot(root)
        (root / 'source.rs').write_text('reviewed source')
        self.git(root, 'update-index', '--assume-unchanged', 'source.rs')
        (root / 'source.rs').write_text('hidden source change')
        with self.assertRaises(ValueError):
            canary.source_snapshot(root)

    def test_real_bulk_source_snapshot_has_bounded_input_and_output(self):
        root = self.repository('large-checkout')
        for index in range(500):
            (root / (str(index).zfill(4) + '-' + 'long-source-name-' * 8 + '.rs')).write_text('public fictional source')
        self.git(root, 'add', '.')
        self.git(root, 'commit', '-qm', 'fictional larger source tree')
        result = subprocess.run([os.sys.executable, '-c', 'from pathlib import Path; import sys; from tools.rust_development_canary import source_snapshot; print(source_snapshot(Path(sys.argv[1]))["trackedFileCount"])', str(root)], cwd=self.previous, capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), '501')

    def test_real_git_snapshot_refuses_changed_submodule_commit_or_source(self):
        root = self.repository('parent')
        child = self.repository('upstream')
        self.git(root, '-c', 'protocol.file.allow=always', 'submodule', 'add', '-q', str(child), 'libs/kairos')
        self.git(root, 'commit', '-qam', 'fictional pinned child')
        baseline = canary.source_snapshot(root)
        self.assertEqual(baseline['gitlinks']['libs/kairos']['commit'], self.git(child, 'rev-parse', 'HEAD'))
        actual = root / 'libs/kairos'
        (actual / 'source.rs').write_text('dirty child source')
        with self.assertRaises(ValueError):
            canary.source_snapshot(root)
        self.git(actual, 'config', 'user.name', 'Fictional Fixture')
        self.git(actual, 'config', 'user.email', 'fixture@example.invalid')
        self.git(actual, 'commit', '-qam', 'different fictional child')
        self.git(root, 'update-index', '--assume-unchanged', 'libs/kairos')
        with self.assertRaises(ValueError):
            canary.source_snapshot(root)

    def test_failed_discovery_retains_missing_and_corrupt_evidence(self):
        path = self.output / 'discovery.json'
        self.assertEqual(canary.finalize_discovery(self.output, 'failure', 'a' * 40), 0)
        self.assertEqual(json.loads(path.read_text())['state'], 'discovery-failed')
        path.write_bytes(b'{partial')
        self.assertEqual(canary.finalize_discovery(self.output, 'failure', 'a' * 40), 1)
        self.assertEqual(path.with_suffix('.json.invalid').read_bytes(), b'{partial')
        path.unlink()
        self.assertEqual(canary.finalize_discovery(self.output, 'success', 'a' * 40), 1)

    def test_actual_cli_missing_receipt_writes_unqualified_terminal_evidence(self):
        script = self.previous / 'tools/rust_development_canary.py'
        result = subprocess.run([os.sys.executable, str(script), 'finalize', '--output', str(self.output), '--outcome', 'failure', '--source', 'a' * 40], capture_output=True)
        self.assertEqual(result.returncode, 1)
        receipt = json.loads((self.output / 'qualification.json').read_text())
        self.assertEqual(receipt['state'], 'evidence-invalid')
        self.assertIsNone(receipt['runtimeTestsExecuted'])
        self.assertFalse(receipt['promoted'])


if __name__ == '__main__':
    unittest.main()
