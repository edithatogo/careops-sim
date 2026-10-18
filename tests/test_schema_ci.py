"""Execute the actual schema CI receipt and committed-lock guard on failures."""
import hashlib
import json
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tempfile
import textwrap
import unittest

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/schema-lock-refresh.yml'


def receipt_script():
    step = WORKFLOW.read_text().split('      - name: Record actual terminal outcomes without promotion\n', 1)[1]
    step = step.split('      - name: Preserve regenerated lock and execution receipt', 1)[0]
    script = textwrap.dedent(step.split('        run: |\n', 1)[1])
    return script.replace('python3', shlex.quote(sys.executable), 1)


class SchemaCiTests(unittest.TestCase):
    def test_actual_receipt_retains_each_failure_and_unexecuted_downstream_stage(self):
        stages = ['resolve', 'install', 'tests', 'audit', 'committed']
        fields = ['resolveOutcome', 'installOutcome', 'testOutcome', 'auditOutcome', 'committedLockOutcome']
        names = ['RESOLVE_OUTCOME', 'INSTALL_OUTCOME', 'TEST_OUTCOME', 'AUDIT_OUTCOME', 'COMMITTED_OUTCOME']
        for position, stage in enumerate(stages):
            with self.subTest(stage=stage), tempfile.TemporaryDirectory() as temporary:
                outcomes = ['success' if i < position else 'failure' if i == position else 'skipped' for i in range(len(stages))]
                env = dict(zip(names, outcomes))
                env['GITHUB_SHA'] = 'a' * 40
                subprocess.run(['/bin/bash', '-euc', receipt_script()], cwd=temporary, env=env, check=True, capture_output=True)
                receipt = json.loads((Path(temporary) / '.artifacts/ci-suite/qualification.json').read_text())
                self.assertEqual([receipt[field] for field in fields], outcomes)
                self.assertIs(receipt['promoted'], False)
                self.assertIsNone(receipt['lockSha256'])

    def test_actual_receipt_binds_the_retained_candidate_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / '.artifacts/ci-suite'
            root.mkdir(parents=True)
            data = b'fictional candidate bytes\n'
            (root / 'requirements-schema-validation.lock').write_bytes(data)
            env = {name: 'success' for name in ['RESOLVE_OUTCOME', 'INSTALL_OUTCOME', 'TEST_OUTCOME', 'AUDIT_OUTCOME', 'COMMITTED_OUTCOME']}
            env['GITHUB_SHA'] = 'b' * 40
            subprocess.run(['/bin/bash', '-euc', receipt_script()], cwd=temporary, env=env, check=True, capture_output=True)
            receipt = json.loads((root / 'qualification.json').read_text())
            self.assertEqual(receipt['lockSha256'], hashlib.sha256(data).hexdigest())
            self.assertEqual(receipt['sourceCommit'], 'b' * 40)
            self.assertIs(receipt['promoted'], False)

    def test_actual_guard_accepts_identical_regeneration_and_refuses_changed_candidate(self):
        step = WORKFLOW.read_text().split('      - name: Refuse uncommitted lock repair in source checks\n', 1)[1]
        command = re.search(r'(?m)^        run: (cmp .+)$', step).group(1)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            candidate = root / '.artifacts/ci-suite/requirements-schema-validation.lock'
            candidate.parent.mkdir(parents=True)
            (root / 'requirements-schema-validation.lock').write_text('qualified bytes\n')
            candidate.write_text('qualified bytes\n')
            result = subprocess.run(['/bin/bash', '-euc', command], cwd=root, capture_output=True)
            self.assertEqual(result.returncode, 0)
            candidate.write_text('changed bytes\n')
            result = subprocess.run(['/bin/bash', '-euc', command], cwd=root, capture_output=True)
            self.assertEqual(result.returncode, 1)


if __name__ == '__main__':
    unittest.main()
