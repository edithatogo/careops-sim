"""Execute real candidate workflow preparation, inventory and failure receipts."""
import hashlib
import json
from pathlib import Path
import platform
import shlex
import subprocess
import sys
import tempfile
import textwrap
import unittest

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/schema-development-canary.yml'
STAGES = ['initialize', 'source', 'bootstrap', 'prepare', 'resolve', 'install', 'inventory', 'tests', 'policy', 'audit', 'canonical']
PACKAGES = {'attrs', 'jsonschema', 'jsonschema-specifications', 'referencing', 'rpds-py'}


def step_script(name):
    part = WORKFLOW.read_text().split('      - name: ' + name + '\n', 1)[1]
    part = part.split('\n      - ', 1)[0]
    script = textwrap.dedent(part.split('        run: |\n', 1)[1])
    script = script.replace('"$CANARY_ROOT/venv/bin/python"', shlex.quote(sys.executable), 1)
    return script.replace('python3', shlex.quote(sys.executable), 1)


def environment(root):
    return {'CANARY_ROOT': str(root / 'candidate'), 'RUNNER_TEMP': str(root),
            'SOURCE_COMMIT': 'a' * 40, 'CANDIDATE_PYTHON': platform.python_version(),
            **{stage.upper() + '_OUTCOME': 'success' for stage in STAGES}}


def execute(name, root, env):
    return subprocess.run(['/bin/bash', '-euc', step_script(name)], cwd=root, env=env, capture_output=True, text=True)


class SchemaCanaryTests(unittest.TestCase):
    def test_preparation_preserves_source_and_refuses_unknown_or_duplicate_roots(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = (ROOT / 'requirements-schema-validation.txt').read_bytes()
            path = root / 'requirements-schema-validation.txt'
            path.write_bytes(source)
            env = environment(root)
            result = execute('Prepare isolated candidate declarations', root, env)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(set((root / 'candidate/candidate.in').read_text().splitlines()), PACKAGES)
            self.assertEqual(path.read_bytes(), source)
            for extra in [b'unknown-package==1\n', b'attrs==26.1.0\n', b'--index-url https://example.invalid\n']:
                path.write_bytes(source + extra)
                self.assertNotEqual(execute('Prepare isolated candidate declarations', root, env).returncode, 0)

    def test_actual_interpreter_inventory_and_receipt_bind_the_candidate(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root / 'candidate'
            output.mkdir()
            env = environment(root)
            result = execute('Inventory the actual target interpreter', root, env)
            self.assertEqual(result.returncode, 0, result.stderr)
            installed = json.loads((output / 'installed.json').read_text())
            self.assertEqual(installed['python'], platform.python_version())
            self.assertTrue(PACKAGES <= set(installed['packages']))
            manifest = (output / 'installed-requirements.txt').read_text()
            self.assertNotIn(';', manifest)
            for name, version in installed['packages'].items():
                self.assertIn(name + '==' + version + '\n', manifest)
            lock = b'fictional exact candidate evidence\n'
            (output / 'candidate.lock').write_bytes(lock)
            result = execute('Record honest terminal qualification', root, env)
            self.assertEqual(result.returncode, 0, result.stderr)
            receipt = json.loads((output / 'qualification.json').read_text())
            self.assertEqual(receipt['state'], 'qualified-schema-candidate')
            self.assertEqual(receipt['sha256']['candidate.lock'], hashlib.sha256(lock).hexdigest())
            self.assertIs(receipt['promoted'], False)
            self.assertIs(receipt['nativeWrites'], False)
            env['CANDIDATE_PYTHON'] = '3.99.0'
            self.assertEqual(execute('Record honest terminal qualification', root, env).returncode, 0)
            receipt = json.loads((output / 'qualification.json').read_text())
            self.assertEqual(receipt['state'], 'evidence-missing-or-runtime-mismatch')

    def test_all_failed_stages_cancelled_missing_and_unexecuted_are_never_promoted(self):
        cases = list(STAGES) + ['cancelled', 'skipped', 'missing']
        for failed in cases:
            with self.subTest(stage=failed), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                env = environment(root)
                if failed in STAGES:
                    position = STAGES.index(failed)
                    for i, stage in enumerate(STAGES):
                        env[stage.upper() + '_OUTCOME'] = 'success' if i < position else 'failure' if i == position else 'skipped'
                elif failed != 'missing':
                    env['TESTS_OUTCOME'] = failed
                if failed == 'initialize':
                    del env['CANARY_ROOT']
                result = execute('Record honest terminal qualification', root, env)
                self.assertEqual(result.returncode, 0, result.stderr)
                output = root / ('schema-development' if failed == 'initialize' else 'candidate')
                receipt = json.loads((output / 'qualification.json').read_text())
                self.assertNotEqual(receipt['state'], 'qualified-schema-candidate')
                self.assertIs(receipt['promoted'], False)
                self.assertEqual(receipt['failedStage'], failed if failed in STAGES else None)

    def test_actual_discovery_matrix_records_unavailable_and_normalises_preview(self):
        full = step_script('Freeze manifest and select exact candidates')
        code = full.split("python - <<'PYCODE'\n", 1)[1].rsplit('\nPYCODE', 1)[0]
        for candidate, expected in [(None, ['3.14.8', '3.15.0']), ('3.16.0-alpha.1', ['3.14.8', '3.15.0', '3.16.0a1'])]:
            with self.subTest(candidate=candidate), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                path = root / '.artifacts/ci-preview/discovery.json'
                path.parent.mkdir(parents=True)
                path.write_text(json.dumps({'candidate': candidate}))
                output = root / 'outputs'
                subprocess.run([sys.executable, '-c', code], cwd=root, env={'GITHUB_OUTPUT': str(output)}, check=True, capture_output=True)
                self.assertEqual(json.loads(output.read_text().split('=', 1)[1])['python'], expected)
                if candidate is None:
                    self.assertIn('unavailable', json.loads(path.read_text())['runtimeTestOutcome'])


if __name__ == '__main__':
    unittest.main()
